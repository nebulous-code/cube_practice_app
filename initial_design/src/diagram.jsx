// OLL pattern diagram — improved 2D. Paper-like, warmer palette, crisp.
// Pattern is a 9-char string with chars: X (yellow top), L/R/T/B (yellow on that side), anything else is gray top.
function PatternDiagram({ pattern, size = 120, tone = 'paper' }) {
  // Color system — muted paper palette
  const palette = tone === 'paper'
    ? {
        face: '#EFEAE0',        // slab background (behind stickers)
        gray: '#D9D3C4',        // non-yellow sticker
        grayStroke: '#C7BFAE',
        yellow: '#E9B949',      // deeper ochre, less neon
        yellowStroke: '#B8901F',
        ink: '#2A2723',
      }
    : {
        face: '#2A2622', gray: '#3A3530', grayStroke: '#4A443D',
        yellow: '#E9B949', yellowStroke: '#B8901F', ink: '#F4EFE3',
      };

  // Canvas math
  const S = size;
  const pad = S * 0.10;
  const strip = S * 0.055;
  const gap = S * 0.018;
  const faceSize = S - 2 * (pad + strip + gap);
  const cellGap = S * 0.012;
  const cell = (faceSize - 2 * cellGap) / 3;
  const faceX = pad + strip + gap;
  const faceY = pad + strip + gap;

  const chars = pattern.split('');

  const cellRect = (r, c) => {
    const ch = chars[r * 3 + c];
    const isYellow = ch === 'X';
    return (
      <rect
        key={`c-${r}-${c}`}
        x={faceX + c * (cell + cellGap)}
        y={faceY + r * (cell + cellGap)}
        width={cell} height={cell}
        rx={S * 0.012}
        fill={isYellow ? palette.yellow : palette.gray}
        stroke={isYellow ? palette.yellowStroke : palette.grayStroke}
        strokeWidth={0.6}
      />
    );
  };

  const sideStrip = (r, c, side) => {
    const ch = chars[r * 3 + c];
    if (ch !== side) return null;
    const x0 = faceX + c * (cell + cellGap);
    const y0 = faceY + r * (cell + cellGap);
    let x, y, w, h;
    if (side === 'T') { x = x0; y = y0 - gap - strip; w = cell; h = strip; }
    if (side === 'B') { x = x0; y = y0 + cell + gap;  w = cell; h = strip; }
    if (side === 'L') { x = x0 - gap - strip; y = y0; w = strip; h = cell; }
    if (side === 'R') { x = x0 + cell + gap;  y = y0; w = strip; h = cell; }
    return (
      <rect key={`s-${r}-${c}-${side}`} x={x} y={y} width={w} height={h}
        rx={S * 0.008}
        fill={palette.yellow} stroke={palette.yellowStroke} strokeWidth={0.6} />
    );
  };

  const strips = [];
  for (let r = 0; r < 3; r++) {
    for (let c = 0; c < 3; c++) {
      ['T','B','L','R'].forEach(s => {
        const el = sideStrip(r, c, s);
        if (el) strips.push(el);
      });
    }
  }

  // Arrow indicating orientation (small ink tick at top-center)
  return (
    <svg width={S} height={S} viewBox={`0 0 ${S} ${S}`} style={{ display: 'block' }}>
      {/* slab */}
      <rect x={pad + strip} y={pad + strip}
            width={S - 2 * (pad + strip)} height={S - 2 * (pad + strip)}
            rx={S * 0.04} fill={palette.face} />
      {/* cells */}
      {[0,1,2].map(r => [0,1,2].map(c => cellRect(r, c)))}
      {/* side strips */}
      {strips}
    </svg>
  );
}

window.PatternDiagram = PatternDiagram;
