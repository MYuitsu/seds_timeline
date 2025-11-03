import { useEffect, useId } from "react";
import type { TimelineSnapshot } from "../../../../timeline-wasm/types/index";
import { mountTimelineView } from "./bindings";

export interface TimelineReactProps {
  snapshot: TimelineSnapshot;
}

export function TimelineReact({ snapshot }: TimelineReactProps) {
  const elementId = useId();

  useEffect(() => {
    mountTimelineView(`#${elementId}`, snapshot);
  }, [elementId, snapshot]);

  return <div id={elementId} data-testid="timeline-react-container" />;
}
