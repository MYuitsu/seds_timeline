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
  recorded_at?: string | null;
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
  generated_at: string;
  critical: CriticalSummary;
  events: TimelineEvent[];
}

export interface SummarizeConfig {
  vital_recent_hours?: number;
  clinical_event_days?: number;
}

export function summarize_bundle(
  bundle: unknown,
  config?: SummarizeConfig
): TimelineSnapshot;
