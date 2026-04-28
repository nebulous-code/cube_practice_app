// App shell with auth gate + bottom tab navigation.
function App() {
  // Auth state machine: 'splash' | 'login' | 'register' | 'verify' | 'forgot' | 'reset' | 'onboarding' | 'app'
  const [authStage, setAuthStage] = useStateP('splash');
  const [user, setUser] = useStateP(null); // {name, email}
  const [pendingEmail, setPendingEmail] = useStateP('');

  const [tab, setTab] = useStateP('home');
  const [route, setRoute] = useStateP(null);
  const [progress, setProgress] = useStateP(() => {
    try { const saved = localStorage.getItem('oll-progress'); if (saved) return JSON.parse(saved); } catch {}
    return seedProgress();
  });
  const [cases, setCases] = useStateP(OLL_CASES);

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
  useEffectP(() => { window.caseById = (id) => cases.find(c => c.id === id); }, [cases]);

  // ── Auth flow ──
  if (authStage === 'splash') return <SplashScreen onContinue={() => setAuthStage('login')} />;
  if (authStage === 'about') return (
    <AboutSection onBack={() => setAuthStage('login')} />
  );
  if (authStage === 'login') return (
    <LoginScreen
      onSubmit={({ email }) => { setUser({ email, name: email.split('@')[0] }); setAuthStage('app'); }}
      onRegister={() => setAuthStage('register')}
      onForgot={() => setAuthStage('forgot')}
      onGuest={() => { setUser(null); setAuthStage('app'); }}
      onOpenAbout={() => setAuthStage('about')}
    />
  );
  if (authStage === 'register') return (
    <RegisterScreen
      onBack={() => setAuthStage('login')}
      onSubmit={({ name, email }) => { setUser({ name, email }); setPendingEmail(email); setAuthStage('verify'); }}
    />
  );
  if (authStage === 'verify') return (
    <VerifyEmailScreen
      email={pendingEmail} onBack={() => setAuthStage('register')}
      onResend={() => {}}
      onVerify={() => setAuthStage('onboarding')}
    />
  );
  if (authStage === 'forgot') return (
    <ForgotPasswordScreen onBack={() => setAuthStage('login')}
      onSubmit={(email) => { setPendingEmail(email); setAuthStage('reset'); }} />
  );
  if (authStage === 'reset') return (
    <ResetPasswordScreen email={pendingEmail} onBack={() => setAuthStage('forgot')}
      onSubmit={() => setAuthStage('login')} />
  );
  if (authStage === 'onboarding') return (
    <OnboardingScreen name={user?.name} onFinish={() => setAuthStage('app')} />
  );

  // ── Routed full-bleed screens ──
  if (route?.type === 'practice') {
    return (
      <PracticeScreen queue={route.queue} progress={progress} onGrade={handleGrade}
        onExit={() => setRoute(null)}
        onViewCase={(id) => setRoute({ type: 'detail', id, from: 'practice', queue: route.queue })} />
    );
  }
  if (route?.type === 'detail') {
    return (
      <CaseDetailScreen caseId={route.id} progress={progress}
        onBack={() => route.from === 'practice' ? setRoute({ type: 'practice', queue: route.queue }) : setRoute(null)}
        onEditCase={handleEditCase} />
    );
  }
  if (route?.type === 'about') {
    return <AboutSection onBack={() => setRoute(user ? { type: 'settings' } : null)} />;
  }
  if (route?.type === 'settings') {
    if (!user) {
      return (
        <GuestUpgradeScreen
          onBack={() => setRoute(null)}
          onSignIn={() => { setRoute(null); setAuthStage('login'); }}
          onOpenAbout={() => setRoute({ type: 'about' })}
          onCreate={({ name, email }) => {
            setUser({ name, email });
            setPendingEmail(email);
            setRoute(null);
            setAuthStage('verify');
          }}
        />
      );
    }
    return (
      <SettingsScreen user={user}
        onBack={() => setRoute(null)}
        onSignOut={() => { setUser(null); setRoute(null); setAuthStage('login'); }}
        onUpdateAccount={(p) => setUser(u => ({ ...(u || {}), ...p }))}
      />
    );
  }

  // ── Tabs ──
  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column', background: paper.bg }}>
      <div style={{ flex: 1, overflow: 'auto' }}>
        {tab === 'home' && (
          <HomeScreen progress={progress}
            user={user}
            onOpenSettings={() => setRoute({ type: 'settings' })}
            onStartPractice={handleStartPractice}
            onBrowse={() => setTab('cases')}
            onOpenStats={() => setTab('stats')} />
        )}
        {tab === 'cases' && (
          <CasesScreen progress={progress}
            onOpenCase={(id) => setRoute({ type: 'detail', id, from: 'cases' })} />
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
      position: 'relative', zIndex: 5, background: paper.bg,
      borderTop: `1px solid ${paper.ruleFaint}`, padding: '8px 0 26px',
      display: 'flex', justifyContent: 'space-around',
    }}>
      {tabs.map(t => {
        const active = tab === t.k; const Icon = t.icon;
        return (
          <button key={t.k} onClick={() => onChange(t.k)} style={{
            background: 'none', border: 'none', padding: '6px 18px',
            display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4,
            cursor: 'pointer', color: active ? paper.ink : paper.inkFaint, fontFamily: fonts.sans,
          }}>
            <Icon active={active} />
            <span style={{ fontSize: 10, letterSpacing: 0.8, textTransform: 'uppercase', fontWeight: active ? 600 : 400 }}>{t.label}</span>
          </button>
        );
      })}
    </div>
  );
}
function HomeIcon({ active }) { return (<svg width="22" height="22" viewBox="0 0 22 22" fill="none"><rect x="3" y="3" width="7" height="7" rx="1.2" stroke="currentColor" strokeWidth="1.4" fill={active ? 'currentColor' : 'none'} /><rect x="12" y="3" width="7" height="7" rx="1.2" stroke="currentColor" strokeWidth="1.4" /><rect x="3" y="12" width="7" height="7" rx="1.2" stroke="currentColor" strokeWidth="1.4" /><rect x="12" y="12" width="7" height="7" rx="1.2" stroke="currentColor" strokeWidth="1.4" fill={active ? 'currentColor' : 'none'}/></svg>); }
function GridIcon() { return (<svg width="22" height="22" viewBox="0 0 22 22" fill="none"><rect x="3" y="3" width="16" height="16" rx="2" stroke="currentColor" strokeWidth="1.4"/><line x1="3" y1="8.5" x2="19" y2="8.5" stroke="currentColor" strokeWidth="1.4"/><line x1="3" y1="14" x2="19" y2="14" stroke="currentColor" strokeWidth="1.4"/></svg>); }
function ChartIcon({ active }) { return (<svg width="22" height="22" viewBox="0 0 22 22" fill="none"><line x1="3" y1="19" x2="19" y2="19" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round"/><rect x="5" y="11" width="3" height="6" rx="0.5" fill={active ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth="1.4"/><rect x="10" y="7" width="3" height="10" rx="0.5" fill={active ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth="1.4"/><rect x="15" y="4" width="3" height="13" rx="0.5" fill={active ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth="1.4"/></svg>); }

window.App = App;
