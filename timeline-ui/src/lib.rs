//! Timeline UI component for the WebAssembly environment.

#[cfg(target_arch = "wasm32")]
mod styles;

#[cfg(target_arch = "wasm32")]
mod wasm_ui {
    use crate::styles;
    use chrono::{DateTime, Duration, NaiveDate, Utc};
    use serde_wasm_bindgen::from_value;
    use std::{
        cmp::Ordering,
        collections::{BTreeMap, HashMap, HashSet},
    };
    use timeline_core::{
        CriticalItem, CriticalSummary, DiagnosticKind, DiagnosticSnapshot, EventCategory, Severity,
        TimelineEvent, TimelineSnapshot, VitalSnapshot, VitalTrend,
    };
    use wasm_bindgen::prelude::*;
    use web_sys::{console, Document, Element, HtmlInputElement, Window};
    use yew::events::InputEvent;
    use yew::prelude::*;
    use yew::TargetCast;

    const VITAL_EVENT_KEYWORDS: &[&str] = &[
        "heart rate",
        "blood pressure",
        "respiratory rate",
        "spo2",
        "oxygen saturation",
        "temperature",
        "pulse",
    ];

    const LAB_EVENT_KEYWORDS: &[&str] = &[
        "lactate",
        "troponin",
        "culture",
        "panel",
        "cbc",
        "chemistry",
        "creatinine",
        "glucose",
        "magnesium",
        "blood gas",
    ];

    const IMAGING_EVENT_KEYWORDS: &[&str] = &[
        "ct",
        "mri",
        "x-ray",
        "xray",
        "ultrasound",
        "radiograph",
    ];

    const TIMELINE_BUCKET_COLUMNS: &[(&str, &str)] = &[
        ("Vitals", "Vitals"),
        ("Labs", "Labs"),
        ("Imaging", "Imaging"),
        ("Observations", "Observations"),
        ("Medications", "Medications"),
        ("Procedures", "Procedures"),
        ("Encounters", "Encounters"),
        ("Conditions", "Conditions"),
        ("Documents", "Documents"),
        ("Notes", "Notes"),
        ("Events", "Other"),
    ];

    #[derive(Clone, Default, PartialEq)]
    struct FilterState {
        severity: Option<Severity>,
        query: String,
    }

    #[derive(Debug, Default, Clone, Copy)]
    struct SeverityCounts {
        total: usize,
        critical: usize,
        high: usize,
        moderate: usize,
        low: usize,
        info: usize,
    }

    struct DayRow<'a> {
        label: String,
        key: String,
        summary: String,
        default_collapsed: bool,
        is_expanded: bool,
        event_count: usize,
        buckets: HashMap<&'static str, Vec<&'a TimelineEvent>>,
    }

    struct GroupedEvents<'a> {
        title: String,
        events: Vec<&'a TimelineEvent>,
    }

    #[derive(Clone, Copy)]
    enum ChartMode {
        TimelinePerDay,
        SummaryByDay,
    }

    struct MeasurementChartData<'a> {
        unit: Option<String>,
        series: Vec<MeasurementSeries<'a>>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    }

    struct MeasurementSeries<'a> {
        label: String,
        points: Vec<SeriesPoint<'a>>,
    }

    struct SeriesPoint<'a> {
        timestamp: DateTime<Utc>,
        value: f64,
        _marker: std::marker::PhantomData<&'a TimelineEvent>,
    }

    struct ParsedMeasurement {
        unit: Option<String>,
        values: Vec<NamedValue>,
    }

    struct NamedValue {
        label: String,
        value: f64,
    }

    #[derive(Clone, Copy, PartialEq)]
    enum CardVariant {
        Neutral,
        Alert,
        Allergy,
        Medication,
        Condition,
        Vitals,
        Diagnostics,
        Insights,
    }

    impl CardVariant {
        fn data_attr(self) -> &'static str {
            match self {
                CardVariant::Neutral => "neutral",
                CardVariant::Alert => "alert",
                CardVariant::Allergy => "allergy",
                CardVariant::Medication => "medication",
                CardVariant::Condition => "condition",
                CardVariant::Vitals => "vitals",
                CardVariant::Diagnostics => "diagnostics",
                CardVariant::Insights => "insights",
            }
        }
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
        let expanded_groups = use_state(|| HashSet::<String>::new());
        let expanded_snapshot = (*expanded_groups).clone();
        let mut filtered_events: Vec<&TimelineEvent> = snapshot
            .events
            .iter()
            .filter(|event| event_matches_filters(event, &filters_value))
            .collect();

        filtered_events.sort_by(|a, b| compare_datetimes(b.occurred_at, a.occurred_at));

        let grouped_events = group_events_by_day(&filtered_events);
        let severity_counts = tally_severity(&filtered_events);
        let event_count_label = format_event_count(&severity_counts);
        let snapshot_recency = format_relative_time(Some(snapshot.generated_at))
            .unwrap_or_else(|| "just now".to_string());

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

        let events_view = if filtered_events.is_empty() {
            html! { <div class="timeline-empty">{"No events match the current filters."}</div> }
        } else {
            render_category_grid(grouped_events, expanded_groups.clone(), expanded_snapshot)
        };

        html! {
            <div class="timeline-root">
                <aside class="critical-column">
                    <header class="critical-header">
                        <span class="critical-eyebrow">{"Emergency status"}</span>
                        <h2>{"Priority information"}</h2>
                        <p class="critical-subhead">{ format!("Snapshot generated {snapshot_recency}") }</p>
                    </header>
                    { render_code_status(&snapshot.critical) }
                    { render_trend_insights(&snapshot.critical) }
                    { render_vitals(&snapshot.critical.recent_vitals) }
                    { render_diagnostics(&snapshot.critical) }
                    { render_vital_trends(&snapshot.critical) }
                    { render_critical_card("Clinical alerts", &snapshot.critical.alerts, "No urgent alerts.", CardVariant::Alert ) }
                    { render_critical_card("Severe allergies", &snapshot.critical.allergies, "No high-risk allergies recorded.", CardVariant::Allergy) }
                    { render_critical_card("Active medications", &snapshot.critical.medications, "No active medications.", CardVariant::Medication ) }
                    { render_critical_card("High-risk chronic conditions", &snapshot.critical.chronic_conditions, "No high-risk chronic conditions recorded.", CardVariant::Condition) }
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
                        <div class="toolbar-summary">
                            <span class="toolbar-count">{ event_count_label }</span>
                            { build_severity_badges(&severity_counts) }
                        </div>
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
                    <div class="timeline-events">
                        { events_view }
                    </div>
                </section>
            </div>
        }
    }

    fn tally_severity(events: &[&TimelineEvent]) -> SeverityCounts {
        let mut counts = SeverityCounts::default();

        for event in events {
            counts.total += 1;
            match event.severity {
                Severity::Critical => counts.critical += 1,
                Severity::High => counts.high += 1,
                Severity::Moderate => counts.moderate += 1,
                Severity::Low => counts.low += 1,
                Severity::Info => counts.info += 1,
            }
        }

        counts
    }

    fn format_event_count(counts: &SeverityCounts) -> String {
        match counts.total {
            0 => "No events in view".to_string(),
            1 => "1 event".to_string(),
            total => {
                if counts.critical > 0 {
                    format!("{total} events ({} critical)", counts.critical)
                } else if counts.high > 0 {
                    format!("{total} events ({} high severity)", counts.high)
                } else if counts.moderate > 0 {
                    format!("{total} events ({} moderate)", counts.moderate)
                } else if counts.low > 0 {
                    format!("{total} events ({} low)", counts.low)
                } else {
                    format!("{total} events")
                }
            }
        }
    }

    fn build_severity_badges(counts: &SeverityCounts) -> Html {
        let entries = [
            (Severity::Critical, counts.critical),
            (Severity::High, counts.high),
            (Severity::Moderate, counts.moderate),
            (Severity::Low, counts.low),
            (Severity::Info, counts.info),
        ];

        if counts.total == 0 {
            return Html::default();
        }

        html! {
            <ul class="severity-summary" aria-label="Events by severity">
                {
                    for entries.into_iter().filter(|(_, count)| *count > 0).map(|(severity, count)| {
                        html! {
                            <li class="severity-summary-item" data-level={severity_level(severity)}>
                                <span class="severity-summary-label">{ severity_label(severity) }</span>
                                <span class="severity-summary-count">{ count }</span>
                            </li>
                        }
                    })
                }
            </ul>
        }
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

    fn render_critical_card(
        title: &str,
        items: &[CriticalItem],
        empty_label: &str,
        variant: CardVariant,
    ) -> Html {
        html! {
            <section class="critical-card" data-variant={variant.data_attr()}>
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

        let icon = if status_level == "affirm" { "✓" } else { "!" };

        html! {
            <section class="critical-card code-status" data-variant="code-status">
                <header>
                    <div class="code-status-heading">
                        <span class="code-status-icon" data-level={status_level} aria-hidden="true">{ icon }</span>
                        <h3>{"Code status"}</h3>
                    </div>
                    <span class="critical-pill" data-level={status_level}>{ helper_text }</span>
                </header>
                <p class="code-status-value" data-level={status_level}>{ status_text }</p>
            </section>
        }
    }

    fn render_vitals(vitals: &[VitalSnapshot]) -> Html {
        html! {
            <section class="critical-card" data-variant={CardVariant::Vitals.data_attr()}>
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
            <section class="critical-card trend-card" data-variant={CardVariant::Vitals.data_attr()}>
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

    fn render_diagnostics(summary: &CriticalSummary) -> Html {
        let labs: Vec<&DiagnosticSnapshot> = summary
            .recent_diagnostics
            .iter()
            .filter(|item| matches!(item.kind, DiagnosticKind::Lab))
            .collect();

        let imaging: Vec<&DiagnosticSnapshot> = summary
            .recent_diagnostics
            .iter()
            .filter(|item| matches!(item.kind, DiagnosticKind::Imaging))
            .collect();

        if labs.is_empty() && imaging.is_empty() {
            return Html::default();
        }

        let total = labs.len() + imaging.len();
        let labs_html = if labs.is_empty() {
            Html::default()
        } else {
            render_diagnostic_group("Labs", labs)
        };
        let imaging_html = if imaging.is_empty() {
            Html::default()
        } else {
            render_diagnostic_group("Imaging", imaging)
        };

        html! {
            <section class="critical-card diagnostics-card" data-variant={CardVariant::Diagnostics.data_attr()}>
                <header>
                    <h3>{"Recent diagnostics"}</h3>
                    <span class="critical-count">{ total }</span>
                </header>
                <div class="diagnostic-groups">
                    { labs_html }
                    { imaging_html }
                </div>
            </section>
        }
    }

    fn render_diagnostic_group(label: &str, items: Vec<&DiagnosticSnapshot>) -> Html {
        html! {
            <div class="diagnostic-group">
                <h4>{ label }</h4>
                <ul class="diagnostic-list">
                    { for items.into_iter().map(render_diagnostic_item) }
                </ul>
            </div>
        }
    }

    fn render_diagnostic_item(item: &DiagnosticSnapshot) -> Html {
        let severity_level = severity_level(item.severity);
        let severity_label = severity_label(item.severity);
        let relative = format_relative_time(item.recorded_at);

        html! {
            <li class="diagnostic-item">
                <div class="diagnostic-header">
                    <span class="diagnostic-name">{ item.name.clone() }</span>
                    <span class="severity-badge" data-level={severity_level}>{ severity_label }</span>
                </div>
                <div class="diagnostic-value">{ item.value.clone() }</div>
                <div class="diagnostic-meta">
                    { relative.map(|text| html! { <span>{ text }</span> }).unwrap_or_default() }
                </div>
            </li>
        }
    }

    fn render_trend_insights(summary: &CriticalSummary) -> Html {
        let mut items: Vec<Html> = Vec::new();

        for trend in &summary.vital_trends {
            let numeric_points: Vec<_> = trend
                .points
                .iter()
                .filter_map(|point| point.value.map(|value| (point, value)))
                .collect();

            if numeric_points.len() < 2 {
                continue;
            }

            let (first_point, first_value) = numeric_points.first().copied().unwrap();
            let (last_point, last_value) = numeric_points.last().copied().unwrap();
            let delta = last_value - first_value;

            if delta.abs() < 0.5 {
                continue;
            }

            let direction = if delta > 0.0 { "up" } else { "down" };
            let arrow = if delta > 0.0 { "↑" } else { "↓" };
            let unit_suffix = trend.unit.as_deref().unwrap_or("");
            let change_value = format_numeric(delta.abs());
            let change_summary = if unit_suffix.is_empty() {
                format!("{arrow}{change_value}")
            } else {
                format!("{arrow}{change_value} {unit_suffix}")
            };

            let span_text = format_duration_span(first_point.recorded_at, last_point.recorded_at)
                .unwrap_or_else(|| "recent readings".to_string());
            let change_text = format!("{change_summary} in {span_text}");

            let start_label = first_point
                .label
                .clone()
                .unwrap_or_else(|| format_measurement(first_value, trend.unit.as_deref()));
            let end_label = last_point
                .label
                .clone()
                .unwrap_or_else(|| format_measurement(last_value, trend.unit.as_deref()));
            let detail_text = format!("{start_label} → {end_label}");

            let range_text = format_time_range(first_point.recorded_at, last_point.recorded_at);
            let relative_text = format_relative_time(last_point.recorded_at);

            items.push(html! {
                <li class="insight-item" data-trend={direction}>
                    <div class="insight-header">
                        <span class="insight-arrow" aria-hidden="true">{ arrow }</span>
                        <span class="insight-name">{ trend.name.clone() }</span>
                    </div>
                    <div class="insight-change">{ change_text }</div>
                    <div class="insight-detail">{ detail_text }</div>
                    <div class="insight-meta">
                        {
                            range_text
                                .map(|text| html! { <span class="insight-range">{ text }</span> })
                                .unwrap_or_default()
                        }
                        {
                            relative_text
                                .map(|text| html! { <span class="insight-relative">{ format!("Last recorded {text}") }</span> })
                                .unwrap_or_default()
                        }
                    </div>
                </li>
            });
        }

        if items.is_empty() {
            Html::default()
        } else {
            html! {
                <section class="critical-card insights-card" data-variant={CardVariant::Insights.data_attr()}>
                    <header>
                        <h3>{"Trend insights"}</h3>
                        <span class="critical-count">{ items.len() }</span>
                    </header>
                    <ul class="insight-list">
                        { for items }
                    </ul>
                </section>
            }
        }
    }

    fn render_trend_item(trend: &VitalTrend) -> Html {
        let numeric_values: Vec<f64> = trend.points.iter().filter_map(|p| p.value).collect();
        let chart_data = trend_to_chart_data(trend);
        let chart_html = chart_data
            .as_ref()
            .map(|data| build_measurement_chart(data, Severity::Info, ChartMode::SummaryByDay))
            .unwrap_or_else(|| html! { <div class="trend-fallback">{"Not enough data to render a chart."}</div> });

        let latest_label = trend
            .points
            .iter()
            .rev()
            .find_map(|p| p.label.clone())
            .unwrap_or_else(|| "--".to_string());
        let relative = trend
            .points
            .iter()
            .rev()
            .find_map(|p| format_relative_time(p.recorded_at));
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
                    { chart_html }
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

    fn trend_to_chart_data(trend: &VitalTrend) -> Option<MeasurementChartData<'static>> {
        let mut points: Vec<SeriesPoint<'static>> = trend
            .points
            .iter()
            .filter_map(|point| {
                let timestamp = point.recorded_at?;
                let value = point.value?;
                Some(SeriesPoint {
                    timestamp,
                    value,
                    _marker: std::marker::PhantomData,
                })
            })
            .collect();

        if points.len() < 2 {
            return None;
        }

        points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        let start = points.first()?.timestamp;
        let end = points.last()?.timestamp;

        let series = MeasurementSeries {
            label: trend.name.clone(),
            points,
        };

        Some(MeasurementChartData {
            unit: trend.unit.clone(),
            series: vec![series],
            start,
            end,
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

    fn format_duration_span(
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Option<String> {
        let start = start?;
        let end = end?;
        let mut delta = end.signed_duration_since(start);
        if delta.num_seconds().abs() < 1 {
            return Some("moments".to_string());
        }

        if delta.num_seconds() < 0 {
            delta = -delta;
        }

        let total_minutes = delta.num_minutes();
        if total_minutes < 1 {
            return Some("moments".to_string());
        }

        let hours = total_minutes / 60;
        let minutes = total_minutes % 60;
        let mut parts = Vec::new();

        if hours > 0 {
            let unit = if hours == 1 { "hour" } else { "hours" };
            parts.push(format!("{hours} {unit}"));
        }

        if minutes > 0 {
            let unit = if minutes == 1 { "minute" } else { "minutes" };
            parts.push(format!("{minutes} {unit}"));
        }

        Some(parts.join(" "))
    }

    fn format_time_range(
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Option<String> {
        match (start, end) {
            (Some(start), Some(end)) => Some(format!(
                "{} -> {}",
                format_clock_time(start),
                format_clock_time(end)
            )),
            _ => None,
        }
    }

    fn format_clock_time(timestamp: DateTime<Utc>) -> String {
        timestamp.format("%H:%M").to_string()
    }

    fn format_numeric(value: f64) -> String {
        if value.abs() >= 10.0 {
            format!("{value:.0}")
        } else {
            format!("{value:.1}")
        }
    }

    fn format_measurement(value: f64, unit: Option<&str>) -> String {
        let numeric = format_numeric(value);
        match unit {
            Some(unit) if !unit.is_empty() => format!("{numeric} {unit}"),
            _ => numeric,
        }
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

    fn render_category_grid(
        grouped_events: Vec<(String, Vec<&TimelineEvent>)>,
        expanded_groups: UseStateHandle<HashSet<String>>,
        expanded_snapshot: HashSet<String>,
    ) -> Html {
        let mut bucket_totals: HashMap<&'static str, usize> = HashMap::new();
        let mut day_rows: Vec<DayRow<'_>> = Vec::new();

        for (index, (label, events)) in grouped_events.into_iter().enumerate() {
            let key = group_storage_key(&label, &events);
            let default_collapsed = should_collapse_group(index, &label, &events);
            let is_expanded = expanded_snapshot.contains(&key) || !default_collapsed;
            let summary = summarize_group(&events);
            let event_count = events.len();
            let mut buckets: HashMap<&'static str, Vec<&TimelineEvent>> = HashMap::new();

            for event in &events {
                let bucket = categorize_event_for_summary(event);
                *bucket_totals.entry(bucket).or_insert(0) += 1;
                buckets.entry(bucket).or_default().push(*event);
            }

            for bucket_events in buckets.values_mut() {
                bucket_events.sort_by(|a, b| compare_datetimes(b.occurred_at, a.occurred_at));
            }

            day_rows.push(DayRow {
                label,
                key,
                summary,
                default_collapsed,
                is_expanded,
                event_count,
                buckets,
            });
        }

        html! {
            <div class="timeline-category-grid">
                <div class="timeline-category-head">
                    <div class="timeline-category-corner">{"Day"}</div>
                    {
                        for TIMELINE_BUCKET_COLUMNS.iter().map(|(bucket, heading)| {
                            let count = bucket_totals.get(bucket).copied().unwrap_or(0);
                            render_category_header_cell(*heading, count, bucket_slug(bucket))
                        })
                    }
                </div>
                {
                    for day_rows.iter().map(|row| {
                        render_category_day_row(row, expanded_groups.clone())
                    })
                }
            </div>
        }
    }

    fn render_category_header_cell(heading: &str, count: usize, bucket_slug: &'static str) -> Html {
        html! {
            <div class="timeline-category-head-cell" data-bucket={bucket_slug}>
                <span class="timeline-category-title">{ heading }</span>
                <span class="timeline-category-count">{ count }</span>
            </div>
        }
    }

    fn render_category_day_row(
        row: &DayRow<'_>,
        expanded_groups: UseStateHandle<HashSet<String>>,
    ) -> Html {
        let is_collapsed = row.default_collapsed && !row.is_expanded;

        html! {
            <div class="timeline-category-row" data-group-key={row.key.clone()}>
                { render_day_row_header(row, expanded_groups.clone()) }
                {
                    for TIMELINE_BUCKET_COLUMNS.iter().map(|(bucket, _)| {
                        let events = row.buckets.get(bucket);
                        render_category_cell(events, is_collapsed)
                    })
                }
            </div>
        }
    }

    fn render_day_row_header(
        row: &DayRow<'_>,
        expanded_groups: UseStateHandle<HashSet<String>>,
    ) -> Html {
        let label = row.label.clone();
        let summary_text = row.summary.clone();
        let event_count = row.event_count;
        let default_collapsed = row.default_collapsed;
        let is_expanded = row.is_expanded;
        let key = row.key.clone();

        let button = if default_collapsed {
            let handle = expanded_groups.clone();
            let key_clone = key.clone();
            let text = if is_expanded { "Collapse" } else { "Expand" };
            html! {
                <button
                    type="button"
                    class="group-toggle"
                    aria-expanded={is_expanded.to_string()}
                    onclick={Callback::from(move |_| {
                        let mut next = (*handle).clone();
                        if next.contains(&key_clone) {
                            next.remove(&key_clone);
                        } else {
                            next.insert(key_clone.clone());
                        }
                        handle.set(next);
                    })}
                >
                    { text }
                </button>
            }
        } else {
            Html::default()
        };

        html! {
            <div
                class={classes!(
                    "timeline-category-label",
                    (!is_expanded && default_collapsed).then_some("is-collapsed"),
                )}
            >
                <span class="timeline-day-name">{ label }</span>
                <span class="timeline-day-count">{ format!("{event_count} events") }</span>
                {
                    (!is_expanded && default_collapsed)
                        .then(|| html! { <span class="timeline-day-summary">{ summary_text.clone() }</span> })
                        .unwrap_or_default()
                }
                { button }
            </div>
        }
    }

    fn render_category_cell(events: Option<&Vec<&TimelineEvent>>, is_collapsed: bool) -> Html {
        if is_collapsed {
            return html! {
                <div class="timeline-category-cell is-collapsed">
                    <span class="timeline-category-placeholder">{"Collapsed"}</span>
                </div>
            };
        }

        let Some(events) = events else {
            return html! {
                <div class="timeline-category-cell is-empty">
                    <span class="timeline-category-placeholder">{"--"}</span>
                </div>
            };
        };

        if events.is_empty() {
            return html! {
                <div class="timeline-category-cell is-empty">
                    <span class="timeline-category-placeholder">{"--"}</span>
                </div>
            };
        }

        let grouped = group_events_by_title(events.as_slice());

        html! {
            <div class="timeline-category-cell">
                {
                    for grouped.iter().map(|group| render_grouped_category(group))
                }
            </div>
        }
    }

    fn group_events_by_title<'a>(events: &'a [&'a TimelineEvent]) -> Vec<GroupedEvents<'a>> {
        let mut grouped: BTreeMap<String, Vec<&'a TimelineEvent>> = BTreeMap::new();

        for event in events {
            grouped.entry(event.title.clone()).or_default().push(*event);
        }

        let mut groups: Vec<GroupedEvents<'a>> = grouped
            .into_iter()
            .map(|(title, mut list)| {
                list.sort_by(|a, b| compare_datetimes(b.occurred_at, a.occurred_at));
                GroupedEvents { title, events: list }
            })
            .collect();

        groups.sort_by(|a, b| {
            let latest_a = a
                .events
                .first()
                .and_then(|event| event.occurred_at);
            let latest_b = b
                .events
                .first()
                .and_then(|event| event.occurred_at);
            compare_datetimes(latest_b, latest_a)
        });

        groups
    }

    fn render_grouped_category(group: &GroupedEvents<'_>) -> Html {
        let severity = group
            .events
            .iter()
            .fold(Severity::Info, |current, event| {
                if event.severity < current {
                    event.severity
                } else {
                    current
                }
            });
        let severity_label = severity_label(severity);
        let severity_level = severity_level(severity);
        let count = group.events.len();
        let count_label = if count == 1 {
            "1 entry".to_string()
        } else {
            format!("{count} entries")
        };

        let latest = group
            .events
            .first()
            .and_then(|event| event.occurred_at);
        let earliest = group
            .events
            .last()
            .and_then(|event| event.occurred_at);

        let range_label = format_time_range(earliest, latest);
        let relative_label = format_relative_time(latest);
        let chart_data = collect_measurement_series(group.events.as_slice());
        let (chart_html, has_chart) = if let Some(data) = chart_data.as_ref() {
            (
                render_measurement_panel(data, severity, ChartMode::TimelinePerDay),
                true,
            )
        } else {
            (Html::default(), false)
        };

        let mut meta: Vec<Html> = Vec::new();

        if let Some(range) = range_label {
            meta.push(html! {
                <span class="timeline-group-range">{ format!("Range {range}") }</span>
            });
        }

        if let Some(relative) = relative_label {
            meta.push(html! {
                <span class="timeline-group-relative">{ format!("Last reading {relative}") }</span>
            });
        }

        let key = group
            .events
            .first()
            .map(|event| event.id.clone())
            .unwrap_or_else(|| group.title.clone());

        html! {
            <div class="timeline-category-group" key={key}>
                <header class="timeline-group-header">
                    <span class="timeline-group-title">{ group.title.clone() }</span>
                    <span class="timeline-group-count">{ count_label }</span>
                    <span class="severity-badge" data-level={severity_level}>{ severity_label }</span>
                </header>
                {
                    if meta.is_empty() {
                        Html::default()
                    } else {
                        html! {
                            <div class="timeline-group-meta">
                                { for meta }
                            </div>
                        }
                    }
                }
                { chart_html }
                {
                    if has_chart {
                        let summary_label = if count == 1 {
                            "View entry".to_string()
                        } else {
                            format!("View {count} entries")
                        };

                        html! {
                            <details class="timeline-group-details">
                                <summary>{ summary_label }</summary>
                                <ul class="timeline-cell-list">
                                    { for group.events.iter().map(|event| render_event(*event)) }
                                </ul>
                            </details>
                        }
                    } else {
                        html! {
                            <ul class="timeline-cell-list">
                                { for group.events.iter().map(|event| render_event(*event)) }
                            </ul>
                        }
                    }
                }
            </div>
        }
    }
        fn collect_measurement_series<'a>(
            events: &[&'a TimelineEvent],
        ) -> Option<MeasurementChartData<'a>> {
            let mut series_map: BTreeMap<String, Vec<SeriesPoint<'a>>> = BTreeMap::new();
            let mut unit: Option<String> = None;
            let mut start: Option<DateTime<Utc>> = None;
            let mut end: Option<DateTime<Utc>> = None;

            for event in events {
                let detail = match event.detail.as_ref() {
                    Some(detail) => detail,
                    None => continue,
                };

                let timestamp = match event.occurred_at {
                    Some(timestamp) => timestamp,
                    None => continue,
                };

                let parsed = parse_measurement_values(&event.title, detail);
                let Some(parsed) = parsed else { continue };

                if start.map_or(true, |current| timestamp < current) {
                    start = Some(timestamp);
                }
                if end.map_or(true, |current| timestamp > current) {
                    end = Some(timestamp);
                }

                if unit.is_none() {
                    unit = parsed.unit.clone();
                }

                for value in parsed.values {
                    series_map
                        .entry(value.label.clone())
                        .or_default()
                        .push(SeriesPoint {
                            timestamp,
                            value: value.value,
                            _marker: std::marker::PhantomData,
                        });
                }
            }

            if series_map.is_empty() {
                return None;
            }

            let start = start?;
            let end = end?;

            let mut series: Vec<MeasurementSeries<'a>> = series_map
                .into_iter()
                .map(|(label, mut points)| {
                    points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                    MeasurementSeries { label, points }
                })
                .collect();

            series.sort_by(|a, b| match (a.label.as_str(), b.label.as_str()) {
                ("Systolic", "Diastolic") => std::cmp::Ordering::Less,
                ("Diastolic", "Systolic") => std::cmp::Ordering::Greater,
                _ => a.label.cmp(&b.label),
            });

            let total_points: usize = series.iter().map(|series| series.points.len()).sum();
            if total_points < 2 {
                return None;
            }

            Some(MeasurementChartData { unit, series, start, end })
        }

        fn parse_measurement_values(title: &str, detail: &str) -> Option<ParsedMeasurement> {
            let trimmed = detail.trim();
            if trimmed.is_empty() {
                return None;
            }

            let lower_title = title.to_lowercase();

            if lower_title.contains("blood pressure") && trimmed.contains('/') {
                let mut parts = trimmed.splitn(2, '/');
                let systolic_part = parts.next()?.trim();
                let diastolic_part = parts.next()?.trim();

                let (systolic_value, _) = parse_value_and_unit(systolic_part)?;
                let (diastolic_value, unit) = parse_value_and_unit(diastolic_part)?;

                return Some(ParsedMeasurement {
                    unit,
                    values: vec![
                        NamedValue {
                            label: "Systolic".to_string(),
                            value: systolic_value,
                        },
                        NamedValue {
                            label: "Diastolic".to_string(),
                            value: diastolic_value,
                        },
                    ],
                });
            }

            let (value, unit) = parse_value_and_unit(trimmed)?;
            let label = if lower_title.is_empty() {
                "Measurement".to_string()
            } else {
                title.to_string()
            };

            Some(ParsedMeasurement {
                unit,
                values: vec![NamedValue { label, value }],
            })
        }

        fn parse_value_and_unit(segment: &str) -> Option<(f64, Option<String>)> {
            let mut chars = segment.chars().peekable();

            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_digit() || ch == '.' || ch == '-' {
                    break;
                }
                chars.next();
            }

            let mut number = String::new();

            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_digit() || ch == '.' || (ch == '-' && number.is_empty()) {
                    number.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }

            if number.is_empty() {
                return None;
            }

            let value = number.parse::<f64>().ok()?;
            let remainder: String = chars.collect();
            let unit = remainder.trim();
            let unit = if unit.is_empty() {
                None
            } else {
                Some(unit.to_string())
            };

            Some((value, unit))
        }

    fn render_measurement_panel(
        data: &MeasurementChartData<'_>,
        severity: Severity,
        mode: ChartMode,
    ) -> Html {
        let severity_level = severity_level(severity);
        let chart = build_measurement_chart(data, severity, mode);
        let unit_hint = data.unit.as_deref();

        let stats = data
            .series
            .iter()
            .map(|series| render_series_stats(series, unit_hint))
            .collect::<Vec<_>>();

        html! {
            <div class="timeline-group-chart" data-severity={severity_level}>
                <div class="timeline-group-stats" data-series-count={data.series.len().to_string()}>
                    { for stats }
                </div>
                { chart }
            </div>
        }
    }

    fn build_measurement_chart(
        data: &MeasurementChartData<'_>,
        severity: Severity,
        mode: ChartMode,
    ) -> Html {
        const VIEW_WIDTH: f64 = 260.0;
        const VIEW_HEIGHT: f64 = 120.0;
        const LEFT_PAD: f64 = 52.0;
        const RIGHT_PAD: f64 = 16.0;
        const TOP_PAD: f64 = 14.0;
        const BOTTOM_PAD: f64 = 34.0;

        let plot_width = VIEW_WIDTH - LEFT_PAD - RIGHT_PAD;
        let plot_height = VIEW_HEIGHT - TOP_PAD - BOTTOM_PAD;
        let severity_class = format!("is-{}", severity_level(severity));

        let mut min_value = f64::INFINITY;
        let mut max_value = f64::NEG_INFINITY;
        for series in &data.series {
            for point in &series.points {
                min_value = min_value.min(point.value);
                max_value = max_value.max(point.value);
            }
        }

        if !min_value.is_finite() || !max_value.is_finite() {
            return Html::default();
        }

        if (max_value - min_value).abs() < f64::EPSILON {
            let padding = (max_value.abs().max(1.0)) * 0.05;
            min_value -= padding;
            max_value += padding;
        }

        let span = (max_value - min_value).abs();
        let padding = if span < 5.0 { 1.0 } else { span * 0.1 };
        let axis_min = min_value - padding;
        let axis_max = max_value + padding;
        let axis_range = (axis_max - axis_min).max(1e-3);

        let duration = data.end.signed_duration_since(data.start);
        let mut total_seconds = duration.num_seconds() as f64;
        if total_seconds.abs() < 60.0 {
            total_seconds = 60.0;
        }

        let mut path_elements: Vec<Html> = Vec::new();
        let mut point_elements: Vec<Html> = Vec::new();

        for (index, series) in data.series.iter().enumerate() {
            let mut path = String::new();
            let mut first_point = true;

            for point in &series.points {
                let offset = point
                    .timestamp
                    .signed_duration_since(data.start)
                    .num_seconds() as f64;
                let ratio_x = (offset / total_seconds).clamp(0.0, 1.0);
                let x = LEFT_PAD + ratio_x * plot_width;

                let ratio_y = ((point.value - axis_min) / axis_range).clamp(0.0, 1.0);
                let y = TOP_PAD + (1.0 - ratio_y) * plot_height;

                if first_point {
                    path.push_str(&format!("M{:.2},{:.2}", x, y));
                    first_point = false;
                } else {
                    path.push_str(&format!(" L{:.2},{:.2}", x, y));
                }

                let point_class = classes!(
                    "timeline-chart-point",
                    severity_class.clone(),
                    (index > 0).then_some("is-secondary")
                );

                let tooltip_time = match mode {
                    ChartMode::TimelinePerDay => format_clock_time(point.timestamp),
                    ChartMode::SummaryByDay => point.timestamp.format("%m/%d %H:%M").to_string(),
                };

                let tooltip_value = format_measurement(point.value, data.unit.as_deref());

                point_elements.push(html! {
                    <circle
                        class={point_class.clone()}
                        cx={format!("{:.2}", x)}
                        cy={format!("{:.2}", y)}
                        r="3"
                    >
                        <title>{ format!("{tooltip_time} – {tooltip_value}") }</title>
                    </circle>
                });
            }

            if !path.is_empty() {
                let line_class = classes!(
                    "timeline-chart-line",
                    severity_class.clone(),
                    (index > 0).then_some("is-secondary")
                );

                path_elements.push(html! {
                    <path class={line_class} d={path.clone()} />
                });
            }
        }

        let y_ticks = build_value_ticks(axis_min, axis_max, data.unit.as_deref());
        let x_ticks = build_time_ticks(data, mode, total_seconds);

        let grid_lines: Vec<Html> = y_ticks
            .iter()
            .map(|(value, label)| {
                let ratio = ((value - axis_min) / axis_range).clamp(0.0, 1.0);
                let y = TOP_PAD + (1.0 - ratio) * plot_height;
                html! {
                    <g class="timeline-chart-grid-row">
                        <line
                            x1={format!("{:.2}", LEFT_PAD)}
                            y1={format!("{:.2}", y)}
                            x2={format!("{:.2}", LEFT_PAD + plot_width)}
                            y2={format!("{:.2}", y)}
                        />
                        <text
                            x={format!("{:.2}", LEFT_PAD - 8.0)}
                            y={format!("{:.2}", y + 4.0)}
                            class="timeline-chart-tick"
                        >
                            { label.clone() }
                        </text>
                    </g>
                }
            })
            .collect();

        let column_lines: Vec<Html> = x_ticks
            .iter()
            .map(|(ratio, label)| {
                let x = LEFT_PAD + ratio * plot_width;
                html! {
                    <g class="timeline-chart-grid-col">
                        <line
                            x1={format!("{:.2}", x)}
                            y1={format!("{:.2}", TOP_PAD)}
                            x2={format!("{:.2}", x)}
                            y2={format!("{:.2}", TOP_PAD + plot_height)}
                        />
                        <text
                            x={format!("{:.2}", x)}
                            y={format!("{:.2}", TOP_PAD + plot_height + 16.0)}
                            class="timeline-chart-tick"
                        >
                            { label.clone() }
                        </text>
                    </g>
                }
            })
            .collect();

        let mode_label = match mode {
            ChartMode::TimelinePerDay => "timeline",
            ChartMode::SummaryByDay => "summary",
        };

        html! {
            <div class="timeline-group-plot" data-mode={mode_label}>
                <svg
                    viewBox={format!("0 0 {:.0} {:.0}", VIEW_WIDTH, VIEW_HEIGHT)}
                    class="timeline-group-chart-plot"
                    role="img"
                    aria-hidden="true"
                >
                    <rect
                        class="timeline-chart-surface"
                        x={format!("{:.2}", LEFT_PAD)}
                        y={format!("{:.2}", TOP_PAD)}
                        width={format!("{:.2}", plot_width)}
                        height={format!("{:.2}", plot_height)}
                        rx="6"
                        ry="6"
                    />
                    { for grid_lines }
                    { for column_lines }
                    <line
                        class="timeline-chart-axis-line"
                        x1={format!("{:.2}", LEFT_PAD)}
                        y1={format!("{:.2}", TOP_PAD + plot_height)}
                        x2={format!("{:.2}", LEFT_PAD + plot_width)}
                        y2={format!("{:.2}", TOP_PAD + plot_height)}
                    />
                    <line
                        class="timeline-chart-axis-line"
                        x1={format!("{:.2}", LEFT_PAD)}
                        y1={format!("{:.2}", TOP_PAD)}
                        x2={format!("{:.2}", LEFT_PAD)}
                        y2={format!("{:.2}", TOP_PAD + plot_height)}
                    />
                    { for path_elements }
                    { for point_elements }
                </svg>
            </div>
        }
    }

    fn build_value_ticks(min: f64, max: f64, unit: Option<&str>) -> Vec<(f64, String)> {
        if !min.is_finite() || !max.is_finite() {
            return Vec::new();
        }

        let mut values = Vec::new();
        let mut push_value = |value: f64| {
            if values
                .iter()
                .all(|existing: &f64| (existing - value).abs() > 0.05)
            {
                values.push(value);
            }
        };

        push_value(min);
        push_value((min + max) / 2.0);
        push_value(max);

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        values
            .into_iter()
            .map(|value| (value, format_axis_value(value, unit)))
            .collect()
    }

    fn build_time_ticks(
        data: &MeasurementChartData<'_>,
        mode: ChartMode,
        total_seconds: f64,
    ) -> Vec<(f64, String)> {
        match mode {
            ChartMode::TimelinePerDay => build_hour_ticks(data, total_seconds),
            ChartMode::SummaryByDay => build_day_ticks(data, total_seconds),
        }
    }

    fn build_hour_ticks(
        data: &MeasurementChartData<'_>,
        total_seconds: f64,
    ) -> Vec<(f64, String)> {
        let mut times: Vec<DateTime<Utc>> = data
            .series
            .iter()
            .flat_map(|series| series.points.iter().map(|point| point.timestamp))
            .collect();

        times.sort();
        times.dedup();

        if times.is_empty() {
            times.push(data.start);
            times.push(data.end);
        } else {
            if *times.first().unwrap() != data.start {
                times.insert(0, data.start);
            }
            if *times.last().unwrap() != data.end {
                times.push(data.end);
            }
        }

        if times.len() > 5 {
            let mid = data.start + Duration::seconds((total_seconds / 2.0).round() as i64);
            times = vec![data.start, mid, data.end];
        }

        times
            .into_iter()
            .map(|timestamp| {
                let offset = timestamp
                    .signed_duration_since(data.start)
                    .num_seconds() as f64;
                let ratio = (offset / total_seconds).clamp(0.0, 1.0);
                (ratio, format_clock_time(timestamp))
            })
            .collect()
    }

    fn build_day_ticks(
        data: &MeasurementChartData<'_>,
        total_seconds: f64,
    ) -> Vec<(f64, String)> {
        let mut days: BTreeMap<NaiveDate, DateTime<Utc>> = BTreeMap::new();

        for series in &data.series {
            for point in &series.points {
                days.entry(point.timestamp.date_naive())
                    .or_insert(point.timestamp);
            }
        }

        if days.is_empty() {
            days.insert(data.start.date_naive(), data.start);
            days.insert(data.end.date_naive(), data.end);
        }

        let mut ticks: Vec<(f64, String)> = Vec::new();

        for timestamp in days.values() {
            let offset = timestamp
                .signed_duration_since(data.start)
                .num_seconds() as f64;
            let ratio = (offset / total_seconds).clamp(0.0, 1.0);
            ticks.push((ratio, timestamp.format("%m/%d").to_string()));
        }

        ticks
    }

    fn render_series_stats(series: &MeasurementSeries<'_>, unit: Option<&str>) -> Html {
        if series.points.is_empty() {
            return Html::default();
        }

        let latest = series.points.last().unwrap();
        let first = series.points.first().unwrap();

        let mut min_value = f64::INFINITY;
        let mut max_value = f64::NEG_INFINITY;

        for point in &series.points {
            min_value = min_value.min(point.value);
            max_value = max_value.max(point.value);
        }

        let latest_label = format_measurement(latest.value, unit);
        let min_label = format_measurement(min_value, unit);
        let max_label = format_measurement(max_value, unit);
        let (delta_label, delta_trend) = format_delta_display(latest.value - first.value, unit);

        html! {
            <div class="timeline-group-stat-block">
                <div class="stat-header">
                    <span class="stat-label">{ series.label.clone() }</span>
                    <span class="stat-value">{ latest_label }</span>
                </div>
                <div class="stat-meta">
                    <span class="stat-delta" data-trend={delta_trend}>{ delta_label }</span>
                    <span class="stat-range">{ format!("Low {min_label} • High {max_label}") }</span>
                </div>
            </div>
        }
    }

    fn format_axis_value(value: f64, unit: Option<&str>) -> String {
        let numeric = format_numeric(value);
        match unit {
            Some(unit) if !unit.is_empty() => format!("{numeric} {unit}"),
            _ => numeric,
        }
    }

    fn format_delta_display(delta: f64, unit: Option<&str>) -> (String, &'static str) {
        let threshold = 0.1;
        if delta.abs() < threshold {
            return ("Δ 0".to_string(), "steady");
        }

        let arrow = if delta > 0.0 { "↑" } else { "↓" };
        let magnitude = format_numeric(delta.abs());
        let unit_suffix = unit.map(|u| format!(" {u}")).unwrap_or_default();
        let label = format!("{arrow}{magnitude}{unit_suffix}");
        let trend = if delta > 0.0 { "up" } else { "down" };
        (label, trend)
    }

    fn bucket_slug(bucket: &str) -> &'static str {
        match bucket {
            "Vitals" => "vitals",
            "Labs" => "labs",
            "Imaging" => "imaging",
            "Observations" => "observations",
            "Medications" => "medications",
            "Procedures" => "procedures",
            "Encounters" => "encounters",
            "Conditions" => "conditions",
            "Documents" => "documents",
            "Notes" => "notes",
            _ => "other",
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

    fn group_storage_key(label: &str, events: &[&TimelineEvent]) -> String {
        let first_id = events
            .first()
            .map(|event| event.id.as_str())
            .unwrap_or("");
        format!("{label}-{first_id}")
    }

    fn should_collapse_group(index: usize, label: &str, events: &[&TimelineEvent]) -> bool {
        if index == 0 || events.len() <= 2 {
            return false;
        }

        label != "Today" && label != "Yesterday"
    }

    fn summarize_group(events: &[&TimelineEvent]) -> String {
        let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();

        for event in events {
            let bucket = categorize_event_for_summary(event);
            *counts.entry(bucket).or_insert(0) += 1;
        }

        let mut phrases: Vec<String> = counts
            .into_iter()
            .map(|(bucket, count)| format_bucket_phrase(bucket, count))
            .collect();
        phrases.sort();
        phrases.join(", ")
    }

    fn categorize_event_for_summary(event: &TimelineEvent) -> &'static str {
        match event.category {
            EventCategory::Observation => {
                let title = event.title.to_lowercase();
                if is_vital_title(&title) {
                    "Vitals"
                } else if is_imaging_title(&title) {
                    "Imaging"
                } else if is_lab_title(&title) {
                    "Labs"
                } else {
                    "Observations"
                }
            }
            EventCategory::Medication => "Medications",
            EventCategory::Condition => "Conditions",
            EventCategory::Procedure => "Procedures",
            EventCategory::Encounter => "Encounters",
            EventCategory::Document => "Documents",
            EventCategory::Note => "Notes",
            EventCategory::Other => "Events",
        }
    }

    fn is_vital_title(title: &str) -> bool {
        VITAL_EVENT_KEYWORDS.iter().any(|keyword| title.contains(keyword))
    }

    fn is_lab_title(title: &str) -> bool {
        LAB_EVENT_KEYWORDS.iter().any(|keyword| title.contains(keyword))
    }

    fn is_imaging_title(title: &str) -> bool {
        IMAGING_EVENT_KEYWORDS
            .iter()
            .any(|keyword| title.split_whitespace().any(|token| token == *keyword))
    }

    fn format_bucket_phrase(bucket: &str, count: usize) -> String {
        match bucket {
            "Vitals" => pluralize(count, "vital", "vitals"),
            "Labs" => pluralize(count, "lab", "labs"),
            "Imaging" => pluralize(count, "study", "studies"),
            "Observations" => pluralize(count, "observation", "observations"),
            "Medications" => pluralize(count, "medication", "medications"),
            "Conditions" => pluralize(count, "condition", "conditions"),
            "Procedures" => pluralize(count, "procedure", "procedures"),
            "Encounters" => pluralize(count, "encounter", "encounters"),
            "Documents" => pluralize(count, "document", "documents"),
            "Notes" => pluralize(count, "note", "notes"),
            _ => pluralize(count, "event", "events"),
        }
    }

    fn pluralize(count: usize, singular: &str, plural: &str) -> String {
        if count == 1 {
            format!("1 {singular}")
        } else {
            format!("{count} {plural}")
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
