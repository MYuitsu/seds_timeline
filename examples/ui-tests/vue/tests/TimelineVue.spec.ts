import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import type { TimelineSnapshot } from "../../../../timeline-wasm/types/index";
import { setMountImplementation } from "../src/bindings";
import { TimelineVue } from "../src/TimelineVue";

const SAMPLE_SNAPSHOT: TimelineSnapshot = {
  generated_at: "2025-11-02T11:00:00Z",
  critical: {
    allergies: [],
    medications: [{ label: "Heparin", severity: "moderate" }],
    chronic_conditions: [],
    code_status: null,
    alerts: [],
    recent_vitals: []
  },
  events: []
};

describe("TimelineVue", () => {
  it("gọi mountTimelineView khi component mounted", async () => {
    const spy = vi.fn();
    setMountImplementation(spy);

    const wrapper = mount(TimelineVue, {
      props: {
        snapshot: SAMPLE_SNAPSHOT
      }
    });

    const container = wrapper.find('[data-testid="timeline-vue-container"]');
    expect(container.exists()).toBe(true);
    expect(container.attributes().id).toBeTruthy();
    expect(spy).toHaveBeenCalledWith(`#${container.attributes().id}`, SAMPLE_SNAPSHOT);
  });
});
