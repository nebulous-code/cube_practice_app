// Lightweight SRS + letter-grade system.
// We store per-case history: array of { rating: 0..3, ts }
// rating: 0=Fail, 1=Hard, 2=Good, 3=Easy
// Using a simplified SM-2-ish algorithm.

const RATINGS = [
  { key: 'fail', label: 'Fail', value: 0, color: '#B84A3F' },
  { key: 'hard', label: 'Hard', value: 1, color: '#C68A3A' },
  { key: 'good', label: 'Good', value: 2, color: '#5E7C58' },
  { key: 'easy', label: 'Easy', value: 3, color: '#3F6B8A' },
];

// Compute a composite 0..100 score from a case's history.
// Recent reviews weigh more. Fresh cases (no reviews) return null.
function scoreFromHistory(history) {
  if (!history || history.length === 0) return null;
  // Exponential recency weights, most recent = 1.0, each older = 0.7x
  let num = 0, den = 0;
  const recent = history.slice(-10); // cap to last 10
  for (let i = 0; i < recent.length; i++) {
    const w = Math.pow(0.75, recent.length - 1 - i);
    const v = recent[i].rating; // 0..3
    num += w * (v / 3); // normalize to 0..1
    den += w;
  }
  const base = num / den; // 0..1
  // Penalty for very few reps
  const repBonus = Math.min(1, recent.length / 4);
  return Math.round(base * 100 * (0.7 + 0.3 * repBonus));
}

function scoreToGrade(score) {
  if (score == null) return 'New';
  if (score >= 88) return 'A';
  if (score >= 75) return 'B';
  if (score >= 60) return 'C';
  if (score >= 45) return 'D';
  return 'F';
}

const GRADE_META = {
  'A':   { color: '#5E7C58', bg: '#E6EBDD', label: 'Solid' },
  'B':   { color: '#6D7A4E', bg: '#EAEADA', label: 'Mostly there' },
  'C':   { color: '#B08A3A', bg: '#F2E8CE', label: 'Needs work' },
  'D':   { color: '#C2753A', bg: '#F2DFCA', label: 'Weak' },
  'F':   { color: '#B84A3F', bg: '#F2D2CD', label: 'Failing' },
  'New': { color: '#7A6E5A', bg: '#EDE8DB', label: 'Not yet practiced' },
};

// Next interval (days) given rating — used for a simple "due" display
function nextIntervalDays(prevInterval, rating) {
  if (rating === 0) return 0; // fail -> review same day
  if (prevInterval == null || prevInterval === 0) {
    return [0, 1, 2, 4][rating];
  }
  const mult = [0.2, 1.2, 2.5, 3.5][rating];
  return Math.max(1, Math.round(prevInterval * mult));
}

// Given full progress map { [caseId]: { history, interval, lastReviewed } },
// return summary stats.
function summarizeProgress(progress, cases) {
  const perCase = cases.map(c => {
    const p = progress[c.id] || {};
    const score = scoreFromHistory(p.history);
    return { id: c.id, score, grade: scoreToGrade(score), history: p.history || [], lastReviewed: p.lastReviewed };
  });
  const counts = { A: 0, B: 0, C: 0, D: 0, F: 0, New: 0 };
  perCase.forEach(x => counts[x.grade]++);
  const overallScore = perCase
    .filter(x => x.score != null)
    .reduce((s, x) => s + x.score, 0) / Math.max(1, perCase.filter(x => x.score != null).length);
  return { perCase, counts, overallScore: Math.round(overallScore || 0) };
}

// Build practice queue: weakest cases first, plus new cases interleaved
function buildQueue(progress, cases, { scope = 'weakest', size = 20, filter = null } = {}) {
  let pool = cases;
  if (filter?.priority) pool = pool.filter(c => filter.priority.includes(c.priority));
  if (filter?.group)    pool = pool.filter(c => filter.group.includes(c.group));

  const withScore = pool.map(c => {
    const p = progress[c.id] || {};
    const score = scoreFromHistory(p.history);
    return { case: c, score, grade: scoreToGrade(score) };
  });

  if (scope === 'weakest') {
    // Sort: F, D, C, B, New mixed in, then A last
    const order = { F: 0, D: 1, C: 2, B: 3, New: 4, A: 5 };
    withScore.sort((a, b) => order[a.grade] - order[b.grade] || (a.score ?? 0) - (b.score ?? 0));
  } else if (scope === 'all') {
    // shuffle
    for (let i = withScore.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [withScore[i], withScore[j]] = [withScore[j], withScore[i]];
    }
  }
  return withScore.slice(0, size).map(x => x.case);
}

Object.assign(window, {
  RATINGS, scoreFromHistory, scoreToGrade, GRADE_META,
  nextIntervalDays, summarizeProgress, buildQueue,
});
