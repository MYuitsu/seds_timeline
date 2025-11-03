import { ComponentFixture, TestBed } from "@angular/core/testing";
import { describe, expect, it, vi } from "vitest";
import type { TimelineSnapshot } from "../../../../timeline-wasm/types/index";
import { setMountImplementation } from "../src/bindings";
import { TimelineAngularComponent } from "../src/timeline-angular.component";

const SAMPLE_SNAPSHOT: TimelineSnapshot = {
  generated_at: "2025-11-02T10:00:00Z",
  critical: {
    allergies: [{ label: "Morphine", severity: "high" }],
    medications: [],
    chronic_conditions: [],
    code_status: null,
    alerts: [],
    recent_vitals: []
  },
  events: []
};

describe("TimelineAngularComponent", () => {
  it("gọi mountTimelineView sau khi view init", async () => {
    const spy = vi.fn();
    setMountImplementation(spy);

    await TestBed.configureTestingModule({
      declarations: [TimelineAngularComponent]
    }).compileComponents();

    const fixture: ComponentFixture<TimelineAngularComponent> = TestBed.createComponent(
      TimelineAngularComponent,
    );
    fixture.componentInstance.snapshot = SAMPLE_SNAPSHOT;
    fixture.detectChanges();

    const host = fixture.nativeElement.querySelector(".timeline-angular-host");
    expect(host).toBeTruthy();
    expect(host.id).toBeTruthy();
    expect(spy).toHaveBeenCalledWith(`#${host.id}`, SAMPLE_SNAPSHOT);
  });
});
