//! Logic lõi xây dựng timeline và bảng thông tin quan trọng.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Cấu hình điều chỉnh thứ tự ưu tiên và các ngưỡng.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineConfig {
    /// Khoảng thời gian (giờ) coi là "gần đây" cho các chỉ số sống.
    pub vital_recent_hours: u32,
    /// Khoảng thời gian (ngày) coi là sự kiện lâm sàng đáng chú ý.
    pub clinical_event_days: u32,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            vital_recent_hours: 6,
            clinical_event_days: 30,
        }
    }
}

/// Mức độ ưu tiên hiển thị trên timeline.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    High,
    Moderate,
    Low,
    Info,
}

/// Thông tin quan trọng cần hiển thị tức thời.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VitalTrend {
    pub name: String,
    pub unit: Option<String>,
    pub points: Vec<VitalTrendPoint>,
}

impl Default for VitalTrend {
    fn default() -> Self {
        Self {
            name: String::new(),
            unit: None,
            points: Vec::new(),
        }
    }
}

/// Một điểm dữ liệu trong biểu đồ chỉ số sống.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VitalTrendPoint {
    pub recorded_at: Option<DateTime<Utc>>,
    pub value: Option<f64>,
    pub label: Option<String>,
}

impl Default for VitalTrendPoint {
    fn default() -> Self {
        Self {
            recorded_at: None,
            value: None,
            label: None,
        }
    }
}

/// Kết quả xét nghiệm hoặc chẩn đoán hình ảnh gần nhất.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticSnapshot {
    pub name: String,
    pub value: String,
    pub recorded_at: Option<DateTime<Utc>>,
    pub severity: Severity,
    pub kind: DiagnosticKind,
    pub unit: Option<String>,
}

impl Default for DiagnosticSnapshot {
    fn default() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
            recorded_at: None,
            severity: Severity::Info,
            kind: DiagnosticKind::Lab,
            unit: None,
        }
    }
}

/// Phân loại dữ liệu chẩn đoán.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
    Lab,
    Imaging,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CriticalSummary {
    pub allergies: Vec<CriticalItem>,
    pub medications: Vec<CriticalItem>,
    pub chronic_conditions: Vec<CriticalItem>,
    pub code_status: Option<String>,
    pub alerts: Vec<CriticalItem>,
    pub recent_vitals: Vec<VitalSnapshot>,
    #[serde(default)]
    pub vital_trends: Vec<VitalTrend>,
    #[serde(default)]
    pub recent_diagnostics: Vec<DiagnosticSnapshot>,
}

/// Mục thông tin trọng yếu (dị ứng, thuốc, cảnh báo).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CriticalItem {
    pub label: String,
    pub detail: Option<String>,
    pub severity: Severity,
}

/// Ảnh chụp chỉ số sống.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VitalSnapshot {
    pub name: String,
    pub value: String,
    pub recorded_at: Option<DateTime<Utc>>,
    pub numeric_value: Option<f64>,
    pub unit: Option<String>,
}

/// Một sự kiện trong timeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineEvent {
    pub id: String,
    pub category: EventCategory,
    pub title: String,
    pub detail: Option<String>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub severity: Severity,
    pub source: Option<ResourceReference>,
}

/// Nhãn phân loại để trình bày timeline.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventCategory {
    Encounter,
    Procedure,
    Condition,
    Medication,
    Observation,
    Document,
    Note,
    Other,
}

/// Liên kết ngược tới resource gốc (FHIR reference, URL...).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceReference {
    pub system: Option<String>,
    pub reference: Option<String>,
    pub display: Option<String>,
}

/// Kết quả tổng hợp cuối cùng.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineSnapshot {
    pub generated_at: DateTime<Utc>,
    pub critical: CriticalSummary,
    pub events: Vec<TimelineEvent>,
}

impl TimelineSnapshot {
    /// Khởi tạo snapshot từ các thành phần đã chuẩn bị.
    pub fn new(critical: CriticalSummary, mut events: Vec<TimelineEvent>) -> Self {
        events.sort_by_key(|event| event.occurred_at);
        Self {
            generated_at: Utc::now(),
            critical,
            events,
        }
    }

    /// Truy cập bảng thông tin trọng yếu.
    pub fn critical_panel(&self) -> &CriticalSummary {
        &self.critical
    }

    /// Danh sách sự kiện đã sắp xếp theo thời gian.
    pub fn timeline(&self) -> &[TimelineEvent] {
        &self.events
    }
}

/// Lỗi chung khi tạo timeline.
#[derive(Debug, thiserror::Error)]
pub enum TimelineError {
    #[error("Dữ liệu đầu vào thiếu thông tin tối thiểu")]
    MissingData,
    #[error("Không đọc được dữ liệu: {0}")]
    Parse(String),
    #[error("Lỗi khác: {0}")]
    Other(String),
}

/// Tiện ích dựng snapshot rỗng (dùng cho mock/testing).
pub fn empty_snapshot() -> TimelineSnapshot {
    TimelineSnapshot {
        generated_at: Utc::now(),
        critical: CriticalSummary::default(),
        events: Vec::new(),
    }
}
