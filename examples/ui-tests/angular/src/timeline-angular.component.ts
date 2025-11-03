import { AfterViewInit, Component, ElementRef, Input } from "@angular/core";
import type { TimelineSnapshot } from "../../../../timeline-wasm/types/index";
import { mountTimelineView } from "./bindings";

@Component({
  selector: "timeline-angular",
  template: '<div class="timeline-angular-host"></div>'
})
export class TimelineAngularComponent implements AfterViewInit {
  @Input({ required: true }) snapshot!: TimelineSnapshot;

  constructor(private host: ElementRef<HTMLElement>) {}

  ngAfterViewInit(): void {
    const hostId = this.ensureId(this.host.nativeElement);
    mountTimelineView(`#${hostId}`, this.snapshot);
  }

  private ensureId(element: HTMLElement): string {
    if (!element.id) {
      element.id = `timeline-angular-${Math.random().toString(36).slice(2)}`;
    }
    return element.id;
  }
}
