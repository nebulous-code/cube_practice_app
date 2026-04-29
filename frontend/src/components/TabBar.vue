<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

interface Tab {
  to: string
  label: string
  // Inline SVG icon — kept lightweight per the paper aesthetic.
  icon: string
}

// SVG paths only; the wrapping <svg> is in the template.
const ICON_PRACTICE =
  'M12 4 L20 12 L12 20 L4 12 Z'
const ICON_CASES =
  'M4 4 H10 V10 H4 Z M14 4 H20 V10 H14 Z M4 14 H10 V20 H4 Z M14 14 H20 V20 H14 Z'
const ICON_PROGRESS =
  'M4 20 V14 M10 20 V8 M16 20 V4 M22 20 H2'

const TABS: ReadonlyArray<Tab> = [
  { to: '/', label: 'Practice', icon: ICON_PRACTICE },
  { to: '/cases', label: 'Cases', icon: ICON_CASES },
  { to: '/progress', label: 'Progress', icon: ICON_PROGRESS },
]

function isActive(to: string): boolean {
  if (to === '/') return route.path === '/'
  return route.path === to || route.path.startsWith(`${to}/`)
}

const activePath = computed(() => route.path)
</script>

<template>
  <nav class="tab-bar" :data-active="activePath">
    <RouterLink
      v-for="tab in TABS"
      :key="tab.to"
      :to="tab.to"
      class="tab"
      :class="{ active: isActive(tab.to) }"
    >
      <svg
        class="icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path :d="tab.icon" />
      </svg>
      <span class="label">{{ tab.label }}</span>
    </RouterLink>
  </nav>
</template>

<style scoped>
.tab-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 5;
  display: flex;
  background: var(--paper-bg);
  border-top: 1px solid var(--paper-rule-faint);
  padding: 8px 0 calc(env(safe-area-inset-bottom, 0px) + 18px);
}

.tab {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 4px;
  text-decoration: none;
  color: var(--paper-ink-faint);
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 0.4px;
  cursor: pointer;
}

.tab.active {
  color: var(--paper-ink);
}

.icon {
  width: 22px;
  height: 22px;
}

.label {
  margin: 0;
}
</style>
