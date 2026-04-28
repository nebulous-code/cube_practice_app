<script setup lang="ts">
import { ref } from 'vue'
import Field from './Field.vue'

defineProps<{
  label?: string
  modelValue: string
  placeholder?: string
  hint?: string | null
  autofocus?: boolean
  error?: string | null
  autocomplete?: string
}>()

defineEmits<{ 'update:modelValue': [value: string] }>()

const shown = ref(false)
</script>

<template>
  <Field
    :label="label ?? 'Password'"
    :type="shown ? 'text' : 'password'"
    :model-value="modelValue"
    :placeholder="placeholder"
    :hint="hint"
    :autofocus="autofocus"
    :error="error"
    :autocomplete="autocomplete"
    @update:model-value="$emit('update:modelValue', $event)"
  >
    <template #right>
      <button class="toggle" type="button" @click="shown = !shown">
        {{ shown ? 'Hide' : 'Show' }}
      </button>
    </template>
  </Field>
</template>

<style scoped>
.toggle {
  background: none;
  border: none;
  padding: 4px 0 4px 8px;
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 0.6px;
  color: var(--paper-ink-muted);
  text-transform: uppercase;
  cursor: pointer;
}
</style>
