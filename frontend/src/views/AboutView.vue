<script setup lang="ts">
import StaticPageView from './StaticPageView.vue'
</script>

<template>
  <StaticPageView eyebrow="Cube Practice" title="About">
    <p class="placeholder">Placeholder copy — replace before public launch.</p>

    <p>
      Cube Practice is a quiet place to drill Rubik's cube algorithms with
      spaced repetition. Build muscle memory for the cases you don't yet know,
      and keep the ones you do sharp.
    </p>

    <h2>Why another app</h2>
    <p>
      Most cube trainers focus on raw speed. Cube Practice focuses on
      retention — surfacing each case at the right interval so it sticks.
    </p>

    <h2>Status</h2>
    <p>
      Currently in early development. Authentication is live; practice and
      progress views land in subsequent milestones.
    </p>
  </StaticPageView>
</template>
