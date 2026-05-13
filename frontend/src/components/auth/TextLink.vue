<script setup lang="ts">
defineProps<{ accent?: boolean; type?: 'button' | 'submit' }>()
</script>

<template>
  <button class="link" :class="{ accent }" :type="type ?? 'button'">
    <slot />
  </button>
</template>

<style scoped>
.link {
  background: none;
  border: none;
  padding: 0;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink);
  font-weight: 400;
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 3px;
  text-decoration-color: var(--paper-rule);
}

.link.accent {
  color: var(--paper-accent);
  font-weight: 500;
  text-decoration-color: var(--paper-accent);
}
</style>
