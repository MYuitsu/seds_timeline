//! Bộ chuyển đổi FHIR JSON thành `TimelineSnapshot` v0.2 với phân tích mở rộng.

use std::collections::HashMap;

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde_json::Value;
use timeline_core::{
    CriticalItem, CriticalSummary, EventCategory, ResourceReference, Severity, TimelineConfig,
    TimelineError, TimelineEvent, TimelineSnapshot, VitalSnapshot,
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

    let entries = bundle
        .get("entry")
        .and_then(Value::as_array)
        .ok_or(TimelineError::MissingData)?;

    let anchor = compute_anchor(entries);
    let mut aggregate = AggregateData::with_anchor(anchor);

    for entry in entries {
        let Some(resource) = entry.get("resource") else {
            continue;
        };

        match resource
            .get("resourceType")
            .and_then(Value::as_str)
            .unwrap_or_default()
        {
            "Patient" => aggregate.handle_patient(resource),
            "AllergyIntolerance" => aggregate.handle_allergy(resource),
            "MedicationStatement" => aggregate.handle_medication(resource),
            "MedicationRequest" => aggregate.handle_medication(resource),
            "Condition" => aggregate.handle_condition(resource, config),
            "Observation" => aggregate.handle_observation(resource, config),
            "Procedure" => aggregate.handle_procedure(resource),
            "Encounter" => aggregate.handle_encounter(resource),
            "DocumentReference" | "Composition" => aggregate.handle_document(resource),
            _ => {}
        }
    }

    Ok(aggregate.finalize(config))
}

#[derive(Default)]
struct AggregateData {
    anchor: Option<DateTime<Utc>>,
    alerts: Vec<CriticalItem>,
    allergies: Vec<CriticalItem>,
    medications: Vec<CriticalItem>,
    chronic_conditions: Vec<CriticalItem>,
    code_status: Option<CodeStatusRecord>,
    vitals: HashMap<String, VitalSnapshot>,
    events: Vec<TimelineEvent>,
}

impl AggregateData {
    fn with_anchor(anchor: Option<DateTime<Utc>>) -> Self {
        Self {
            anchor,
            ..Self::default()
        }
    }

    fn handle_patient(&mut self, resource: &Value) {
        if let Some(name) = extract_patient_name(resource) {
            let mut detail_parts = Vec::new();

            if let Some(age) = extract_patient_age(resource) {
                detail_parts.push(format!("{age} tuổi"));
            }

            if let Some(gender) = resource.get("gender").and_then(Value::as_str) {
                detail_parts.push(match gender {
                    "male" => "Nam".to_string(),
                    "female" => "Nữ".to_string(),
                    other => format!("Giới tính: {other}"),
                });
            }

            let detail = if detail_parts.is_empty() {
                None
            } else {
                Some(detail_parts.join(" | "))
            };

            self.alerts.push(CriticalItem {
                label: format!("Bệnh nhân: {name}"),
                detail,
                severity: Severity::Info,
            });
        }
    }

    fn handle_allergy(&mut self, resource: &Value) {
        let Some(label) = resource.get("code").and_then(extract_codeable_text) else {
            return;
        };

        let severity = map_allergy_severity(resource);
        let mut details = Vec::new();

        if let Some(category) = resource
            .get("category")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        {
            if !category.is_empty() {
                details.push(format!(
                    "Loại: {}",
                    category
                        .into_iter()
                        .map(|s| capitalize_first(s))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        if let Some(reactions) = summarize_reactions(resource) {
            details.push(format!("Phản ứng: {reactions}"));
        }

        if let Some(criticality) = resource.get("criticality").and_then(Value::as_str) {
            details.push(format!("Độ nguy kịch: {}", criticality.to_uppercase()));
        }

        let recorded_at = extract_datetime(resource, &["recordedDate", "onsetDateTime"]);

        let detail = if details.is_empty() {
            None
        } else {
            Some(details.join(" | "))
        };

        let item = CriticalItem {
            label: format!("Dị ứng: {label}"),
            detail: detail.clone(),
            severity,
        };

        self.allergies.push(item);

        self.events.push(TimelineEvent {
            id: resource_id(resource, "allergy"),
            category: EventCategory::Condition,
            title: format!("Ghi nhận dị ứng với {label}"),
            detail,
            occurred_at: recorded_at,
            severity,
            source: make_reference(resource),
        });
    }

    fn handle_medication(&mut self, resource: &Value) {
        let medication = resource
            .get("medicationCodeableConcept")
            .and_then(extract_codeable_text)
            .or_else(|| {
                resource.get("medicationReference").and_then(|value| {
                    value
                        .get("display")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
            })
            .unwrap_or_else(|| "Thuốc chưa rõ".to_string());

        let status = resource
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown");

        let severity = match status {
            "active" | "intended" => Severity::High,
            "on-hold" => Severity::Moderate,
            "completed" => Severity::Low,
            _ => Severity::Moderate,
        };

        let mut details = Vec::new();
        details.push(format!("Trạng thái: {}", status.to_uppercase()));

        if let Some(reason) = resource
            .get("reasonCode")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(extract_codeable_text)
        {
            details.push(format!("Chỉ định: {reason}"));
        }

        if let Some(dose) = summarize_dosage(resource) {
            details.push(dose);
        }

        let recorded_at = extract_datetime(
            resource,
            &[
                "effectiveDateTime",
                "effectivePeriod",
                "dateAsserted",
                "authoredOn",
            ],
        );

        let detail = Some(details.join(" | "));

        let item = CriticalItem {
            label: format!("Thuốc: {medication}"),
            detail: detail.clone(),
            severity,
        };
        self.medications.push(item);

        self.events.push(TimelineEvent {
            id: resource_id(resource, "medication"),
            category: EventCategory::Medication,
            title: format!("{medication}"),
            detail,
            occurred_at: recorded_at,
            severity,
            source: make_reference(resource),
        });
    }

    fn handle_condition(&mut self, resource: &Value, config: &TimelineConfig) {
        let Some(condition_name) = resource.get("code").and_then(extract_codeable_text) else {
            return;
        };

        let recorded_at = extract_datetime(
            resource,
            &["recordedDate", "onsetDateTime", "onsetDate", "assertedDate"],
        );

        if !is_recent_event(self.anchor, recorded_at, config.clinical_event_days) {
            return;
        }

        let severity = map_condition_severity(&condition_name);

        let mut details = Vec::new();
        if let Some(status) = extract_status_code(resource.get("clinicalStatus")) {
            details.push(format!("Trạng thái: {status}"));
        }
        if let Some(severity_text) = extract_status_code(resource.get("severity")) {
            details.push(format!("Mức độ: {severity_text}"));
        }

        let item = CriticalItem {
            label: format!("Bệnh mạn: {condition_name}"),
            detail: if details.is_empty() {
                None
            } else {
                Some(details.join(" | "))
            },
            severity,
        };

        self.chronic_conditions.push(item.clone());

        self.events.push(TimelineEvent {
            id: resource_id(resource, "condition"),
            category: EventCategory::Condition,
            title: condition_name,
            detail: item.detail,
            occurred_at: recorded_at,
            severity,
            source: make_reference(resource),
        });
    }

    fn handle_observation(&mut self, resource: &Value, _config: &TimelineConfig) {
        let name = resource
            .get("code")
            .and_then(extract_codeable_text)
            .unwrap_or_else(|| "Observation".to_string());

        if observation_is_code_status(resource) {
            if let Some(value) = observation_value_text(resource) {
                let recorded_at = extract_observation_timestamp(resource);
                let severity = Severity::Critical;
                self.code_status = match &self.code_status {
                    Some(existing) if is_more_recent(existing.recorded_at, recorded_at) => {
                        self.code_status.clone()
                    }
                    _ => Some(CodeStatusRecord { value, recorded_at }),
                };

                self.events.push(TimelineEvent {
                    id: resource_id(resource, "code-status"),
                    category: EventCategory::Observation,
                    title: "Cập nhật code status".to_string(),
                    detail: self.code_status.as_ref().map(|cs| cs.value.clone()),
                    occurred_at: recorded_at,
                    severity,
                    source: make_reference(resource),
                });
            }
            return;
        }

        let detail = match summarize_observation_value(resource) {
            Some(detail) => detail,
            None => return,
        };

        let recorded_at = extract_observation_timestamp(resource);
        let severity = classify_observation(&name, resource, &detail);

        let event = TimelineEvent {
            id: resource_id(resource, "observation"),
            category: EventCategory::Observation,
            title: name.clone(),
            detail: Some(detail.clone()),
            occurred_at: recorded_at,
            severity,
            source: make_reference(resource),
        };

        if let Some(vital_label) = infer_vital_label(&name) {
            let snapshot = VitalSnapshot {
                name: vital_label.to_string(),
                value: detail.clone(),
                recorded_at,
            };
            self.upsert_vital(snapshot);
        }

        self.events.push(event);
    }

    fn handle_procedure(&mut self, resource: &Value) {
        let name = resource
            .get("code")
            .and_then(extract_codeable_text)
            .unwrap_or_else(|| "Thủ thuật".to_string());
        let recorded_at = extract_datetime(resource, &["performedDateTime", "performedPeriod"]);
        let severity = Severity::Moderate;

        self.events.push(TimelineEvent {
            id: resource_id(resource, "procedure"),
            category: EventCategory::Procedure,
            title: name,
            detail: extract_status_code(resource.get("status")),
            occurred_at: recorded_at,
            severity,
            source: make_reference(resource),
        });
    }

    fn handle_encounter(&mut self, resource: &Value) {
        let label = resource
            .get("class")
            .and_then(extract_codeable_text)
            .unwrap_or_else(|| {
                resource
                    .get("type")
                    .and_then(Value::as_array)
                    .and_then(|arr| arr.first())
                    .and_then(extract_codeable_text)
                    .unwrap_or_else(|| "Encounter".to_string())
            });

        let recorded_at = extract_datetime(resource, &["period"]);

        self.events.push(TimelineEvent {
            id: resource_id(resource, "encounter"),
            category: EventCategory::Encounter,
            title: format!("Chăm sóc: {label}"),
            detail: resource
                .get("reasonCode")
                .and_then(Value::as_array)
                .and_then(|arr| arr.first())
                .and_then(extract_codeable_text),
            occurred_at: recorded_at,
            severity: Severity::Info,
            source: make_reference(resource),
        });
    }

    fn handle_document(&mut self, resource: &Value) {
        let title = resource
            .get("type")
            .and_then(extract_codeable_text)
            .or_else(|| {
                resource
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "Tài liệu lâm sàng".to_string());

        let recorded_at = extract_datetime(resource, &["date", "created"]);

        self.events.push(TimelineEvent {
            id: resource_id(resource, "document"),
            category: EventCategory::Document,
            title,
            detail: resource
                .get("content")
                .and_then(Value::as_array)
                .and_then(|arr| arr.first())
                .and_then(|content| {
                    content
                        .get("attachment")
                        .and_then(|attachment| attachment.get("title"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                }),
            occurred_at: recorded_at,
            severity: Severity::Low,
            source: make_reference(resource),
        });
    }

    fn upsert_vital(&mut self, snapshot: VitalSnapshot) {
        match self.vitals.get_mut(&snapshot.name) {
            Some(existing) => {
                if is_more_recent(snapshot.recorded_at, existing.recorded_at) {
                    *existing = snapshot;
                }
            }
            None => {
                self.vitals.insert(snapshot.name.clone(), snapshot);
            }
        }
    }

    fn finalize(mut self, config: &TimelineConfig) -> TimelineSnapshot {
        self.alerts.sort_by_key(|item| item.severity);
        self.allergies.sort_by_key(|item| item.severity);
        self.medications.sort_by_key(|item| item.severity);
        self.chronic_conditions.sort_by_key(|item| item.severity);

        let mut vital_values: Vec<VitalSnapshot> = self
            .vitals
            .into_values()
            .filter(|vital| {
                is_recent_vital(self.anchor, vital.recorded_at, config.vital_recent_hours)
            })
            .collect();
        vital_values.sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at));

        let critical = CriticalSummary {
            allergies: self.allergies,
            medications: self.medications,
            chronic_conditions: self.chronic_conditions,
            code_status: self.code_status.map(|cs| cs.value),
            alerts: self.alerts,
            recent_vitals: vital_values,
        };

        TimelineSnapshot::new(critical, self.events)
    }
}

#[derive(Clone)]
struct CodeStatusRecord {
    value: String,
    recorded_at: Option<DateTime<Utc>>,
}

fn compute_anchor(entries: &[Value]) -> Option<DateTime<Utc>> {
    entries
        .iter()
        .filter_map(|entry| entry.get("resource"))
        .filter_map(resource_timestamp)
        .max()
}

fn resource_timestamp(resource: &Value) -> Option<DateTime<Utc>> {
    let resource_type = resource.get("resourceType").and_then(Value::as_str)?;
    match resource_type {
        "Observation" => extract_observation_timestamp(resource),
        "Condition" => extract_datetime(
            resource,
            &["recordedDate", "onsetDateTime", "onsetDate", "assertedDate"],
        ),
        "MedicationStatement" => extract_datetime(
            resource,
            &[
                "effectiveDateTime",
                "effectivePeriod",
                "dateAsserted",
                "authoredOn",
            ],
        ),
        "MedicationRequest" => extract_datetime(
            resource,
            &["authoredOn", "effectiveDateTime", "effectivePeriod"],
        ),
        "AllergyIntolerance" => {
            extract_datetime(resource, &["recordedDate", "onsetDateTime", "onsetDate"])
        }
        "Procedure" => extract_datetime(resource, &["performedDateTime", "performedPeriod"]),
        "Encounter" => extract_datetime(resource, &["period"]),
        "DocumentReference" | "Composition" => extract_datetime(resource, &["date", "created"]),
        _ => extract_datetime(resource, &["effectiveDateTime", "issued", "date"]),
    }
}

fn extract_patient_name(resource: &Value) -> Option<String> {
    let names = resource.get("name")?.as_array()?;
    let name = names.first()?;
    let given = name
        .get("given")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(Value::as_str)
        .unwrap_or("");
    let family = name.get("family").and_then(Value::as_str).unwrap_or("");
    let full = format!("{given} {family}").trim().to_string();
    if full.is_empty() {
        None
    } else {
        Some(full)
    }
}

fn extract_patient_age(resource: &Value) -> Option<i32> {
    let birth_date = resource
        .get("birthDate")
        .and_then(Value::as_str)
        .and_then(parse_date)?;
    let today = Utc::now().date_naive();
    let mut age = today.year() - birth_date.year();

    let has_had_birthday = if (today.month(), today.day()) >= (birth_date.month(), birth_date.day())
    {
        true
    } else {
        false
    };

    if !has_had_birthday {
        age -= 1;
    }

    if age >= 0 {
        Some(age)
    } else {
        None
    }
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn extract_codeable_text(value: &Value) -> Option<String> {
    if let Some(text) = value.get("text").and_then(Value::as_str) {
        if !text.trim().is_empty() {
            return Some(text.trim().to_string());
        }
    }

    if let Some(codings) = value.get("coding").and_then(Value::as_array) {
        for coding in codings {
            if let Some(display) = coding.get("display").and_then(Value::as_str) {
                if !display.trim().is_empty() {
                    return Some(display.trim().to_string());
                }
            }
            if let Some(code) = coding.get("code").and_then(Value::as_str) {
                if !code.trim().is_empty() {
                    return Some(code.trim().to_string());
                }
            }
        }
    }

    None
}

fn resource_id(resource: &Value, fallback: &str) -> String {
    resource
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| format!("{fallback}-unknown"))
}

fn make_reference(resource: &Value) -> Option<ResourceReference> {
    let resource_type = resource.get("resourceType").and_then(Value::as_str)?;
    let id = resource.get("id").and_then(Value::as_str)?.to_string();
    Some(ResourceReference {
        system: Some("FHIR".to_string()),
        reference: Some(format!("{resource_type}/{id}")),
        display: resource.get("code").and_then(extract_codeable_text),
    })
}

fn capitalize_first(input: &str) -> String {
    let mut chars = input.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn summarize_reactions(resource: &Value) -> Option<String> {
    let reactions = resource.get("reaction")?.as_array()?;
    let mut parts = Vec::new();
    for reaction in reactions {
        if let Some(manifestations) = reaction.get("manifestation").and_then(Value::as_array) {
            for manifestation in manifestations {
                if let Some(text) = extract_codeable_text(manifestation) {
                    parts.push(text);
                }
            }
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

fn map_allergy_severity(resource: &Value) -> Severity {
    if let Some(severity) = resource.get("criticality").and_then(Value::as_str) {
        return match severity {
            "high" | "unable-to-assess" => Severity::Critical,
            "low" => Severity::Moderate,
            _ => Severity::Moderate,
        };
    }

    if let Some(reactions) = resource.get("reaction").and_then(Value::as_array) {
        for reaction in reactions {
            if let Some(severity) = reaction.get("severity").and_then(Value::as_str) {
                return match severity {
                    "severe" => Severity::Critical,
                    "moderate" => Severity::High,
                    "mild" => Severity::Moderate,
                    _ => Severity::Moderate,
                };
            }
        }
    }

    Severity::Moderate
}

fn summarize_dosage(resource: &Value) -> Option<String> {
    let dosage = resource.get("dosage")?.as_array()?.first()?;
    let mut parts = Vec::new();

    if let Some(text) = dosage.get("text").and_then(Value::as_str) {
        parts.push(text.trim().to_string());
    }

    if let Some(route) = dosage
        .get("route")
        .and_then(extract_codeable_text)
        .filter(|s| !s.is_empty())
    {
        parts.push(format!("Đường dùng: {route}"));
    }

    if let Some(rate) = dosage.get("rateQuantity").and_then(format_quantity_value) {
        parts.push(format!("Tốc độ: {rate}"));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" | "))
    }
}

fn format_quantity_value(value: &Value) -> Option<String> {
    let magnitude = value.get("value")?.as_f64()?;
    let unit = value.get("unit").and_then(Value::as_str).unwrap_or("");
    let number = format_numeric(magnitude);
    if unit.is_empty() {
        Some(number)
    } else {
        Some(format!("{number} {unit}"))
    }
}

fn format_numeric(value: f64) -> String {
    if (value.fract() - 0.0).abs() < f64::EPSILON {
        format!("{value:.0}")
    } else if (value * 10.0).fract().abs() < f64::EPSILON {
        format!("{value:.1}")
    } else {
        format!("{value}")
    }
}

fn extract_datetime(resource: &Value, fields: &[&str]) -> Option<DateTime<Utc>> {
    for field in fields {
        let Some(value) = resource.get(*field) else {
            continue;
        };

        if let Some(text) = value.as_str() {
            if let Some(dt) = parse_datetime(text) {
                return Some(dt);
            }
        }

        if let Some(obj) = value.as_object() {
            if let Some(end) = obj.get("end").and_then(Value::as_str) {
                if let Some(dt) = parse_datetime(end) {
                    return Some(dt);
                }
            }
            if let Some(start) = obj.get("start").and_then(Value::as_str) {
                if let Some(dt) = parse_datetime(start) {
                    return Some(dt);
                }
            }
        }
    }
    None
}

fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

fn map_condition_severity(condition: &str) -> Severity {
    let normalized = condition.to_lowercase();
    if normalized.contains("sepsis")
        || normalized.contains("shock")
        || normalized.contains("arrest")
        || normalized.contains("respiratory failure")
    {
        Severity::Critical
    } else if normalized.contains("pneumonia")
        || normalized.contains("infarction")
        || normalized.contains("stroke")
        || normalized.contains("pulmonary embolism")
    {
        Severity::High
    } else {
        Severity::Moderate
    }
}

fn extract_status_code(value: Option<&Value>) -> Option<String> {
    let value = value?;
    if let Some(text) = extract_codeable_text(value) {
        return Some(text);
    }
    value.as_str().map(|s| s.to_string())
}

fn observation_is_code_status(resource: &Value) -> bool {
    let check_text = |text: &str| {
        let lower = text.to_lowercase();
        lower.contains("code status")
            || lower.contains("dnr")
            || lower.contains("do not resuscitate")
            || lower.contains("resuscitation status")
            || lower.contains("advance directive")
    };

    if let Some(text) = resource.get("code").and_then(extract_codeable_text) {
        if check_text(&text) {
            return true;
        }
    }

    false
}

fn observation_value_text(resource: &Value) -> Option<String> {
    if let Some(value) = resource.get("valueCodeableConcept") {
        return extract_codeable_text(value);
    }
    if let Some(value) = resource.get("valueString").and_then(Value::as_str) {
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

fn summarize_observation_value(resource: &Value) -> Option<String> {
    if let Some(quantity) = resource.get("valueQuantity") {
        return format_quantity_value(quantity);
    }

    if let Some(value_string) = resource.get("valueString").and_then(Value::as_str) {
        if !value_string.is_empty() {
            return Some(value_string.to_string());
        }
    }

    if let Some(value_concept) = resource.get("valueCodeableConcept") {
        if let Some(text) = extract_codeable_text(value_concept) {
            return Some(text);
        }
    }

    if let Some(components) = resource.get("component").and_then(Value::as_array) {
        if let Some(bp) = summarize_blood_pressure(components) {
            return Some(bp);
        }

        let mut parts = Vec::new();
        for component in components {
            let label = component
                .get("code")
                .and_then(extract_codeable_text)
                .unwrap_or_else(|| "Thành phần".to_string());
            if let Some(quantity) = component.get("valueQuantity") {
                if let Some(value) = format_quantity_value(quantity) {
                    parts.push(format!("{label}: {value}"));
                }
            }
        }
        if !parts.is_empty() {
            return Some(parts.join(" | "));
        }
    }

    None
}

fn summarize_blood_pressure(components: &[Value]) -> Option<String> {
    let mut systolic: Option<String> = None;
    let mut diastolic: Option<String> = None;
    let mut unit: Option<String> = None;

    for component in components {
        let label = component
            .get("code")
            .and_then(extract_codeable_text)
            .unwrap_or_else(|| "".to_string())
            .to_lowercase();

        if let Some(quantity) = component.get("valueQuantity") {
            if systolic.is_none() && label.contains("systolic") {
                if let Some(value) = format_quantity_value(quantity) {
                    unit = quantity
                        .get("unit")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    systolic = Some(value.split_whitespace().next().unwrap_or("").to_string());
                }
            }

            if diastolic.is_none() && label.contains("diastolic") {
                if let Some(value) = format_quantity_value(quantity) {
                    unit = quantity
                        .get("unit")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    diastolic = Some(value.split_whitespace().next().unwrap_or("").to_string());
                }
            }
        }
    }

    match (systolic, diastolic) {
        (Some(sys), Some(dia)) => {
            let unit = unit.unwrap_or_else(|| "mmHg".to_string());
            Some(format!("{sys}/{dia} {unit}"))
        }
        _ => None,
    }
}

fn extract_observation_timestamp(resource: &Value) -> Option<DateTime<Utc>> {
    extract_datetime(
        resource,
        &[
            "effectiveDateTime",
            "effectiveInstant",
            "effectivePeriod",
            "issued",
        ],
    )
}

fn classify_observation(name: &str, resource: &Value, detail: &str) -> Severity {
    let normalized = name.to_lowercase();

    if let Some(severity) = severity_from_interpretation(resource) {
        return severity;
    }

    if normalized.contains("heart rate") || normalized.contains("pulse") {
        if let Some(value) = parse_value_quantity(resource) {
            return match value {
                v if v >= 140.0 => Severity::Critical,
                v if v >= 120.0 => Severity::High,
                v if v <= 40.0 => Severity::Critical,
                v if v <= 50.0 => Severity::High,
                _ => Severity::Moderate,
            };
        }
    }

    if normalized.contains("respiratory rate") {
        if let Some(value) = parse_value_quantity(resource) {
            return match value {
                v if v >= 35.0 => Severity::Critical,
                v if v >= 28.0 => Severity::High,
                v if v <= 8.0 => Severity::Critical,
                v if v <= 10.0 => Severity::High,
                _ => Severity::Moderate,
            };
        }
    }

    if normalized.contains("spo2") || normalized.contains("oxygen saturation") {
        if let Some(value) = parse_value_quantity(resource) {
            return match value {
                v if v < 85.0 => Severity::Critical,
                v if v < 92.0 => Severity::High,
                _ => Severity::Moderate,
            };
        }
    }

    if normalized.contains("blood pressure") {
        if let Some((sys, dia)) = parse_blood_pressure_from_detail(detail) {
            if sys >= 200 || dia >= 120 {
                return Severity::Critical;
            }
            if sys >= 180 || dia >= 110 {
                return Severity::High;
            }
            if sys <= 80 || dia <= 50 {
                return Severity::High;
            }
            return Severity::Moderate;
        }
    }

    if normalized.contains("lactate") {
        if let Some(value) = parse_value_quantity(resource) {
            return match value {
                v if v >= 4.0 => Severity::Critical,
                v if v >= 2.0 => Severity::High,
                _ => Severity::Moderate,
            };
        }
    }

    Severity::Info
}

fn severity_from_interpretation(resource: &Value) -> Option<Severity> {
    let interpretation = resource.get("interpretation")?.as_array()?;
    for entry in interpretation {
        if let Some(code) = extract_codeable_text(entry) {
            let lower = code.to_lowercase();
            if lower.contains("critical") || lower == "hh" || lower == "ll" {
                return Some(Severity::Critical);
            }
            if lower == "h" || lower == "l" || lower.contains("abnormal") {
                return Some(Severity::High);
            }
        }
    }
    None
}

fn parse_value_quantity(resource: &Value) -> Option<f64> {
    if let Some(quantity) = resource.get("valueQuantity") {
        return quantity.get("value").and_then(Value::as_f64);
    }
    None
}

fn parse_blood_pressure_from_detail(detail: &str) -> Option<(i32, i32)> {
    let parts: Vec<&str> = detail.split_whitespace().collect();
    for part in parts {
        if let Some((sys, dia)) = part.split_once('/') {
            let sys_val = sys.parse::<i32>().ok()?;
            let dia_val = dia.parse::<i32>().ok()?;
            return Some((sys_val, dia_val));
        }
    }
    None
}

fn infer_vital_label(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    if lower.contains("heart rate") || lower.contains("pulse") {
        Some("Mạch")
    } else if lower.contains("spo2") || lower.contains("oxygen saturation") {
        Some("SpO2")
    } else if lower.contains("blood pressure") {
        Some("HA")
    } else if lower.contains("respiratory rate") {
        Some("Nhịp thở")
    } else if lower.contains("temperature") {
        Some("Nhiệt độ")
    } else {
        None
    }
}

fn is_recent_vital(
    anchor: Option<DateTime<Utc>>,
    recorded_at: Option<DateTime<Utc>>,
    window_hours: u32,
) -> bool {
    let Some(anchor) = anchor else {
        return true;
    };
    let Some(recorded_at) = recorded_at else {
        return true;
    };

    let threshold = window_hours as i64;
    let delta_hours = anchor.signed_duration_since(recorded_at).num_hours();
    delta_hours.abs() <= threshold
}

fn is_recent_event(
    anchor: Option<DateTime<Utc>>,
    recorded_at: Option<DateTime<Utc>>,
    window_days: u32,
) -> bool {
    let Some(anchor) = anchor else {
        return true;
    };
    let Some(recorded_at) = recorded_at else {
        return true;
    };
    let threshold = window_days as i64;
    let delta_days = anchor.signed_duration_since(recorded_at).num_days();
    delta_days.abs() <= threshold
}

fn is_more_recent(candidate: Option<DateTime<Utc>>, current: Option<DateTime<Utc>>) -> bool {
    match (candidate, current) {
        (Some(a), Some(b)) => a > b,
        (Some(_), None) => true,
        _ => false,
    }
}
