<script setup lang="ts">
defineProps<{ size?: number }>()

// Pattern from initial_design/src/logo.jsx — a 3×3 grid where the
// top-left, full middle row, and bottom-right are filled (X). Reads
// as both a cube face and a stylized "OLL" hint.
const grid: ReadonlyArray<ReadonlyArray<'X' | 'O'>> = [
  ['X', 'O', 'O'],
  ['X', 'X', 'X'],
  ['O', 'O', 'X'],
]

// Geometry expressed against a 100×100 viewBox so callers control
// rendered size via the `size` prop without recomputing positions.
const PAD = 8
const GAP = 4
const CELL = (100 - PAD * 2 - GAP * 2) / 3

const cells = grid.flatMap((row, r) =>
  row.map((ch, c) => ({
    key: `${r}-${c}`,
    x: PAD + c * (CELL + GAP),
    y: PAD + r * (CELL + GAP),
    fill: ch === 'X' ? 'var(--logo-dark)' : 'var(--logo-light)',
  })),
)
</script>

<template>
  <svg
    :width="size ?? 44"
    :height="size ?? 44"
    viewBox="0 0 100 100"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
    class="logo-mark"
  >
    <rect x="0" y="0" width="100" height="100" rx="18" fill="var(--logo-frame)" />
    <rect
      v-for="cell in cells"
      :key="cell.key"
      :x="cell.x"
      :y="cell.y"
      :width="CELL"
      :height="CELL"
      rx="2.5"
      :fill="cell.fill"
    />
  </svg>
</template>

<style scoped>
.logo-mark {
  display: block;
  --logo-frame: #1f1b16;
  --logo-dark: #8c5a2b;
  --logo-light: #e8dcc4;
}
</style>
