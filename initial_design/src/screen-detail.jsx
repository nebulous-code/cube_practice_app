// Case detail screen — unified edit mode for algorithm + result.
function CaseDetailScreen({ caseId, progress, onBack, onEditCase }) {
  const c = caseById(caseId);
  const [editing, setEditing] = useStateP(false);
  const [draftAlg, setDraftAlg] = useStateP('');
  const [draftName, setDraftName] = useStateP('');
  const [draftResultId, setDraftResultId] = useStateP(0);
  const [draftRotation, setDraftRotation] = useStateP(0);

  useEffectP(() => {
    if (c && editing) {
      setDraftAlg(c.algorithm);
      setDraftName(c.name || '');
      setDraftResultId(c.result.id);
      setDraftRotation(c.result.rotation);
    }
  }, [editing, c?.id]);

  if (!c) return null;

  const resultCase = caseById(c.result.id);
  const resultPattern = resultCase ? rotatePattern(resultCase.pattern, c.result.rotation) : c.pattern;
  const score = scoreFromHistory(progress[c.id]?.history);
  const grade = scoreToGrade(score);
  const history = progress[c.id]?.history || [];

  // Preview result while editing
  const previewResultCase = caseById(draftResultId);
  const previewResultPattern = previewResultCase
    ? rotatePattern(previewResultCase.pattern, draftRotation)
    : c.pattern;

  const saveEdits = () => {
    const id = parseInt(draftResultId);
    onEditCase(c.id, {
      algorithm: draftAlg.trim() || c.algorithm,
      name: draftName.trim() || null,
      result: {
        id: caseById(id) ? id : c.result.id,
        rotation: ((draftRotation % 4) + 4) % 4,
      },
    });
    setEditing(false);
  };

  return (
    <div style={{ background: paper.bg, minHeight: '100%', paddingBottom: 40 }}>
      {/* header */}
      <div style={{ padding: '52px 22px 10px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
          <button onClick={onBack} style={{
            background: 'none', border: 'none', padding: 0,
            fontFamily: fonts.sans, fontSize: 13, color: paper.inkMuted, cursor: 'pointer',
          }}>← Back</button>
          {!editing ? (
            <button onClick={() => setEditing(true)} style={{
              background: 'none', border: 'none', padding: 0,
              fontFamily: fonts.sans, fontSize: 12, color: paper.accent,
              cursor: 'pointer', letterSpacing: 0.6, textTransform: 'uppercase',
            }}>Edit</button>
          ) : (
            <div style={{ display: 'flex', gap: 12 }}>
              <button onClick={() => setEditing(false)} style={{
                background: 'none', border: 'none', padding: 0,
                fontFamily: fonts.sans, fontSize: 12, color: paper.inkMuted,
                cursor: 'pointer', letterSpacing: 0.6, textTransform: 'uppercase',
              }}>Cancel</button>
              <button onClick={saveEdits} style={{
                background: 'none', border: 'none', padding: 0,
                fontFamily: fonts.sans, fontSize: 12, color: paper.accent, fontWeight: 600,
                cursor: 'pointer', letterSpacing: 0.6, textTransform: 'uppercase',
              }}>Save</button>
            </div>
          )}
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <GradePip grade={grade} size={32} />
          <div style={{ flex: 1 }}>
            <Eyebrow>Case {String(c.id).padStart(2, '0')} · {GROUP_LABELS[c.group]}</Eyebrow>
            {editing ? (
              <input
                value={draftName} onChange={e => setDraftName(e.target.value)}
                placeholder="Give it a nickname…"
                style={{
                  width: '100%', background: 'transparent',
                  border: 'none', borderBottom: `1px solid ${paper.rule}`,
                  fontFamily: fonts.serif, fontSize: 26, letterSpacing: -0.4,
                  color: paper.ink, padding: '2px 0', marginTop: 2, outline: 'none',
                }}
              />
            ) : (
              <div style={{
                fontFamily: fonts.serif, fontSize: 28, lineHeight: 1.05,
                letterSpacing: -0.5, color: paper.ink, marginTop: 2,
              }}>
                {c.name || <span style={{ color: paper.inkFaint, fontStyle: 'italic' }}>Unnamed</span>}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* pattern + meta */}
      <div style={{ padding: '14px 22px 0' }}>
        <Card pad={22} style={{ display: 'flex', gap: 18, alignItems: 'center' }}>
          <PatternDiagram pattern={c.pattern} size={120} />
          <div style={{ flex: 1, minWidth: 0 }}>
            <Eyebrow style={{ marginBottom: 6 }}>Group</Eyebrow>
            <div style={{
              fontFamily: fonts.serif, fontSize: 16, color: paper.ink,
              marginBottom: 12, fontStyle: 'italic',
            }}>
              {GROUP_LABELS[c.group]}
            </div>
            <Eyebrow style={{ marginBottom: 4 }}>Priority</Eyebrow>
            <Chip tone={c.priority === '+' ? 'accent' : c.priority === '*' ? 'warn' : 'neutral'}>
              {PRIORITY_LABELS[c.priority]}
            </Chip>
          </div>
        </Card>
      </div>

      {/* Algorithm */}
      <div style={{ padding: '14px 22px 0' }}>
        <Card>
          <Eyebrow style={{ marginBottom: 8 }}>Algorithm</Eyebrow>
          {editing ? (
            <>
              <textarea
                value={draftAlg} onChange={e => setDraftAlg(e.target.value)}
                rows={2}
                style={{
                  width: '100%', background: paper.bg,
                  border: `1px solid ${paper.rule}`, borderRadius: 8,
                  padding: '10px 12px', resize: 'vertical',
                  fontFamily: fonts.mono, fontSize: 16, color: paper.ink,
                  lineHeight: 1.5, letterSpacing: 0.3, outline: 'none',
                  boxSizing: 'border-box',
                }}
              />
              <div style={{
                fontFamily: fonts.sans, fontSize: 11, color: paper.inkFaint,
                marginTop: 6, letterSpacing: 0.3,
              }}>
                Changing the algorithm? Update the result below to match.
              </div>
            </>
          ) : (
            <div style={{
              fontFamily: fonts.mono, fontSize: 17, color: paper.ink,
              lineHeight: 1.6, letterSpacing: 0.3,
            }}>
              {c.algorithm}
            </div>
          )}
        </Card>
      </div>

      {/* Result */}
      <div style={{ padding: '14px 22px 0' }}>
        <Card>
          <Eyebrow style={{ marginBottom: 10 }}>Result after algorithm</Eyebrow>
          <div style={{ display: 'flex', gap: 14, alignItems: 'center' }}>
            <PatternDiagram pattern={editing ? previewResultPattern : resultPattern} size={92} />
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{
                fontFamily: fonts.serif, fontSize: 17, color: paper.ink, lineHeight: 1.2,
              }}>
                Case {String(editing ? draftResultId : c.result.id).padStart(2, '0')}
                {(editing ? previewResultCase : resultCase)?.name && (
                  <span style={{ color: paper.inkMuted, fontStyle: 'italic' }}>
                    {' · '}{(editing ? previewResultCase : resultCase).name}
                  </span>
                )}
              </div>
              <div style={{ fontFamily: fonts.sans, fontSize: 12, color: paper.inkFaint, marginTop: 2 }}>
                {['no rotation','90° clockwise','180°','90° counter-clockwise'][editing ? draftRotation : c.result.rotation]}
              </div>
            </div>
          </div>

          {editing && (
            <div style={{ marginTop: 16 }}>
              <Rule style={{ marginBottom: 14 }} />
              <Eyebrow style={{ marginBottom: 8 }}>Result case</Eyebrow>
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 14 }}>
                <input
                  type="number" min="1" max="57"
                  value={draftResultId}
                  onChange={e => setDraftResultId(parseInt(e.target.value) || 0)}
                  style={{
                    width: 72, padding: '8px 10px',
                    border: `1px solid ${paper.rule}`, borderRadius: 8,
                    fontFamily: fonts.mono, fontSize: 15, background: paper.bg,
                    color: paper.ink, outline: 'none',
                  }}
                />
                <span style={{ fontFamily: fonts.sans, fontSize: 12, color: paper.inkFaint }}>
                  Case number (1–57)
                </span>
              </div>
              <Eyebrow style={{ marginBottom: 8 }}>Rotation</Eyebrow>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr 1fr', gap: 6 }}>
                {[0,1,2,3].map(r => (
                  <button key={r} onClick={() => setDraftRotation(r)} style={{
                    padding: '9px 4px', borderRadius: 8,
                    border: `1px solid ${draftRotation === r ? paper.ink : paper.rule}`,
                    background: draftRotation === r ? paper.ink : 'transparent',
                    color: draftRotation === r ? paper.bg : paper.ink,
                    fontFamily: fonts.sans, fontSize: 11, cursor: 'pointer',
                    letterSpacing: 0.3,
                  }}>
                    {['0°','90° CW','180°','90° CCW'][r]}
                  </button>
                ))}
              </div>
            </div>
          )}
        </Card>
      </div>

      {/* History */}
      <div style={{ padding: '14px 22px 0' }}>
        <Eyebrow style={{ marginBottom: 10 }}>Recent practice</Eyebrow>
        {history.length === 0 ? (
          <div style={{
            fontFamily: fonts.serif, fontStyle: 'italic', fontSize: 15,
            color: paper.inkFaint, padding: '4px 0',
          }}>Not yet practiced.</div>
        ) : (
          <Card pad={14}>
            <div style={{ display: 'flex', gap: 4, marginBottom: 12 }}>
              {history.slice(-14).map((h, i) => (
                <div key={i} style={{
                  flex: 1, height: 26, borderRadius: 3,
                  background: RATINGS[h.rating].color,
                }} />
              ))}
            </div>
            {/* Legend */}
            <div style={{
              display: 'flex', justifyContent: 'space-between', alignItems: 'center',
              marginBottom: 8,
            }}>
              {RATINGS.map(r => (
                <div key={r.key} style={{
                  display: 'flex', alignItems: 'center', gap: 5,
                  fontFamily: fonts.sans, fontSize: 10, color: paper.inkMuted,
                  letterSpacing: 0.3,
                }}>
                  <div style={{ width: 8, height: 8, borderRadius: 2, background: r.color }} />
                  {r.label}
                </div>
              ))}
            </div>
            <Rule color={paper.ruleFaint} style={{ marginBottom: 8 }} />
            <div style={{ fontFamily: fonts.sans, fontSize: 11, color: paper.inkFaint, letterSpacing: 0.3 }}>
              {history.length} reviews · grade {grade} · score {score}/100
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}

window.CaseDetailScreen = CaseDetailScreen;
