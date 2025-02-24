<template>
  <div
    ref="panelRef"
    :class="
      clsx(
        'si-panel',
        'z-20 dark:text-white pointer-events-auto relative',
        isTopOrBottom
          ? 'border-shade-100 bg-white dark:bg-neutral-900'
          : 'dark:border-neutral-600 border-neutral-300 bg-white dark:bg-neutral-800',
        {
          left: 'border-r-2 ',
          right: 'border-l-2',
          top: 'border-b shadow-[0_4px_4px_0_rgba(0,0,0,0.15)]',
          bottom: 'border-t shadow-[0_-4px_4px_0_rgba(0,0,0,0.15)]',
        }[side],
        `${side}-0`,
      )
    "
    :style="{
      ...(resizeable && {
        [isTopOrBottom ? 'height' : 'width']: `${currentSize}px`,
      }),
    }"
  >
    <PanelResizingHandle
      v-if="resizeable"
      class="si-panel__resizer"
      :panelSide="side"
      @resize-start="onResizeStart"
      @resize-move="onResizeMove"
      @resize-reset="resetSize"
    />
    <div class="si-panel__inner absolute w-full h-full flex flex-col">
      <!-- most uses will just have a single child -->
      <slot />

      <!-- but here we give the option for 2 resizable child sections within -->
      <div
        v-if="$slots.subpanel1"
        :style="{
          height: disableSubpanelResizing
            ? 'auto'
            : `${subpanelSplitPercent * 100}%`,
        }"
        class="relative overflow-hidden"
      >
        <slot name="subpanel1" />
      </div>
      <PanelResizingHandle
        v-if="$slots.subpanel1 && $slots.subpanel2 && !disableSubpanelResizing"
        :panelSide="isTopOrBottom ? 'left' : 'top'"
        :class="isTopOrBottom ? 'h-full' : 'w-full'"
        :style="{ top: `${subpanelSplitPercent * 100}%` }"
        @resize-start="onSubpanelResizeStart"
        @resize-move="onSubpanelResizeMove"
        @resize-reset="onSubpanelResizeReset"
      />

      <div
        v-if="$slots.subpanel2"
        :style="{
          height: disableSubpanelResizing
            ? 'auto'
            : `${100 - subpanelSplitPercent * 100}%`,
        }"
        class="grow relative overflow-hidden"
      >
        <slot name="subpanel2" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, PropType, ref } from "vue";
import * as _ from "lodash-es";
import clsx from "clsx";
import PanelResizingHandle from "./PanelResizingHandle.vue";

const props = defineProps({
  rememberSizeKey: { type: String, required: true },
  side: {
    type: String as PropType<"left" | "right" | "top" | "bottom">,
    required: true,
  },
  hidden: { type: Boolean },
  sizeClasses: { type: String, default: "" },
  resizeable: { type: Boolean, default: true },
  minSizeRatio: { type: Number, default: 0.1 },
  minSize: { type: Number, default: 200 },
  maxSizeRatio: { type: Number, default: 0.45 },
  maxSize: { type: Number },
  defaultSize: { type: Number, default: 320 },
  disableSubpanelResizing: { type: Boolean },
  defaultSubpanelSplit: { type: Number, default: 0.5 },
});

const isTopOrBottom = computed(
  () => props.side === "top" || props.side === "bottom",
);

const panelRef = ref<HTMLDivElement>();
const currentSize = ref(0);

const setSize = (newSize: number) => {
  let finalSize = newSize;

  // TODO: make sure these checks don't conflict with each other

  if (props.minSize) {
    if (finalSize < props.minSize) finalSize = props.minSize;
  }
  if (props.minSizeRatio) {
    const limit =
      (isTopOrBottom.value ? window.innerHeight : window.innerWidth) *
      props.minSizeRatio;
    if (finalSize < limit) finalSize = limit;
  }

  if (props.maxSize) {
    if (finalSize > props.maxSize) finalSize = props.maxSize;
  }
  if (props.maxSizeRatio) {
    const limit =
      (isTopOrBottom.value ? window.innerHeight : window.innerWidth) *
      props.maxSizeRatio;

    if (finalSize > limit) finalSize = limit;
  }
  currentSize.value = finalSize;
  if (finalSize === props.defaultSize) {
    window.localStorage.removeItem(primarySizeLocalStorageKey.value);
  } else {
    window.localStorage.setItem(
      primarySizeLocalStorageKey.value,
      `${finalSize}`,
    );
  }
};

const primarySizeLocalStorageKey = computed(
  () => `${props.rememberSizeKey}-size`,
);
const subpanelSplitLocalStorageKey = computed(
  () => `${props.rememberSizeKey}-split`,
);

const beginResizeValue = ref(0);
const onResizeStart = () => {
  beginResizeValue.value = currentSize.value;
};

const onResizeMove = (delta: number) => {
  const adjustedDelta =
    props.side === "right" || props.side === "bottom" ? delta : -delta;
  setSize(beginResizeValue.value + adjustedDelta);
};

const resetSize = (useDefaultSize = true) => {
  if (props.defaultSize && useDefaultSize) {
    setSize(props.defaultSize);
  }
};

const onWindowResize = () => {
  // may change the size because min/max ratio of window size may have changed
  if (props.resizeable) setSize(currentSize.value);
  else currentSize.value = props.defaultSize;
};

const debounceForResize = _.debounce(onWindowResize, 20);
const windowResizeObserver = new ResizeObserver(debounceForResize);

// subpanel resizing
const subpanelSplitPercent = ref(props.defaultSubpanelSplit);

onMounted(() => {
  const storedSplit = window.localStorage.getItem(
    subpanelSplitLocalStorageKey.value,
  );
  if (storedSplit) subpanelSplitPercent.value = parseFloat(storedSplit);
});

const totalAvailableSize = ref(0);
const subpanel1SizePx = computed(
  () => totalAvailableSize.value * subpanelSplitPercent.value,
);
let subpanelResizeStartPanel1Size: number;
function onSubpanelResizeStart() {
  const boundingRect = panelRef.value?.getBoundingClientRect();
  if (!boundingRect) return;
  totalAvailableSize.value = isTopOrBottom.value
    ? boundingRect.width
    : boundingRect.height;
  subpanelResizeStartPanel1Size = subpanel1SizePx.value;
}
function onSubpanelResizeMove(delta: number) {
  const newPanel1SizePx = subpanelResizeStartPanel1Size - delta;
  setSubpanelSplit(newPanel1SizePx / totalAvailableSize.value);
}
function onSubpanelResizeReset() {
  setSubpanelSplit(props.defaultSubpanelSplit);
}
function setSubpanelSplit(newSplitPercent: number) {
  if (newSplitPercent < 0.2) {
    subpanelSplitPercent.value = 0.2;
  } else if (newSplitPercent > 0.8) {
    subpanelSplitPercent.value = 0.8;
  } else {
    subpanelSplitPercent.value = newSplitPercent;
  }

  if (subpanelSplitPercent.value === props.defaultSubpanelSplit) {
    window.localStorage.removeItem(subpanelSplitLocalStorageKey.value);
  } else {
    window.localStorage.setItem(
      subpanelSplitLocalStorageKey.value,
      `${subpanelSplitPercent.value}`,
    );
  }
}

onMounted(() => {
  if (props.resizeable) {
    const storedSize = window.localStorage.getItem(
      primarySizeLocalStorageKey.value,
    );
    if (storedSize) setSize(parseInt(storedSize));
    else setSize(props.defaultSize);
  } else {
    window.localStorage.removeItem(primarySizeLocalStorageKey.value);
  }

  windowResizeObserver.observe(document.body);
});

onBeforeUnmount(() => {
  windowResizeObserver.unobserve(document.body);
});

defineExpose({
  setSize,
  resetSize,
});
</script>
