// Simple OLL-themed logo mark.
// A 3x3 grid where the center, NE, SW are filled — reads as both a cube face
// and a stylized "OLL" hint. Plus a small wordmark.

function LogoMark({ size = 48, dark = '#8C5A2B', light = '#E8DCC4', frame = '#1F1B16' }) {
  const S = size;
  const pad = S * 0.08;
  const inner = S - pad * 2;
  const gap = S * 0.04;
  const cell = (inner - gap * 2) / 3;
  // Pattern: X = dark brown, O = light brown.
  // Top-left, bottom-right, and full middle row are dark.
  const grid = [
    ['X', 'O', 'O'],
    ['X', 'X', 'X'],
    ['O', 'O', 'X'],
  ];
  const fill = (ch) => ch === 'X' ? dark : light;
  return (
    <svg width={S} height={S} viewBox={`0 0 ${S} ${S}`} style={{ display: 'block' }}>
      <rect x="0" y="0" width={S} height={S} rx={S * 0.18} fill={frame} />
      {[0,1,2].map(r => [0,1,2].map(c => (
        <rect key={`${r}-${c}`}
          x={pad + c * (cell + gap)} y={pad + r * (cell + gap)}
          width={cell} height={cell} rx={S * 0.025}
          fill={fill(grid[r][c])} />
      )))}
    </svg>
  );
}

function LogoWord({ size = 22, color, italic = true, style = {} }) {
  return (
    <span style={{
      fontFamily: fonts.serif, fontSize: size, fontWeight: 500,
      letterSpacing: -0.5, color: color || paper.ink,
      ...style,
    }}>
      OLL<span style={{ fontStyle: italic ? 'italic' : 'normal', color: paper.inkMuted }}> practice</span>
    </span>
  );
}

function LogoLockup({ size = 48, gap = 12, wordSize, color, style = {} }) {
  return (
    <div style={{ display: 'inline-flex', alignItems: 'center', gap, ...style }}>
      <LogoMark size={size} />
      <LogoWord size={wordSize || Math.round(size * 0.5)} color={color} />
    </div>
  );
}

Object.assign(window, { LogoMark, LogoWord, LogoLockup });
