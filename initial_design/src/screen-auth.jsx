// All auth-related screens.

// Splash — logo, brief tag, fades into login. (We render statically; auth state controls flow.)
function SplashScreen({ onContinue }) {
  useEffectP(() => {
    const t = setTimeout(onContinue, 1400);
    return () => clearTimeout(t);
  }, []);
  return (
    <div style={{
      background: paper.bg, height: '100%', display: 'flex',
      flexDirection: 'column', alignItems: 'center', justifyContent: 'center',
      gap: 22,
    }}>
      <LogoMark size={88} />
      <LogoWord size={26} />
      <div style={{
        fontFamily: fonts.serif, fontStyle: 'italic', fontSize: 14,
        color: paper.inkFaint, marginTop: 4, letterSpacing: 0.2,
      }}>
        a quiet place to drill OLL
      </div>
    </div>
  );
}

function LoginScreen({ onSubmit, onRegister, onForgot, onGuest, onOpenAbout }) {
  const [email, setEmail] = useStateP('');
  const [password, setPassword] = useStateP('');
  return (
    <AuthShell footer={
      <div style={{ display: 'flex', flexDirection: 'column', gap: 10, alignItems: 'center' }}>
        <div style={{ fontFamily: fonts.sans, fontSize: 13, color: paper.inkMuted }}>
          New here? <TextLink onClick={onRegister} accent>Create an account</TextLink>
        </div>
        <TextLink onClick={onOpenAbout}>About, terms & privacy</TextLink>
      </div>
    }>
      <AuthHeader mark eyebrow="Welcome back" title="Sign in" sub="Pick up where you left off." />
      <Field label="Email" type="email" value={email} onChange={setEmail} placeholder="you@example.com" autoFocus />
      <PasswordField value={password} onChange={setPassword} placeholder="••••••••" />
      <div style={{ textAlign: 'right', marginTop: -4, marginBottom: 8 }}>
        <TextLink onClick={onForgot}>Forgot password?</TextLink>
      </div>
      <PrimaryCTA onClick={() => onSubmit({ email, password })} disabled={!email || !password}>
        Sign in
      </PrimaryCTA>
      <div style={{
        display: 'flex', alignItems: 'center', gap: 12,
        margin: '24px 0', color: paper.inkFaint,
      }}>
        <div style={{ flex: 1, height: 1, background: paper.rule }} />
        <span style={{ fontFamily: fonts.sans, fontSize: 11, letterSpacing: 1, textTransform: 'uppercase' }}>or</span>
        <div style={{ flex: 1, height: 1, background: paper.rule }} />
      </div>
      <button onClick={onGuest} style={{
        width: '100%', background: 'transparent', color: paper.ink,
        border: `1px solid ${paper.rule}`, borderRadius: 12, padding: '14px',
        fontFamily: fonts.sans, fontSize: 14, fontWeight: 500, cursor: 'pointer',
        letterSpacing: 0.2,
      }}>
        Continue as guest
      </button>
      <div style={{
        fontFamily: fonts.sans, fontSize: 11, color: paper.inkFaint,
        textAlign: 'center', marginTop: 10, lineHeight: 1.5,
      }}>
        Guest progress is stored on this device only.
      </div>
    </AuthShell>
  );
}

function RegisterScreen({ onSubmit, onBack }) {
  const [name, setName] = useStateP('');
  const [email, setEmail] = useStateP('');
  const [password, setPassword] = useStateP('');
  const [confirm, setConfirm] = useStateP('');
  const mismatch = confirm && password !== confirm;
  const valid = name && email.includes('@') && password.length >= 8 && !mismatch;
  return (
    <AuthShell onBack={onBack} footer={
      <div style={{ fontFamily: fonts.sans, fontSize: 11, color: paper.inkFaint, lineHeight: 1.5 }}>
        By creating an account you agree to our <TextLink>Terms</TextLink> and <TextLink>Privacy Policy</TextLink>.
      </div>
    }>
      <AuthHeader title="Create your account" sub="Track progress across devices." />
      <Field label="Display name" value={name} onChange={setName} placeholder="What should we call you?" autoFocus />
      <Field label="Email" type="email" value={email} onChange={setEmail} placeholder="you@example.com" />
      <PasswordField value={password} onChange={setPassword} placeholder="At least 8 characters" hint={password && password.length < 8 ? `${8 - password.length} more characters` : null} />
      <PasswordField label="Confirm password" value={confirm} onChange={setConfirm} placeholder="••••••••" error={mismatch ? "Passwords don't match" : null} />
      <PrimaryCTA onClick={() => onSubmit({ name, email, password })} disabled={!valid}>
        Create account
      </PrimaryCTA>
    </AuthShell>
  );
}

function VerifyEmailScreen({ email, onVerify, onResend, onBack }) {
  const [digits, setDigits] = useStateP(['', '', '', '', '', '']);
  const refs = digits.map(() => useRef(null));

  const setDigit = (i, val) => {
    const v = val.replace(/\D/g, '').slice(-1);
    setDigits(d => { const nd = [...d]; nd[i] = v; return nd; });
    if (v && i < 5) refs[i + 1].current?.focus();
  };
  const handleKey = (i, e) => {
    if (e.key === 'Backspace' && !digits[i] && i > 0) refs[i - 1].current?.focus();
  };
  const code = digits.join('');
  const valid = code.length === 6;

  return (
    <AuthShell onBack={onBack}>
      <AuthHeader eyebrow="One more step" title="Verify your email" sub={
        <>We sent a 6-digit code to <span style={{ color: paper.ink, fontStyle: 'normal' }}>{email}</span>.</>
      } />
      <div style={{
        fontFamily: fonts.sans, fontSize: 11, letterSpacing: 1.2,
        color: paper.inkFaint, textTransform: 'uppercase', marginBottom: 10, fontWeight: 500,
      }}>Enter code</div>
      <div style={{ display: 'flex', gap: 8, marginBottom: 20 }}>
        {digits.map((d, i) => (
          <input
            key={i} ref={refs[i]} value={d}
            onChange={e => setDigit(i, e.target.value)}
            onKeyDown={e => handleKey(i, e)}
            inputMode="numeric" maxLength={1} size={1} autoFocus={i === 0}
            style={{
              flex: 1, minWidth: 0, width: 0, height: 56, textAlign: 'center', padding: 0,
              background: paper.card, border: `1px solid ${d ? paper.ink : paper.rule}`,
              borderRadius: 10, fontFamily: fonts.mono, fontSize: 24,
              color: paper.ink, outline: 'none',
            }}
          />
        ))}
      </div>
      <PrimaryCTA onClick={() => onVerify(code)} disabled={!valid}>Verify</PrimaryCTA>
      <div style={{ textAlign: 'center', marginTop: 22 }}>
        <span style={{ fontFamily: fonts.sans, fontSize: 13, color: paper.inkMuted }}>
          Didn't get it? </span>
        <TextLink onClick={onResend}>Resend code</TextLink>
      </div>
    </AuthShell>
  );
}

function ForgotPasswordScreen({ onSubmit, onBack }) {
  const [email, setEmail] = useStateP('');
  const [sent, setSent] = useStateP(false);
  if (sent) {
    return (
      <AuthShell onBack={onBack}>
        <AuthHeader eyebrow="Check your inbox" title="Reset link sent" sub={
          <>If an account exists for <span style={{ color: paper.ink, fontStyle: 'normal' }}>{email}</span>, you'll receive a 6-digit code shortly.</>
        } />
        <PrimaryCTA onClick={() => onSubmit(email)}>Enter code</PrimaryCTA>
        <div style={{ textAlign: 'center', marginTop: 18 }}>
          <TextLink onClick={() => setSent(false)}>Use a different email</TextLink>
        </div>
      </AuthShell>
    );
  }
  return (
    <AuthShell onBack={onBack}>
      <AuthHeader title="Forgot password" sub="Enter your account email and we'll send you a code to reset it." />
      <Field label="Email" type="email" value={email} onChange={setEmail} placeholder="you@example.com" autoFocus />
      <PrimaryCTA onClick={() => setSent(true)} disabled={!email.includes('@')}>Send reset code</PrimaryCTA>
    </AuthShell>
  );
}

function ResetPasswordScreen({ email, onSubmit, onBack }) {
  const [code, setCode] = useStateP('');
  const [pw, setPw] = useStateP('');
  const [confirm, setConfirm] = useStateP('');
  const mismatch = confirm && pw !== confirm;
  const valid = code.length === 6 && pw.length >= 8 && !mismatch;
  return (
    <AuthShell onBack={onBack}>
      <AuthHeader title="Reset password" sub="Enter the code we sent and choose a new password." />
      <Field label="Reset code" value={code} onChange={v => setCode(v.replace(/\D/g,'').slice(0,6))} placeholder="6-digit code" />
      <PasswordField label="New password" value={pw} onChange={setPw} placeholder="At least 8 characters" />
      <PasswordField label="Confirm new password" value={confirm} onChange={setConfirm} error={mismatch ? "Passwords don't match" : null} />
      <PrimaryCTA onClick={() => onSubmit({ code, password: pw })} disabled={!valid}>Update password</PrimaryCTA>
    </AuthShell>
  );
}

function OnboardingScreen({ name, onFinish }) {
  const [step, setStep] = useStateP(0);
  const steps = [
    {
      eyebrow: `Hello ${name || 'there'}`,
      title: 'Practice OLL with intention',
      body: 'See a pattern. Execute the algorithm on your cube. Verify the resulting shape. Grade yourself honestly — Fail, Hard, Good, or Easy.',
    },
    {
      eyebrow: 'How it works',
      title: 'Weakest cases come first',
      body: 'Your grades drive a queue that surfaces the cases you struggle with most. Practice a little every day to keep your overall grade up.',
    },
  ];
  const s = steps[step];
  return (
    <div style={{
      background: paper.bg, height: '100%', display: 'flex', flexDirection: 'column',
      padding: '64px 26px 28px',
    }}>
      <div style={{ flex: 1 }}>
        <div style={{ marginBottom: 28 }}><LogoMark size={40} /></div>
        <Eyebrow style={{ marginBottom: 10 }}>{s.eyebrow}</Eyebrow>
        <div style={{
          fontFamily: fonts.serif, fontSize: 34, lineHeight: 1.05,
          letterSpacing: -1, color: paper.ink, marginBottom: 14,
        }}>{s.title}</div>
        <div style={{
          fontFamily: fonts.serif, fontStyle: 'italic', fontSize: 18,
          lineHeight: 1.5, color: paper.inkMuted,
        }}>{s.body}</div>
      </div>
      <div style={{ display: 'flex', gap: 6, justifyContent: 'center', marginBottom: 18 }}>
        {steps.map((_, i) => (
          <div key={i} style={{
            width: i === step ? 22 : 6, height: 6, borderRadius: 3,
            background: i === step ? paper.ink : paper.rule,
            transition: 'width 200ms',
          }} />
        ))}
      </div>
      <PrimaryCTA onClick={() => step < steps.length - 1 ? setStep(step + 1) : onFinish()}>
        {step < steps.length - 1 ? 'Next' : 'Start practicing'}
      </PrimaryCTA>
    </div>
  );
}

function SettingsScreen({ user, onBack, onSignOut, onChangePassword, onUpdateAccount }) {
  const [name, setName] = useStateP(user?.name || '');
  const [email, setEmail] = useStateP(user?.email || '');
  const [section, setSection] = useStateP(null); // null | 'security' | 'about'

  if (section === 'security') {
    return <SecuritySection onBack={() => setSection(null)} onSignOut={onSignOut} />;
  }
  if (section === 'about') {
    return <AboutSection onBack={() => setSection(null)} />;
  }

  return (
    <div style={{ background: paper.bg, minHeight: '100%', paddingBottom: 40 }}>
      <div style={{ padding: '52px 22px 10px' }}>
        <button onClick={onBack} style={{
          background: 'none', border: 'none', padding: 0, marginBottom: 16,
          fontFamily: fonts.sans, fontSize: 13, color: paper.inkMuted, cursor: 'pointer',
        }}>← Back</button>
        <Eyebrow style={{ marginBottom: 8 }}>Settings</Eyebrow>
        <div style={{
          fontFamily: fonts.serif, fontSize: 34, letterSpacing: -0.8, color: paper.ink,
        }}>Your account</div>
      </div>

      <div style={{ padding: '14px 22px 0' }}>
        <Card pad={20}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 14, marginBottom: 18 }}>
            <Avatar name={name} size={56} />
            <div>
              <div style={{ fontFamily: fonts.serif, fontSize: 20, color: paper.ink, lineHeight: 1.1 }}>
                {name || 'Guest'}
              </div>
              <div style={{ fontFamily: fonts.sans, fontSize: 12, color: paper.inkMuted, marginTop: 2 }}>
                {email || 'Local account'}
              </div>
            </div>
          </div>
          <Field label="Display name" value={name} onChange={setName} />
          <Field label="Email" type="email" value={email} onChange={setEmail} />
          <PrimaryCTA onClick={() => onUpdateAccount({ name, email })}>Save changes</PrimaryCTA>
        </Card>
      </div>

      <div style={{ padding: '20px 22px 0' }}>
        <Eyebrow style={{ marginBottom: 12 }}>Security</Eyebrow>
        <SettingsRow title="Password & sign-in" sub="Change password, sign out everywhere" onClick={() => setSection('security')} />
      </div>

      <div style={{ padding: '20px 22px 0' }}>
        <Eyebrow style={{ marginBottom: 12 }}>App</Eyebrow>
        <SettingsRow title="About OLL Practice" sub="Version, terms, privacy" onClick={() => setSection('about')} />
      </div>

      <div style={{ padding: '28px 22px 0' }}>
        <button onClick={onSignOut} style={{
          width: '100%', background: 'transparent', color: '#B84A3F',
          border: `1px solid ${paper.rule}`, borderRadius: 12, padding: '14px',
          fontFamily: fonts.sans, fontSize: 14, fontWeight: 500, cursor: 'pointer',
        }}>
          Sign out
        </button>
      </div>
    </div>
  );
}

function Avatar({ name, size = 40 }) {
  const initials = (name || '?').split(' ').map(s => s[0]).filter(Boolean).slice(0, 2).join('').toUpperCase();
  return (
    <div style={{
      width: size, height: size, borderRadius: size / 2,
      background: paper.accentBg, color: paper.accent,
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      fontFamily: fonts.serif, fontWeight: 500,
      fontSize: Math.round(size * 0.4), letterSpacing: 0.2,
      border: `1px solid ${paper.accent}30`,
    }}>{initials || '·'}</div>
  );
}

function SettingsRow({ title, sub, onClick, danger }) {
  return (
    <button onClick={onClick} style={{
      display: 'flex', alignItems: 'center', justifyContent: 'space-between',
      width: '100%', textAlign: 'left', background: paper.card,
      border: `1px solid ${paper.ruleFaint}`, borderRadius: 12,
      padding: '14px 16px', cursor: 'pointer', marginBottom: 8,
      fontFamily: fonts.sans, color: danger ? '#B84A3F' : paper.ink,
    }}>
      <div>
        <div style={{ fontSize: 15, fontWeight: 500, letterSpacing: -0.1 }}>{title}</div>
        {sub && <div style={{ fontSize: 12, color: paper.inkMuted, marginTop: 2 }}>{sub}</div>}
      </div>
      <div style={{ color: paper.inkFaint, fontSize: 18 }}>›</div>
    </button>
  );
}

function SecuritySection({ onBack, onSignOut }) {
  const [cur, setCur] = useStateP('');
  const [pw, setPw] = useStateP('');
  const [confirm, setConfirm] = useStateP('');
  const mismatch = confirm && pw !== confirm;
  return (
    <div style={{ background: paper.bg, minHeight: '100%', paddingBottom: 40 }}>
      <div style={{ padding: '52px 22px 10px' }}>
        <button onClick={onBack} style={{
          background: 'none', border: 'none', padding: 0, marginBottom: 16,
          fontFamily: fonts.sans, fontSize: 13, color: paper.inkMuted, cursor: 'pointer',
        }}>← Back</button>
        <Eyebrow style={{ marginBottom: 8 }}>Security</Eyebrow>
        <div style={{ fontFamily: fonts.serif, fontSize: 30, letterSpacing: -0.8, color: paper.ink }}>
          Password & sign-in
        </div>
      </div>
      <div style={{ padding: '20px 22px 0' }}>
        <Card pad={20}>
          <Eyebrow style={{ marginBottom: 12 }}>Change password</Eyebrow>
          <PasswordField label="Current password" value={cur} onChange={setCur} />
          <PasswordField label="New password" value={pw} onChange={setPw} hint="At least 8 characters" />
          <PasswordField label="Confirm new password" value={confirm} onChange={setConfirm} error={mismatch ? "Passwords don't match" : null} />
          <PrimaryCTA disabled={!cur || pw.length < 8 || mismatch}>Update password</PrimaryCTA>
        </Card>
      </div>
      <div style={{ padding: '20px 22px 0' }}>
        <Eyebrow style={{ marginBottom: 12 }}>Sessions</Eyebrow>
        <SettingsRow title="Sign out everywhere" sub="Ends all sessions on other devices" />
      </div>
    </div>
  );
}

function AboutSection({ onBack }) {
  return (
    <div style={{ background: paper.bg, minHeight: '100%', paddingBottom: 40 }}>
      <div style={{ padding: '52px 22px 10px' }}>
        <button onClick={onBack} style={{
          background: 'none', border: 'none', padding: 0, marginBottom: 16,
          fontFamily: fonts.sans, fontSize: 13, color: paper.inkMuted, cursor: 'pointer',
        }}>← Back</button>
        <Eyebrow style={{ marginBottom: 8 }}>About</Eyebrow>
        <div style={{ fontFamily: fonts.serif, fontSize: 30, letterSpacing: -0.8, color: paper.ink }}>
          OLL Practice
        </div>
      </div>
      <div style={{ padding: '24px 22px 0', display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
        <LogoMark size={64} />
        <div style={{ fontFamily: fonts.serif, fontSize: 18, marginTop: 14, color: paper.ink }}>OLL Practice</div>
        <div style={{ fontFamily: fonts.sans, fontSize: 12, color: paper.inkFaint, marginTop: 4 }}>Version 0.1.0</div>
      </div>
      <div style={{ padding: '28px 22px 0' }}>
        <SettingsRow title="Terms of Service" />
        <SettingsRow title="Privacy Policy" />
        <SettingsRow title="Acknowledgements" />
      </div>
    </div>
  );
}

function GuestUpgradeScreen({ onBack, onCreate, onSignIn, onOpenAbout }) {
  const [name, setName] = useStateP('');
  const [email, setEmail] = useStateP('');
  const [password, setPassword] = useStateP('');
  const [confirm, setConfirm] = useStateP('');
  const mismatch = confirm && password !== confirm;
  const valid = name && email.includes('@') && password.length >= 8 && !mismatch;
  return (
    <AuthShell onBack={onBack} footer={
      <div style={{ display: 'flex', flexDirection: 'column', gap: 10, alignItems: 'center' }}>
        <div style={{ fontFamily: fonts.sans, fontSize: 13, color: paper.inkMuted }}>
          Already have an account? <TextLink onClick={onSignIn} accent>Sign in</TextLink>
        </div>
        <TextLink onClick={onOpenAbout}>About, terms & privacy</TextLink>
      </div>
    }>
      <AuthHeader
        eyebrow="You're practicing as a guest"
        title="Save your progress"
        sub="Create an account to keep your grades, streak, and history across devices."
      />
      <div style={{
        background: paper.accentBg, borderRadius: 12, padding: '12px 14px',
        marginBottom: 22, fontFamily: fonts.sans, fontSize: 12,
        color: paper.accent, lineHeight: 1.5,
      }}>
        Your guest progress will be carried over to your new account.
      </div>
      <Field label="Display name" value={name} onChange={setName} placeholder="What should we call you?" autoFocus />
      <Field label="Email" type="email" value={email} onChange={setEmail} placeholder="you@example.com" />
      <PasswordField value={password} onChange={setPassword} placeholder="At least 8 characters" hint={password && password.length < 8 ? `${8 - password.length} more characters` : null} />
      <PasswordField label="Confirm password" value={confirm} onChange={setConfirm} placeholder="••••••••" error={mismatch ? "Passwords don't match" : null} />
      <PrimaryCTA onClick={() => onCreate({ name, email, password })} disabled={!valid}>
        Create account
      </PrimaryCTA>
    </AuthShell>
  );
}

Object.assign(window, {
  SplashScreen, LoginScreen, RegisterScreen, VerifyEmailScreen,
  ForgotPasswordScreen, ResetPasswordScreen, OnboardingScreen,
  SettingsScreen, Avatar, GuestUpgradeScreen,
});
