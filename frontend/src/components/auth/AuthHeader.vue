<script setup lang="ts">
import LogoMark from './LogoMark.vue'

defineProps<{
  eyebrow?: string
  title: string
  sub?: string
  mark?: boolean
}>()
</script>

<template>
  <div class="header">
    <div v-if="mark" class="mark"><LogoMark :size="44" /></div>
    <div v-if="eyebrow" class="eyebrow">{{ eyebrow }}</div>
    <h1 class="title">{{ title }}</h1>
    <p v-if="sub" class="sub"><slot>{{ sub }}</slot></p>
    <p v-else-if="$slots.default" class="sub"><slot /></p>
  </div>
</template>

<style scoped>
.header {
  margin-bottom: 32px;
}

.mark {
  margin-bottom: 22px;
  color: var(--paper-ink);
}

.eyebrow {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin-bottom: 10px;
}

.title {
  font-family: var(--font-serif);
  font-size: 34px;
  line-height: 1.05;
  letter-spacing: -1px;
  color: var(--paper-ink);
  margin: 0;
}

.sub {
  font-family: var(--font-serif);
  font-style: italic;
  font-size: 16px;
  color: var(--paper-ink-muted);
  margin: 8px 0 0;
  line-height: 1.4;
}
</style>
