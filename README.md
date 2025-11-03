# SEDS Timeline

Thư viện Rust giúp tóm tắt nhanh bệnh án theo chuẩn FHIR cho bối cảnh cấp cứu, kèm thành phần giao diện WebAssembly.

## Cấu trúc workspace

- `timeline-core`: mô hình dữ liệu và logic sắp xếp timeline.
- `timeline-fhir`: adapter để chuyển JSON FHIR thành `TimelineSnapshot`.
- `timeline-wasm`: bridge WASM/JavaScript với API trung lập framework.
- `timeline-ui`: web component dựng giao diện "Critical Overview" + timeline.
- `examples/cli`: ví dụ CLI xử lý bundle FHIR.
- `examples/ui-tests`: bộ test tích hợp UI giả lập cho React, Angular, Vue (dùng Vitest + jsdom).
- `examples/ui-demo`: ứng dụng Vite nhỏ hiển thị timeline trực tiếp trong trình duyệt.

## Chạy demo giao diện

1. Build các crate WASM:
	```powershell
	wasm-pack build timeline-wasm --target web --out-dir pkg/timeline-wasm --out-name timeline_wasm
	wasm-pack build timeline-ui --target web --out-dir pkg/timeline-ui --out-name timeline_ui
	```
2. Cài đặt phụ thuộc và chạy Vite dev server:
	```powershell
	cd examples/ui-demo
	npm install
	npm run dev
	```
3. Mở trình duyệt tới địa chỉ Vite hiển thị (mặc định `http://localhost:5173`) để xem component timeline render từ file `public/sample_bundle.json`.

## Trạng thái

Mới khởi tạo skeleton. Chưa có logic tóm tắt thực tế.
