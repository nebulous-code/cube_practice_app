// Practice flashcard screen — the core flow.
// Shows: pattern (front) -> algorithm + expected result (back) -> grade buttons.
const { useState: useStateP, useEffect: useEffectP, useMemo: useMemoP } = React;

function PracticeScreen({ queue, progress, onGrade, onExit, onViewCase }) {
  const [index, setIndex] = useStateP(0);
  const [revealed, setRevealed] = useStateP(false);
  const [sessionResults, setSessionResults] = useStateP([]); // {id, rating}

  const current = queue[index];
  const done = index >= queue.length;

  useEffectP(() => { setRevealed(false); }, [index]);

  if (done) {
    return <SessionComplete results={sessionResults} onExit={onExit} queue={queue} />;
  }
  if (!current) return null;

  const resultCase = caseById(current.result.id);
  const resultPattern = resultCase
    ? rotatePattern(resultCase.pattern, current.result.rotation)
    : current.pattern;

  const handleGrade = (rating) => {
    setSessionResults(r => [...r, { id: current.id, rating }]);
    onGrade(current.id, rating);
    setTimeout(() => setIndex(i => i + 1), 150);
  };

  const caseHistory = progress[current.id]?.history || [];
  const curScore = scoreFromHistory(caseHistory);
  const curGrade = scoreToGrade(curScore);

  return (
    <div style={{
      display: 'flex', flexDirection: 'column', height: '100%',
      background: paper.bg, fontFamily: fonts.sans, color: paper.ink,
    }}>
      {/* top bar */}
      <div style={{
        display: 'flex', justifyContent: 'space-between', alignItems: 'center',
        padding: '56px 20px 10px',
      }}>
        <button onClick={onExit} style={{
          background: 'none', border: 'none', padding: 0,
          fontFamily: fonts.sans, fontSize: 13, color: paper.inkMuted,
          cursor: 'pointer', display: 'flex', alignItems: 'center', gap: 6,
        }}>
          <span style={{ fontSize: 16, lineHeight: 1 }}>×</span> End session
        </button>
        <Eyebrow>{index + 1} of {queue.length}</Eyebrow>
      </div>

      {/* progress bar */}
      <div style={{ padding: '0 20px 10px', display: 'flex', gap: 3 }}>
        {queue.map((_, i) => {
          const past = sessionResults[i];
          let bg = paper.ruleFaint;
          if (past) {
            const rt = RATINGS[past.rating];
            bg = rt.color;
          } else if (i === index) bg = paper.ink;
          return <div key={i} style={{ flex: 1, height: 3, background: bg, borderRadius: 2 }} />;
        })}
      </div>

      {/* card body */}
      <div style={{
        flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center',
        justifyContent: 'flex-start', padding: '10px 20px 20px',
        overflow: 'auto',
      }}>
        {/* case header — small & muted */}
        <div style={{
          display: 'flex', alignItems: 'center', gap: 10, marginBottom: 4,
          fontFamily: fonts.sans, fontSize: 12, color: paper.inkMuted,
          letterSpacing: 0.6, textTransform: 'uppercase',
        }}>
          <span>Case {String(current.id).padStart(2, '0')}</span>
          {current.name && <span style={{ color: paper.inkFaint }}>· {current.name}</span>}
          <GradePip grade={curGrade} size={18} />
        </div>

        {/* big pattern */}
        <div style={{ marginTop: 20, marginBottom: 20 }}>
          <PatternDiagram pattern={current.pattern} size={240} />
        </div>

        {!revealed ? (
          <>
            <div style={{
              fontFamily: fonts.serif, fontSize: 22, fontStyle: 'italic',
              color: paper.inkMuted, marginBottom: 4,
            }}>
              Execute, then check.
            </div>
            <div style={{
              fontFamily: fonts.sans, fontSize: 13, color: paper.inkFaint,
              marginBottom: 24, textAlign: 'center', maxWidth: 280,
            }}>
              From a solved yellow top, apply your algorithm and verify the resulting shape matches.
            </div>
            <Button size="lg" onClick={() => setRevealed(true)} style={{ width: '100%', maxWidth: 300 }}>
              Reveal answer
            </Button>
          </>
        ) : (
          <RevealedContent
            current={current}
            resultCase={resultCase}
            resultPattern={resultPattern}
            onViewCase={onViewCase}
            onGrade={handleGrade}
          />
        )}
      </div>
    </div>
  );
}

function RevealedContent({ current, resultCase, resultPattern, onViewCase, onGrade }) {
  return (
    <>
      {/* Algorithm */}
      <div style={{ width: '100%', marginBottom: 16 }}>
        <Eyebrow style={{ marginBottom: 6 }}>Algorithm</Eyebrow>
        <div style={{
          fontFamily: fonts.mono, fontSize: 17, color: paper.ink,
          lineHeight: 1.6, padding: '10px 14px',
          background: paper.card, border: `1px solid ${paper.ruleFaint}`,
          borderRadius: 10, letterSpacing: 0.5,
        }}>
          {current.algorithm}
        </div>
      </div>

      {/* Expected result */}
      <div style={{ width: '100%', marginBottom: 18 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline', marginBottom: 8 }}>
          <Eyebrow>Should become</Eyebrow>
          <button onClick={() => onViewCase(current.id)} style={{
            background: 'none', border: 'none', padding: 0,
            fontFamily: fonts.sans, fontSize: 11, color: paper.accent,
            cursor: 'pointer', letterSpacing: 0.4, textTransform: 'uppercase',
          }}>
            Wrong? Edit →
          </button>
        </div>
        <div style={{
          display: 'flex', gap: 14, alignItems: 'center',
          background: paper.card, border: `1px solid ${paper.ruleFaint}`,
          borderRadius: 10, padding: 12,
        }}>
          <PatternDiagram pattern={resultPattern} size={78} />
          <div style={{ flex: 1, minWidth: 0 }}>
            <div style={{
              fontFamily: fonts.serif, fontSize: 18, color: paper.ink,
              lineHeight: 1.2, marginBottom: 2,
            }}>
              Case {String(current.result.id).padStart(2, '0')}
              {resultCase?.name && <span style={{ color: paper.inkMuted, fontStyle: 'italic' }}> · {resultCase.name}</span>}
            </div>
            <div style={{ fontFamily: fonts.sans, fontSize: 12, color: paper.inkFaint }}>
              {current.result.rotation === 0 ? 'no rotation' :
               current.result.rotation === 1 ? 'rotated 90° CW' :
               current.result.rotation === 2 ? 'rotated 180°' : 'rotated 90° CCW'}
            </div>
          </div>
        </div>
      </div>

      {/* Grade buttons */}
      <div style={{ width: '100%' }}>
        <Eyebrow style={{ marginBottom: 8 }}>How did it go?</Eyebrow>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
          {RATINGS.map(r => (
            <button key={r.key} onClick={() => onGrade(r.value)} style={{
              display: 'flex', flexDirection: 'column', alignItems: 'flex-start',
              padding: '10px 14px', borderRadius: 12,
              border: `1px solid ${r.color}40`,
              background: `${r.color}14`,
              color: paper.ink, cursor: 'pointer',
              fontFamily: fonts.sans,
            }}>
              <span style={{ fontSize: 15, fontWeight: 600, letterSpacing: -0.1 }}>{r.label}</span>
              <span style={{ fontSize: 11, color: paper.inkMuted, marginTop: 2 }}>
                {r.key === 'fail' && 'Missed it'}
                {r.key === 'hard' && 'Got it slowly'}
                {r.key === 'good' && 'Solid recall'}
                {r.key === 'easy' && 'Instant'}
              </span>
            </button>
          ))}
        </div>
      </div>
    </>
  );
}

function SessionComplete({ results, onExit, queue }) {
  const counts = { 0: 0, 1: 0, 2: 0, 3: 0 };
  results.forEach(r => counts[r.rating]++);
  const avg = results.reduce((s, r) => s + r.rating, 0) / (results.length || 1);
  return (
    <div style={{
      display: 'flex', flexDirection: 'column', height: '100%', background: paper.bg,
      padding: 24, fontFamily: fonts.sans, color: paper.ink,
    }}>
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', justifyContent: 'center', alignItems: 'center' }}>
        <Eyebrow style={{ marginBottom: 14 }}>Session complete</Eyebrow>
        <div style={{
          fontFamily: fonts.serif, fontSize: 54, lineHeight: 1, marginBottom: 6,
          letterSpacing: -1,
        }}>
          {results.length} cases
        </div>
        <div style={{
          fontFamily: fonts.serif, fontStyle: 'italic', fontSize: 19,
          color: paper.inkMuted, marginBottom: 40,
        }}>
          {avg >= 2.3 ? 'A confident round.' :
           avg >= 1.5 ? 'Steady progress.' :
           avg >= 0.8 ? 'Grinding away.' : 'Tough set — reschedule soon.'}
        </div>

        <div style={{
          width: '100%', background: paper.card, borderRadius: 14,
          border: `1px solid ${paper.ruleFaint}`, padding: 18,
        }}>
          {RATINGS.map(r => (
            <div key={r.key} style={{
              display: 'flex', alignItems: 'center', gap: 10, padding: '6px 0',
            }}>
              <div style={{ width: 8, height: 8, borderRadius: 4, background: r.color }} />
              <div style={{ flex: 1, fontSize: 14 }}>{r.label}</div>
              <div style={{ fontFamily: fonts.serif, fontSize: 18, color: paper.ink }}>
                {counts[r.value]}
              </div>
            </div>
          ))}
        </div>
      </div>
      <Button size="lg" onClick={onExit} style={{ width: '100%' }}>Back to home</Button>
    </div>
  );
}

window.PracticeScreen = PracticeScreen;
