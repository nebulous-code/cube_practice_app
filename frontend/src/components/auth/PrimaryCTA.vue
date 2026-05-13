<script setup lang="ts">
defineProps<{ disabled?: boolean; type?: 'button' | 'submit' }>()
</script>

<template>
  <button :type="type ?? 'button'" class="cta" :disabled="disabled">
    <slot />
  </button>
</template>

<style scoped>
.cta {
  width: 100%;
  background: var(--paper-ink);
  color: var(--paper-bg);
  border: none;
  border-radius: 12px;
  padding: 15px;
  font-family: var(--font-sans);
  font-size: 15px;
  font-weight: 600;
  letter-spacing: 0.2px;
  cursor: pointer;
  margin-top: 8px;
  transition: opacity 120ms;
}

.cta:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
