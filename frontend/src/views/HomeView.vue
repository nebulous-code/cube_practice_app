<script setup lang="ts">
// Placeholder dashboard for milestone 1.
// Real content arrives in milestone 2 onward.
</script>

<template>
  <main class="placeholder">
    <div class="card">
      <p class="eyebrow">Milestone 1</p>
      <h1>OLL Practice — scaffolding</h1>
      <p class="body">
        Auth and user account flows ship in this milestone. The practice/cases/progress views
        come online in later milestones.
      </p>
    </div>
  </main>
</template>

<style scoped>
.placeholder {
  min-height: 100vh;
  display: grid;
  place-items: center;
  padding: var(--space-8) var(--space-6);
}

.card {
  max-width: 480px;
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: var(--radius-lg);
  padding: var(--space-8);
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
  margin: 0 0 var(--space-4);
  color: var(--paper-ink);
  line-height: 1.1;
}

.body {
  font-family: var(--font-serif);
  font-style: italic;
  color: var(--paper-ink-muted);
  line-height: 1.5;
  margin: 0;
}
</style>
