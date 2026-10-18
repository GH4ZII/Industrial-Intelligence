"""Per-asset Isolation Forest models."""

from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass

import numpy as np
import pandas as pd
from sklearn.ensemble import IsolationForest

from .scoring import map_decision_to_score, zscore_anomaly

WINDOW = 60
MIN_TRAIN_ROWS = 20


@dataclass
class AssetModel:
    model: IsolationForest | None
    feature_names: list[str]
    means: dict[str, float]
    stds: dict[str, float]


def _pivot_series(rows: list[dict]) -> pd.DataFrame:
    if not rows:
        return pd.DataFrame()
    frame = pd.DataFrame(rows)
    frame["time"] = pd.to_datetime(frame["time"], utc=True)
    pivoted = (
        frame.pivot_table(
            index="time",
            columns="metric",
            values="value",
            aggfunc="last",
        )
        .sort_index()
        .ffill()
        .dropna(how="any")
    )
    return pivoted


def build_feature_matrix(pivoted: pd.DataFrame) -> tuple[np.ndarray, list[str], pd.DataFrame]:
    if pivoted.empty:
        return np.empty((0, 0)), [], pivoted

    rolled_mean = pivoted.rolling(WINDOW, min_periods=1).mean()
    rolled_std = pivoted.rolling(WINDOW, min_periods=1).std().fillna(0.0)

    parts = [pivoted]
    feature_names = list(pivoted.columns)
    extra_names: list[str] = []
    for column in pivoted.columns:
        parts.append(rolled_mean[column].rename(f"{column}__mean"))
        parts.append(rolled_std[column].rename(f"{column}__std"))
        extra_names.extend([f"{column}__mean", f"{column}__std"])

    features = pd.concat(parts, axis=1)
    names = feature_names + extra_names
    matrix = features.to_numpy(dtype=float)
    return matrix, names, features


def train_asset_model(rows: list[dict]) -> AssetModel | None:
    pivoted = _pivot_series(rows)
    matrix, names, _ = build_feature_matrix(pivoted)
    if matrix.shape[0] < MIN_TRAIN_ROWS or matrix.shape[1] == 0:
        means = {col: float(pivoted[col].mean()) for col in pivoted.columns} if not pivoted.empty else {}
        stds = {col: float(pivoted[col].std() or 0.0) for col in pivoted.columns} if not pivoted.empty else {}
        return AssetModel(model=None, feature_names=list(pivoted.columns), means=means, stds=stds)

    means = {col: float(pivoted[col].mean()) for col in pivoted.columns}
    stds = {col: float(pivoted[col].std() or 0.0) for col in pivoted.columns}
    model = IsolationForest(
        n_estimators=100,
        contamination=0.05,
        random_state=42,
    )
    model.fit(matrix)
    return AssetModel(model=model, feature_names=names, means=means, stds=stds)


def score_latest(model: AssetModel, rows: list[dict]) -> tuple[float, dict[str, float]] | None:
    pivoted = _pivot_series(rows)
    if pivoted.empty:
        return None

    latest_values = {col: float(pivoted.iloc[-1][col]) for col in pivoted.columns}

    if model.model is None:
        zs = [
            zscore_anomaly(latest_values[col], model.means.get(col, latest_values[col]), model.stds.get(col, 0.0))
            for col in pivoted.columns
        ]
        score = max(zs) if zs else 0.0
        return score, latest_values

    matrix, names, _ = build_feature_matrix(pivoted)
    if matrix.shape[0] == 0:
        return None

    row = matrix[-1].reshape(1, -1)
    # Align columns if the live feature set drifted.
    if names != model.feature_names:
        aligned = np.zeros((1, len(model.feature_names)))
        index = {name: i for i, name in enumerate(names)}
        for j, name in enumerate(model.feature_names):
            if name in index:
                aligned[0, j] = row[0, index[name]]
        row = aligned

    decision = float(model.model.decision_function(row)[0])
    return map_decision_to_score(decision), latest_values


def group_rows_by_asset(rows: list[dict]) -> dict[str, list[dict]]:
    grouped: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        grouped[row["asset_id"]].append(row)
    return grouped
