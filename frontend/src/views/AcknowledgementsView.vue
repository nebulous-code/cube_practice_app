<script setup lang="ts">
import StaticPageView from './StaticPageView.vue'
</script>

<template>
  <StaticPageView eyebrow="Credits" title="Acknowledgements">
    <h2>About</h2>
    <p>
      I'm a software developer who enjoys solving the Rubik's cube and wanted a systematic way to practice on a regular basis.
    </p>

    <h2>Algorithms and case references</h2>
    <p>
      The OLL algorithms used in Quiet Cube come from <strong><em>Speedsolving the Cube</em> by Dan Harris</strong>.
    </p>
    <p>
      The case numbering convention follows the <a href="https://www.speedsolving.com/wiki/index.php/OLL" target="_blank" rel="noopener noreferrer">Speedsolving.com wiki's OLL page</a>, which is the de facto standard reference in the cubing community.
    </p>

    <h2>Inspiration</h2>
    <p>Quiet Cube wouldn't exist without these projects and resources:</p>
    <ul>
      <li><strong><a href="https://apps.ankiweb.net/" target="_blank" rel="noopener noreferrer">Anki</a></strong> — for proving how powerful spaced repetition can be as a learning tool, and for inspiring the SM-2 review approach used here.</li>
      <li><strong><a href="https://jperm.net/" target="_blank" rel="noopener noreferrer">J Perm</a></strong> — for setting the bar on clear, accessible cubing instruction and online practice tools.</li>
      <li><strong><em>Speedsolving the Cube</em> by Dan Harris</strong> — the source of the algorithms and a foundational reference for anyone learning to solve the cube methodically.</li>
      <li><strong><a href="https://www.speedsolving.com/wiki/" target="_blank" rel="noopener noreferrer">Speedsolving.com wiki</a></strong> — the cubing community's collective knowledge base, including the OLL case numbering this app follows.</li>
    </ul>

    <h2>Fonts</h2>
    <p>
      Quiet Cube uses three typefaces, all licensed under the <a href="https://scripts.sil.org/OFL" target="_blank" rel="noopener noreferrer">SIL Open Font License 1.1</a>:
    </p>
    <ul>
      <li><strong>Newsreader</strong></li>
      <li><strong>Inter Tight</strong></li>
      <li><strong>JetBrains Mono</strong></li>
    </ul>

    <h2>Anti-bot protection</h2>
    <p>
      This site is protected by reCAPTCHA and the Google <a href="https://policies.google.com/privacy" target="_blank" rel="noopener noreferrer">Privacy Policy</a> and <a href="https://policies.google.com/terms" target="_blank" rel="noopener noreferrer">Terms of Service</a> apply.
    </p>

    <h2>Open-source dependencies</h2>
    <p>
      Quiet Cube is built on top of many open-source libraries from the npm and Rust crates ecosystems. License texts for those dependencies are preserved in <code>node_modules</code> and <code>Cargo.lock</code> per the conventions of each ecosystem.
    </p>

    <h2>License and source</h2>
    <p>
      Quiet Cube is source-available under the <a href="https://polyformproject.org/licenses/noncommercial/1.0.0/" target="_blank" rel="noopener noreferrer">Polyform Noncommercial License 1.0.0</a>. You're welcome to use, modify, and share the code for any noncommercial purpose.
    </p>
    <p>
      If you'd like to support the project, you can do so via <a href="https://github.com/sponsors/nebulous-code" target="_blank" rel="noopener noreferrer">GitHub Sponsors</a>.
    </p>

    <h2>Contact</h2>
    <p>
      <strong>nebulous-code</strong> — <strong>me@nebulouscode.com</strong>
    </p>
  </StaticPageView>
</template>
