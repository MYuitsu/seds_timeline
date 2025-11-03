//! Thành phần giao diện timeline cho môi trường WebAssembly.

#[cfg(target_arch = "wasm32")]
mod styles;

#[cfg(target_arch = "wasm32")]
mod wasm_ui {
    use crate::styles;
    use chrono::{Duration, Utc};
    use serde_wasm_bindgen::from_value;
    use timeline_core::{
        CriticalItem, CriticalSummary, Severity, TimelineEvent, TimelineSnapshot, VitalSnapshot,
    };
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
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
        let filtered_events: Vec<&TimelineEvent> = snapshot
            .events
            .iter()
            .filter(|event| event_matches_filters(event, &filters_value))
            .collect();

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

        html! {
            <div class="timeline-root">
                <aside class="critical-column">
                    <header class="critical-header">
                        <span class="critical-eyebrow">{"Trạng thái cấp cứu"}</span>
                        <h2>{"Thông tin ưu tiên"}</h2>
                        <p>{"Các điểm cần chú ý ngay cho bác sĩ cấp cứu."}</p>
                    </header>
                    { render_code_status(&snapshot.critical) }
                    { render_critical_card("Cảnh báo lâm sàng", &snapshot.critical.alerts, "Không có cảnh báo khẩn") }
                    { render_critical_card("Dị ứng nghiêm trọng", &snapshot.critical.allergies, "Chưa ghi nhận dị ứng nguy hiểm") }
                    { render_critical_card("Thuốc đang dùng", &snapshot.critical.medications, "Không có thuốc hoạt động" ) }
                    { render_critical_card("Bệnh mạn quan trọng", &snapshot.critical.chronic_conditions, "Chưa ghi nhận bệnh nền nguy cơ") }
                    { render_vitals(&snapshot.critical.recent_vitals) }
                </aside>
                <section class="timeline-column" aria-live="polite">
                    <header class="timeline-toolbar">
                        <div class="toolbar-group">
                            <span class="toolbar-label">{"Bộ lọc"}</span>
                            { severity_controls }
                        </div>
                        <div class="toolbar-search">
                            <input
                                type="search"
                                placeholder="Lọc theo từ khóa (ví dụ: huyết áp, sepsis)"
                                value={filters_value.query.clone()}
                                oninput={on_search}
                                aria-label="Tìm kiếm sự kiện theo từ khóa"
                            />
                            <button type="button" onclick={on_clear_filters.clone()} aria-label="Xóa bộ lọc">{"Đặt lại"}</button>
                        </div>
                    </header>
                    <ul class="timeline-events">
                        {
                            if filtered_events.is_empty() {
                                html! {
                                    <li class="timeline-empty">{"Không có sự kiện nào khớp bộ lọc hiện tại."}</li>
                                }
                            } else {
                                html! { for filtered_events.into_iter().map(render_event) }
                            }
                        }
                    </ul>
                </section>
            </div>
        }
    }

    fn render_severity_filters(filters: UseStateHandle<FilterState>) -> Html {
        let options = [
            (Severity::Critical, "Chỉ Critical"),
            (Severity::High, "≥ High"),
            (Severity::Moderate, "≥ Moderate"),
        ];

        html! {
            <div class="filter-chips" role="group" aria-label="Lọc theo mức độ nghiêm trọng">
                {
                    for options.into_iter().map(|(level, label)| {
                        let filters = filters.clone();
                        let is_active = filters.severity == Some(level);
                        let level_attr = severity_level(level);
                        let onclick = Callback::from(move |_| {
                            let mut next = (*filters).clone();
                            if next.severity == Some(level) {
                                next.severity = None;
                            } else {
                                next.severity = Some(level);
                            }
                            filters.set(next);
                        });

                        html! {
                            <button
                                type="button"
                                class={classes!("filter-chip", is_active.then_some("is-active"))}
                                data-level={level_attr}
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
        let (status_text, status_level) = match summary.code_status.as_ref() {
            Some(value) => (value.clone(), "critical"),
            None => ("Chưa có code status".to_string(), "info"),
        };

        html! {
            <section class="critical-card code-status">
                <header>
                    <h3>{"Code status"}</h3>
                    <span class="critical-pill" data-level={status_level}>{"Đảm bảo thông tin"}</span>
                </header>
                <p class={classes!("code-status-value", status_level)}>{ status_text }</p>
            </section>
        }
    }

    fn render_vitals(vitals: &[VitalSnapshot]) -> Html {
        html! {
            <section class="critical-card">
                <header>
                    <h3>{"Chỉ số sống gần đây"}</h3>
                    <span class="critical-count">{ vitals.len() }</span>
                </header>
                <ul class="vital-list">
                    {
                        if vitals.is_empty() {
                            html! { <li class="critical-empty">{"Chưa ghi nhận chỉ số sống trong khoảng thời gian cấu hình."}</li> }
                        } else {
                            html! { for vitals.iter().map(render_vital_item) }
                        }
                    }
                </ul>
            </section>
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
        html! {
            <li class="vital-item">
                <span class="vital-name">{ vital.name.clone() }</span>
                <span class="vital-value">{ vital.value.clone() }</span>
                <span class="vital-time">{ timestamp }</span>
            </li>
        }
    }

    fn render_event(event: &TimelineEvent) -> Html {
        let severity_label = severity_label(event.severity);
        let severity_level = severity_level(event.severity);
        let timestamp = format_timestamp(event.occurred_at);
        let relative = format_relative_time(event.occurred_at);
        let category = format!("{:?}", event.category);

        html! {
            <li class="timeline-event">
                <div class="timeline-meta">
                    <span class="timeline-time">{ timestamp }</span>
                    { relative.map(|text| html! { <span class="timeline-relative">{ text }</span> }).unwrap_or_default() }
                    <span class="timeline-category" data-category={category.to_lowercase()}>{ category }</span>
                    <span class="timeline-severity" data-level={severity_level}>{ severity_label }</span>
                </div>
                <div class="timeline-body">
                    <h3 class="timeline-title">{ event.title.clone() }</h3>
                    { event.detail.as_ref().map(render_event_detail).unwrap_or_default() }
                </div>
            </li>
        }
    }

    fn render_event_detail(detail: &String) -> Html {
        html! { <p class="timeline-detail">{ detail.clone() }</p> }
    }

    fn format_timestamp(timestamp: Option<chrono::DateTime<Utc>>) -> String {
        timestamp
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "--".to_string())
    }

    fn format_relative_time(timestamp: Option<chrono::DateTime<Utc>>) -> Option<String> {
        let timestamp = timestamp?;
        let now = Utc::now();
        let delta = now.signed_duration_since(timestamp);
        let label = if delta.num_seconds() >= 0 {
            "cách đây"
        } else {
            "trong"
        };
        let abs_delta: Duration = delta.abs();

        if abs_delta.num_days() >= 1 {
            Some(format!("{label} {} ngày", abs_delta.num_days()))
        } else if abs_delta.num_hours() >= 1 {
            Some(format!("{label} {} giờ", abs_delta.num_hours()))
        } else if abs_delta.num_minutes() >= 1 {
            Some(format!("{label} {} phút", abs_delta.num_minutes()))
        } else {
            Some(format!("{label} vài giây"))
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

        let mut haystack = vec![
            event.title.to_lowercase(),
            format!("{:?}", event.category).to_lowercase(),
        ];
        if let Some(detail) = &event.detail {
            haystack.push(detail.to_lowercase());
        }

        haystack.iter().any(|text| text.contains(&query))
    }

    #[wasm_bindgen]
    pub fn mount_timeline_view(selector: &str, snapshot: JsValue) -> Result<(), JsValue> {
        let window: Window =
            web_sys::window().ok_or_else(|| JsValue::from_str("Không có window"))?;
        let document: Document = window
            .document()
            .ok_or_else(|| JsValue::from_str("Không truy cập được document"))?;

        let target: Element = document
            .query_selector(selector)
            .map_err(|err| JsValue::from_str(&format!("Selector lỗi: {err:?}")))?
            .ok_or_else(|| JsValue::from_str("Không tìm thấy element theo selector"))?;

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
        "timeline-ui chỉ hỗ trợ biên dịch target wasm32",
    ))
}
