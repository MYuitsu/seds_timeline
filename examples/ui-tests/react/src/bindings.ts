import type { TimelineSnapshot } from "../../../../timeline-wasm/types/index";

type MountImpl = (selector: string, snapshot: TimelineSnapshot) => void;

let implementation: MountImpl | null = null;

export function setMountImplementation(fn: MountImpl) {
  implementation = fn;
}

export function mountTimelineView(selector: string, snapshot: TimelineSnapshot): void {
  if (!implementation) {
    throw new Error(
      "mountTimelineView chưa được cấu hình. Gọi setMountImplementation với wasm mount_timeline_view trước.",
    );
  }

  implementation(selector, snapshot);
}
