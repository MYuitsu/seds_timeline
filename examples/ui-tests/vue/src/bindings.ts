import type { TimelineSnapshot } from "../../../../timeline-wasm/types/index";

type MountImpl = (selector: string, snapshot: TimelineSnapshot) => void;

let implementation: MountImpl | null = null;

export function setMountImplementation(fn: MountImpl) {
  implementation = fn;
}

export function mountTimelineView(selector: string, snapshot: TimelineSnapshot): void {
  if (!implementation) {
    throw new Error(
      "mountTimelineView chưa được cấu hình cho Vue. Gọi setMountImplementation trước.",
    );
  }

  implementation(selector, snapshot);
}
