declare module "../../../pkg/timeline-wasm/timeline_wasm.js" {
  export default function init(module?: RequestInfo): Promise<unknown>;
  export function summarize_bundle(bundle: unknown, config?: unknown): any;
}

declare module "../../../pkg/timeline-ui/timeline_ui.js" {
  export default function init(module?: RequestInfo): Promise<unknown>;
  export function mount_timeline_view(selector: string, snapshot: any): void;
}
