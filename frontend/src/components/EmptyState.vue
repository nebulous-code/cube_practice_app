<script setup lang="ts">
// Reusable zero-data treatment. Used by every view's empty branch so the
// visual language stays consistent with the paper aesthetic. See
// docs/milestones/05_polish_and_static_pages.md §5.
//
// All slots are optional. Typical use: title + body + cta. The icon slot
// renders above the title for views that want a small visual anchor (e.g.
// the 404 page); leave it empty otherwise.
</script>

<template>
  <section class="empty">
    <div v-if="$slots.icon" class="icon">
      <slot name="icon" />
    </div>
    <h2 v-if="$slots.title" class="title">
      <slot name="title" />
    </h2>
    <p v-if="$slots.body" class="body">
      <slot name="body" />
    </p>
    <div v-if="$slots.cta" class="cta">
      <slot name="cta" />
    </div>
  </section>
</template>

<style scoped>
.empty {
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: var(--radius-md);
  padding: var(--space-6) var(--space-5);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  text-align: center;
}

.icon {
  color: var(--paper-ink-faint);
  margin-bottom: var(--space-1);
}

.title {
  font-family: var(--font-serif);
  font-style: italic;
  font-weight: 500;
  font-size: 20px;
  letter-spacing: -0.2px;
  color: var(--paper-ink);
  margin: 0;
}

.body {
  font-family: var(--font-sans);
  font-size: 13px;
  line-height: 1.5;
  color: var(--paper-ink-muted);
  margin: 0;
  max-width: 36ch;
}

.cta {
  margin-top: var(--space-2);
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
  justify-content: center;
}
</style>
