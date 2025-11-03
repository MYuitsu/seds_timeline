//! Bridge WASM <-> JavaScript trung lập framework.

use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use timeline_core::{TimelineConfig, TimelineError};
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
struct JsTimelineConfig {
    #[serde(default)]
    vital_recent_hours: Option<u32>,
    #[serde(default)]
    clinical_event_days: Option<u32>,
}

impl From<JsTimelineConfig> for TimelineConfig {
    fn from(cfg: JsTimelineConfig) -> Self {
        let mut base = TimelineConfig::default();
        if let Some(hours) = cfg.vital_recent_hours {
            base.vital_recent_hours = hours;
        }
        if let Some(days) = cfg.clinical_event_days {
            base.clinical_event_days = days;
        }
        base
    }
}

#[wasm_bindgen]
pub fn summarize_bundle(
    input_bundle: JsValue,
    config: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let bundle_value = from_value::<serde_json::Value>(input_bundle)
        .map_err(|err| JsValue::from_str(&format!("Không đọc được JSON bundle: {err}")))?;

    let cfg = match config {
        Some(js_cfg) => {
            let cfg: JsTimelineConfig = from_value(js_cfg)
                .map_err(|err| JsValue::from_str(&format!("Không đọc được config: {err}")))?;
            TimelineConfig::from(cfg)
        }
        None => TimelineConfig::default(),
    };

    let snapshot = timeline_fhir::summarize_bundle_value(&bundle_value, &cfg)
        .map_err(|err| JsValue::from_str(&format_timeline_error(err)))?;

    to_value(&snapshot)
        .map_err(|err| JsValue::from_str(&format!("Không serialize snapshot: {err}")))
}

fn format_timeline_error(err: TimelineError) -> String {
    format!("Timeline error: {err}")
}
