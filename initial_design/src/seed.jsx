// Seed data — plausible practice history so the dashboard has signal.
// Deterministic pseudo-random based on case id.

function seedProgress() {
  const now = Date.now();
  const day = 86400000;
  const progress = {};

  // Cases 49,50,51,52 (OCLL/Solves) = mostly Good/Easy → A's and B's
  // Cases 12-17 (small_L) = recently learning → mix of B/C/D
  // Dot cases (1-7) = hardest → F/D
  // Fish cases = mix

  const profiles = {
    solid:    () => [2, 3, 2, 3, 2, 2, 3, 2, 3, 2],
    learning: () => [1, 2, 1, 2, 2, 1, 2, 2],
    weak:     () => [0, 1, 0, 1, 2, 1, 0, 1],
    rough:    () => [0, 0, 1, 0, 1, 0, 1],
    fresh:    () => [2, 2],
    new:      () => null,
  };

  const assign = (id) => {
    // OCLL solves — known well
    if ([49, 50, 51, 52].includes(id)) return 'solid';
    if ([55, 56, 57].includes(id)) return 'fresh';
    // Lightning bolts / squares — learning
    if ([20, 21, 22, 23, 24, 25].includes(id)) return 'learning';
    // P shapes — learning
    if ([35, 36, 37, 38].includes(id)) return 'learning';
    // Small L — rough (no nicknames, recently added)
    if ([12, 13, 14, 15, 16, 17].includes(id)) return 'rough';
    // Fish
    if ([18, 19, 31, 32].includes(id)) return 'learning';
    // Dot — hardest
    if ([1, 2, 3, 4, 5, 6, 7, 30].includes(id)) return 'weak';
    // Awkward
    if ([39, 40, 41, 42].includes(id)) return 'rough';
    // W shapes
    if ([33, 34].includes(id)) return 'learning';
    // Corners correct
    if ([53, 54].includes(id)) return 'weak';
    // Knight moves
    if ([26, 27, 28, 29].includes(id)) return 'new';
    // T, C, I, Lightning -43/44 — not studying
    return 'new';
  };

  OLL_CASES.forEach((c, idx) => {
    const prof = assign(c.id);
    const ratings = profiles[prof]();
    if (!ratings) return;
    const history = ratings.map((r, i) => ({
      rating: r,
      ts: now - (ratings.length - i) * day * (1 + (idx % 3)),
    }));
    progress[c.id] = {
      history,
      lastReviewed: history[history.length - 1].ts,
      interval: 2,
    };
  });

  return progress;
}

window.seedProgress = seedProgress;
