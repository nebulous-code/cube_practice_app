<script setup lang="ts">
import StaticPageView from './StaticPageView.vue'
</script>

<template>
  <StaticPageView eyebrow="Credits" title="Acknowledgements">
    <p class="placeholder">Placeholder copy — fill in before public launch.</p>

    <p>
      Quiet Cube stands on the work of the global cubing community —
      decades of algorithm research, naming conventions, and shared notation.
    </p>

    <h2>Open source</h2>
    <p>
      Built with Vue, Vite, Pinia, and Vue Router on the frontend; Rust,
      Axum, sqlx, and tokio on the backend. Auth uses argon2id and JWT.
    </p>

    <h2>Algorithms</h2>
    <p>
      OLL, PLL, and other case sets follow widely used community
      numbering. Specific algorithm sources will be credited here as the
      case sets ship.
    </p>

    <h2>Design</h2>
    <p>
      Typography uses Newsreader, Inter Tight, and JetBrains Mono via Google
      Fonts.
    </p>

    <h2>Support the project</h2>
    <p>
      Quiet Cube is a solo side project. If it's useful to you, drop a
      star on
      <a
        href="https://github.com/nebulous-code/cube_practice_app"
        target="_blank"
        rel="noopener noreferrer"
        >github.com/nebulous-code/cube_practice_app</a
      >
      — it's the easiest free way to say thanks and helps other cubers
      find the app.
    </p>
  </StaticPageView>
</template>
