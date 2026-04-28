<script setup lang="ts">
defineProps<{ size?: number }>()
</script>

<template>
  <svg
    :width="size ?? 44"
    :height="size ?? 44"
    viewBox="0 0 44 44"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
  >
    <rect x="2" y="2" width="40" height="40" rx="6" stroke="currentColor" stroke-width="1.4" />
    <line x1="15" y1="2" x2="15" y2="42" stroke="currentColor" stroke-width="1.2" />
    <line x1="29" y1="2" x2="29" y2="42" stroke="currentColor" stroke-width="1.2" />
    <line x1="2" y1="15" x2="42" y2="15" stroke="currentColor" stroke-width="1.2" />
    <line x1="2" y1="29" x2="42" y2="29" stroke="currentColor" stroke-width="1.2" />
  </svg>
</template>
