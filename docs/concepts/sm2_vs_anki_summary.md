# SM-2 vs. Anki: Why "SM-2" Means Two Different Things

## The mismatch in one sentence

The original SM-2 algorithm uses a 0–5 grading scale; Anki uses a 4-button UI (Again / Hard / Good / Easy) with a *modified* algorithm that's only loosely derived from SM-2. Both are commonly called "SM-2," which is why the project's design doc (faithful to canonical SM-2) and prototype UI (modeled on Anki) appear to conflict.

---

## Canonical SM-2 (Piotr Wozniak, 1988)

The algorithm published in the original SuperMemo paper. This is what `docs/Cube_Practice_Design_Doc.md:163-184` currently specifies.

### Inputs
A quality grade `q ∈ {0, 1, 2, 3, 4, 5}` per review:

| Grade | Meaning |
|------:|---------|
| 5 | Perfect response |
| 4 | Correct response after a hesitation |
| 3 | Correct response, recalled with serious difficulty |
| 2 | Incorrect response; the correct one seemed easy to recall |
| 1 | Incorrect response; the correct one was remembered |
| 0 | Complete blackout |

### Update rule
```
if q < 3:
    repetitions = 0
    interval_days = 1
else:
    if repetitions == 0: interval_days = 1
    elif repetitions == 1: interval_days = 6
    else: interval_days = round(interval_days * ease_factor)
    repetitions += 1

ease_factor = ease_factor + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))
ease_factor = max(1.3, ease_factor)
due_date = today + interval_days
```

### Notable consequences
- Any grade `q < 3` is treated as a failure and resets `repetitions` to 0.
- The ease delta is a smooth quadratic in `q`. Small differences in grade produce small differences in ease.
- A 6-button grading UI is implied, which is genuinely awkward on mobile.

---

## Anki's "SM-2" (what most people experience)

Anki's official docs say its scheduler is "derived from SM-2" but has been "significantly changed." The 4-button UI (Again / Hard / Good / Easy) is not just a re-skin — the algorithm under it is different too.

### Differences from canonical SM-2

| | Canonical SM-2 | Anki |
|--|--|--|
| Grading scale | 0–5 (six values) | 4 buttons |
| Failure paths | Three (`q ∈ {0,1,2}`) | One (`Again`) |
| Ease update | Smooth formula in `q` | Flat: `Hard = −0.15`, `Good = 0`, `Easy = +0.15` |
| Initial intervals | Fixed: 1 day → 6 days → `interval × ease` | User-configurable "learning steps" before the card graduates |
| Ease during learning | Affected by every grade | Untouched until the card graduates — prevents "ease hell" |
| Easy bonus | None | Configurable extra interval multiplier on Easy |
| Late reviews | Not modeled | Bonus applied if a card was answered correctly past its due date |
| Ease floor | 1.3 | 1.3 |

### Why Anki diverged
- A 4-button UI is friendlier on mobile and matches user mental models better.
- Multiple failure grades didn't add value in practice — most failures are alike, and ease drift on the success side is what actually controls scheduling.
- The "ease hell" failure mode (a card stuck repeatedly failing → very low ease → very short intervals → fails again) is fixed by isolating learning from ease updates.

---

## Implications for this project

### The original §1.1 of `outstanding_decision.md`
The proposal "map 0&1 → button 0, 2&3 → button 1, 4 → button 2, 5 → button 3" maps SM-2 grades *to* buttons, but at runtime the direction is reversed: a user clicks one of four buttons, and we need to derive a single SM-2 grade. The cleanest reduction in that direction (`Fail=0, Hard=2, Good=4, Easy=5`) still has a problem: under canonical SM-2 any grade below 3 resets `repetitions` to 0, so a "Hard" press would wipe the card's progress — the opposite of what Anki users expect.

### Three coherent paths

1. **Adopt Anki's modified algorithm wholesale.** Keep the 4-button UI from the prototype. Replace the canonical SM-2 ease formula with Anki's flat ±0.15 rule. Keep the SM-2 *data shape* (`ease_factor`, `interval_days`, `repetitions`, `due_date`) so the schema in `Cube_Practice_Design_Doc.md` §3 is unchanged — only §4 (the algorithm) needs to be rewritten. Recommended.

2. **Stay strictly canonical SM-2.** Switch the UI to six grading buttons (or a slider 0–5). Faithful to the original paper, worse mobile UX, and at odds with what users coming from Anki expect.

3. **Hybrid: 4-button UI, canonical formula.** Map buttons to `{1, 3, 4, 5}` (avoiding the reset-zone except for an explicit Fail). Preserves the canonical math but loses the Anki semantic that "Hard = got it slowly, count it as a success."

### Recommendation
Option 1. The prototype already assumes Anki's UX, the algorithm change is small and well-documented, and §1.2 of `outstanding_decision.md` (scheduling model) collapses into the same answer: keep SM-2's data shape, use Anki's update rule on top of it.

If we go with option 1, `Cube_Practice_Design_Doc.md` §4 should be rewritten to describe Anki's rule explicitly rather than the canonical formula.

---

## Sources
- [What spaced repetition algorithm does Anki use? — Anki FAQs](https://faqs.ankiweb.net/what-spaced-repetition-algorithm.html)
- [Anki SRS Algorithm — Julien Sobczak](https://juliensobczak.com/inspect/2022/05/30/anki-srs/)
- [The Anki SM-2 Spaced Repetition Algorithm — RemNote Help Center](https://help.remnote.com/en/articles/6026144-the-anki-sm-2-spaced-repetition-algorithm)
- [SM-2 Algorithm Pseudo-code — Anki Forums](https://forums.ankiweb.net/t/sm-2-algorithm-pseudo-code/8350)
