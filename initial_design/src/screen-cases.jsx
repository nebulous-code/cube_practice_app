// Cases browser — list view grouped by shape.
function CasesScreen({ progress, onOpenCase }) {
  const [filter, setFilter] = useStateP('all'); // 'all' | 'learning' | 'dot' | 'solves' | 'group'
  const [search, setSearch] = useStateP('');

  const filtered = OLL_CASES.filter(c => {
    if (search) {
      const s = search.toLowerCase();
      const matches =
        String(c.id).includes(s) ||
        (c.name && c.name.toLowerCase().includes(s)) ||
        c.algorithm.toLowerCase().includes(s) ||
        GROUP_LABELS[c.group].toLowerCase().includes(s);
      if (!matches) return false;
    }
    if (filter === 'all') return true;
    if (filter === 'learning') return c.priority === 'L';
    if (filter === 'dot') return c.priority === '*';
    if (filter === 'solves') return c.priority === '+';
    return true;
  });

  // Group by shape
  const grouped = {};
  filtered.forEach(c => {
    if (!grouped[c.group]) grouped[c.group] = [];
    grouped[c.group].push(c);
  });
  const groupOrder = Object.keys(GROUP_LABELS).filter(g => grouped[g]);

  return (
    <div style={{ background: paper.bg, minHeight: '100%', paddingBottom: 90 }}>
      {/* Header */}
      <div style={{ padding: '56px 22px 10px' }}>
        <Eyebrow style={{ marginBottom: 8 }}>Reference</Eyebrow>
        <div style={{
          fontFamily: fonts.serif, fontSize: 36, letterSpacing: -0.8,
          lineHeight: 1, color: paper.ink,
        }}>
          All cases <span style={{ color: paper.inkFaint, fontStyle: 'italic' }}>57</span>
        </div>
      </div>

      {/* Search */}
      <div style={{ padding: '14px 22px 0' }}>
        <input
          value={search} onChange={e => setSearch(e.target.value)}
          placeholder="Search nickname, number, algorithm…"
          style={{
            width: '100%', background: paper.card, border: `1px solid ${paper.ruleFaint}`,
            borderRadius: 10, padding: '11px 14px',
            fontFamily: fonts.sans, fontSize: 14, color: paper.ink,
            outline: 'none', boxSizing: 'border-box',
          }}
        />
      </div>

      {/* Filter chips */}
      <div style={{
        display: 'flex', gap: 6, padding: '14px 22px 4px', overflowX: 'auto',
      }}>
        {[
          { k: 'all', label: 'All' },
          { k: 'learning', label: 'Learning' },
          { k: 'dot', label: 'Dot (hard)' },
          { k: 'solves', label: 'Known' },
        ].map(f => (
          <button key={f.k} onClick={() => setFilter(f.k)} style={{
            border: `1px solid ${filter === f.k ? paper.ink : paper.rule}`,
            background: filter === f.k ? paper.ink : 'transparent',
            color: filter === f.k ? paper.bg : paper.inkMuted,
            borderRadius: 999, padding: '6px 12px',
            fontFamily: fonts.sans, fontSize: 12, cursor: 'pointer',
            whiteSpace: 'nowrap', letterSpacing: 0.2,
          }}>{f.label}</button>
        ))}
      </div>

      {/* Grouped list */}
      <div style={{ padding: '16px 22px 0' }}>
        {groupOrder.map(g => (
          <div key={g} style={{ marginBottom: 24 }}>
            <div style={{
              display: 'flex', justifyContent: 'space-between', alignItems: 'baseline',
              marginBottom: 10,
            }}>
              <div style={{
                fontFamily: fonts.serif, fontSize: 18, fontStyle: 'italic',
                color: paper.ink, letterSpacing: -0.2,
              }}>
                {GROUP_LABELS[g]}
              </div>
              <Eyebrow>{grouped[g].length}</Eyebrow>
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 10 }}>
              {grouped[g].map(c => (
                <CaseTile key={c.id} c={c} progress={progress} onClick={() => onOpenCase(c.id)} />
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function CaseTile({ c, progress, onClick }) {
  const score = scoreFromHistory(progress[c.id]?.history);
  const grade = scoreToGrade(score);
  return (
    <button onClick={onClick} style={{
      background: paper.card, border: `1px solid ${paper.ruleFaint}`,
      borderRadius: 10, padding: 8, cursor: 'pointer',
      display: 'flex', flexDirection: 'column', gap: 4,
      fontFamily: fonts.sans, textAlign: 'left',
    }}>
      <div style={{ position: 'relative' }}>
        <PatternDiagram pattern={c.pattern} size={90} />
        <div style={{ position: 'absolute', top: 0, right: 0 }}>
          <GradePip grade={grade} size={18} />
        </div>
      </div>
      <div style={{
        fontFamily: fonts.serif, fontSize: 13, color: paper.ink, lineHeight: 1.1,
        marginTop: 2,
      }}>
        {String(c.id).padStart(2, '0')}
      </div>
      {c.name && (
        <div style={{
          fontFamily: fonts.sans, fontSize: 10, color: paper.inkMuted,
          letterSpacing: 0.2, lineHeight: 1.1,
          overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        }}>
          {c.name}
        </div>
      )}
    </button>
  );
}

window.CasesScreen = CasesScreen;
