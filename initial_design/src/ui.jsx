// Small UI primitives in the paper aesthetic.
const { useState, useEffect, useMemo, useRef } = React;

// Paper palette
const paper = {
  bg: '#F4EFE3',            // main page bg
  bgAlt: '#EDE6D4',         // section alt
  card: '#FBF7ED',          // elevated card
  ink: '#1F1B16',           // primary text
  inkMuted: '#6E6455',
  inkFaint: '#9C907C',
  rule: '#D9CEB7',
  ruleFaint: '#E5DCC8',
  accent: '#8C5A2B',        // warm ink
  accentBg: '#E8DCC4',
  yellow: '#E9B949',
};

const fonts = {
  serif: '"Newsreader", "Georgia", serif',
  sans: '"Inter Tight", "Inter", system-ui, sans-serif',
  mono: '"JetBrains Mono", "SF Mono", ui-monospace, monospace',
};

// A hairline rule
function Rule({ color = paper.rule, style = {} }) {
  return <div style={{ height: 1, background: color, ...style }} />;
}

// Chip used for tags / priority badges
function Chip({ children, tone = 'neutral', size = 'sm', style = {} }) {
  const tones = {
    neutral: { bg: 'transparent', border: paper.rule, color: paper.inkMuted },
    ink:     { bg: paper.ink, border: paper.ink, color: paper.bg },
    accent:  { bg: paper.accentBg, border: paper.accentBg, color: paper.accent },
    warn:    { bg: '#F2DFCA', border: '#E5C69C', color: '#8C5A2B' },
  };
  const t = tones[tone] || tones.neutral;
  const sz = size === 'sm'
    ? { fontSize: 10.5, padding: '2px 7px', letterSpacing: 0.3 }
    : { fontSize: 12, padding: '3px 9px', letterSpacing: 0.3 };
  return (
    <span style={{
      display: 'inline-flex', alignItems: 'center', gap: 4,
      border: `1px solid ${t.border}`, background: t.bg, color: t.color,
      borderRadius: 999, textTransform: 'uppercase',
      fontFamily: fonts.sans, fontWeight: 500,
      ...sz, ...style,
    }}>{children}</span>
  );
}

// Grade pip — little colored tag showing A/B/C/D/F/New
function GradePip({ grade, size = 24 }) {
  const meta = GRADE_META[grade] || GRADE_META.New;
  return (
    <div style={{
      width: size, height: size, borderRadius: size / 2,
      background: meta.bg, color: meta.color,
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      fontFamily: fonts.serif, fontWeight: 600,
      fontSize: grade === 'New' ? Math.round(size * 0.36) : Math.round(size * 0.5),
      letterSpacing: grade === 'New' ? 0 : -0.5,
      lineHeight: 1,
    }}>
      {grade === 'New' ? '·' : grade}
    </div>
  );
}

// Small label used for section headers, all caps
function Eyebrow({ children, style = {} }) {
  return (
    <div style={{
      fontFamily: fonts.sans, fontSize: 10.5, letterSpacing: 1.6,
      textTransform: 'uppercase', color: paper.inkFaint, fontWeight: 500,
      ...style,
    }}>{children}</div>
  );
}

// Button
function Button({ children, onClick, variant = 'primary', size = 'md', style = {}, disabled }) {
  const variants = {
    primary: { bg: paper.ink, color: paper.bg, border: paper.ink },
    ghost:   { bg: 'transparent', color: paper.ink, border: paper.rule },
    quiet:   { bg: 'transparent', color: paper.inkMuted, border: 'transparent' },
    accent:  { bg: paper.accent, color: paper.bg, border: paper.accent },
  };
  const sizes = {
    sm: { padding: '7px 12px', fontSize: 13, height: 32 },
    md: { padding: '11px 16px', fontSize: 14, height: 42 },
    lg: { padding: '14px 20px', fontSize: 15, height: 50 },
  };
  const v = variants[variant];
  const s = sizes[size];
  return (
    <button onClick={onClick} disabled={disabled} style={{
      display: 'inline-flex', alignItems: 'center', justifyContent: 'center', gap: 8,
      background: v.bg, color: v.color, border: `1px solid ${v.border}`,
      borderRadius: 999, cursor: disabled ? 'not-allowed' : 'pointer',
      fontFamily: fonts.sans, fontWeight: 500, letterSpacing: 0.1,
      opacity: disabled ? 0.4 : 1,
      transition: 'transform 120ms ease, opacity 120ms',
      ...s, ...style,
    }}>{children}</button>
  );
}

// Card container
function Card({ children, style = {}, pad = 20 }) {
  return (
    <div style={{
      background: paper.card, borderRadius: 14,
      border: `1px solid ${paper.ruleFaint}`,
      padding: pad, ...style,
    }}>{children}</div>
  );
}

Object.assign(window, { paper, fonts, Rule, Chip, GradePip, Eyebrow, Button, Card });
