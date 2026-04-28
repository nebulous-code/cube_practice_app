// Home / Dashboard screen — with streak KPIs and colored standings.

// Streak: consecutive days (ending today or yesterday) with at least one review.
function computeStreak(progress) {
  const dates = new Set();
  Object.values(progress).forEach(p => {
    (p.history || []).forEach(h => {
      const d = new Date(h.ts);
      dates.add(`${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`);
    });
  });
  const key = (d) => `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`;
  let streak = 0;
  const cursor = new Date();
  // Allow today OR yesterday as streak anchor (grace for not having practiced yet today)
  if (!dates.has(key(cursor))) cursor.setDate(cursor.getDate() - 1);
  while (dates.has(key(cursor))) {
    streak++;
    cursor.setDate(cursor.getDate() - 1);
  }
  return streak;
}

function reviewsToday(progress) {
  const now = new Date();
  const startOfDay = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  let n = 0;
  Object.values(progress).forEach(p => {
    (p.history || []).forEach(h => { if (h.ts >= startOfDay) n++; });
  });
  return n;
}

function HomeScreen({ progress, onStartPractice, onBrowse, onOpenStats, user, onOpenSettings }) {
  const summary = summarizeProgress(progress, OLL_CASES);
  const weakCount = summary.counts.F + summary.counts.D + summary.counts.C;
  const dueCount = Math.max(weakCount, 8);
  const streak = computeStreak(progress);
  const todayReviews = reviewsToday(progress);

  const previewCases = buildQueue(progress, OLL_CASES, { scope: 'weakest', size: 3 });

  const today = new Date();
  const dateStr = today.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' });

  const overallGrade = scoreToGrade(summary.overallScore);

  return (
    <div style={{
      background: paper.bg, minHeight: '100%', fontFamily: fonts.sans,
      color: paper.ink, paddingBottom: 90,
    }}>
      {/* Masthead */}
      <div style={{ padding: '56px 22px 12px', position: 'relative' }}>
        <button onClick={onOpenSettings} style={{
          position: 'absolute', top: 56, right: 22,
          background: 'none', border: 'none', padding: 0, cursor: 'pointer',
          display: 'flex', alignItems: 'center', gap: 6,
        }}>
          {!user && (
            <span style={{
              fontFamily: fonts.sans, fontSize: 10, letterSpacing: 1.2,
              textTransform: 'uppercase', color: paper.accent, fontWeight: 600,
            }}>
              Sign up
            </span>
          )}
          <Avatar name={user?.name} size={38} />
        </button>
        <Eyebrow style={{ marginBottom: 10 }}>{dateStr}</Eyebrow>
        <div style={{
          fontFamily: fonts.serif, fontSize: 42, lineHeight: 1.02,
          letterSpacing: -1.2, color: paper.ink,
        }}>
          OLL
          <span style={{ fontStyle: 'italic', color: paper.inkMuted }}> practice</span>
        </div>
      </div>

      {/* Streak KPI row */}
      <div style={{ padding: '14px 22px 0' }}>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 8 }}>
          <KpiCard label="Streak" value={streak} unit={streak === 1 ? 'day' : 'days'} emphasis />
          <KpiCard label="Today" value={todayReviews} unit={todayReviews === 1 ? 'review' : 'reviews'} />
          <KpiCard label="Due" value={dueCount} unit="cases" />
        </div>
      </div>

      {/* Main CTA — weakest cases */}
      <div style={{ padding: '16px 22px 0' }}>
        <div style={{
          background: paper.ink, color: paper.bg, borderRadius: 18,
          padding: '22px 22px 20px', position: 'relative', overflow: 'hidden',
        }}>
          <div style={{
            fontFamily: fonts.sans, fontSize: 11, letterSpacing: 1.4,
            textTransform: 'uppercase', opacity: 0.55, marginBottom: 10,
          }}>Today's queue</div>
          <div style={{
            fontFamily: fonts.serif, fontSize: 38, lineHeight: 1, letterSpacing: -1,
            marginBottom: 4,
          }}>{dueCount} cases</div>
          <div style={{
            fontFamily: fonts.serif, fontStyle: 'italic', fontSize: 16,
            opacity: 0.7, marginBottom: 20,
          }}>
            weakest first · B's and below
          </div>

          <div style={{ display: 'flex', gap: 10, marginBottom: 18 }}>
            {previewCases.map((c, i) => (
              <div key={c.id} style={{
                background: paper.bg, borderRadius: 10, padding: 6,
                flex: i === 0 ? 1.1 : 1,
              }}>
                <PatternDiagram pattern={c.pattern} size={i === 0 ? 72 : 60} />
              </div>
            ))}
            <div style={{
              flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center',
              background: 'rgba(255,255,255,0.05)', borderRadius: 10,
              fontFamily: fonts.serif, fontSize: 22, color: paper.bg, opacity: 0.5,
            }}>
              +{Math.max(0, dueCount - 3)}
            </div>
          </div>

          <button onClick={() => onStartPractice('weakest')} style={{
            width: '100%', background: paper.bg, color: paper.ink,
            border: 'none', padding: '14px', borderRadius: 12,
            fontFamily: fonts.sans, fontSize: 15, fontWeight: 600,
            cursor: 'pointer', letterSpacing: 0.2,
          }}>
            Begin session →
          </button>
        </div>
      </div>

      {/* Standing — now with color */}
      <div style={{ padding: '28px 22px 0' }}>
        <Eyebrow style={{ marginBottom: 14 }}>Standing</Eyebrow>
        <Card pad={18}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 14, marginBottom: 14 }}>
            <div style={{
              width: 64, height: 64, borderRadius: 14,
              background: GRADE_META[overallGrade].bg,
              color: GRADE_META[overallGrade].color,
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              fontFamily: fonts.serif, fontSize: 42, fontWeight: 500,
              letterSpacing: -1.5, lineHeight: 1,
            }}>
              {overallGrade === 'New' ? '·' : overallGrade}
            </div>
            <div style={{ flex: 1 }}>
              <div style={{ fontFamily: fonts.sans, fontSize: 13, color: paper.inkMuted }}>
                Overall · {summary.overallScore}/100
              </div>
              <div style={{
                fontFamily: fonts.serif, fontStyle: 'italic',
                fontSize: 16, color: paper.ink, marginTop: 2,
              }}>
                {GRADE_META[overallGrade].label.toLowerCase()}
              </div>
            </div>
          </div>

          {/* Grade distribution bar using grade colors */}
          <div style={{ display: 'flex', height: 8, borderRadius: 5, overflow: 'hidden', marginBottom: 14 }}>
            {['A','B','C','D','F','New'].map(g => {
              const n = summary.counts[g];
              if (!n) return null;
              const w = (n / OLL_CASES.length) * 100;
              return (
                <div key={g} style={{
                  width: `${w}%`, background: GRADE_META[g].color,
                  opacity: g === 'New' ? 0.35 : 1,
                }} />
              );
            })}
          </div>

          <div style={{ display: 'flex', justifyContent: 'space-between', gap: 4 }}>
            {['A','B','C','D','F','New'].map(g => (
              <div key={g} style={{
                flex: 1, textAlign: 'center',
                background: GRADE_META[g].bg, borderRadius: 8,
                padding: '8px 2px',
              }}>
                <div style={{
                  fontFamily: fonts.serif, fontSize: 18,
                  color: GRADE_META[g].color, fontWeight: 600, lineHeight: 1,
                }}>
                  {summary.counts[g]}
                </div>
                <div style={{
                  fontFamily: fonts.sans, fontSize: 9.5, letterSpacing: 0.6,
                  color: GRADE_META[g].color, textTransform: 'uppercase',
                  marginTop: 4, opacity: 0.8,
                }}>
                  {g}
                </div>
              </div>
            ))}
          </div>
        </Card>
      </div>

      {/* Quick actions */}
      <div style={{ padding: '22px 22px 0' }}>
        <Eyebrow style={{ marginBottom: 14 }}>Or choose a scope</Eyebrow>
        <div style={{ display: 'grid', gap: 10 }}>
          <ScopeRow
            title="By priority"
            sub="Dots · Learning · Known"
            onClick={() => onBrowse('priority')}
          />
          <ScopeRow
            title="By shape group"
            sub="Fish · Squares · Lightning · +11 more"
            onClick={() => onBrowse('group')}
          />
          <ScopeRow
            title="All 57 cases"
            sub="Random draw from the full deck"
            onClick={() => onStartPractice('all')}
          />
        </div>
      </div>

      <div style={{ padding: '28px 22px 0' }}>
        <button onClick={onOpenStats} style={{
          background: 'transparent', border: 'none', padding: 0,
          fontFamily: fonts.sans, fontSize: 13, color: paper.accent,
          cursor: 'pointer', letterSpacing: 0.4,
        }}>
          See full progress →
        </button>
      </div>
    </div>
  );
}

function KpiCard({ label, value, unit, emphasis }) {
  return (
    <div style={{
      background: emphasis ? paper.accentBg : paper.card,
      border: `1px solid ${emphasis ? paper.accentBg : paper.ruleFaint}`,
      borderRadius: 12, padding: '12px 14px',
    }}>
      <div style={{
        fontFamily: fonts.sans, fontSize: 9.5, letterSpacing: 1.2,
        color: emphasis ? paper.accent : paper.inkFaint,
        textTransform: 'uppercase', fontWeight: 500,
      }}>
        {label}
      </div>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 5, marginTop: 6 }}>
        <div style={{
          fontFamily: fonts.serif, fontSize: 28, lineHeight: 1,
          color: emphasis ? paper.accent : paper.ink, letterSpacing: -0.8,
          fontWeight: 500,
        }}>
          {value}
        </div>
        <div style={{
          fontFamily: fonts.sans, fontSize: 10, letterSpacing: 0.3,
          color: emphasis ? paper.accent : paper.inkMuted, opacity: 0.8,
        }}>
          {unit}
        </div>
      </div>
    </div>
  );
}

function ScopeRow({ title, sub, onClick }) {
  return (
    <button onClick={onClick} style={{
      display: 'flex', alignItems: 'center', justifyContent: 'space-between',
      width: '100%', textAlign: 'left', background: paper.card,
      border: `1px solid ${paper.ruleFaint}`, borderRadius: 12,
      padding: '14px 16px', cursor: 'pointer',
      fontFamily: fonts.sans, color: paper.ink,
    }}>
      <div>
        <div style={{ fontSize: 15, fontWeight: 500, letterSpacing: -0.1 }}>{title}</div>
        <div style={{ fontSize: 12, color: paper.inkMuted, marginTop: 2 }}>{sub}</div>
      </div>
      <div style={{ color: paper.inkFaint, fontSize: 18 }}>›</div>
    </button>
  );
}

window.HomeScreen = HomeScreen;
