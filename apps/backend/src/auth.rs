use argon2::{
    password_hash::{
        phc::PasswordHash,
        PasswordHasher,
        PasswordVerifier,
    },
    Argon2,
};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    models::{
        LoginRequest,
        LoginResponse,
        LoginUser,
        RegisterRequest,
        Role,
        UserPublic,
    },
    AppState,
};


pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<UserPublic>, (StatusCode, String)> {

    let (user_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&state.pool)
            .await
            .map_err(internal_error)?;


    // Første bruker blir admin.
    // Senere brukere blir operator.
    let role_name =
        if user_count == 0 {
            "admin"
        } else {
            "operator"
        };


    let password_hash =
        Argon2::default()
            .hash_password(request.password.as_bytes())
            .map_err(|error| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error.to_string(),
                )
            })?
            .to_string();


    let role_id: i32 =
        sqlx::query_scalar(
            "SELECT id FROM roles WHERE name = $1"
        )
        .bind(role_name)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;


    let result: (i64, String) =
        sqlx::query_as(
            r#"
            INSERT INTO users (
                email,
                password_hash,
                role_id
            )
            VALUES ($1, $2, $3)
            RETURNING id, email
            "#,
        )
        .bind(&request.email)
        .bind(password_hash)
        .bind(role_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;


    Ok(Json(UserPublic {
        id: result.0,
        email: result.1,
        role: role_name.to_string(),
    }))
}


pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {

    let user =
        sqlx::query_as::<_, LoginUser>(
            r#"
            SELECT
                u.id,
                u.email,
                u.password_hash,
                r.name AS role
            FROM users u
            JOIN roles r
                ON r.id = u.role_id
            WHERE u.email = $1
            "#,
        )
        .bind(&request.email)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid email or password".to_string(),
        ))?;


    let parsed_hash =
        PasswordHash::new(&user.password_hash)
            .map_err(|_| (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid stored password hash".to_string(),
            ))?;


    if Argon2::default()
        .verify_password(
            request.password.as_bytes(),
            &parsed_hash,
        )
        .is_err()
    {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid email or password".to_string(),
        ));
    }


    let token = Uuid::new_v4();

    let expires_at =
        Utc::now() + Duration::hours(12);


    sqlx::query(
        r#"
        INSERT INTO sessions (
            token,
            user_id,
            expires_at
        )
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(token)
    .bind(user.id)
    .bind(expires_at)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;


    Ok(Json(LoginResponse {
        token,
        expires_at,
        user: UserPublic {
            id: user.id,
            email: user.email,
            role: user.role,
        },
    }))
}


pub async fn me(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<UserPublic>, (StatusCode, String)> {

    let user =
        authenticate(&headers, &state).await?;

    Ok(Json(user))
}


pub async fn get_users(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<UserPublic>>, (StatusCode, String)> {

    let current_user =
        authenticate(&headers, &state).await?;


    if current_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Admin access required".to_string(),
        ));
    }


    let users =
        sqlx::query_as::<_, UserPublic>(
            r#"
            SELECT
                u.id,
                u.email,
                r.name AS role
            FROM users u
            JOIN roles r
                ON r.id = u.role_id
            ORDER BY u.id
            "#,
        )
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;


    Ok(Json(users))
}


pub async fn get_roles(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<Role>>, (StatusCode, String)> {

    authenticate(&headers, &state).await?;


    let roles =
        sqlx::query_as::<_, Role>(
            "SELECT id, name FROM roles ORDER BY id"
        )
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;


    Ok(Json(roles))
}


async fn authenticate(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<UserPublic, (StatusCode, String)> {

    let authorization =
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header".to_string(),
            ))?;


    let token_string =
        authorization
            .strip_prefix("Bearer ")
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Expected Bearer token".to_string(),
            ))?;


    let token =
        Uuid::parse_str(token_string)
            .map_err(|_| (
                StatusCode::UNAUTHORIZED,
                "Invalid session token".to_string(),
            ))?;


    sqlx::query_as::<_, UserPublic>(
        r#"
        SELECT
            u.id,
            u.email,
            r.name AS role
        FROM sessions s
        JOIN users u
            ON u.id = s.user_id
        JOIN roles r
            ON r.id = u.role_id
        WHERE
            s.token = $1
            AND s.expires_at > NOW()
        "#,
    )
    .bind(token)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or((
        StatusCode::UNAUTHORIZED,
        "Session expired or invalid".to_string(),
    ))
}


fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {}", error),
    )
}