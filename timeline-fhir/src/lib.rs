//! Bộ chuyển đổi FHIR JSON thành `TimelineSnapshot`.

use chrono::{DateTime, Utc};
use serde_json::Value;
use timeline_core::{
    empty_snapshot, CriticalItem, EventCategory, ResourceReference, Severity, TimelineConfig,
    TimelineError, TimelineEvent, TimelineSnapshot,
};

/// Tổng hợp từ chuỗi JSON.
pub fn summarize_bundle_str(
    bundle_json: &str,
    config: &TimelineConfig,
) -> Result<TimelineSnapshot, TimelineError> {
    let value: Value =
        serde_json::from_str(bundle_json).map_err(|err| TimelineError::Parse(err.to_string()))?;
    summarize_bundle_value(&value, config)
}

/// Tổng hợp từ `serde_json::Value`.
pub fn summarize_bundle_value(
    bundle: &Value,
    config: &TimelineConfig,
) -> Result<TimelineSnapshot, TimelineError> {
    let bundle_type = bundle
        .get("resourceType")
        .and_then(Value::as_str)
        .ok_or_else(|| TimelineError::MissingData)?;

    if bundle_type != "Bundle" {
        return Err(TimelineError::Parse(format!(
            "resourceType mong đợi là Bundle, nhận {bundle_type}"
        )));
    }

    // TODO: thay thế mock bằng logic phân tích thực sự.
    let mut snapshot = empty_snapshot();

    if let Some(patient) = bundle
        .get("entry")
        .and_then(Value::as_array)
        .and_then(|entries| entries.iter().find_map(|entry| entry.get("resource")))
    {
        if let Some(name) = extract_patient_name(patient) {
            snapshot.critical.alerts.push(CriticalItem {
                label: format!("Bệnh nhân: {name}"),
                detail: None,
                severity: Severity::Info,
            });
        }
    }

    // Ví dụ demo: lấy các Observation gần đây làm sự kiện.
    let events = extract_observation_events(bundle, config)?;
    snapshot.events = events;

    Ok(snapshot)
}

fn extract_patient_name(patient: &Value) -> Option<String> {
    let names = patient.get("name")?.as_array()?;
    let first = names.first()?;
    let given = first
        .get("given")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first()?.as_str())
        .unwrap_or("");
    let family = first.get("family").and_then(Value::as_str).unwrap_or("");
    let full = format!("{given} {family}").trim().to_string();
    if full.is_empty() {
        None
    } else {
        Some(full)
    }
}

fn extract_observation_events(
    bundle: &Value,
    _config: &TimelineConfig,
) -> Result<Vec<TimelineEvent>, TimelineError> {
    let mut events = Vec::new();

    let entries = match bundle.get("entry").and_then(Value::as_array) {
        Some(entries) => entries,
        None => return Ok(events),
    };

    for entry in entries {
        let resource = match entry.get("resource") {
            Some(resource) => resource,
            None => continue,
        };

        let resource_type = match resource.get("resourceType").and_then(Value::as_str) {
            Some(rt) => rt,
            None => continue,
        };

        if resource_type != "Observation" {
            continue;
        }

        let id = resource
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("observation")
            .to_string();

        let title = resource
            .get("code")
            .and_then(|code| code.get("text"))
            .and_then(Value::as_str)
            .unwrap_or("Observation")
            .to_string();

        let value = resource
            .get("valueQuantity")
            .and_then(format_quantity)
            .or_else(|| {
                resource
                    .get("valueString")
                    .and_then(Value::as_str)
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "Không có giá trị".to_string());

        let issued = resource
            .get("issued")
            .and_then(Value::as_str)
            .and_then(parse_datetime);

        events.push(TimelineEvent {
            id,
            category: EventCategory::Observation,
            title,
            detail: Some(value),
            occurred_at: issued,
            severity: Severity::Info,
            source: Some(ResourceReference {
                system: Some("FHIR".into()),
                reference: resource
                    .get("id")
                    .and_then(Value::as_str)
                    .map(|s| s.to_string()),
                display: None,
            }),
        });
    }

    Ok(events)
}

fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

fn format_quantity(value: &Value) -> Option<String> {
    let magnitude = value.get("value")?.as_f64()?;
    let unit = value.get("unit").and_then(Value::as_str).unwrap_or("");
    if unit.is_empty() {
        Some(format!("{magnitude}"))
    } else {
        Some(format!("{magnitude} {unit}"))
    }
}
