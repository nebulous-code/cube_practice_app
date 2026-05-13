<script setup lang="ts">
import { useRouter } from 'vue-router'

defineProps<{
  eyebrow: string
  title: string
}>()

const router = useRouter()

function goBack() {
  if (window.history.length > 1) router.back()
  else router.push('/')
}
</script>

<template>
  <main class="page">
    <button class="back" type="button" @click="goBack">← Back</button>

    <header>
      <p class="eyebrow">{{ eyebrow }}</p>
      <h1>{{ title }}</h1>
    </header>

    <article class="prose">
      <slot />
    </article>
  </main>
</template>

<style scoped>
.page {
  min-height: 100vh;
  background: var(--paper-bg);
  padding: 52px 22px 40px;
  max-width: 720px;
  margin: 0 auto;
  color: var(--paper-ink);
}

.back {
  background: none;
  border: none;
  padding: 0;
  margin-bottom: 16px;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
  cursor: pointer;
}

header {
  margin-bottom: 28px;
}

.eyebrow {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin: 0 0 var(--space-3);
}

h1 {
  font-family: var(--font-serif);
  font-size: 30px;
  letter-spacing: -0.6px;
  color: var(--paper-ink);
  margin: 0;
  line-height: 1.1;
}

.prose {
  font-family: var(--font-serif);
  font-size: 16px;
  line-height: 1.65;
  color: var(--paper-ink-muted);
}

.prose :deep(h2) {
  font-family: var(--font-sans);
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.3px;
  text-transform: uppercase;
  color: var(--paper-ink);
  margin: 32px 0 12px;
}

.prose :deep(h3) {
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.2px;
  text-transform: uppercase;
  color: var(--paper-ink);
  margin: 22px 0 10px;
}

.prose :deep(p) {
  margin: 0 0 14px;
}

.prose :deep(strong) {
  color: var(--paper-ink);
  font-weight: 600;
}

.prose :deep(code) {
  font-family: var(--font-mono);
  font-size: 0.88em;
  background: var(--paper-rule-faint);
  padding: 1px 5px;
  border-radius: var(--radius-sm);
  color: var(--paper-ink);
}

.prose :deep(a) {
  color: var(--paper-accent);
  text-underline-offset: 2px;
}

.prose :deep(ul) {
  margin: 0 0 14px;
  padding-left: 1.2em;
}

.prose :deep(li) {
  margin: 0 0 6px;
}

.prose :deep(.doc-meta) {
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--paper-ink-faint);
  line-height: 1.6;
  margin: 0 0 24px;
}

.prose :deep(.doc-meta strong) {
  color: var(--paper-ink-faint);
  font-weight: 600;
}

.prose :deep(.table-wrap) {
  overflow-x: auto;
  margin: 0 0 14px;
}

.prose :deep(table) {
  font-family: var(--font-sans);
  font-size: 13px;
  line-height: 1.45;
  border-collapse: collapse;
  width: 100%;
  min-width: 480px;
}

.prose :deep(thead th) {
  text-align: left;
  font-weight: 600;
  color: var(--paper-ink);
  border-bottom: 1px solid var(--paper-rule);
  padding: 8px 12px 8px 0;
}

.prose :deep(tbody td) {
  vertical-align: top;
  padding: 10px 12px 10px 0;
  border-bottom: 1px solid var(--paper-rule-faint);
  color: var(--paper-ink-muted);
}

.prose :deep(tbody tr:last-child td) {
  border-bottom: none;
}

.prose :deep(th:last-child),
.prose :deep(td:last-child) {
  padding-right: 0;
}

.prose :deep(.placeholder) {
  font-family: var(--font-sans);
  font-size: 12px;
  letter-spacing: 0.2px;
  color: var(--paper-ink-faint);
  border: 1px dashed var(--paper-rule);
  border-radius: 8px;
  padding: 10px 14px;
  margin: 0 0 18px;
}
</style>
