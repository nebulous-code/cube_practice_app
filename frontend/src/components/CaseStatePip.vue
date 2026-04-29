<script setup lang="ts">
import { computed } from 'vue'

import type { CaseState } from '@/stores/cases'

const props = withDefaults(
  defineProps<{
    state: CaseState
    size?: number
    showLabel?: boolean
  }>(),
  { size: 12, showLabel: false },
)

const STATE_LABEL: Record<CaseState, string> = {
  not_started: 'Not started',
  learning: 'Learning',
  due: 'Due',
  mastered: 'Mastered',
}

const label = computed(() => STATE_LABEL[props.state])

const tooltip = computed(() => label.value)
</script>

<template>
  <span class="pip" :data-state="state" :title="tooltip" :aria-label="label">
    <svg
      :width="size"
      :height="size"
      viewBox="0 0 12 12"
      class="dot"
      aria-hidden="true"
    >
      <circle
        v-if="state === 'not_started'"
        cx="6"
        cy="6"
        r="4.2"
        fill="none"
        stroke="currentColor"
        stroke-width="1"
      />
      <g v-else-if="state === 'learning'">
        <circle
          cx="6"
          cy="6"
          r="4.2"
          fill="none"
          stroke="currentColor"
          stroke-width="1"
        />
        <path d="M 6 1.8 A 4.2 4.2 0 0 1 6 10.2 Z" fill="currentColor" />
      </g>
      <circle
        v-else-if="state === 'due'"
        cx="6"
        cy="6"
        r="4.2"
        fill="currentColor"
      />
      <g v-else-if="state === 'mastered'">
        <circle cx="6" cy="6" r="4.2" fill="currentColor" />
        <path
          d="M 3.5 6.2 L 5.3 7.8 L 8.5 4.4"
          fill="none"
          stroke="var(--paper-bg)"
          stroke-width="1.2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </g>
    </svg>
    <span v-if="showLabel" class="text">{{ label }}</span>
  </span>
</template>

<style scoped>
.pip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.dot {
  display: block;
}

.text {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 0.4px;
  color: var(--paper-ink-muted);
}

.pip[data-state='not_started'] {
  color: var(--paper-ink-faint);
}

.pip[data-state='learning'] {
  color: var(--paper-accent);
}

.pip[data-state='due'] {
  color: #c97a2c; /* warm warning, on-paper */
}

.pip[data-state='mastered'] {
  color: #4a7c3e; /* muted green, on-paper */
}
</style>
