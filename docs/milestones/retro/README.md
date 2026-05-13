# retro/

Closed milestones live here once they've been retrospected. The active
phase docs (`docs/milestones/01_*.md` etc.) capture *what was built*; the
retro pass that moves them here adds the **§13 "Looking back"** section —
what surprised, what we'd do differently, what the design call turned out
to actually mean in practice.

## Why split active vs retro?

Active milestone docs are working artifacts during the build — open
questions get resolved inline, the story list ticks down, the "done when"
list runs as the QA pass. Once a milestone ships and has been
retrospected, the doc stops being load-bearing — it's a record, not a
plan. Moving it here clears the active list so post-MVP work can take
over `milestones/` without drowning in shipped phases.

## Workflow

1. Milestone ships.
2. Walk the §13 prompts in the milestone doc. Fill them in. Be honest
   about what you'd change — the value of a retro is the friction it
   captures, not a victory lap.
3. `git mv docs/milestones/0X_*.md docs/milestones/retro/`.
4. Add a one-line entry to `docs/CHANGELOG.md` if it isn't already
   there — that's the long-term tracker of "what shipped when."

## What lives here

(Empty for now. M1–M7 will move in once each retro is written.)
