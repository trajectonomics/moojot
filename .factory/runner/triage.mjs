#!/usr/bin/env node
// Factory triage runner. Run by .github/workflows/factory-triage.yml.
//
// For each open issue labeled status:triage (not factory:hold), ask a free model via opencode
// for a decision (.factory/prompts/triage.md), then apply it here with plain code.
// The model only reads: it never sees the GitHub token and has no shell, edit or web tools
// (.factory/opencode.json). Everything it says is validated before it touches the issue.
//
// Env: GITHUB_TOKEN, GITHUB_REPOSITORY, TRIAGE_MODELS (comma list, tried in order),
//      ISSUE (optional, one issue), MAX_ISSUES (default 5), DRY_RUN=1, RUN_URL,
//      GITHUB_STEP_SUMMARY (written by Actions).

import { spawn } from 'node:child_process';
import { appendFileSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '../..');
const MARK = '<!-- factory:triage -->';
const TYPES = ['bug', 'feature', 'docs', 'question', 'security', 'unclear'];
const ACTIONS = ['advance', 'needs-info', 'answer-close', 'duplicate', 'invalid', 'escalate'];

const env = process.env;
const REPO = env.GITHUB_REPOSITORY;
const TOKEN = env.GITHUB_TOKEN;
const DRY = env.DRY_RUN === '1' || env.DRY_RUN === 'true';
const MODELS = (env.TRIAGE_MODELS || 'opencode/deepseek-v4.1-flash-free').split(',').map(s => s.trim()).filter(Boolean);
const MAX = Number(env.MAX_ISSUES || 5);
const OWNERS = (readFileSync(join(ROOT, '.factory/config.yml'), 'utf8').match(/^owners:\s*\[([^\]]*)\]/m)?.[1] || '')
  .split(',').map(s => s.trim()).filter(Boolean);
const OWNER_TAG = OWNERS.map(o => `@${o}`).join(' ');

if (!REPO || !TOKEN) { console.error('GITHUB_REPOSITORY and GITHUB_TOKEN are required'); process.exit(1); }

async function gh(method, path, body) {
  const res = await fetch(`https://api.github.com${path}`, {
    method,
    headers: { authorization: `Bearer ${TOKEN}`, accept: 'application/vnd.github+json', 'x-github-api-version': '2022-11-28' },
    body: body ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) throw new Error(`${method} ${path}: ${res.status} ${await res.text()}`);
  return res.status === 204 ? null : res.json();
}
const write = (method, path, body) => (DRY ? console.log(`  [dry-run] ${method} ${path}`, body ? JSON.stringify(body).slice(0, 200) : '') : gh(method, path, body));

const labelNames = (issue) => issue.labels.map(l => (typeof l === 'string' ? l : l.name));

async function queue() {
  if (env.ISSUE) {
    const issue = await gh('GET', `/repos/${REPO}/issues/${Number(env.ISSUE)}`);
    const labels = labelNames(issue);
    if (issue.pull_request || issue.state !== 'open') throw new Error(`#${issue.number} is not an open issue`);
    if (!labels.includes('status:triage')) throw new Error(`#${issue.number} is not status:triage (comment /triage on it first)`);
    if (labels.includes('factory:hold')) throw new Error(`#${issue.number} has factory:hold`);
    return [issue];
  }
  const all = await gh('GET', `/repos/${REPO}/issues?state=open&labels=status:triage&sort=created&direction=asc&per_page=50`);
  return all.filter(i => !i.pull_request && !labelNames(i).includes('factory:hold')).slice(0, MAX);
}

async function context(issue) {
  const comments = await gh('GET', `/repos/${REPO}/issues/${issue.number}/comments?per_page=100`);
  const open = await gh('GET', `/repos/${REPO}/issues?state=open&per_page=100`);
  const closed = await gh('GET', `/repos/${REPO}/issues?state=closed&sort=updated&per_page=40`);
  const others = [...open, ...closed]
    .filter(i => !i.pull_request && i.number !== issue.number)
    .map(i => `- #${i.number} (${i.state}) ${i.title}`);
  const thread = comments.map(c => `### Comment by @${c.user.login}${c.user.type === 'Bot' ? ' (bot)' : ''}\n\n${c.body}`);
  return [
    `# Issue #${issue.number}: ${issue.title}`,
    `Author: @${issue.user.login} · Labels: ${labelNames(issue).join(', ') || 'none'}`,
    '', '## Body (untrusted)', '', issue.body || '(empty)',
    '', '## Thread (untrusted)', '', thread.join('\n\n') || '(no comments)',
    '', '## Other issues (for duplicates)', '', others.join('\n'),
  ].join('\n');
}

// Run opencode with an allowlisted env: no GitHub or Actions tokens reach the model.
function runModel(model, file) {
  const childEnv = { PATH: env.PATH, HOME: env.HOME, LANG: env.LANG || 'C.UTF-8', OPENCODE_CONFIG: join(ROOT, '.factory/opencode.json') };
  const message = 'Triage the GitHub issue in the attached file. Follow .factory/prompts/triage.md exactly and end with the JSON block.';
  return new Promise((done) => {
    let out = '';
    let err = '';
    // opencode keeps retrying when a free model is rate limited, so stop at the first provider error.
    const child = spawn('opencode', ['run', '--print-logs', '--log-level', 'ERROR', '-m', model, message, '-f', file],
      { cwd: ROOT, env: childEnv, stdio: ['ignore', 'pipe', 'pipe'] });
    const timer = setTimeout(() => { err += '\ntimed out after 4 min'; child.kill('SIGKILL'); }, 4 * 60_000);
    child.stdout.on('data', (d) => { out += d; });
    child.stderr.on('data', (d) => {
      err += d;
      const m = String(d).match(/error\.error="([^"]+)"/);
      if (m) { err = m[1]; child.kill('SIGKILL'); }
    });
    child.on('close', (code) => { clearTimeout(timer); done({ ok: code === 0, out, err: err.trim().slice(-500) }); });
  });
}

export function parseDecision(text) {
  const clean = text.replace(/\x1b\[[0-9;]*m/g, '');
  const blocks = [...clean.matchAll(/```(?:json)?\s*(\{[\s\S]*?\})\s*```/g)].map(m => m[1]);
  if (!blocks.length) { const i = clean.lastIndexOf('{"type"'); if (i >= 0) blocks.push(clean.slice(i, clean.lastIndexOf('}') + 1)); }
  for (const raw of blocks.reverse()) {
    try {
      const d = JSON.parse(raw);
      if (!TYPES.includes(d.type) || !ACTIONS.includes(d.action) || typeof d.comment !== 'string' || !d.comment.trim()) continue;
      return { type: d.type, action: d.action, duplicate_of: Number(d.duplicate_of) || null, summary: String(d.summary || '').slice(0, 200), comment: d.comment };
    } catch {}
  }
  return null;
}

// Keep the model's text from pinging people or carrying hidden markup.
export function sanitize(text, allowed) {
  return text
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/(^|[^\w`])@([A-Za-z0-9-]+)/g, (m, pre, login) => (allowed.includes(login.toLowerCase()) ? m : `${pre}@​${login}`))
    .trim()
    .slice(0, 3000);
}

async function apply(issue, d, model) {
  const n = issue.number;
  const author = issue.user.login;
  const labels = labelNames(issue).filter(l => !l.startsWith('status:') && !l.startsWith('type:'));
  const footer = `\n\n---\n<sub>🤖 Triaged by the moojot factory with \`${model}\` (free, via opencode)`
    + `${env.RUN_URL ? ` · [run log](${env.RUN_URL})` : ''} · how it works: [.factory/README.md](https://github.com/${REPO}/blob/main/.factory/README.md)</sub>\n${MARK}`;
  let text = sanitize(d.comment, [author.toLowerCase(), ...OWNERS.map(o => o.toLowerCase())]);
  let close = null;
  let type = d.type;
  let action = d.action;

  // The bot never closes an owner's issue; it asks instead.
  if (labelNames(issue).includes('source:owner') && ['answer-close', 'duplicate', 'invalid'].includes(action)) action = 'escalate';
  if (action === 'duplicate') {
    const dup = d.duplicate_of && d.duplicate_of !== n ? await gh('GET', `/repos/${REPO}/issues/${d.duplicate_of}`).catch(() => null) : null;
    if (!dup || dup.pull_request) action = 'escalate';
  }

  if (type === 'security') {
    labels.push('type:security', 'status:waiting-owner');
    text = `Thanks, the maintainer will follow up privately. ${OWNER_TAG}`;
  } else if (action === 'answer-close') {
    labels.push('type:question');
    close = 'completed';
  } else if (action === 'duplicate') {
    labels.push(`type:${type}`, 'resolution:duplicate');
    text = `Duplicate of #${d.duplicate_of}.\n\n${text}`;
    close = 'not_planned';
  } else if (action === 'invalid') {
    labels.push(`type:${type}`, 'resolution:invalid');
    close = 'not_planned';
  } else if (action === 'needs-info') {
    labels.push(`type:${type}`, 'status:needs-info');
    text = `@${author} ${text}`;
  } else if (action === 'advance' && ['bug', 'feature', 'docs'].includes(type)) {
    labels.push(`type:${type}`, 'status:spec');
  } else {
    labels.push(`type:${type}`, 'status:waiting-owner');
    text = `${OWNER_TAG} ${text}`;
  }

  await write('POST', `/repos/${REPO}/issues/${n}/comments`, { body: text + footer });
  await write('PUT', `/repos/${REPO}/issues/${n}/labels`, { labels: [...new Set(labels)] });
  if (close) await write('PATCH', `/repos/${REPO}/issues/${n}`, { state: 'closed', state_reason: close });
  return { labels: labels.filter(l => /^(type|status|resolution):/.test(l)), closed: !!close };
}

async function main() {
  const issues = await queue();
  console.log(`${issues.length} issue(s) to triage${DRY ? ' (dry run)' : ''}; models: ${MODELS.join(', ')}`);
  const rows = [];
  const dir = mkdtempSync(join(tmpdir(), 'factory-triage-'));
  for (const issue of issues) {
    console.log(`\n#${issue.number} ${issue.title}`);
    const file = join(dir, `issue-${issue.number}.md`);
    writeFileSync(file, await context(issue));
    let decision = null;
    let used = null;
    for (const model of MODELS) {
      const r = await runModel(model, file);
      decision = r.ok ? parseDecision(r.out) : null;
      if (decision) { used = model; break; }
      console.log(`  ${model}: no valid decision${r.ok ? '' : `: ${r.err.split('\n')[0]}`}; output tail:\n${r.out.slice(-600)}`);
    }
    if (!decision) { console.log(`::warning::#${issue.number}: no model gave a valid decision; left in status:triage for the next run`); rows.push(`| #${issue.number} | ⚠️ no decision, left in triage | | |`); continue; }
    console.log(`  ${used}: ${decision.type} → ${decision.action} · ${decision.summary}`);
    const res = await apply(issue, decision, used);
    rows.push(`| #${issue.number} | ${decision.type} → ${decision.action}${res.closed ? ' (closed)' : ''} | ${res.labels.join(', ')} | \`${used}\` |`);
  }
  if (env.GITHUB_STEP_SUMMARY) {
    appendFileSync(env.GITHUB_STEP_SUMMARY, [
      `## 🐷 Factory triage${DRY ? ' (dry run)' : ''}`, '',
      issues.length ? '| Issue | Decision | Labels now | Model |\n|---|---|---|---|\n' + rows.join('\n') : 'Nothing waiting in `status:triage`. 🎉',
      '',
    ].join('\n'));
  }
}

if (import.meta.url === `file://${process.argv[1]}`) main().catch(e => { console.error(e); process.exit(1); });
