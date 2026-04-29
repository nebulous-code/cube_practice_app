<script setup lang="ts">
import StaticPageView from './StaticPageView.vue'
</script>

<template>
  <StaticPageView eyebrow="Legal" title="Privacy Policy">
    <p class="placeholder">
      Placeholder copy — final privacy policy is being drafted. Replace this
      view's body before public launch.
    </p>

    <h2>What we store</h2>
    <ul>
      <li>Your email and display name.</li>
      <li>A salted hash of your password — never the password itself.</li>
      <li>Your practice history and review schedule, once that lands.</li>
      <li>Session records used to keep you signed in across visits.</li>
    </ul>

    <h2>What we don't store</h2>
    <ul>
      <li>Payment data — there is no paid tier yet.</li>
      <li>Third-party tracking or advertising identifiers.</li>
    </ul>

    <h2>Email</h2>
    <p>
      We send transactional email only — verification codes, password resets,
      and account-change confirmations. No marketing.
    </p>

    <h2>Third parties</h2>
    <p>
      We rely on a small set of services to operate: Neon (database hosting),
      Render (application hosting), Resend (transactional email), and Google
      reCAPTCHA (bot detection on signup).
    </p>

    <h2>Your data, your call</h2>
    <p>
      You can update your email, change your password, or sign out everywhere
      from settings. Account deletion is on the roadmap.
    </p>
  </StaticPageView>
</template>
