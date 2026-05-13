<script setup lang="ts">
import StaticPageView from './StaticPageView.vue'
</script>

<template>
  <StaticPageView eyebrow="Quiet Cube" title="About">
    <p>
      Quiet Cube is a focused trainer for the 57 cases of OLL — the third step of the CFOP method and, in my experience, the wall most CFOP cubers hit on their way to faster solves.
    </p>
    <p>
      It started as a stack of index cards with each OLL pattern drawn on the front and my algorithm on the back. I'd flip through them with a solved cube in hand, comparing the result of the algorithm on my cube to the picture on the back of the card. It worked, but it was a slow, messy system.
    </p>
    <p>
      Quiet Cube is the digital version of those cards. Same loop, with spaced repetition handling which case you see next.
    </p>

    <h2>The philosophy</h2>
    <p>
      The app applies the <strong>SM-2 algorithm</strong> — the same spaced-repetition method Anki uses — to OLL practice. Weak cases come back sooner. Strong cases get out of your way. That way the limited time I have to practice goes towards the algorithms I haven't locked in yet.
    </p>
    <p>
      The interaction is deliberately physical. You see the case, execute the algorithm on a real cube in front of you, flip the card, and check whether your cube matches the expected pattern. Then you grade yourself honestly. No timer, no leaderboard, no opponents. Just you and the cube.
    </p>
    <p>
      Quiet Cube is also, well, quiet. No ads. No social features. No third-party trackers stitched into the page. The app collects the minimum information needed to run a user account, and nothing else. Quiet Cube is the app I wanted for myself. If it helps someone else, even better.
    </p>
    <p>
      Cubing helps me put everything else in my life down for a while. The same puzzle every time, but never the same solution twice. This app is built to protect that feeling, not interrupt it.
    </p>

    <h2>About me</h2>
    <p>
      I'm a full-stack developer with over a decade of experience across product, dev ops, and engineering — though web apps and analytics tools are where I'm happiest. I tend to use software to solve pain points in my hobbies, and Quiet Cube is one expression of that.
    </p>
    <p>
      I've been cubing for over five years. I solve around the 60-second mark using CFOP, and I'm hoping that finally learning full OLL through Quiet Cube gets me consistently under a minute. If it works for me, it might work for you.
    </p>
    <p>
      I'm based in Texas. Outside of cubing and code, I camp, collect Pokémon cards, and write science fiction. Mostly things that make me feel like a kid or like I'm on an adventure.
    </p>

    <h2>Contact</h2>
    <p>
      Questions, bugs, suggestions, or just want to say hi? Email me at <strong>me@nebulouscode.com</strong>.
    </p>
    <p>
      More of my work is at <a href="https://github.com/nebulous-code" target="_blank" rel="noopener noreferrer">github.com/nebulous-code</a>.
    </p>
  </StaticPageView>
</template>
