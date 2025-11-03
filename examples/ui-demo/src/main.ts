/// <reference types="vite/client" />

import initTimelineWasm, { summarize_bundle } from "../../../pkg/timeline-wasm/timeline_wasm.js";
import initTimelineUi, { mount_timeline_view } from "../../../pkg/timeline-ui/timeline_ui.js";

async function loadBundle() {
  const response = await fetch("/sample_bundle.json");
  if (!response.ok) {
    throw new Error(`Không tải được sample_bundle.json: ${response.status}`);
  }
  return (await response.json()) as unknown;
}

async function bootstrap() {
  const statusEl = document.createElement("div");
  statusEl.id = "timeline-status";
  statusEl.textContent = "Đang tải mô-đun WASM...";
  document.body.prepend(statusEl);

  await Promise.all([initTimelineWasm(), initTimelineUi()]);

  statusEl.textContent = "Đang xây dựng bản tóm tắt...";

  const bundle = await loadBundle();
  const snapshot = summarize_bundle(bundle);

  statusEl.textContent = "Đang mount giao diện...";

  mount_timeline_view("#timeline-root", snapshot);

  statusEl.remove();
}

bootstrap().catch((err) => {
  const statusEl = document.getElementById("timeline-status") ?? document.createElement("div");
  statusEl.id = "timeline-status";
  statusEl.style.color = "red";
  statusEl.textContent = `Lỗi: ${String(err)}`;
  document.body.prepend(statusEl);
  // eslint-disable-next-line no-console
  console.error(err);
});
