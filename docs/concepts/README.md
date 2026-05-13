# concepts/

Evergreen explanatory docs about the philosophy and domain knowledge that
underpins Quiet Cube. These aren't reference material (the live spec is
`docs/ARCHITECTURE.md`) and they aren't a phase log (that's
`docs/CHANGELOG.md` and `docs/milestones/`). They're the *why* of the system —
the parts that need explaining to someone working on the code, and that
don't change when the code changes.

## Contents

- [`sm2_vs_anki_summary.md`](sm2_vs_anki_summary.md) — why this app uses
  Anki's variant of SM-2 rather than canonical SM-2. Read this before
  touching `backend/src/srs/` or thinking about new grading semantics.
- [`oll_practice.md`](oll_practice.md) — the 57 OLL cases as a
  human-readable reference (number, nickname, algorithm, pattern, result).
  The seed migration `0003_seed_oll_cases.sql` is the source of truth for
  the database; this doc is the source of truth for *understanding* what
  the app is teaching.
