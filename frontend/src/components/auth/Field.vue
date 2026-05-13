<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  label: string
  modelValue: string
  type?: string
  placeholder?: string
  hint?: string | null
  autofocus?: boolean
  error?: string | null
  autocomplete?: string
}>()

defineEmits<{ 'update:modelValue': [value: string] }>()

const focused = ref(false)
</script>

<template>
  <label class="field">
    <span class="label" :class="{ 'label-error': !!error }">{{ label }}</span>
    <span class="input-row" :class="{ focused, error: !!error }">
      <input
        :type="type ?? 'text'"
        :value="modelValue"
        :placeholder="placeholder"
        :autofocus="autofocus"
        :autocomplete="autocomplete"
        @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        @focus="focused = true"
        @blur="focused = false"
      />
      <slot name="right" />
    </span>
    <span v-if="error" class="error-text">{{ error }}</span>
    <span v-else-if="hint" class="hint">{{ hint }}</span>
  </label>
</template>

<style scoped>
.field {
  display: block;
  margin-bottom: 16px;
}

.label {
  display: block;
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.2px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin-bottom: 6px;
}

.label-error {
  color: var(--paper-error);
}

.input-row {
  display: flex;
  align-items: center;
  background: var(--paper-card);
  border: 1px solid var(--paper-rule);
  border-radius: var(--radius-md);
  padding: 0 14px;
  transition: border-color 120ms;
}

.input-row.focused {
  border-color: var(--paper-ink);
}

.input-row.error {
  border-color: var(--paper-error);
}

input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  padding: 14px 0;
  font-family: var(--font-sans);
  font-size: 15px;
  color: var(--paper-ink);
  min-width: 0;
}

.hint,
.error-text {
  display: block;
  font-family: var(--font-sans);
  font-size: 11px;
  margin-top: 6px;
  letter-spacing: 0.2px;
}

.hint {
  color: var(--paper-ink-faint);
}

.error-text {
  color: var(--paper-error);
}
</style>
