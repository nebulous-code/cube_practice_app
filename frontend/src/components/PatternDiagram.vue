<script setup lang="ts">
import { computed } from 'vue'

// Vue port of initial_design/src/diagram.jsx. Renders a 9-character OLL
// pattern as the cube top face with optional side strips. Geometry is
// expressed against a 100×100 viewBox so callers control the rendered
// size with the `size` prop without recomputing positions.

const props = withDefaults(
  defineProps<{
    pattern: string
    size?: number
  }>(),
  { size: 120 },
)

// Palette — paper aesthetic, mirrors the prototype.
const FACE = '#EFEAE0'
const GRAY = '#D9D3C4'
const GRAY_STROKE = '#C7BFAE'
const YELLOW = '#E9B949'
const YELLOW_STROKE = '#B8901F'

// Geometry against a 100-unit viewBox (S = 100).
const PAD = 10
const STRIP = 5.5
const GAP = 1.8
const CELL_GAP = 1.2
const CELL = (100 - 2 * (PAD + STRIP + GAP) - 2 * CELL_GAP) / 3
const FACE_X = PAD + STRIP + GAP
const FACE_Y = FACE_X
const SLAB_X = PAD + STRIP
const SLAB_W = 100 - 2 * (PAD + STRIP)

interface Cell {
  key: string
  x: number
  y: number
  fill: string
  stroke: string
}

interface Strip {
  key: string
  x: number
  y: number
  width: number
  height: number
}

const cells = computed<Cell[]>(() => {
  const out: Cell[] = []
  for (let r = 0; r < 3; r++) {
    for (let c = 0; c < 3; c++) {
      const ch = props.pattern.charAt(r * 3 + c)
      const yellow = ch === 'X'
      out.push({
        key: `c-${r}-${c}`,
        x: FACE_X + c * (CELL + CELL_GAP),
        y: FACE_Y + r * (CELL + CELL_GAP),
        fill: yellow ? YELLOW : GRAY,
        stroke: yellow ? YELLOW_STROKE : GRAY_STROKE,
      })
    }
  }
  return out
})

const strips = computed<Strip[]>(() => {
  const out: Strip[] = []
  for (let r = 0; r < 3; r++) {
    for (let c = 0; c < 3; c++) {
      const ch = props.pattern.charAt(r * 3 + c)
      const x0 = FACE_X + c * (CELL + CELL_GAP)
      const y0 = FACE_Y + r * (CELL + CELL_GAP)
      let strip: Omit<Strip, 'key'> | null = null
      if (ch === 'T') {
        strip = { x: x0, y: y0 - GAP - STRIP, width: CELL, height: STRIP }
      } else if (ch === 'B') {
        strip = { x: x0, y: y0 + CELL + GAP, width: CELL, height: STRIP }
      } else if (ch === 'L') {
        strip = { x: x0 - GAP - STRIP, y: y0, width: STRIP, height: CELL }
      } else if (ch === 'R') {
        strip = { x: x0 + CELL + GAP, y: y0, width: STRIP, height: CELL }
      }
      if (strip) out.push({ key: `s-${r}-${c}-${ch}`, ...strip })
    }
  }
  return out
})
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 100 100"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
    class="pattern-diagram"
  >
    <rect
      :x="SLAB_X"
      :y="SLAB_X"
      :width="SLAB_W"
      :height="SLAB_W"
      rx="4"
      :fill="FACE"
    />
    <rect
      v-for="cell in cells"
      :key="cell.key"
      :x="cell.x"
      :y="cell.y"
      :width="CELL"
      :height="CELL"
      rx="1.2"
      :fill="cell.fill"
      :stroke="cell.stroke"
      stroke-width="0.6"
    />
    <rect
      v-for="strip in strips"
      :key="strip.key"
      :x="strip.x"
      :y="strip.y"
      :width="strip.width"
      :height="strip.height"
      rx="0.8"
      :fill="YELLOW"
      :stroke="YELLOW_STROKE"
      stroke-width="0.6"
    />
  </svg>
</template>

<style scoped>
.pattern-diagram {
  display: block;
}
</style>
