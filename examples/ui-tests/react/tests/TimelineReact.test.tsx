import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { TimelineSnapshot } from "../../../../timeline-wasm/types/index";
import { TimelineReact } from "../src/TimelineReact";
import { setMountImplementation } from "../src/bindings";

const SAMPLE_SNAPSHOT: TimelineSnapshot = {
  generated_at: "2025-11-02T10:00:00Z",
  critical: {
    allergies: [{ label: "Penicillin", severity: "critical" }],
    medications: [],
    chronic_conditions: [],
    code_status: null,
    alerts: [],
    recent_vitals: [],
  },
  events: [],
};

describe("TimelineReact", () => {
  it("gọi mountTimelineView với selector và snapshot", () => {
    const mountSpy = vi.fn();
    setMountImplementation(mountSpy);

    const { getByTestId } = render(<TimelineReact snapshot={SAMPLE_SNAPSHOT} />);
    const container = getByTestId("timeline-react-container");

    expect(container.id).toBeTruthy();
    expect(mountSpy).toHaveBeenCalledWith(`#${container.id}`, SAMPLE_SNAPSHOT);
  });
});
