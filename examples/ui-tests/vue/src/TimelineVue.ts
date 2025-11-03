import { defineComponent, h, onMounted, ref } from "vue";
import type { TimelineSnapshot } from "../../../../timeline-wasm/types/index";
import { mountTimelineView } from "./bindings";

export const TimelineVue = defineComponent({
  name: "TimelineVue",
  props: {
    snapshot: {
      type: Object as () => TimelineSnapshot,
      required: true
    }
  },
  setup(props: { snapshot: TimelineSnapshot }) {
    const elementId = ref(`timeline-vue-${Math.random().toString(36).slice(2)}`);

    onMounted(() => {
      mountTimelineView(`#${elementId.value}`, props.snapshot);
    });

    return () =>
      h("div", {
        id: elementId.value,
        "data-testid": "timeline-vue-container"
      });
  }
});
