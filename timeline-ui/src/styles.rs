#![cfg(target_arch = "wasm32")]

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Node};

const STYLE_TAG_SELECTOR: &str = "style[data-timeline-ui]";

/// Default CSS for the component along with easy-to-override design tokens.
pub const DEFAULT_STYLES: &str = r#"
:root {
  --timeline-font-family: 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --timeline-bg: #ffffff;
  --timeline-card-bg: #ffffff;
  --timeline-card-border: rgba(148, 163, 184, 0.28);
  --timeline-radius: 16px;
  --timeline-text: #1f2933;
  --timeline-muted: #52606d;
  --timeline-muted-strong: #3f4c5a;
  --timeline-heading: #11181c;
  --timeline-surface: #f8fafc;
  --timeline-critical-text: #8b3700;
  --timeline-pill-affirm-bg: rgba(16, 185, 129, 0.14);
  --timeline-pill-affirm-text: #047857;
  --timeline-pill-warning-bg: rgba(220, 104, 3, 0.16);
  --timeline-pill-warning-text: #b54708;
  --timeline-hot-bg: rgba(255, 247, 236, 0.65);
  --timeline-group-accent: rgba(71, 84, 103, 0.18);
  --timeline-trend-border: rgba(148, 163, 184, 0.38);
  --timeline-trend-path: #2563eb;
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
  display: grid;
  gap: 26px;
  padding: 28px;
  grid-template-columns: minmax(300px, 0.9fr) minmax(420px, 1.6fr);
  box-shadow: 0 24px 48px rgba(15, 23, 42, 0.1);
}

.critical-column {
  display: flex;
  flex-direction: column;
  .timeline-group-chart {
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: rgba(255, 255, 255, 0.95);
    border: 1px dashed rgba(148, 163, 184, 0.35);
    border-radius: calc(var(--timeline-radius) - 18px);
    padding: 12px;
  }

  .timeline-group-stats {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .timeline-group-stat {
    font-size: 0.78rem;
    color: var(--timeline-muted-strong);
    background: rgba(71, 84, 103, 0.14);
    border-radius: 999px;
    padding: 4px 10px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .timeline-group-stat[data-kind="latest"] {
    background: rgba(59, 130, 246, 0.15);
    color: #1d4ed8;
  }

  .timeline-group-stat[data-kind="high"] {
    background: var(--timeline-severity-high-bg);
    color: var(--timeline-severity-high);
  }

  .timeline-group-stat[data-kind="low"] {
    background: var(--timeline-severity-low-bg);
    color: var(--timeline-severity-low);
  }

  .timeline-group-chart-plot {
    width: 100%;
    height: 64px;
    border-radius: 12px;
    border: 1px dashed var(--timeline-trend-border);
    background: #ffffff;
    color: var(--timeline-trend-path);
  }

  .timeline-group-chart-plot path {
    fill: none;
    stroke: currentColor;
    stroke-width: 2.2;
    stroke-linejoin: round;
    stroke-linecap: round;
  }

  .timeline-group-chart-plot circle {
    fill: currentColor;
  }

  .timeline-group-axis {
    display: flex;
    justify-content: space-between;
    font-size: 0.72rem;
    color: var(--timeline-muted);
    font-variant-numeric: tabular-nums;
  }

  .timeline-group-details {
    border-top: 1px dashed rgba(148, 163, 184, 0.28);
    padding-top: 8px;
  }

  .timeline-group-details summary {
    cursor: pointer;
    font-size: 0.8rem;
    color: var(--timeline-muted-strong);
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    list-style: none;
  }

  .timeline-group-details summary::-webkit-details-marker {
    display: none;
  }

  .timeline-group-details summary::marker {
    content: "";
  }

  .timeline-group-details summary::after {
    content: "+";
    font-size: 0.8rem;
    line-height: 1;
    transition: transform 120ms ease;
    color: inherit;
  }

  .timeline-group-details[open] summary::after {
    content: "-";
  }

  .timeline-group-details ul {
    margin-top: 10px;
  }

  gap: 18px;
  position: sticky;
  top: 18px;
  align-self: start;
}

.critical-header {
  background: linear-gradient(120deg, rgba(248, 204, 84, 0.22), rgba(255, 247, 236, 0.55));
  border: 1px solid rgba(247, 192, 108, 0.5);
  border-radius: calc(var(--timeline-radius) - 6px);
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.critical-eyebrow {
  text-transform: uppercase;
  letter-spacing: 0.12em;
  font-size: 0.72rem;
  color: var(--timeline-critical-text);
  font-weight: 600;
}

.critical-header h2 {
  margin: 0;
  font-size: 1.22rem;
  color: var(--timeline-critical-text);
}

.critical-header p {
  margin: 0;
  color: var(--timeline-muted);
  font-size: 0.93rem;
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
  line-height: 1.45;
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
  background: rgba(71, 84, 103, 0.12);
  color: var(--timeline-muted-strong);
}

.critical-pill[data-level="affirm"] {
  background: var(--timeline-pill-affirm-bg);
  color: var(--timeline-pill-affirm-text);
}

.critical-pill[data-level="warning"] {
  background: var(--timeline-pill-warning-bg);
  color: var(--timeline-pill-warning-text);
}

.code-status-value {
  margin: 0;
  font-weight: 600;
  font-size: 1.05rem;
  color: var(--timeline-heading);
}

.code-status-value[data-level="warning"] {
  color: var(--timeline-pill-warning-text);
}

.code-status-value[data-level="affirm"] {
  color: var(--timeline-pill-affirm-text);
}

.vital-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.vital-item {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  font-size: 0.92rem;
  color: var(--timeline-muted-strong);
}

.vital-text {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.vital-name {
  font-weight: 600;
}

.vital-value {
  color: var(--timeline-heading);
  font-weight: 600;
}

.vital-unit {
  font-size: 0.82rem;
  color: var(--timeline-muted);
}

.vital-meta {
  display: flex;
  gap: 10px;
  font-size: 0.82rem;
  color: var(--timeline-muted);
}

.vital-relative {
  font-style: italic;
}

.trend-card {
  padding-top: 18px;
}

.trend-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.insights-card {
  padding-top: 18px;
}

.insight-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.insight-item {
  border: 1px solid var(--timeline-trend-border);
  border-radius: 12px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  background: var(--timeline-surface);
  font-size: 0.9rem;
  color: var(--timeline-muted-strong);
}

.insight-item[data-trend="up"] .insight-change {
  color: var(--timeline-severity-high);
}

.insight-item[data-trend="down"] .insight-change {
  color: var(--timeline-severity-critical);
}

.insight-title {
  font-weight: 600;
  color: var(--timeline-heading);
}

.insight-change {
  margin: 0;
}

.insight-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  font-size: 0.78rem;
  color: var(--timeline-muted);
}

.insight-range::before {
  content: "";
  display: inline-block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--timeline-group-accent);
  margin-right: 6px;
}

.trend-item {
  border: 1px solid var(--timeline-trend-border);
  border-radius: 12px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  background: var(--timeline-surface);
}

.trend-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
}

.trend-name {
  font-weight: 600;
  color: var(--timeline-heading);
}

.trend-unit {
  font-size: 0.8rem;
  color: var(--timeline-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.trend-content {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.trend-chart {
  width: 100%;
  height: 48px;
  border-radius: 10px;
  border: 1px dashed var(--timeline-trend-border);
  background: #ffffff;
  color: var(--timeline-trend-path);
}

.trend-chart path {
  fill: none;
  stroke: currentColor;
  stroke-width: 2.2;
  stroke-linejoin: round;
  stroke-linecap: round;
}

.trend-chart circle {
  fill: currentColor;
}

.trend-fallback {
  font-size: 0.84rem;
  color: var(--timeline-muted);
  font-style: italic;
}

.trend-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 0.82rem;
  color: var(--timeline-muted);
}

.trend-latest {
  font-weight: 600;
  color: var(--timeline-heading);
}

.trend-delta[data-trend="up"] {
  color: var(--timeline-severity-high);
}

.trend-delta[data-trend="down"] {
  color: var(--timeline-severity-critical);
}

.trend-delta[data-trend="steady"] {
  color: var(--timeline-muted);
}

.hot-strip {
  background: var(--timeline-hot-bg);
  border: 1px solid rgba(247, 192, 108, 0.45);
  border-radius: calc(var(--timeline-radius) - 8px);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.hot-strip h3 {
  margin: 0;
  font-size: 0.95rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--timeline-critical-text);
}

.hot-strip ul {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.hot-item {
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(247, 192, 108, 0.35);
  border-radius: 12px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.hot-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
}

.hot-title {
  font-weight: 600;
  color: var(--timeline-heading);
}

.hot-severity {
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  border-radius: 999px;
  padding: 2px 8px;
}

.hot-severity[data-level="critical"] {
  background: var(--timeline-severity-critical-bg);
  color: var(--timeline-severity-critical);
}

.hot-severity[data-level="high"] {
  background: var(--timeline-severity-high-bg);
  color: var(--timeline-severity-high);
}

.hot-detail {
  margin: 0;
  color: var(--timeline-muted);
  font-size: 0.88rem;
  line-height: 1.4;
}

.hot-meta {
  display: flex;
  gap: 12px;
  font-size: 0.78rem;
  color: var(--timeline-muted);
  align-items: center;
}

.hot-category {
  text-transform: uppercase;
  letter-spacing: 0.12em;
}

.timeline-column {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.timeline-updated {
  margin: 0;
  font-size: 0.82rem;
  color: var(--timeline-muted);
}

.timeline-toolbar {
  background: var(--timeline-surface);
  border: 1px solid rgba(148, 163, 184, 0.3);
  border-radius: calc(var(--timeline-radius) - 8px);
  padding: 18px;
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

.toolbar-summary {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.toolbar-label {
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--timeline-muted);
  font-weight: 600;
}

.toolbar-count {
  font-size: 0.82rem;
  color: var(--timeline-muted);
  background: rgba(71, 84, 103, 0.12);
  border-radius: 999px;
  padding: 4px 10px;
}

.severity-summary {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.severity-summary-item {
  display: flex;
  align-items: center;
  gap: 6px;
  border-radius: 999px;
  padding: 4px 10px;
  background: rgba(71, 84, 103, 0.12);
  font-size: 0.78rem;
  color: var(--timeline-muted-strong);
}

.severity-summary-item[data-level="critical"] {
  background: var(--timeline-severity-critical-bg);
  color: var(--timeline-severity-critical);
}

.severity-summary-item[data-level="high"] {
  background: var(--timeline-severity-high-bg);
  color: var(--timeline-severity-high);
}

.severity-summary-item[data-level="moderate"] {
  background: var(--timeline-severity-moderate-bg);
  color: var(--timeline-severity-moderate);
}

.severity-summary-item[data-level="low"] {
  background: var(--timeline-severity-low-bg);
  color: var(--timeline-severity-low);
}

.severity-summary-item[data-level="info"] {
  background: var(--timeline-severity-info-bg);
  color: var(--timeline-severity-info);
}

.severity-summary-label {
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.severity-summary-count {
  font-variant-numeric: tabular-nums;
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
  transition: background 120ms ease, color 120ms ease, border 120ms ease, transform 120ms ease;
}

.filter-chip:hover,
.filter-chip:focus-visible {
  border-color: var(--timeline-severity-high);
  outline: none;
  transform: translateY(-1px);
}

.filter-chip.is-active[data-level="all"] {
  background: rgba(71, 84, 103, 0.12);
  color: var(--timeline-muted-strong);
  border-color: transparent;
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

.toolbar-search input:focus-visible {
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
  margin: 0;
  padding: 0;
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

.timeline-category-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-x: auto;
  padding-bottom: 6px;
}

.timeline-category-grid::-webkit-scrollbar {
  height: 6px;
}

.timeline-category-grid::-webkit-scrollbar-thumb {
  background: rgba(148, 163, 184, 0.4);
  border-radius: 999px;
}

.timeline-category-head,
.timeline-category-row {
  display: flex;
  gap: 12px;
}

.timeline-category-head {
  align-items: stretch;
}

.timeline-category-row {
  align-items: flex-start;
}

.timeline-category-corner,
.timeline-category-label {
  flex: 0 0 184px;
  background: rgba(248, 250, 252, 0.7);
  border: 1px solid rgba(148, 163, 184, 0.28);
  border-radius: calc(var(--timeline-radius) - 12px);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.timeline-category-corner {
  font-size: 0.78rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--timeline-muted-strong);
  font-weight: 600;
  justify-content: center;
}

.timeline-category-label {
  background: rgba(248, 250, 252, 0.9);
}

.timeline-category-label.is-collapsed {
  background: rgba(248, 250, 252, 0.55);
  border-style: dashed;
}

.timeline-category-title {
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--timeline-muted-strong);
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.timeline-category-count {
  font-size: 0.78rem;
  color: var(--timeline-muted);
  font-variant-numeric: tabular-nums;
}

.timeline-category-head-cell {
  flex: 0 0 260px;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(148, 163, 184, 0.28);
  border-radius: calc(var(--timeline-radius) - 12px);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.timeline-category-head-cell.is-collapsed {
  background: rgba(248, 250, 252, 0.55);
  border-style: dashed;
}

.timeline-day-name {
  font-size: 0.9rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--timeline-muted-strong);
}

.timeline-day-count {
  font-size: 0.78rem;
  color: var(--timeline-muted);
}

.timeline-day-summary {
  font-size: 0.78rem;
  color: var(--timeline-muted);
  font-style: italic;
}

.group-toggle {
  border: 1px solid rgba(148, 163, 184, 0.4);
  background: #ffffff;
  border-radius: 999px;
  padding: 4px 12px;
  font-size: 0.72rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--timeline-muted-strong);
  cursor: pointer;
  align-self: start;
}

.group-toggle:hover,
.group-toggle:focus-visible {
  border-color: var(--timeline-severity-high);
  color: var(--timeline-severity-high);
  outline: none;
}

.timeline-category-cell {
  flex: 0 0 260px;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(148, 163, 184, 0.22);
  border-radius: calc(var(--timeline-radius) - 12px);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.timeline-category-cell.is-empty {
  background: rgba(248, 250, 252, 0.55);
  border-style: dashed;
  align-items: center;
  justify-content: center;
}

.timeline-category-group {
  border: 1px solid rgba(148, 163, 184, 0.24);
  border-radius: calc(var(--timeline-radius) - 16px);
  padding: 10px 12px;
  background: rgba(248, 250, 252, 0.8);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.timeline-group-header {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  justify-content: flex-start;
}

.timeline-group-title {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--timeline-heading);
}

.timeline-group-count {
  font-size: 0.78rem;
  color: var(--timeline-muted);
  background: rgba(71, 84, 103, 0.14);
  border-radius: 999px;
  padding: 2px 10px;
}

.timeline-group-header .severity-badge {
  margin-left: auto;
}

.timeline-group-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  font-size: 0.78rem;
  color: var(--timeline-muted);
}

.timeline-group-range::before {
  content: "";
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: rgba(71, 84, 103, 0.3);
  display: inline-block;
  margin-right: 6px;
}

.timeline-group-relative {
  font-style: italic;
}

.timeline-category-cell.is-collapsed {
  background: rgba(248, 250, 252, 0.4);
  border-style: dashed;
  align-items: center;
  justify-content: center;
}

.timeline-category-placeholder {
  font-size: 0.78rem;
  color: var(--timeline-muted);
  font-style: italic;
}

.timeline-category-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.timeline-cell-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.timeline-cell-empty {
  font-size: 0.8rem;
  color: var(--timeline-muted);
  font-style: italic;
}

.timeline-grid-empty {
  background: rgba(248, 250, 252, 0.75);
  border: 1px dashed rgba(148, 163, 184, 0.5);
  border-radius: calc(var(--timeline-radius) - 10px);
  padding: 18px;
  text-align: center;
  color: var(--timeline-muted);
  font-style: italic;
}

.timeline-cell-list .timeline-event {
  border-left-width: 3px;
  padding-left: 14px;
  box-shadow: none;
}

.timeline-cell-list .timeline-event::before {
  display: none;
}

.timeline-group ul {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.timeline-event {
  border-left: 4px solid rgba(148, 163, 184, 0.4);
  padding: 12px 16px 12px 18px;
  position: relative;
  background: rgba(248, 250, 252, 0.7);
  border-radius: calc(var(--timeline-radius) - 10px);
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
  top: 18px;
}

.timeline-event.is-critical {
  border-left-color: var(--timeline-severity-critical);
  background: rgba(180, 35, 24, 0.08);
}

.timeline-event.is-critical::before {
  border-color: var(--timeline-severity-critical);
}

.timeline-event.is-high {
  border-left-color: var(--timeline-severity-high);
  background: rgba(220, 104, 3, 0.08);
}

.timeline-event.is-high::before {
  border-color: var(--timeline-severity-high);
}

.timeline-event.is-moderate {
  border-left-color: var(--timeline-severity-moderate);
  background: rgba(6, 118, 71, 0.08);
}

.timeline-event.is-moderate::before {
  border-color: var(--timeline-severity-moderate);
}

.timeline-event.is-low {
  border-left-color: var(--timeline-severity-low);
  background: rgba(11, 83, 148, 0.08);
}

.timeline-event.is-low::before {
  border-color: var(--timeline-severity-low);
}

.timeline-event.is-info {
  border-left-color: rgba(71, 84, 103, 0.35);
}

.timeline-event.is-info::before {
  border-color: rgba(71, 84, 103, 0.35);
}

.timeline-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  font-size: 0.84rem;
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

.timeline-source {
  display: flex;
  gap: 8px;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--timeline-muted);
  margin-top: 10px;
  flex-wrap: wrap;
}

.timeline-source-system {
  font-weight: 600;
  color: var(--timeline-muted-strong);
}

.timeline-source-display {
  background: rgba(71, 84, 103, 0.16);
  border-radius: 999px;
  padding: 2px 8px;
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
    padding: 18px;
  }

  .vital-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
  }

  .hot-item {
    padding: 10px;
  }

  .timeline-event {
    padding-left: 14px;
  }

  .timeline-event::before {
    left: -6px;
  }

  .timeline-meta {
    flex-direction: column;
    align-items: flex-start;
  }

  .toolbar-search {
    flex-direction: column;
    align-items: stretch;
  }

  .toolbar-search button {
    width: 100%;
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
