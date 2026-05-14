// Weekly operator-stats digest. See docs/milestones/08_operator_stats.md.
//
// Runs on a GitHub Actions schedule (`.github/workflows/operator-stats.yml`)
// against a read-only Neon role and emails a digest via Resend. Also runnable
// locally with the same env vars set in the shell.
//
// Required env:
//   STATS_DATABASE_URL  read-only Neon connection string (stats_readonly role)
//   RESEND_API_KEY      Resend API key (same as the backend's)
//   OPERATOR_EMAIL      recipient address
// Optional env:
//   EMAIL_FROM          defaults to the verified Quiet Cube sender below

import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import pg from 'pg';

const { Client } = pg;

const __dirname = dirname(fileURLToPath(import.meta.url));
const HISTORY_PATH = join(__dirname, 'operator_stats_history.json');

const RESEND_ENDPOINT = 'https://api.resend.com/emails';
const DEFAULT_FROM = 'Quiet Cube <noreply@mail.nebulouscode.com>';
const SPARKLINE_WEEKS = 8;

function requireEnv(key) {
  const v = process.env[key];
  if (!v) throw new Error(`Missing required env var: ${key}`);
  return v;
}

// Returns the Monday of the week containing `date` as an ISO date string
// (UTC). The history file keys off this so re-running on the same calendar
// week updates the existing entry instead of appending a duplicate.
function mondayOfWeek(date = new Date()) {
  const d = new Date(date);
  const day = d.getUTCDay();
  const offset = day === 0 ? 6 : day - 1;
  d.setUTCDate(d.getUTCDate() - offset);
  d.setUTCHours(0, 0, 0, 0);
  return d.toISOString().slice(0, 10);
}

async function queryStats(databaseUrl) {
  const client = new Client({ connectionString: databaseUrl });
  await client.connect();
  try {
    const { rows } = await client.query(`
      WITH user_aggs AS (
        SELECT
          COUNT(*)::int AS total,
          COUNT(*) FILTER (WHERE email_verified)::int AS verified,
          COUNT(*) FILTER (WHERE NOT email_verified)::int AS unverified,
          COUNT(*) FILTER (
            WHERE created_at >= NOW() - INTERVAL '7 days'
          )::int AS signups_this_week,
          COUNT(*) FILTER (
            WHERE created_at >= NOW() - INTERVAL '14 days'
              AND created_at <  NOW() - INTERVAL '7 days'
          )::int AS signups_last_week
        FROM users
      ),
      deletion_aggs AS (
        SELECT
          COUNT(*) FILTER (
            WHERE deleted_at >= NOW() - INTERVAL '7 days'
          )::int AS deletions_this_week,
          COUNT(*) FILTER (
            WHERE deleted_at >= NOW() - INTERVAL '14 days'
              AND deleted_at <  NOW() - INTERVAL '7 days'
          )::int AS deletions_last_week
        FROM account_deletions
      )
      SELECT * FROM user_aggs, deletion_aggs;
    `);
    return rows[0];
  } finally {
    await client.end();
  }
}

function readHistory() {
  if (!existsSync(HISTORY_PATH)) return [];
  const raw = readFileSync(HISTORY_PATH, 'utf8').trim();
  return raw ? JSON.parse(raw) : [];
}

function writeHistory(history) {
  writeFileSync(HISTORY_PATH, JSON.stringify(history, null, 2) + '\n');
}

// Replace the entry for the current week if one exists; otherwise append.
// Keeps the array sorted oldest-first so callers can slice from the tail
// for "most recent N weeks".
function upsertEntry(history, entry) {
  const idx = history.findIndex((e) => e.week_of === entry.week_of);
  if (idx >= 0) {
    history[idx] = entry;
  } else {
    history.push(entry);
    history.sort((a, b) => a.week_of.localeCompare(b.week_of));
  }
  return history;
}

function formatSigned(n) {
  return n >= 0 ? `+${n}` : `${n}`;
}

function trendArrow(thisWeek, lastWeek) {
  if (thisWeek > lastWeek) return 'up ↑';
  if (thisWeek < lastWeek) return 'down ↓';
  return 'sideways →';
}

// Renders the last SPARKLINE_WEEKS history entries most-recent-first as
// "week_of    signed_delta   bar" rows. Bar height normalises to the max
// |delta| in the window. Zero weeks render as a dot so the column lines up.
function sparkline(history) {
  if (history.length === 0) return '  (no prior history)';
  const recent = history.slice(-SPARKLINE_WEEKS).reverse();
  const maxAbs = Math.max(1, ...recent.map((e) => Math.abs(e.net_7d)));
  const blocks = '▁▂▃▄▅▆▇█';
  return recent
    .map((e) => {
      const norm = Math.min(
        blocks.length - 1,
        Math.floor((Math.abs(e.net_7d) / maxAbs) * (blocks.length - 1)),
      );
      const bar = e.net_7d === 0 ? '·' : blocks[norm];
      return `  ${e.week_of}   ${formatSigned(e.net_7d).padStart(4)}  ${bar}`;
    })
    .join('\n');
}

function buildDigest(stats, history) {
  const weekOf = mondayOfWeek();
  const netThisWeek = stats.signups_this_week - stats.deletions_this_week;
  const netLastWeek = stats.signups_last_week - stats.deletions_last_week;
  return [
    `Quiet Cube — week of ${weekOf}`,
    '',
    `Total users:        ${stats.total}       (${stats.verified} verified, ${stats.unverified} unverified)`,
    `  Net this week:    ${formatSigned(netThisWeek)}`,
    `  New signups:      ${stats.signups_this_week}`,
    `  Deletions:        ${stats.deletions_this_week}`,
    '',
    `Last week:          ${formatSigned(netLastWeek)} net`,
    `  New signups:      ${stats.signups_last_week}`,
    `  Deletions:        ${stats.deletions_last_week}`,
    '',
    `Trend: ${trendArrow(netThisWeek, netLastWeek)}`,
    '',
    `Net delta, last ${Math.min(SPARKLINE_WEEKS, history.length)} week(s) (most recent first):`,
    sparkline(history),
  ].join('\n');
}

async function sendEmail({ apiKey, from, to, subject, text }) {
  const res = await fetch(RESEND_ENDPOINT, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${apiKey}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ from, to, subject, text }),
  });
  if (!res.ok) {
    const body = await res.text();
    throw new Error(`Resend ${res.status}: ${body}`);
  }
}

// Best-effort: the digest run already failed once when this is called, so
// don't let a Resend hiccup mask the original error. Swallowed exceptions
// land on stderr; the process still exits non-zero from main().catch().
async function sendFailureEmail({ apiKey, from, to, error }) {
  try {
    await sendEmail({
      apiKey,
      from,
      to,
      subject: 'Quiet Cube — Stats run failed',
      text: [
        `Stats run failed at ${new Date().toISOString()}`,
        '',
        `Error: ${error.message}`,
        '',
        error.stack || '(no stack)',
        '',
        'Check the GitHub Actions run for details.',
      ].join('\n'),
    });
  } catch (e) {
    console.error('Failure-email send also failed:', e.message);
  }
}

async function main() {
  const databaseUrl = requireEnv('STATS_DATABASE_URL');
  const apiKey = requireEnv('RESEND_API_KEY');
  const to = requireEnv('OPERATOR_EMAIL');
  const from = process.env.EMAIL_FROM || DEFAULT_FROM;

  const stats = await queryStats(databaseUrl);
  console.log(
    `Stats: total=${stats.total} verified=${stats.verified} unverified=${stats.unverified} signups_7d=${stats.signups_this_week} deletions_7d=${stats.deletions_this_week}`,
  );

  const weekOf = mondayOfWeek();
  const entry = {
    week_of: weekOf,
    total_users: stats.total,
    verified: stats.verified,
    unverified: stats.unverified,
    signups_7d: stats.signups_this_week,
    deletions_7d: stats.deletions_this_week,
    net_7d: stats.signups_this_week - stats.deletions_this_week,
  };

  const history = readHistory();
  upsertEntry(history, entry);
  writeHistory(history);

  const text = buildDigest(stats, history);
  await sendEmail({
    apiKey,
    from,
    to,
    subject: `Quiet Cube — Stats for week of ${weekOf}`,
    text,
  });
  console.log('Digest sent.');
}

main().catch(async (err) => {
  console.error(err);
  if (process.env.RESEND_API_KEY && process.env.OPERATOR_EMAIL) {
    await sendFailureEmail({
      apiKey: process.env.RESEND_API_KEY,
      from: process.env.EMAIL_FROM || DEFAULT_FROM,
      to: process.env.OPERATOR_EMAIL,
      error: err,
    });
  }
  process.exit(1);
});
