// Shared chrome + form primitives for auth screens.

function AuthShell({ children, onBack, footer }) {
  return (
    <div style={{
      background: paper.bg, minHeight: '100%', display: 'flex', flexDirection: 'column',
      fontFamily: fonts.sans, color: paper.ink,
    }}>
      {onBack && (
        <div style={{ padding: '52px 22px 0' }}>
          <button onClick={onBack} style={{
            background: 'none', border: 'none', padding: 0,
            fontFamily: fonts.sans, fontSize: 13, color: paper.inkMuted,
            cursor: 'pointer',
          }}>← Back</button>
        </div>
      )}
      <div style={{ flex: 1, padding: onBack ? '20px 26px 0' : '64px 26px 0' }}>
        {children}
      </div>
      {footer && (
        <div style={{ padding: '20px 26px 28px', textAlign: 'center' }}>{footer}</div>
      )}
    </div>
  );
}

function AuthHeader({ eyebrow, title, sub, mark = false }) {
  return (
    <div style={{ marginBottom: 32 }}>
      {mark && <div style={{ marginBottom: 22 }}><LogoMark size={44} /></div>}
      {eyebrow && <Eyebrow style={{ marginBottom: 10 }}>{eyebrow}</Eyebrow>}
      <div style={{
        fontFamily: fonts.serif, fontSize: 34, lineHeight: 1.05,
        letterSpacing: -1, color: paper.ink,
      }}>
        {title}
      </div>
      {sub && (
        <div style={{
          fontFamily: fonts.serif, fontStyle: 'italic', fontSize: 16,
          color: paper.inkMuted, marginTop: 8, lineHeight: 1.4,
        }}>
          {sub}
        </div>
      )}
    </div>
  );
}

function Field({ label, type = 'text', value, onChange, placeholder, hint, autoFocus, error, rightSlot }) {
  const [focused, setFocused] = useStateP(false);
  return (
    <label style={{ display: 'block', marginBottom: 16 }}>
      <div style={{
        fontFamily: fonts.sans, fontSize: 11, letterSpacing: 1.2,
        color: error ? '#B84A3F' : paper.inkFaint, textTransform: 'uppercase',
        marginBottom: 6, fontWeight: 500,
      }}>{label}</div>
      <div style={{
        display: 'flex', alignItems: 'center',
        background: paper.card,
        border: `1px solid ${error ? '#B84A3F' : focused ? paper.ink : paper.rule}`,
        borderRadius: 10, padding: '0 14px',
        transition: 'border-color 120ms',
      }}>
        <input
          type={type} value={value} onChange={e => onChange(e.target.value)}
          placeholder={placeholder} autoFocus={autoFocus}
          onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}
          style={{
            flex: 1, background: 'transparent', border: 'none', outline: 'none',
            padding: '14px 0', fontFamily: fonts.sans, fontSize: 15,
            color: paper.ink, minWidth: 0,
          }}
        />
        {rightSlot}
      </div>
      {hint && !error && (
        <div style={{ fontFamily: fonts.sans, fontSize: 11, color: paper.inkFaint, marginTop: 6, letterSpacing: 0.2 }}>
          {hint}
        </div>
      )}
      {error && (
        <div style={{ fontFamily: fonts.sans, fontSize: 11, color: '#B84A3F', marginTop: 6, letterSpacing: 0.2 }}>
          {error}
        </div>
      )}
    </label>
  );
}

function PasswordField({ label = 'Password', value, onChange, placeholder, hint, autoFocus, error }) {
  const [shown, setShown] = useStateP(false);
  return (
    <Field
      label={label} type={shown ? 'text' : 'password'}
      value={value} onChange={onChange} placeholder={placeholder}
      hint={hint} autoFocus={autoFocus} error={error}
      rightSlot={
        <button type="button" onClick={() => setShown(s => !s)} style={{
          background: 'none', border: 'none', padding: '4px 0 4px 8px',
          fontFamily: fonts.sans, fontSize: 11, letterSpacing: 0.6,
          color: paper.inkMuted, textTransform: 'uppercase', cursor: 'pointer',
        }}>
          {shown ? 'Hide' : 'Show'}
        </button>
      }
    />
  );
}

// Big bottom CTA used across auth
function PrimaryCTA({ children, onClick, disabled }) {
  return (
    <button onClick={onClick} disabled={disabled} style={{
      width: '100%', background: paper.ink, color: paper.bg,
      border: 'none', borderRadius: 12, padding: '15px',
      fontFamily: fonts.sans, fontSize: 15, fontWeight: 600,
      letterSpacing: 0.2, cursor: disabled ? 'not-allowed' : 'pointer',
      opacity: disabled ? 0.4 : 1, marginTop: 8,
    }}>
      {children}
    </button>
  );
}

function TextLink({ children, onClick, accent = false }) {
  return (
    <button onClick={onClick} style={{
      background: 'none', border: 'none', padding: 0,
      fontFamily: fonts.sans, fontSize: 13,
      color: accent ? paper.accent : paper.ink, fontWeight: accent ? 500 : 400,
      cursor: 'pointer', textDecoration: 'underline', textUnderlineOffset: 3,
      textDecorationColor: accent ? paper.accent : paper.rule,
    }}>{children}</button>
  );
}

Object.assign(window, { AuthShell, AuthHeader, Field, PasswordField, PrimaryCTA, TextLink });
