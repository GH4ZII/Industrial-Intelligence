use chrono::{DateTime, Utc};
use sqlx::FromRow;

pub const TEMP_WARNING: f64 = 90.0;
pub const TEMP_CRITICAL: f64 = 110.0;
pub const VIB_WARNING: f64 = 5.0;
pub const VIB_CRITICAL: f64 = 9.0;
pub const PRESSURE_WARNING: f64 = 3.5;
pub const PRESSURE_CRITICAL: f64 = 2.5;
pub const RPM_DEV_WARNING: f64 = 0.20;
pub const RPM_DEV_CRITICAL: f64 = 0.30;

#[derive(Debug, FromRow)]
pub struct LatestMetric {
    pub site: String,
    pub asset_type: String,
    pub asset_id: String,
    pub metric: String,
    pub value: f64,
    pub time: DateTime<Utc>,
}

#[derive(Debug, PartialEq)]
pub struct FiredAlert {
    pub site: String,
    pub asset_id: String,
    pub severity: &'static str,
    pub alert_type: &'static str,
    pub message: String,
    pub value: f64,
    pub unit: &'static str,
    pub detected_at: DateTime<Utc>,
}

pub fn evaluate_asset(metrics: Vec<&LatestMetric>) -> Vec<FiredAlert> {
    let mut alerts = Vec::new();

    let site = metrics
        .first()
        .map(|m| m.site.as_str())
        .unwrap_or("unknown");
    let asset_id = metrics
        .first()
        .map(|m| m.asset_id.as_str())
        .unwrap_or("unknown");
    let asset_type = metrics
        .first()
        .map(|m| m.asset_type.as_str())
        .unwrap_or("asset");

    for metric in &metrics {
        match metric.metric.as_str() {
            "temperature" => {
                if let Some((severity, alert_type)) =
                    high_threshold(metric.value, TEMP_WARNING, TEMP_CRITICAL, "temperature")
                {
                    alerts.push(FiredAlert {
                        site: site.to_string(),
                        asset_id: asset_id.to_string(),
                        severity,
                        alert_type,
                        message: format!(
                            "High temperature detected on {} {}",
                            asset_type, asset_id
                        ),
                        value: metric.value,
                        unit: "°C",
                        detected_at: metric.time,
                    });
                }
            }
            "vibration" => {
                if let Some((severity, alert_type)) =
                    high_threshold(metric.value, VIB_WARNING, VIB_CRITICAL, "vibration")
                {
                    alerts.push(FiredAlert {
                        site: site.to_string(),
                        asset_id: asset_id.to_string(),
                        severity,
                        alert_type,
                        message: format!(
                            "High vibration detected on {} {}",
                            asset_type, asset_id
                        ),
                        value: metric.value,
                        unit: "mm/s",
                        detected_at: metric.time,
                    });
                }
            }
            "pressure" => {
                if let Some((severity, alert_type)) =
                    low_threshold(metric.value, PRESSURE_WARNING, PRESSURE_CRITICAL)
                {
                    alerts.push(FiredAlert {
                        site: site.to_string(),
                        asset_id: asset_id.to_string(),
                        severity,
                        alert_type,
                        message: format!(
                            "Low pressure detected on {} {}",
                            asset_type, asset_id
                        ),
                        value: metric.value,
                        unit: "bar",
                        detected_at: metric.time,
                    });
                }
            }
            "rpm" => {
                if let Some(nominal) = nominal_rpm(asset_type, asset_id) {
                    let deviation = (metric.value - nominal).abs() / nominal;
                    if let Some((severity, alert_type)) = high_threshold(
                        deviation,
                        RPM_DEV_WARNING,
                        RPM_DEV_CRITICAL,
                        "rpm_deviation",
                    ) {
                        alerts.push(FiredAlert {
                            site: site.to_string(),
                            asset_id: asset_id.to_string(),
                            severity,
                            alert_type,
                            message: format!(
                                "RPM deviation detected on {} {} ({:.0} vs nominal {:.0})",
                                asset_type, asset_id, metric.value, nominal
                            ),
                            value: deviation * 100.0,
                            unit: "%",
                            detected_at: metric.time,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    alerts
}

pub fn high_threshold(
    value: f64,
    warning: f64,
    critical: f64,
    base: &'static str,
) -> Option<(&'static str, &'static str)> {
    if value > critical {
        Some((
            "CRITICAL",
            match base {
                "temperature" => "temperature_critical",
                "vibration" => "vibration_critical",
                "rpm_deviation" => "rpm_deviation_critical",
                _ => "threshold_critical",
            },
        ))
    } else if value > warning {
        Some((
            "WARNING",
            match base {
                "temperature" => "temperature_warning",
                "vibration" => "vibration_warning",
                "rpm_deviation" => "rpm_deviation_warning",
                _ => "threshold_warning",
            },
        ))
    } else {
        None
    }
}

pub fn low_threshold(
    value: f64,
    warning: f64,
    critical: f64,
) -> Option<(&'static str, &'static str)> {
    if value < critical {
        Some(("CRITICAL", "pressure_critical"))
    } else if value < warning {
        Some(("WARNING", "pressure_warning"))
    } else {
        None
    }
}

pub fn nominal_rpm(asset_type: &str, asset_id: &str) -> Option<f64> {
    match (asset_type, asset_id) {
        ("pump", _) | (_, "P101") => Some(1450.0),
        ("motor", _) | (_, "M201") => Some(1450.0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn metric(name: &str, value: f64) -> LatestMetric {
        LatestMetric {
            site: "trondheim".to_string(),
            asset_type: "pump".to_string(),
            asset_id: "P101".to_string(),
            metric: name.to_string(),
            value,
            time: Utc::now(),
        }
    }

    #[test]
    fn temperature_critical_above_110() {
        let row = metric("temperature", 112.0);
        let alerts = evaluate_asset(vec![&row]);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, "CRITICAL");
        assert_eq!(alerts[0].alert_type, "temperature_critical");
    }

    #[test]
    fn vibration_warning_between_thresholds() {
        let row = metric("vibration", 6.2);
        let alerts = evaluate_asset(vec![&row]);
        assert_eq!(alerts[0].severity, "WARNING");
        assert_eq!(alerts[0].alert_type, "vibration_warning");
    }

    #[test]
    fn pressure_critical_below_2_5() {
        let row = metric("pressure", 2.1);
        let alerts = evaluate_asset(vec![&row]);
        assert_eq!(alerts[0].alert_type, "pressure_critical");
    }

    #[test]
    fn normal_values_do_not_alert() {
        let rows = [
            metric("temperature", 63.0),
            metric("vibration", 2.1),
            metric("pressure", 5.0),
            metric("rpm", 1450.0),
        ];
        let refs: Vec<&LatestMetric> = rows.iter().collect();
        assert!(evaluate_asset(refs).is_empty());
    }
}
