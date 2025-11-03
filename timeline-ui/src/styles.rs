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
  --timeline-surface: #f8fafc;
  --timeline-critical-bg: #fff7ec;
  --timeline-critical-border: #f7c06c;
  --timeline-critical-text: #8b3700;
  --timeline-card-bg: #ffffff;
  --timeline-card-border: rgba(148, 163, 184, 0.3);
  --timeline-muted-strong: #3f4c5a;
  --timeline-severity-critical: #b42318;
  --timeline-severity-critical-bg: rgba(180, 35, 24, 0.1);
  --timeline-severity-high: #dc6803;
  --timeline-severity-high-bg: rgba(220, 104, 3, 0.12);
  --timeline-severity-moderate: #067647;
  --timeline-severity-moderate-bg: rgba(6, 118, 71, 0.12);
  --timeline-severity-low: #0b5394;
  --timeline-severity-low-bg: rgba(11, 83, 148, 0.12);
  --timeline-severity-info: #475467;
  --timeline-severity-info-bg: rgba(71, 84, 103, 0.12);
}

.timeline-root {
  font-family: var(--timeline-font-family);
  background: var(--timeline-bg);
  color: var(--timeline-text);
  border-radius: var(--timeline-radius);
  padding: 24px;
  display: grid;
  gap: 24px;
  grid-template-columns: minmax(280px, 0.9fr) minmax(360px, 1.4fr);
  box-shadow: 0 24px 48px rgba(15, 23, 42, 0.1);
}

.critical-column {
  display: flex;
  flex-direction: column;
  gap: 16px;
  position: sticky;
  top: 16px;
  align-self: start;
}

.critical-header {
  background: linear-gradient(120deg, rgba(248, 204, 84, 0.18), rgba(255, 247, 236, 0.4));
  border: 1px solid rgba(247, 192, 108, 0.5);
  border-radius: calc(var(--timeline-radius) - 6px);
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.critical-header h2 {
  margin: 0;
  font-size: 1.2rem;
  color: var(--timeline-critical-text);
}

.critical-header p {
  margin: 0;
  color: var(--timeline-muted);
  font-size: 0.92rem;
}

.critical-eyebrow {
  text-transform: uppercase;
  letter-spacing: 0.12em;
  font-size: 0.7rem;
  color: var(--timeline-critical-text);
  font-weight: 600;
}

.critical-card {
  background: var(--timeline-card-bg);
  border: 1px solid var(--timeline-card-border);
  border-radius: calc(var(--timeline-radius) - 6px);
  padding: 16px;
  box-shadow: 0 12px 28px rgba(15, 23, 42, 0.04);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.critical-card header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.critical-card header h3 {
  margin: 0;
  font-size: 1rem;
  color: var(--timeline-heading);
}

.critical-count {
  font-size: 0.85rem;
  color: var(--timeline-muted);
  background: rgba(71, 84, 103, 0.08);
  border-radius: 999px;
  padding: 2px 10px;
}

.critical-card ul {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.critical-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.critical-item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.critical-label {
  font-weight: 600;
  color: var(--timeline-muted-strong);
}

.critical-detail {
  margin: 0;
  color: var(--timeline-muted);
  font-size: 0.9rem;
  line-height: 1.4;
}

.critical-empty {
  color: var(--timeline-muted);
  font-size: 0.9rem;
  font-style: italic;
}

.critical-pill {
  font-size: 0.72rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  padding: 4px 10px;
  border-radius: 999px;
  background: rgba(71, 84, 103, 0.1);
  color: var(--timeline-muted-strong);
}

.code-status .critical-pill {
  background: var(--timeline-severity-critical-bg);
  color: var(--timeline-severity-critical);
}

.code-status-value {
  margin: 0;
  font-weight: 600;
  font-size: 1.05rem;
  color: var(--timeline-severity-info);
}

.code-status-value.critical {
  color: var(--timeline-severity-critical);
}

.vital-list {
  display: grid;
  gap: 8px;
}

.vital-item {
  display: grid;
  grid-template-columns: 100px 1fr auto;
  gap: 8px;
  font-size: 0.92rem;
  color: var(--timeline-muted-strong);
}

.vital-name {
  font-weight: 600;
}

.vital-value {
  color: var(--timeline-heading);
}

.vital-time {
  font-size: 0.82rem;
  color: var(--timeline-muted);
}

.timeline-column {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.timeline-toolbar {
  background: var(--timeline-surface);
  border: 1px solid rgba(148, 163, 184, 0.3);
  border-radius: calc(var(--timeline-radius) - 8px);
  padding: 16px;
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
}

.toolbar-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toolbar-label {
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--timeline-muted);
  font-weight: 600;
}

.filter-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.filter-chip {
  border: 1px solid rgba(148, 163, 184, 0.5);
  background: #ffffff;
  border-radius: 999px;
  padding: 6px 14px;
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--timeline-muted-strong);
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease, border 120ms ease;
}

.filter-chip:hover {
  border-color: var(--timeline-severity-high);
}

.filter-chip.is-active[data-level="critical"] {
  background: var(--timeline-severity-critical-bg);
  color: var(--timeline-severity-critical);
  border-color: transparent;
}

.filter-chip.is-active[data-level="high"] {
  background: var(--timeline-severity-high-bg);
  color: var(--timeline-severity-high);
  border-color: transparent;
}

.filter-chip.is-active[data-level="moderate"] {
  background: var(--timeline-severity-moderate-bg);
  color: var(--timeline-severity-moderate);
  border-color: transparent;
}

.toolbar-search {
  display: flex;
  align-items: center;
  gap: 10px;
}

.toolbar-search input {
  border: 1px solid rgba(148, 163, 184, 0.5);
  border-radius: 10px;
  padding: 8px 12px;
  min-width: 220px;
  font-size: 0.9rem;
}

.toolbar-search input:focus {
  outline: 2px solid rgba(59, 130, 246, 0.3);
  border-color: rgba(59, 130, 246, 0.5);
}

.toolbar-search button {
  border: none;
  background: rgba(59, 130, 246, 0.12);
  color: #1d4ed8;
  border-radius: 8px;
  padding: 8px 12px;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
}

.toolbar-search button:hover {
  background: rgba(59, 130, 246, 0.2);
}

.timeline-events {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.timeline-event {
  border-left: 4px solid rgba(148, 163, 184, 0.4);
  padding-left: 16px;
  position: relative;
  background: rgba(248, 250, 252, 0.7);
  border-radius: calc(var(--timeline-radius) - 10px);
  padding-top: 12px;
  padding-bottom: 12px;
}

.timeline-event::before {
  content: "";
  position: absolute;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #ffffff;
  border: 2px solid rgba(148, 163, 184, 0.5);
  left: -8px;
  top: 16px;
}

.timeline-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  font-size: 0.85rem;
  color: var(--timeline-muted);
  align-items: center;
  margin-bottom: 8px;
}

.timeline-relative {
  color: var(--timeline-muted);
  font-style: italic;
}

.timeline-category {
  text-transform: uppercase;
  letter-spacing: 0.08em;
  font-weight: 600;
  font-size: 0.72rem;
  background: rgba(71, 84, 103, 0.16);
  border-radius: 999px;
  padding: 4px 10px;
}

.timeline-severity {
  text-transform: uppercase;
  font-weight: 700;
  letter-spacing: 0.08em;
  font-size: 0.72rem;
}

.timeline-severity[data-level="critical"],
.severity-badge[data-level="critical"] {
  color: var(--timeline-severity-critical);
  background: var(--timeline-severity-critical-bg);
  border-radius: 999px;
  padding: 2px 8px;
}

.timeline-severity[data-level="high"],
.severity-badge[data-level="high"] {
  color: var(--timeline-severity-high);
  background: var(--timeline-severity-high-bg);
  border-radius: 999px;
  padding: 2px 8px;
}

.timeline-severity[data-level="moderate"],
.severity-badge[data-level="moderate"] {
  color: var(--timeline-severity-moderate);
  background: var(--timeline-severity-moderate-bg);
  border-radius: 999px;
  padding: 2px 8px;
}

.timeline-severity[data-level="low"],
.severity-badge[data-level="low"] {
  color: var(--timeline-severity-low);
  background: var(--timeline-severity-low-bg);
  border-radius: 999px;
  padding: 2px 8px;
}

.timeline-severity[data-level="info"],
.severity-badge[data-level="info"] {
  color: var(--timeline-severity-info);
  background: var(--timeline-severity-info-bg);
  border-radius: 999px;
  padding: 2px 8px;
}

.timeline-title {
  margin: 0 0 6px 0;
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--timeline-heading);
}

.timeline-detail {
  margin: 0;
  color: var(--timeline-muted-strong);
  font-size: 0.95rem;
  line-height: 1.5;
}

.timeline-empty {
  background: rgba(248, 250, 252, 0.9);
  border: 1px dashed rgba(148, 163, 184, 0.5);
  border-radius: calc(var(--timeline-radius) - 10px);
  padding: 20px;
  text-align: center;
  color: var(--timeline-muted);
  font-style: italic;
}

@media (max-width: 1080px) {
  .timeline-root {
    grid-template-columns: 1fr;
  }

  .critical-column {
    position: static;
  }

  .timeline-toolbar {
    flex-direction: column;
    align-items: stretch;
  }

  .toolbar-search {
    justify-content: space-between;
  }

  .toolbar-search input {
    flex: 1;
    min-width: 0;
  }
}

@media (max-width: 640px) {
  .timeline-root {
    padding: 16px;
  }

  .vital-item {
    grid-template-columns: 1fr;
    gap: 4px;
  }

  .timeline-event {
    padding-left: 12px;
  }

  .timeline-event::before {
    left: -7px;
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
    style_el.set_attribute("data-timeline-ui", "v2")?;
    style_el.set_text_content(Some(DEFAULT_STYLES));
    head.append_child(&style_el.clone().dyn_into::<Node>()?)?;

    Ok(())
}
