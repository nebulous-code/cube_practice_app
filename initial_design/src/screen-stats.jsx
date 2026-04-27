// Stats screen — overall progress view.
function StatsScreen({ progress }) {
  const summary = summarizeProgress(progress, OLL_CASES);
  const total = OLL_CASES.length;

  // Per-group breakdown
  const byGroup = {};
  OLL_CASES.forEach(c => {
    if (!byGroup[c.group]) byGroup[c.group] = { cases: [], scores: [] };
    const score = scoreFromHistory(progress[c.id]?.history);
    byGroup[c.group].cases.push(c);
    byGroup[c.group].scores.push(score);
  });
  const groupRows = Object.keys(GROUP_LABELS).filter(g => byGroup[g]).map(g => {
    const scored = byGroup[g].scores.filter(s => s != null);
    const avg = scored.length ? scored.reduce((a,b) => a+b, 0) / scored.length : null;
    return {
      group: g,
      total: byGroup[g].cases.length,
      practiced: scored.length,
      avg: avg == null ? null : Math.round(avg),
      grade: scoreToGrade(avg),
    };
  });
  groupRows.sort((a, b) => (a.avg ?? -1) - (b.avg ?? -1));

  // Recent activity — count reviews in last 7 / 30 days
  const now = Date.now();
  const day = 86400000;
  let reviews7 = 0, reviews30 = 0;
  Object.values(progress).forEach(p => {
    (p.history || []).forEach(h => {
      if (now - h.ts < 7 * day) reviews7++;
      if (now - h.ts < 30 * day) reviews30++;
    });
  });

  return (
    <div style={{ background: paper.bg, minHeight: '100%', paddingBottom: 90 }}>
      <div style={{ padding: '56px 22px 10px' }}>
        <Eyebrow style={{ marginBottom: 8 }}>Progress</Eyebrow>
        <div style={{
          fontFamily: fonts.serif, fontSize: 38, letterSpacing: -1,
          lineHeight: 1, color: paper.ink,
        }}>
          Where you stand
        </div>
      </div>

      {/* Overall score — big */}
      <div style={{ padding: '20px 22px 0' }}>
        <Card pad={24}>
          <Eyebrow style={{ marginBottom: 12 }}>Overall grade</Eyebrow>
          <div style={{ display: 'flex', alignItems: 'center', gap: 20 }}>
            <div style={{
              fontFamily: fonts.serif, fontSize: 92, lineHeight: 0.9,
              letterSpacing: -3, color: paper.ink,
            }}>
              {scoreToGrade(summary.overallScore)}
            </div>
            <div style={{ flex: 1 }}>
              <div style={{
                fontFamily: fonts.serif, fontStyle: 'italic', fontSize: 19,
                color: paper.inkMuted, lineHeight: 1.2, marginBottom: 8,
              }}>
                {summary.overallScore}/100
              </div>
              {/* Grade distribution bars */}
              <div style={{ display: 'flex', height: 10, borderRadius: 6, overflow: 'hidden' }}>
                {['A','B','C','D','F','New'].map(g => {
                  const n = summary.counts[g];
                  if (!n) return null;
                  const w = (n / total) * 100;
                  return (
                    <div key={g} title={`${g}: ${n}`} style={{
                      width: `${w}%`, background: GRADE_META[g].color,
                      opacity: g === 'New' ? 0.25 : 0.85,
                    }} />
                  );
                })}
              </div>
              <div style={{
                display: 'flex', justifyContent: 'space-between', marginTop: 8,
                fontFamily: fonts.sans, fontSize: 11, color: paper.inkFaint,
              }}>
                <span>{summary.counts.A + summary.counts.B} solid</span>
                <span>{summary.counts.C + summary.counts.D + summary.counts.F} to work on</span>
                <span>{summary.counts.New} new</span>
              </div>
            </div>
          </div>
        </Card>
      </div>

      {/* Activity */}
      <div style={{ padding: '16px 22px 0' }}>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 10 }}>
          <Card pad={16}>
            <Eyebrow style={{ marginBottom: 6 }}>This week</Eyebrow>
            <div style={{ fontFamily: fonts.serif, fontSize: 30, lineHeight: 1, color: paper.ink }}>
              {reviews7}
            </div>
            <div style={{ fontFamily: fonts.sans, fontSize: 11, color: paper.inkFaint, marginTop: 4 }}>
              reviews
            </div>
          </Card>
          <Card pad={16}>
            <Eyebrow style={{ marginBottom: 6 }}>Last 30 days</Eyebrow>
            <div style={{ fontFamily: fonts.serif, fontSize: 30, lineHeight: 1, color: paper.ink }}>
              {reviews30}
            </div>
            <div style={{ fontFamily: fonts.sans, fontSize: 11, color: paper.inkFaint, marginTop: 4 }}>
              reviews
            </div>
          </Card>
        </div>
      </div>

      {/* Per-group */}
      <div style={{ padding: '22px 22px 0' }}>
        <Eyebrow style={{ marginBottom: 12 }}>By shape group</Eyebrow>
        <Card pad={0}>
          {groupRows.map((r, i) => (
            <div key={r.group} style={{
              display: 'flex', alignItems: 'center', gap: 12,
              padding: '12px 16px',
              borderBottom: i < groupRows.length - 1 ? `1px solid ${paper.ruleFaint}` : 'none',
            }}>
              <GradePip grade={r.grade} size={22} />
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{
                  fontFamily: fonts.serif, fontSize: 15, color: paper.ink,
                  letterSpacing: -0.2,
                }}>
                  {GROUP_LABELS[r.group]}
                </div>
                <div style={{ fontFamily: fonts.sans, fontSize: 11, color: paper.inkFaint }}>
                  {r.practiced}/{r.total} practiced
                  {r.avg != null && <span> · avg {r.avg}</span>}
                </div>
              </div>
              {/* micro bar */}
              <div style={{
                width: 64, height: 6, background: paper.ruleFaint, borderRadius: 3,
                overflow: 'hidden',
              }}>
                <div style={{
                  height: '100%', width: `${r.avg ?? 0}%`,
                  background: r.avg != null ? GRADE_META[r.grade].color : 'transparent',
                }} />
              </div>
            </div>
          ))}
        </Card>
      </div>
    </div>
  );
}

window.StatsScreen = StatsScreen;
