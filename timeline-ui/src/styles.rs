#![cfg(target_arch = "wasm32")]

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Node};

const STYLE_TAG_SELECTOR: &str = "style[data-timeline-ui]";

/// CSS mặc định cho component kèm hệ thống biến dễ override.
pub const DEFAULT_STYLES: &str = r#"
:root {
  --timeline-font-family: 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --timeline-bg: #ffffff;
  --timeline-border: #d0d7de;
  --timeline-radius: 14px;
  --timeline-text: #1f2933;
  --timeline-muted: #52606d;
  --timeline-heading: #11181c;
  --timeline-critical-bg: #fff4de;
  --timeline-critical-border: #f7c06c;
  --timeline-critical-text: #8b3700;
  --timeline-severity-critical: #b42318;
  --timeline-severity-high: #dc6803;
  --timeline-severity-moderate: #067647;
  --timeline-severity-low: #0b5394;
  --timeline-severity-info: #475467;
}

.timeline-root {
  font-family: var(--timeline-font-family);
  background: var(--timeline-bg);
  color: var(--timeline-text);
  border: 1px solid var(--timeline-border);
  border-radius: var(--timeline-radius);
  padding: 20px;
  display: grid;
  gap: 20px;
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.08);
}

.timeline-root h2 {
  margin: 0 0 12px 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--timeline-heading);
}

.critical-panel {
  background: var(--timeline-critical-bg);
  border: 1px solid var(--timeline-critical-border);
  border-radius: calc(var(--timeline-radius) - 4px);
  padding: 16px;
}

.critical-panel h2 {
  color: var(--timeline-critical-text);
}

.critical-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 12px;
}

.critical-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.critical-label {
  font-weight: 600;
  color: var(--timeline-critical-text);
}

.critical-detail {
  color: var(--timeline-muted);
  font-size: 0.9rem;
}

.timeline-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.timeline-events {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 16px;
}

.timeline-event {
  border-left: 3px solid var(--timeline-border);
  padding-left: 16px;
  position: relative;
}

.timeline-event::before {
  content: "";
  position: absolute;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--timeline-border);
  left: -6px;
  top: 6px;
}

.timeline-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 0.85rem;
  color: var(--timeline-muted);
}

.timeline-severity {
  text-transform: uppercase;
  font-weight: 600;
  letter-spacing: 0.05em;
}

.timeline-severity[data-level="critical"] {
  color: var(--timeline-severity-critical);
}

.timeline-severity[data-level="high"] {
  color: var(--timeline-severity-high);
}

.timeline-severity[data-level="moderate"] {
  color: var(--timeline-severity-moderate);
}

.timeline-severity[data-level="low"] {
  color: var(--timeline-severity-low);
}

.timeline-severity[data-level="info"] {
  color: var(--timeline-severity-info);
}

.timeline-title {
  margin: 8px 0 4px 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--timeline-heading);
}

.timeline-detail {
  margin: 0;
  color: var(--timeline-muted);
  font-size: 0.95rem;
}

@media (max-width: 640px) {
  .timeline-root {
    padding: 16px;
  }

  .timeline-meta {
    flex-direction: column;
    align-items: flex-start;
  }
}
"#;

pub fn ensure_styles(document: &Document) -> Result<(), JsValue> {
    if document.query_selector(STYLE_TAG_SELECTOR)?.is_some() {
        return Ok(());
    }

    let head = document
        .head()
        .ok_or_else(|| JsValue::from_str("Document không có thẻ <head>"))?;

    let style_el = document.create_element("style")?;
    style_el.set_attribute("data-timeline-ui", "v1")?;
    style_el.set_text_content(Some(DEFAULT_STYLES));
    head.append_child(&style_el.clone().dyn_into::<Node>()?)?;

    Ok(())
}
