# Schema dữ liệu đầu ra

Tài liệu này mô tả cấu trúc JSON mà thư viện trả về sau khi tóm tắt bundle FHIR. Mục tiêu là giúp đội frontend tích hợp dễ dàng và bảo đảm tính tương thích giữa các framework.

## Tổng quan
- `TimelineSnapshot`: đối tượng gốc chứa dấu thời gian sinh, bảng “Critical Overview” và danh sách sự kiện timeline.
- `CriticalSummary`: nhóm thông tin cần hiển thị ngay cho bác sĩ (dị ứng, thuốc, cảnh báo, chỉ số sống).
- `TimelineEvent`: sự kiện chuẩn hóa theo thời gian, có phân loại (encounter, procedure, observation...).
- `ResourceReference`: liên kết ngược tới resource FHIR (nếu cần mở chi tiết).

## TypeScript definitions
Các định nghĩa TypeScript tham khảo được duy trì trong `timeline-wasm/types/index.d.ts`. Phiên bản hiện tại:

```ts
export type Severity = "critical" | "high" | "moderate" | "low" | "info";

export type EventCategory =
  | "Encounter"
  | "Procedure"
  | "Condition"
  | "Medication"
  | "Observation"
  | "Document"
  | "Note"
  | "Other";

export interface CriticalItem {
  label: string;
  detail?: string | null;
  severity: Severity;
}

export interface VitalSnapshot {
  name: string;
  value: string;
  recorded_at?: string | null; // ISO 8601 UTC
}

export interface ResourceReference {
  system?: string | null;
  reference?: string | null;
  display?: string | null;
}

export interface TimelineEvent {
  id: string;
  category: EventCategory;
  title: string;
  detail?: string | null;
  occurred_at?: string | null;
  severity: Severity;
  source?: ResourceReference | null;
}

export interface CriticalSummary {
  allergies: CriticalItem[];
  medications: CriticalItem[];
  chronic_conditions: CriticalItem[];
  code_status?: string | null;
  alerts: CriticalItem[];
  recent_vitals: VitalSnapshot[];
}

export interface TimelineSnapshot {
  generated_at: string; // ISO 8601 UTC
  critical: CriticalSummary;
  events: TimelineEvent[];
}
```

## JSON Schema
File JSON Schema chính thức: `docs/schema/timeline_snapshot.schema.json`.

Schema này có thể dùng để validate đầu ra ở runtime (ví dụ trong automated tests) hoặc tạo typings tự động cho các ngôn ngữ khác.
