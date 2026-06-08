<script setup>
import { computed, ref } from "vue";

const props = defineProps({
  values: {
    type: Array,
    required: true,
    default: () => [],
  },

  tcpValues: {
    type: Array,
    default: () => [],
  },
  width: {
    type: Number,
    default: 160,
  },
  height: {
    type: Number,
    default: 44,
  },
});

const hoveredIndex = ref(null);

const maxVal = computed(() => {
  const allTicks = [...props.values, ...props.tcpValues].filter((v) => v > 0);
  const maxTick = allTicks.length > 0 ? Math.max(...allTicks) : 0;
  return Math.max(maxTick, 200);
});

const buildPoints = (values) => {
  if (!values || values.length < 2) return null;

  const padding = 6;
  const usableHeight = props.height - padding * 2;

  return values.map((val, index) => {
    const x = (index / (values.length - 1)) * props.width;
    const isError = val === 0;

    const y = isError
      ? padding
      : props.height - padding - (val / maxVal.value) * usableHeight;

    return { x, y, val, isError };
  });
};

const pointsData = computed(() => buildPoints(props.values));

const tcpPointsData = computed(() => {
  if (!props.tcpValues || props.tcpValues.length !== props.values.length) {
    return null;
  }
  return buildPoints(props.tcpValues);
});

const tcpLinePath = computed(() => {
  if (!tcpPointsData.value) return "";
  return tcpPointsData.value
    .map((p, i) => `${i === 0 ? "M" : "L"} ${p.x} ${p.y}`)
    .join(" ");
});

const linePath = computed(() => {
  if (!pointsData.value) return "";
  return pointsData.value
    .map((p, i) => `${i === 0 ? "M" : "L"} ${p.x} ${p.y}`)
    .join(" ");
});

const areaPath = computed(() => {
  if (!pointsData.value || !linePath.value) return "";
  return `${linePath.value} L ${props.width} ${props.height} L 0 ${props.height} Z`;
});

const isCurrentlyDown = computed(() => {
  if (!props.values || props.values.length === 0) return false;
  return props.values[props.values.length - 1] === 0;
});

const handleMouseMove = (event) => {
  if (!pointsData.value) return;

  const svg = event.currentTarget;
  const rect = svg.getBoundingClientRect();

  const mouseX = ((event.clientX - rect.left) / rect.width) * props.width;

  let closestIndex = 0;
  let minDiff = Infinity;

  pointsData.value.forEach((p, i) => {
    const diff = Math.abs(p.x - mouseX);
    if (diff < minDiff) {
      minDiff = diff;
      closestIndex = i;
    }
  });

  hoveredIndex.value = closestIndex;
};
</script>

<template>
  <div class="relative flex items-center justify-center">
    <div
      v-if="props.values.length < 2"
      class="text-xs text-zinc-500 animate-pulse"
    >
      Загрузка...
    </div>

    <div v-else class="relative">
      <div
        v-if="hoveredIndex !== null"
        class="absolute bottom-full mb-2 z-50 px-2 py-1 text-[10px] font-semibold text-zinc-100 bg-zinc-950/90 border border-zinc-800 rounded shadow-xl backdrop-blur-sm pointer-events-none transition-all duration-75 whitespace-nowrap transform -translate-x-1/2"
        :style="{ left: `${(pointsData[hoveredIndex].x / width) * 100}%` }"
      >
        <div class="flex items-center gap-1.5">
          <span
            class="w-1.5 h-1.5 rounded-full"
            :class="
              pointsData[hoveredIndex].isError ? 'bg-red-500' : 'bg-emerald-500'
            "
          ></span>
          <span>
            {{
              pointsData[hoveredIndex].isError
                ? "Ошибка соединения"
                : `HTTP ${pointsData[hoveredIndex].val} мс`
            }}
          </span>
        </div>
        <div
          v-if="tcpPointsData && !tcpPointsData[hoveredIndex].isError"
          class="flex items-center gap-1.5 mt-0.5"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-sky-400"></span>
          <span>TCP {{ tcpPointsData[hoveredIndex].val }} мс</span>
        </div>
      </div>

      <svg
        :viewBox="`0 0 ${width} ${height}`"
        :width="width"
        :height="height"
        class="overflow-visible cursor-crosshair"
        @mousemove="handleMouseMove"
        @mouseleave="hoveredIndex = null"
      >
        <defs>
          <linearGradient id="sparkline-grad" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#10b981" stop-opacity="0.12" />
            <stop offset="100%" stop-color="#10b981" stop-opacity="0.0" />
          </linearGradient>
        </defs>

        <path :d="areaPath" fill="url(#sparkline-grad)" />

        <path
          v-if="tcpLinePath"
          :d="tcpLinePath"
          fill="none"
          stroke="#38bdf8"
          stroke-width="1"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-dasharray="3 2"
          class="opacity-60"
        />

        <path
          :d="linePath"
          fill="none"
          stroke="#10b981"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="opacity-80"
        />

        <g>
          <circle
            v-for="(p, i) in pointsData.filter((p) => p.isError)"
            :key="'err-' + i"
            :cx="p.x"
            :cy="p.y"
            r="2.5"
            fill="#ef4444"
          />
        </g>

        <line
          v-if="hoveredIndex !== null"
          :x1="pointsData[hoveredIndex].x"
          y1="0"
          :x2="pointsData[hoveredIndex].x"
          :y2="height"
          stroke="#3f3f46"
          stroke-width="1"
          stroke-dasharray="2 2"
          class="pointer-events-none"
        />

        <circle
          v-if="hoveredIndex !== null"
          :cx="pointsData[hoveredIndex].x"
          :cy="pointsData[hoveredIndex].y"
          r="3.5"
          :fill="pointsData[hoveredIndex].isError ? '#ef4444' : '#10b981'"
          stroke="#18181b"
          stroke-width="1"
          class="pointer-events-none"
        />

        <circle
          v-if="hoveredIndex === null"
          :cx="pointsData[pointsData.length - 1].x"
          :cy="pointsData[pointsData.length - 1].y"
          r="3"
          :fill="isCurrentlyDown ? '#ef4444' : '#10b981'"
        />
      </svg>
    </div>
  </div>
</template>
