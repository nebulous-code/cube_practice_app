// App shell with bottom tab navigation + routing between screens.
function App() {
  const [tab, setTab] = useStateP('home');            // 'home' | 'cases' | 'stats'
  const [route, setRoute] = useStateP(null);           // null | {type: 'practice', scope} | {type: 'detail', id}
  const [progress, setProgress] = useStateP(() => {
    try {
      const saved = localStorage.getItem('oll-progress');
      if (saved) return JSON.parse(saved);
    } catch {}
    return seedProgress();
  });
  const [cases, setCases] = useStateP(OLL_CASES); // mutable for result edits

  useEffectP(() => {
    try { localStorage.setItem('oll-progress', JSON.stringify(progress)); } catch {}
  }, [progress]);

  const handleGrade = (caseId, rating) => {
    setProgress(p => {
      const cur = p[caseId] || { history: [], interval: 0 };
      const newInterval = nextIntervalDays(cur.interval, rating);
      const history = [...(cur.history || []), { rating, ts: Date.now() }];
      return { ...p, [caseId]: { history, interval: newInterval, lastReviewed: Date.now() } };
    });
  };

  const handleStartPractice = (scope) => {
    const queue = buildQueue(progress, cases, { scope, size: scope === 'all' ? 15 : 10 });
    setRoute({ type: 'practice', queue });
  };

  const handleEditCase = (caseId, patch) => {
    setCases(cs => cs.map(c => c.id === caseId ? { ...c, ...patch } : c));
    window.OLL_CASES = window.OLL_CASES.map(c => c.id === caseId ? { ...c, ...patch } : c);
  };

  // Rebind globals if cases changed (for caseById)
  useEffectP(() => {
    window.caseById = (id) => cases.find(c => c.id === id);
  }, [cases]);

  // Render routed screens full-bleed
  if (route?.type === 'practice') {
    return (
      <PracticeScreen
        queue={route.queue}
        progress={progress}
        onGrade={handleGrade}
        onExit={() => setRoute(null)}
        onViewCase={(id) => setRoute({ type: 'detail', id, from: 'practice', queue: route.queue })}
      />
    );
  }
  if (route?.type === 'detail') {
    return (
      <CaseDetailScreen
        caseId={route.id}
        progress={progress}
        onBack={() => {
          if (route.from === 'practice') setRoute({ type: 'practice', queue: route.queue });
          else setRoute(null);
        }}
        onEditCase={handleEditCase}
      />
    );
  }

  // Default tab views
  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column', background: paper.bg }}>
      <div style={{ flex: 1, overflow: 'auto' }}>
        {tab === 'home' && (
          <HomeScreen
            progress={progress}
            onStartPractice={handleStartPractice}
            onBrowse={() => setTab('cases')}
            onOpenStats={() => setTab('stats')}
          />
        )}
        {tab === 'cases' && (
          <CasesScreen
            progress={progress}
            onOpenCase={(id) => setRoute({ type: 'detail', id, from: 'cases' })}
          />
        )}
        {tab === 'stats' && <StatsScreen progress={progress} />}
      </div>
      <TabBar tab={tab} onChange={setTab} />
    </div>
  );
}

function TabBar({ tab, onChange }) {
  const tabs = [
    { k: 'home', label: 'Practice', icon: HomeIcon },
    { k: 'cases', label: 'Cases', icon: GridIcon },
    { k: 'stats', label: 'Progress', icon: ChartIcon },
  ];
  return (
    <div style={{
      position: 'relative', zIndex: 5,
      background: paper.bg,
      borderTop: `1px solid ${paper.ruleFaint}`,
      padding: '8px 0 26px',
      display: 'flex', justifyContent: 'space-around',
    }}>
      {tabs.map(t => {
        const active = tab === t.k;
        const Icon = t.icon;
        return (
          <button key={t.k} onClick={() => onChange(t.k)} style={{
            background: 'none', border: 'none', padding: '6px 18px',
            display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4,
            cursor: 'pointer',
            color: active ? paper.ink : paper.inkFaint,
            fontFamily: fonts.sans,
          }}>
            <Icon active={active} />
            <span style={{
              fontSize: 10, letterSpacing: 0.8, textTransform: 'uppercase',
              fontWeight: active ? 600 : 400,
            }}>{t.label}</span>
          </button>
        );
      })}
    </div>
  );
}

function HomeIcon({ active }) {
  return (
    <svg width="22" height="22" viewBox="0 0 22 22" fill="none">
      <rect x="3" y="3" width="7" height="7" rx="1.2" stroke="currentColor" strokeWidth="1.4" fill={active ? 'currentColor' : 'none'} />
      <rect x="12" y="3" width="7" height="7" rx="1.2" stroke="currentColor" strokeWidth="1.4" />
      <rect x="3" y="12" width="7" height="7" rx="1.2" stroke="currentColor" strokeWidth="1.4" />
      <rect x="12" y="12" width="7" height="7" rx="1.2" stroke="currentColor" strokeWidth="1.4" fill={active ? 'currentColor' : 'none'}/>
    </svg>
  );
}
function GridIcon({ active }) {
  return (
    <svg width="22" height="22" viewBox="0 0 22 22" fill="none">
      <rect x="3" y="3" width="16" height="16" rx="2" stroke="currentColor" strokeWidth="1.4"/>
      <line x1="3" y1="8.5" x2="19" y2="8.5" stroke="currentColor" strokeWidth="1.4"/>
      <line x1="3" y1="14" x2="19" y2="14" stroke="currentColor" strokeWidth="1.4"/>
    </svg>
  );
}
function ChartIcon({ active }) {
  return (
    <svg width="22" height="22" viewBox="0 0 22 22" fill="none">
      <line x1="3" y1="19" x2="19" y2="19" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round"/>
      <rect x="5" y="11" width="3" height="6" rx="0.5" fill={active ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth="1.4"/>
      <rect x="10" y="7" width="3" height="10" rx="0.5" fill={active ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth="1.4"/>
      <rect x="15" y="4" width="3" height="13" rx="0.5" fill={active ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth="1.4"/>
    </svg>
  );
}

window.App = App;
