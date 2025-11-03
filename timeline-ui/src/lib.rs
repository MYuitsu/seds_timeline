//! Timeline UI component for the WebAssembly environment.

#[cfg(target_arch = "wasm32")]
mod styles;

#[cfg(target_arch = "wasm32")]
mod wasm_ui {
    use crate::styles;
    use chrono::{DateTime, Duration, NaiveDate, Utc};
    use serde_wasm_bindgen::from_value;
    use std::cmp::Ordering;
    use timeline_core::{
        CriticalItem, CriticalSummary, EventCategory, Severity, TimelineEvent, TimelineSnapshot,
        VitalSnapshot, VitalTrend,
    };
    use wasm_bindgen::prelude::*;
    use web_sys::{console, Document, Element, HtmlInputElement, Window};
    use yew::events::InputEvent;
    use yew::prelude::*;
    use yew::TargetCast;

    #[derive(Clone, Default, PartialEq)]
    struct FilterState {
        severity: Option<Severity>,
        query: String,
    }

    #[derive(Properties, PartialEq)]
    pub struct TimelineViewProps {
        pub snapshot: TimelineSnapshot,
    }

    #[function_component(TimelineView)]
    fn timeline_view(props: &TimelineViewProps) -> Html {
        let snapshot = &props.snapshot;

        use_effect_with((), |_| {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Err(err) = styles::ensure_styles(&document) {
                        console::error_1(&err);
                    }
                }
            }
            || ()
        });

        let filters = use_state(FilterState::default);
        let filters_value = (*filters).clone();
        let mut filtered_events: Vec<&TimelineEvent> = snapshot
            .events
            .iter()
            .filter(|event| event_matches_filters(event, &filters_value))
            .collect();

        filtered_events.sort_by(|a, b| compare_datetimes(b.occurred_at, a.occurred_at));

        let grouped_events = group_events_by_day(&filtered_events);

        let on_search = {
            let filters = filters.clone();
            Callback::from(move |event: InputEvent| {
                let input: HtmlInputElement = event.target_unchecked_into();
                let mut next = (*filters).clone();
                next.query = input.value();
                filters.set(next);
            })
        };

        let on_clear_filters = {
            let filters = filters.clone();
            Callback::from(move |_| {
                filters.set(FilterState::default());
            })
        };

        let severity_controls = render_severity_filters(filters.clone());

        let event_count_label = match filtered_events.len() {
            1 => "1 event".to_string(),
            count => format!("{count} events"),
        };

        let events_view = if filtered_events.is_empty() {
            html! { <li class="timeline-empty">{"No events match the current filters."}</li> }
        } else {
            html! {
                <>
                    { for grouped_events.into_iter().map(|(label, events)| render_event_group(label, events)) }
                </>
            }
        };

        html! {
            <div class="timeline-root">
                <aside class="critical-column">
                    <header class="critical-header">
                        <span class="critical-eyebrow">{"Emergency status"}</span>
                        <h2>{"Priority information"}</h2>
                        <p>{"Key items for the ED team."}</p>
                    </header>
                    { render_code_status(&snapshot.critical) }
                    { render_critical_card("Clinical alerts", &snapshot.critical.alerts, "No urgent alerts." ) }
                    { render_critical_card("Severe allergies", &snapshot.critical.allergies, "No high-risk allergies recorded.") }
                    { render_critical_card("Active medications", &snapshot.critical.medications, "No active medications." ) }
                    { render_critical_card("High-risk chronic conditions", &snapshot.critical.chronic_conditions, "No high-risk chronic conditions recorded.") }
                    { render_vitals(&snapshot.critical.recent_vitals) }
                    { render_vital_trends(&snapshot.critical) }
                </aside>
                <section class="timeline-column" aria-live="polite">
                    { render_hot_strip(&snapshot.events) }
                    <p class="timeline-updated">{
                        format!(
                            "Updated {}",
                            format_timestamp(Some(snapshot.generated_at))
                        )
                    }</p>
                    <header class="timeline-toolbar">
                        <div class="toolbar-group">
                            <span class="toolbar-label">{"Filters"}</span>
                            { severity_controls }
                        </div>
                        <span class="toolbar-count">{ event_count_label }</span>
                        <div class="toolbar-search">
                            <input
                                type="search"
                                placeholder="Filter by keyword (e.g., blood pressure, sepsis)"
                                value={filters_value.query.clone()}
                                oninput={on_search}
                                aria-label="Search events by keyword"
                            />
                            <button type="button" onclick={on_clear_filters.clone()} aria-label="Clear filters">{"Reset"}</button>
                        </div>
                    </header>
                    <ul class="timeline-events">
                        { events_view }
                    </ul>
                </section>
            </div>
        }
    }

    #[derive(Debug, Clone)]
    struct Sparkline {
        path: String,
        last_x: f64,
        last_y: f64,
    }

    fn render_severity_filters(filters: UseStateHandle<FilterState>) -> Html {
        let options = [
            (None, "All"),
            (Some(Severity::Critical), "Critical only"),
            (Some(Severity::High), "High and above"),
            (Some(Severity::Moderate), "Moderate and above"),
        ];

        html! {
            <div class="filter-chips" role="group" aria-label="Filter by severity">
                {
                    for options.into_iter().map(|(level, label)| {
                        let filters = filters.clone();
                        let is_active = filters.severity == level;
                        let onclick = Callback::from(move |_| {
                            let mut next = (*filters).clone();
                            if next.severity == level {
                                next.severity = None;
                            } else {
                                next.severity = level;
                            }
                            filters.set(next);
                        });

                        html! {
                            <button
                                type="button"
                                class={classes!("filter-chip", is_active.then_some("is-active"))}
                                aria-pressed={is_active.to_string()}
                                data-level={level.map(severity_level).unwrap_or("all")}
                                onclick={onclick}
                            >
                                { label }
                            </button>
                        }
                    })
                }
            </div>
        }
    }

    fn render_critical_card(title: &str, items: &[CriticalItem], empty_label: &str) -> Html {
        html! {
            <section class="critical-card">
                <header>
                    <h3>{ title }</h3>
                    <span class="critical-count">{ items.len() }</span>
                </header>
                <ul>
                    {
                        if items.is_empty() {
                            html! { <li class="critical-empty">{ empty_label }</li> }
                        } else {
                            html! { for items.iter().map(render_critical_item) }
                        }
                    }
                </ul>
            </section>
        }
    }

    fn render_code_status(summary: &CriticalSummary) -> Html {
        let (status_text, status_level, helper_text) = match summary.code_status.as_ref() {
            Some(value) => (value.clone(), "affirm", "Confirmed"),
            None => ("Not documented".to_string(), "warning", "Needs update"),
        };

        html! {
            <section class="critical-card code-status">
                <header>
                    <h3>{"Code status"}</h3>
                    <span class="critical-pill" data-level={status_level}>{ helper_text }</span>
                </header>
                <p class="code-status-value" data-level={status_level}>{ status_text }</p>
            </section>
        }
    }

    fn render_vitals(vitals: &[VitalSnapshot]) -> Html {
        html! {
            <section class="critical-card">
                <header>
                    <h3>{"Recent vital signs"}</h3>
                    <span class="critical-count">{ vitals.len() }</span>
                </header>
                <ul class="vital-list">
                    {
                        if vitals.is_empty() {
                            html! { <li class="critical-empty">{"No recent vital signs in the configured window."}</li> }
                        } else {
                            html! { for vitals.iter().map(render_vital_item) }
                        }
                    }
                </ul>
            </section>
        }
    }

    fn render_vital_trends(summary: &CriticalSummary) -> Html {
        if summary.vital_trends.is_empty() {
            return Html::default();
        }

        html! {
            <section class="critical-card trend-card">
                <header>
                    <h3>{"Vital trends"}</h3>
                    <span class="critical-count">{ summary.vital_trends.len() }</span>
                </header>
                <ul class="trend-list">
                    { for summary.vital_trends.iter().map(render_trend_item) }
                </ul>
            </section>
        }
    }

    fn render_trend_item(trend: &VitalTrend) -> Html {
        let numeric_values: Vec<f64> = trend.points.iter().filter_map(|p| p.value).collect();
        let sparkline = build_sparkline(&numeric_values, 160.0, 40.0);
        let latest_label = trend
            .points
            .last()
            .and_then(|p| p.label.clone())
            .unwrap_or_else(|| "--".to_string());
        let relative = format_relative_time(trend.points.last().and_then(|p| p.recorded_at));
        let delta_text = numeric_delta_display(&numeric_values, trend.unit.as_deref());
        let delta_state = if numeric_values.len() >= 2 {
            let first = numeric_values.first().copied().unwrap_or(0.0);
            let last = numeric_values.last().copied().unwrap_or(0.0);
            let delta = last - first;
            if delta.abs() < 0.1 {
                "steady"
            } else if delta > 0.0 {
                "up"
            } else {
                "down"
            }
        } else {
            "steady"
        };

        html! {
            <li class="trend-item">
                <div class="trend-header">
                    <span class="trend-name">{ trend.name.clone() }</span>
                    {
                        trend.unit.as_ref().map(|unit| html! { <span class="trend-unit">{ unit.clone() }</span> }).unwrap_or_default()
                    }
                </div>
                <div class="trend-content">
                    {
                        if let Some(spark) = sparkline {
                            html! {
                                <svg class="trend-chart" viewBox="0 0 160 40" preserveAspectRatio="none" role="img" aria-label={format!("Trend for {0}", trend.name)}>
                                    <path d={spark.path.clone()} />
                                    <circle cx={format!("{:.2}", spark.last_x)} cy={format!("{:.2}", spark.last_y)} r="2.5" />
                                </svg>
                            }
                        } else {
                            html! { <div class="trend-fallback">{"Not enough data to render a chart."}</div> }
                        }
                    }
                    <div class="trend-meta">
                        <span class="trend-latest">{ latest_label }</span>
                        {
                            relative.map(|text| html! { <span class="trend-time">{ text }</span> }).unwrap_or_default()
                        }
                        {
                            delta_text.map(|text| html! { <span class="trend-delta" data-trend={delta_state}>{ text }</span> }).unwrap_or_default()
                        }
                    </div>
                </div>
            </li>
        }
    }

    fn build_sparkline(values: &[f64], width: f64, height: f64) -> Option<Sparkline> {
        if values.len() < 2 {
            return None;
        }

        let min = values
            .iter()
            .fold(f64::INFINITY, |acc, value| acc.min(*value));
        let max = values
            .iter()
            .fold(f64::NEG_INFINITY, |acc, value| acc.max(*value));
        let span = (max - min).abs();
        let range = if span < f64::EPSILON { 1.0 } else { span };
        let step = width / (values.len() as f64 - 1.0);

        let mut path = String::with_capacity(values.len() * 12);
        let mut last_x = 0.0;
        let mut last_y = height / 2.0;

        for (idx, value) in values.iter().enumerate() {
            let x = step * idx as f64;
            let normalized = (*value - min) / range;
            let y = height - (normalized * height);
            if idx == 0 {
                path.push_str(&format!("M{:.2},{:.2}", x, y));
            } else {
                path.push_str(&format!(" L{:.2},{:.2}", x, y));
            }
            last_x = x;
            last_y = y;
        }

        Some(Sparkline {
            path,
            last_x,
            last_y,
        })
    }

    fn numeric_delta_display(values: &[f64], unit: Option<&str>) -> Option<String> {
        if values.len() < 2 {
            return None;
        }

        let first = values.first().copied().unwrap_or(0.0);
        let last = values.last().copied().unwrap_or(0.0);
        let delta = last - first;
        let formatted = if delta.abs() >= 10.0 {
            format!("{delta:+.0}")
        } else {
            format!("{delta:+.1}")
        };

        let unit_suffix = unit.map(|u| format!(" {u}")).unwrap_or_default();
        Some(format!("Δ {formatted}{unit_suffix}"))
    }

    fn render_critical_item(item: &CriticalItem) -> Html {
        let severity_label = severity_label(item.severity);
        let severity_level = severity_level(item.severity);
        html! {
            <li class="critical-item">
                <div class="critical-item-header">
                    <span class="critical-label">{ item.label.clone() }</span>
                    <span class="severity-badge" data-level={severity_level}>{ severity_label }</span>
                </div>
                { item.detail.as_ref().map(render_detail).unwrap_or_default() }
            </li>
        }
    }

    fn render_detail(detail: &String) -> Html {
        html! { <p class="critical-detail">{ detail.clone() }</p> }
    }

    fn render_vital_item(vital: &VitalSnapshot) -> Html {
        let timestamp = format_timestamp(vital.recorded_at);
        let relative = format_relative_time(vital.recorded_at);
        let unit_to_render = vital.unit.as_ref().and_then(|unit| {
            let unit_lower = unit.to_ascii_lowercase();
            let value_lower = vital.value.to_ascii_lowercase();
            if value_lower.contains(&unit_lower) {
                None
            } else {
                Some(unit.clone())
            }
        });
        html! {
            <li class="vital-item">
                <div class="vital-text">
                    <span class="vital-name">{ vital.name.clone() }</span>
                    <span class="vital-value">{ vital.value.clone() }</span>
                    { unit_to_render.map(|unit| html! { <span class="vital-unit">{ unit }</span> }).unwrap_or_default() }
                </div>
                <div class="vital-meta">
                    <span class="vital-time">{ timestamp }</span>
                    { relative.map(|text| html! { <span class="vital-relative">{ text }</span> }).unwrap_or_default() }
                </div>
            </li>
        }
    }

    fn render_hot_strip(events: &[TimelineEvent]) -> Html {
        let mut urgent: Vec<&TimelineEvent> = events
            .iter()
            .filter(|event| matches!(event.severity, Severity::Critical | Severity::High))
            .collect();
        urgent.sort_by(|a, b| compare_datetimes(b.occurred_at, a.occurred_at));
        urgent.truncate(3);

        if urgent.is_empty() {
            return Html::default();
        }

        html! {
            <aside class="hot-strip" aria-label="Urgent clinical events">
                <h3>{"Priority watchlist"}</h3>
                <ul>
                    { for urgent.into_iter().map(render_hot_item) }
                </ul>
            </aside>
        }
    }

    fn render_hot_item(event: &TimelineEvent) -> Html {
        let relative = format_relative_time(event.occurred_at);
        html! {
            <li class="hot-item">
                <div class="hot-header">
                    <span class="hot-title">{ event.title.clone() }</span>
                    <span class="hot-severity" data-level={severity_level(event.severity)}>{ severity_label(event.severity) }</span>
                </div>
                { event.detail.as_ref().map(|detail| html! { <p class="hot-detail">{ detail.clone() }</p> }).unwrap_or_default() }
                <div class="hot-meta">
                    { relative.map(|text| html! { <span>{ text }</span> }).unwrap_or_default() }
                    <span class="hot-category">{ category_label(event.category) }</span>
                </div>
            </li>
        }
    }

    fn render_event_group(label: String, events: Vec<&TimelineEvent>) -> Html {
        html! {
            <li class="timeline-group">
                <div class="timeline-group-header">
                    <span>{ label }</span>
                </div>
                <ul>
                    { for events.into_iter().map(render_event) }
                </ul>
            </li>
        }
    }

    fn render_event(event: &TimelineEvent) -> Html {
        let severity_label = severity_label(event.severity);
        let severity_level = severity_level(event.severity);
        let timestamp = format_timestamp(event.occurred_at);
        let relative = format_relative_time(event.occurred_at);
        let category = category_label(event.category);
        let severity_class = format!("is-{}", severity_level);

        html! {
            <li class={classes!("timeline-event", severity_class)}>
                <div class="timeline-meta">
                    <span class="timeline-time">{ timestamp }</span>
                    { relative.map(|text| html! { <span class="timeline-relative">{ text }</span> }).unwrap_or_default() }
                    <span class="timeline-category">{ category }</span>
                    <span class="timeline-severity" data-level={severity_level}>{ severity_label }</span>
                </div>
                <div class="timeline-body">
                    <h3 class="timeline-title">{ event.title.clone() }</h3>
                    { event.detail.as_ref().map(render_event_detail).unwrap_or_default() }
                    { render_event_source(event) }
                </div>
            </li>
        }
    }

    fn render_event_detail(detail: &String) -> Html {
        html! { <p class="timeline-detail">{ detail.clone() }</p> }
    }

    fn render_event_source(event: &TimelineEvent) -> Html {
        let Some(source) = event.source.as_ref() else {
            return Html::default();
        };

        let system = source.system.as_deref().unwrap_or("FHIR");
        let display = source
            .display
            .clone()
            .or_else(|| source.reference.clone())
            .unwrap_or_else(|| "Unknown source".to_string());

        html! {
            <div class="timeline-source">
                <span class="timeline-source-system">{ system }</span>
                <span class="timeline-source-display">{ display }</span>
            </div>
        }
    }

    fn category_label(category: EventCategory) -> &'static str {
        match category {
            EventCategory::Encounter => "Encounter",
            EventCategory::Procedure => "Procedure",
            EventCategory::Condition => "Condition",
            EventCategory::Medication => "Medication",
            EventCategory::Observation => "Observation",
            EventCategory::Document => "Document",
            EventCategory::Note => "Note",
            EventCategory::Other => "Other",
        }
    }

    fn severity_label(severity: Severity) -> &'static str {
        match severity {
            Severity::Critical => "Critical",
            Severity::High => "High",
            Severity::Moderate => "Moderate",
            Severity::Low => "Low",
            Severity::Info => "Info",
        }
    }

    fn severity_level(severity: Severity) -> &'static str {
        match severity {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Moderate => "moderate",
            Severity::Low => "low",
            Severity::Info => "info",
        }
    }

    fn event_matches_filters(event: &TimelineEvent, filters: &FilterState) -> bool {
        if let Some(level) = filters.severity {
            if event.severity > level {
                return false;
            }
        }

        let query = filters.query.trim().to_lowercase();
        if query.is_empty() {
            return true;
        }

        let haystack = [
            Some(event.title.as_str()),
            event.detail.as_deref(),
            Some(category_label(event.category)),
            event.source.as_ref().and_then(|s| s.display.as_deref()),
            event.source.as_ref().and_then(|s| s.reference.as_deref()),
        ];

        haystack
            .into_iter()
            .flatten()
            .any(|text| text.to_lowercase().contains(&query))
    }

    fn format_timestamp(timestamp: Option<DateTime<Utc>>) -> String {
        timestamp
            .map(|dt| dt.format("%m/%d/%Y %H:%M").to_string())
            .unwrap_or_else(|| "--".to_string())
    }

    fn format_relative_time(timestamp: Option<DateTime<Utc>>) -> Option<String> {
        let timestamp = timestamp?;
        let now = Utc::now();
        let delta = now.signed_duration_since(timestamp);
        let is_future = delta.num_seconds() < 0;
        let abs_delta: Duration = delta.abs();

        if abs_delta.num_days() >= 1 {
            let value = abs_delta.num_days();
            let unit = if value == 1 { "day" } else { "days" };
            if is_future {
                Some(format!("in {value} {unit}"))
            } else {
                Some(format!("{value} {unit} ago"))
            }
        } else if abs_delta.num_hours() >= 1 {
            let value = abs_delta.num_hours();
            let unit = if value == 1 { "hour" } else { "hours" };
            if is_future {
                Some(format!("in {value} {unit}"))
            } else {
                Some(format!("{value} {unit} ago"))
            }
        } else if abs_delta.num_minutes() >= 1 {
            let value = abs_delta.num_minutes();
            let unit = if value == 1 { "minute" } else { "minutes" };
            if is_future {
                Some(format!("in {value} {unit}"))
            } else {
                Some(format!("{value} {unit} ago"))
            }
        } else {
            if is_future {
                Some("in moments".to_string())
            } else {
                Some("just now".to_string())
            }
        }
    }

    fn format_day_label(timestamp: Option<DateTime<Utc>>) -> String {
        let Some(dt) = timestamp else {
            return "Unknown time".to_string();
        };

        let today: NaiveDate = Utc::now().date_naive();
        let date = dt.date_naive();
        let delta_days = today.signed_duration_since(date).num_days();

        if delta_days == 0 {
            "Today".to_string()
        } else if delta_days == 1 {
            "Yesterday".to_string()
        } else if delta_days == -1 {
            "Tomorrow".to_string()
        } else if (2..=6).contains(&delta_days) {
            format!("{delta_days} days ago")
        } else if (-6..=-2).contains(&delta_days) {
            format!("in {} days", delta_days.abs())
        } else {
            dt.format("%m/%d/%Y").to_string()
        }
    }

    fn group_events_by_day<'a>(
        events: &'a [&'a TimelineEvent],
    ) -> Vec<(String, Vec<&'a TimelineEvent>)> {
        let mut groups: Vec<(String, Vec<&'a TimelineEvent>)> = Vec::new();
        let mut current_label: Option<String> = None;
        let mut bucket: Vec<&'a TimelineEvent> = Vec::new();

        for event in events {
            let label = format_day_label(event.occurred_at);
            match current_label {
                Some(ref current) if current == &label => {
                    bucket.push(*event);
                }
                _ => {
                    if let Some(current) = current_label.replace(label.clone()) {
                        groups.push((current, std::mem::take(&mut bucket)));
                    }
                    bucket.push(*event);
                    current_label = Some(label);
                }
            }
        }

        if let Some(label) = current_label {
            groups.push((label, bucket));
        }

        groups
    }

    fn compare_datetimes(a: Option<DateTime<Utc>>, b: Option<DateTime<Utc>>) -> Ordering {
        match (a, b) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => Ordering::Equal,
        }
    }

    #[wasm_bindgen]
    pub fn mount_timeline_view(selector: &str, snapshot: JsValue) -> Result<(), JsValue> {
        let window: Window =
            web_sys::window().ok_or_else(|| JsValue::from_str("window is not available"))?;
        let document: Document = window
            .document()
            .ok_or_else(|| JsValue::from_str("document is not accessible"))?;

        let target: Element = document
            .query_selector(selector)
            .map_err(|err| JsValue::from_str(&format!("Selector error: {err:?}")))?
            .ok_or_else(|| JsValue::from_str("Element not found for selector"))?;

        let snapshot: TimelineSnapshot = from_value(snapshot)?;

        yew::Renderer::<TimelineView>::with_root_and_props(target, TimelineViewProps { snapshot })
            .render();
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_ui::mount_timeline_view;

#[cfg(not(target_arch = "wasm32"))]
pub fn mount_timeline_view(_: &str, _: wasm_bindgen::JsValue) -> Result<(), wasm_bindgen::JsValue> {
    Err(wasm_bindgen::JsValue::from_str(
        "timeline-ui only supports the wasm32 compilation target",
    ))
}
