<script setup lang="ts">
</script>

<template>
  <main class="stub">
    <div class="card">
      <p class="eyebrow">Progress</p>
      <h1>Coming soon</h1>
      <p class="body">
        Per-case mastery, learning state, and stats arrive in milestone 4.
        First we ship case data and the study loop.
      </p>
    </div>
  </main>
</template>

<style scoped>
.stub {
  min-height: calc(100vh - 90px);
  display: grid;
  place-items: center;
  padding: 48px 22px 40px;
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
  line-height: 1.1;
  color: var(--paper-ink);
  margin: 0 0 var(--space-4);
}

.body {
  font-family: var(--font-serif);
  font-style: italic;
  color: var(--paper-ink-muted);
  line-height: 1.5;
  margin: 0;
}
</style>
