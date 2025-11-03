//! Thành phần giao diện timeline cho môi trường WebAssembly.

use timeline_core::TimelineSnapshot;

#[cfg(target_arch = "wasm32")]
mod styles;

#[cfg(target_arch = "wasm32")]
mod wasm_ui {
    use super::TimelineSnapshot;
    use crate::styles;
    use serde_wasm_bindgen::from_value;
    use wasm_bindgen::prelude::*;
    use web_sys::{console, Document, Element, Window};
    use yew::prelude::*;

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

        html! {
            <div class="timeline-root">
                <section class="critical-panel">
                    <h2>{"Critical Overview"}</h2>
                    <ul class="critical-list">
                        { for snapshot.critical.alerts.iter().map(render_item) }
                        { for snapshot.critical.allergies.iter().map(render_item) }
                        { for snapshot.critical.medications.iter().map(render_item) }
                    </ul>
                </section>
                <section class="timeline-section">
                    <h2>{"Timeline"}</h2>
                    <ul class="timeline-events">
                        { for snapshot.events.iter().map(render_event) }
                    </ul>
                </section>
            </div>
        }
    }

    fn render_item(item: &timeline_core::CriticalItem) -> Html {
        html! {
            <li class="critical-item">
                <span class="critical-label">{ item.label.clone() }</span>
                {
                    item.detail
                        .as_ref()
                        .map(|detail| html! {<span class="critical-detail">{ detail.clone() }</span>})
                        .unwrap_or_default()
                }
            </li>
        }
    }

    fn render_event(event: &timeline_core::TimelineEvent) -> Html {
        let severity_label = format!("{:?}", event.severity);
        let severity_level = match event.severity {
            timeline_core::Severity::Critical => "critical",
            timeline_core::Severity::High => "high",
            timeline_core::Severity::Moderate => "moderate",
            timeline_core::Severity::Low => "low",
            timeline_core::Severity::Info => "info",
        };
        let timestamp = event
            .occurred_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "--".to_string());

        html! {
            <li class="timeline-event">
                <div class="timeline-meta">
                    <span class="timeline-time">{ timestamp }</span>
                    <span class="timeline-category">{ format!("{:?}", event.category) }</span>
                    <span class="timeline-severity" data-level={severity_level}>{ severity_label }</span>
                </div>
                <div class="timeline-body">
                    <h3 class="timeline-title">{ event.title.clone() }</h3>
                    {
                        event.detail
                            .as_ref()
                            .map(|detail| html! {<p class="timeline-detail">{ detail.clone() }</p>})
                            .unwrap_or_default()
                    }
                </div>
            </li>
        }
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
