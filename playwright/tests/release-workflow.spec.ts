import { test, expect } from '@playwright/test';
import { BamboozleClient, BamboozleAssertBuilder, MatchKey } from '@bamboozle/sdk';
import { spawnSync, SpawnSyncReturns } from 'child_process';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';

// All tests run serially — they share a single Bamboozle instance and need
// route cleanup to be deterministic.
test.describe.configure({ mode: 'serial' });

const bamboozle = new BamboozleClient({ baseUrl: 'http://localhost:19090' });
const MOCK = 'http://localhost:18080';
const SCRIPTS = path.resolve(__dirname, '../../.github/scripts/release-wf');

// Short owner/repo used throughout to keep Bamboozle pattern strings readable.
const REPO = 'rwt/repo';

// ─── helpers ──────────────────────────────────────────────────────────────────

function run(
  script: string,
  env: Record<string, string>,
  cwd?: string,
): SpawnSyncReturns<string> {
  return spawnSync('bash', [path.join(SCRIPTS, script)], {
    cwd: cwd ?? process.cwd(),
    env: {
      PATH: process.env.PATH ?? '/usr/bin:/bin',
      HOME: process.env.HOME ?? '/root',
      GITHUB_TOKEN: 'test-token',
      GITHUB_REPOSITORY: REPO,
      GITHUB_API_BASE_URL: MOCK,
      ...env,
    },
    encoding: 'utf8',
  });
}

function makeTempDir(): string {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'bam-rw-'));
}

function removeTempDir(dir: string): void {
  try { fs.rmSync(dir, { recursive: true, force: true }); } catch { /* ignore */ }
}

function initGitRepo(dir: string): void {
  const opts = { cwd: dir };
  spawnSync('git', ['init'], opts);
  spawnSync('git', ['config', 'user.email', 'ci@test.local'], opts);
  spawnSync('git', ['config', 'user.name', 'CI Test'], opts);
  spawnSync('git', ['commit', '--allow-empty', '-m', 'root'], opts);
}

function gitTag(dir: string, tag: string): void {
  spawnSync('git', ['commit', '--allow-empty', '-m', `tag ${tag}`], { cwd: dir });
  spawnSync('git', ['tag', tag], { cwd: dir });
}

async function cleanUp(keys: MatchKey[]): Promise<void> {
  for (const key of keys) {
    try { await bamboozle.clearCalls(key.verb, key.pattern); } catch { /* ignore */ }
    try { await bamboozle.deleteRoute(key.verb, key.pattern); } catch { /* ignore */ }
  }
}

// ─── determine-since-date ─────────────────────────────────────────────────────

test.describe('determine-since-date', () => {
  let keys: MatchKey[] = [];
  let tmpDirs: string[] = [];

  test.beforeEach(() => {
    if (process.platform === 'win32') test.skip();
    keys = [];
    tmpDirs = [];
  });
  test.afterEach(async () => {
    await cleanUp(keys);
    for (const d of tmpDirs) removeTempDir(d);
  });

  test('versioned tag — returns committer date of previous tag in same family', () => {
    const dir = makeTempDir();
    tmpDirs.push(dir);
    initGitRepo(dir);
    gitTag(dir, 'app/v1.0.0');
    gitTag(dir, 'app/v1.1.0');

    const expected = spawnSync(
      'git', ['log', '-1', '--format=%aI', 'app/v1.0.0'],
      { cwd: dir, encoding: 'utf8' },
    ).stdout.trim();

    const result = run('determine-since-date.sh', { TAG: 'app/v1.1.0' }, dir);

    expect(result.status).toBe(0);
    expect(result.stdout.trim()).toBe(expected);
  });

  test('versioned tag — first release with no prior tag returns epoch', () => {
    const dir = makeTempDir();
    tmpDirs.push(dir);
    initGitRepo(dir);
    gitTag(dir, 'app/v1.0.0');

    const result = run('determine-since-date.sh', { TAG: 'app/v1.0.0' }, dir);

    expect(result.status).toBe(0);
    expect(result.stdout.trim()).toBe('1970-01-01T00:00:00Z');
  });

  test('versioned tag — sdk-style prefix is stripped correctly', () => {
    const dir = makeTempDir();
    tmpDirs.push(dir);
    initGitRepo(dir);
    gitTag(dir, 'sdks/npm/v0.9.0');
    gitTag(dir, 'sdks/npm/v1.0.0');

    const expected = spawnSync(
      'git', ['log', '-1', '--format=%aI', 'sdks/npm/v0.9.0'],
      { cwd: dir, encoding: 'utf8' },
    ).stdout.trim();

    const result = run('determine-since-date.sh', { TAG: 'sdks/npm/v1.0.0' }, dir);

    expect(result.status).toBe(0);
    expect(result.stdout.trim()).toBe(expected);
  });

  test('non-versioned tag — returns created_at from existing GitHub Release', async () => {
    const key: MatchKey = { verb: 'GET', pattern: `repos/${REPO}/releases/tags/app%2Fnightly` };
    keys.push(key);
    await bamboozle.upsertRoute({
      match: key,
      response: {
        status: '200',
        headers: { 'Content-Type': 'application/json' },
        content: JSON.stringify({ id: 1, tag_name: 'app/nightly', created_at: '2024-06-15T00:00:00Z' }),
      },
    });

    const result = run('determine-since-date.sh', { TAG: 'app/nightly' });

    expect(result.status).toBe(0);
    expect(result.stdout.trim()).toBe('2024-06-15T00:00:00Z');
    expect(await bamboozle.assert(key.verb, key.pattern, { calledExactly: 1 })).toBeTruthy();
  });

  test('non-versioned tag — falls back to ~25h ago when no release exists', async () => {
    const key: MatchKey = { verb: 'GET', pattern: `repos/${REPO}/releases/tags/app%2Fnightly` };
    keys.push(key);
    await bamboozle.upsertRoute({
      match: key,
      response: { status: '404', content: 'not found' },
    });

    const before = new Date(Date.now() - 26 * 3600 * 1000);
    const after  = new Date(Date.now() - 24 * 3600 * 1000);

    const result = run('determine-since-date.sh', { TAG: 'app/nightly' });

    expect(result.status).toBe(0);
    const sinceDate = new Date(result.stdout.trim());
    expect(sinceDate.getTime()).toBeGreaterThan(before.getTime());
    expect(sinceDate.getTime()).toBeLessThan(after.getTime());
    expect(await bamboozle.assert(key.verb, key.pattern, { calledExactly: 1 })).toBeTruthy();
  });
});

// ─── collect-prs ──────────────────────────────────────────────────────────────

test.describe('collect-prs', () => {
  const SEARCH_KEY: MatchKey = { verb: 'GET', pattern: 'search/issues' };
  let keys: MatchKey[] = [];
  let tmpDirs: string[] = [];

  test.beforeEach(() => {
    if (process.platform === 'win32') test.skip();
    keys = [];
    tmpDirs = [];
  });
  test.afterEach(async () => {
    await cleanUp(keys);
    for (const d of tmpDirs) removeTempDir(d);
  });

  test('single label — writes PR list to output file', async () => {
    keys.push(SEARCH_KEY);
    await bamboozle.upsertRoute({
      match: SEARCH_KEY,
      response: {
        status: '200',
        headers: { 'Content-Type': 'application/json' },
        content: JSON.stringify({
          items: [
            { number: 42, title: 'Fix the bug', html_url: 'https://github.com/rwt/repo/pull/42', user: { login: 'alice' } },
            { number: 43, title: 'Add the feature', html_url: 'https://github.com/rwt/repo/pull/43', user: { login: 'bob' } },
          ],
        }),
      },
    });

    const dir = makeTempDir();
    tmpDirs.push(dir);
    const outputFile = path.join(dir, 'notes.md');

    const result = run('collect-prs.sh', {
      SINCE_DATE: '2020-01-01T00:00:00Z',
      AREA_LABELS: 'area: core',
      OUTPUT_FILE: outputFile,
    });

    expect(result.status).toBe(0);
    const notes = fs.readFileSync(outputFile, 'utf8');
    expect(notes).toContain('## What\'s Changed');
    expect(notes).toContain('Fix the bug');
    expect(notes).toContain('[#42]');
    expect(notes).toContain('@alice');
    expect(notes).toContain('Add the feature');
    expect(notes).toContain('[#43]');
    expect(notes).toContain('@bob');
    expect(await bamboozle.assert(SEARCH_KEY.verb, SEARCH_KEY.pattern, { calledExactly: 1 })).toBeTruthy();
  });

  test('single label — includes label and since date in search query', async () => {
    keys.push(SEARCH_KEY);
    await bamboozle.upsertRoute({
      match: SEARCH_KEY,
      response: {
        status: '200',
        headers: { 'Content-Type': 'application/json' },
        content: JSON.stringify({ items: [] }),
      },
    });

    const dir = makeTempDir();
    tmpDirs.push(dir);

    run('collect-prs.sh', {
      SINCE_DATE: '2024-03-01T00:00:00Z',
      AREA_LABELS: 'area: sdks/npm',
      OUTPUT_FILE: path.join(dir, 'notes.md'),
    });

    expect(await bamboozle.assert(SEARCH_KEY.verb, SEARCH_KEY.pattern, {
      expression: new BamboozleAssertBuilder()
        .with(({ query }) => query.q.contains('area: sdks/npm'))
        .and()
        .with(({ query }) => query.q.contains('2024-03-01T00:00:00Z')),
    })).toBeTruthy();
  });

  test('multiple labels — issues one API call per label', async () => {
    keys.push(SEARCH_KEY);
    await bamboozle.upsertRoute({
      match: SEARCH_KEY,
      response: {
        status: '200',
        headers: { 'Content-Type': 'application/json' },
        content: JSON.stringify({ items: [] }),
      },
    });

    const dir = makeTempDir();
    tmpDirs.push(dir);

    run('collect-prs.sh', {
      SINCE_DATE: '2020-01-01T00:00:00Z',
      AREA_LABELS: 'area: core,area: ci',
      OUTPUT_FILE: path.join(dir, 'notes.md'),
    });

    expect(await bamboozle.assert(SEARCH_KEY.verb, SEARCH_KEY.pattern, { calledExactly: 2 })).toBeTruthy();
  });

  test('duplicate PRs across labels — appear exactly once in output', async () => {
    keys.push(SEARCH_KEY);
    // First call returns PRs #1 and #2; second call returns PRs #2 and #3.
    // PR #2 should appear only once after deduplication.
    await bamboozle.upsertRoute({
      match: SEARCH_KEY,
      setState: 'called',
      response: {
        status: '200',
        headers: { 'Content-Type': 'application/json' },
        content: [
          '{% if previousContext == nil %}',
          JSON.stringify({ items: [
            { number: 1, title: 'PR One',   html_url: 'https://github.com/rwt/repo/pull/1', user: { login: 'alice' } },
            { number: 2, title: 'PR Two',   html_url: 'https://github.com/rwt/repo/pull/2', user: { login: 'bob' } },
          ]}),
          '{% else %}',
          JSON.stringify({ items: [
            { number: 2, title: 'PR Two',   html_url: 'https://github.com/rwt/repo/pull/2', user: { login: 'bob' } },
            { number: 3, title: 'PR Three', html_url: 'https://github.com/rwt/repo/pull/3', user: { login: 'carol' } },
          ]}),
          '{% endif %}',
        ].join(''),
      },
    });

    const dir = makeTempDir();
    tmpDirs.push(dir);
    const outputFile = path.join(dir, 'notes.md');

    run('collect-prs.sh', {
      SINCE_DATE: '2020-01-01T00:00:00Z',
      AREA_LABELS: 'area: core,area: ci',
      OUTPUT_FILE: outputFile,
    });

    const notes = fs.readFileSync(outputFile, 'utf8');
    const pr2Occurrences = (notes.match(/\[#2\]/g) ?? []).length;
    expect(pr2Occurrences).toBe(1);
    expect(notes).toContain('[#1]');
    expect(notes).toContain('[#3]');
    expect(await bamboozle.assert(SEARCH_KEY.verb, SEARCH_KEY.pattern, { calledExactly: 2 })).toBeTruthy();
  });

  test('no PRs found — writes fallback message', async () => {
    keys.push(SEARCH_KEY);
    await bamboozle.upsertRoute({
      match: SEARCH_KEY,
      response: {
        status: '200',
        headers: { 'Content-Type': 'application/json' },
        content: JSON.stringify({ items: [] }),
      },
    });

    const dir = makeTempDir();
    tmpDirs.push(dir);
    const outputFile = path.join(dir, 'notes.md');

    run('collect-prs.sh', {
      SINCE_DATE: '2020-01-01T00:00:00Z',
      AREA_LABELS: 'area: core',
      OUTPUT_FILE: outputFile,
    });

    const notes = fs.readFileSync(outputFile, 'utf8');
    expect(notes).toContain('## What\'s Changed');
    expect(notes).toContain('No changes found for the specified areas.');
  });
});

// ─── create-or-update-release ─────────────────────────────────────────────────

test.describe('create-or-update-release', () => {
  let keys: MatchKey[] = [];
  let tmpDirs: string[] = [];

  // Shared mock routes reused across most tests.
  const NEW_RELEASE_TAG_KEY: MatchKey   = { verb: 'GET',   pattern: `repos/${REPO}/releases/tags/test%2Fnew` };
  const EXIST_RELEASE_TAG_KEY: MatchKey = { verb: 'GET',   pattern: `repos/${REPO}/releases/tags/test%2Fexisting` };
  const CREATE_KEY: MatchKey            = { verb: 'POST',  pattern: `repos/${REPO}/releases` };
  const EDIT_KEY: MatchKey              = { verb: 'PATCH', pattern: `repos/${REPO}/releases/99999` };

  async function registerMockRoutes(): Promise<void> {
    await bamboozle.upsertRoute({
      match: NEW_RELEASE_TAG_KEY,
      response: { status: '404', content: 'not found' },
    });
    await bamboozle.upsertRoute({
      match: EXIST_RELEASE_TAG_KEY,
      response: {
        status: '200',
        headers: { 'Content-Type': 'application/json' },
        content: JSON.stringify({ id: 99999, tag_name: 'test/existing' }),
      },
    });
    await bamboozle.upsertRoute({
      match: CREATE_KEY,
      response: {
        status: '201',
        headers: { 'Content-Type': 'application/json' },
        content: JSON.stringify({ id: 1, tag_name: 'test/new' }),
      },
    });
    await bamboozle.upsertRoute({
      match: EDIT_KEY,
      response: {
        status: '200',
        headers: { 'Content-Type': 'application/json' },
        content: JSON.stringify({ id: 99999, tag_name: 'test/existing' }),
      },
    });
  }

  test.beforeEach(async () => {
    if (process.platform === 'win32') test.skip();
    keys = [NEW_RELEASE_TAG_KEY, EXIST_RELEASE_TAG_KEY, CREATE_KEY, EDIT_KEY];
    tmpDirs = [];
    await registerMockRoutes();
  });
  test.afterEach(async () => {
    await cleanUp(keys);
    for (const d of tmpDirs) removeTempDir(d);
  });

  function writeNotes(dir: string, content = '## What\'s Changed\n\n- test PR ([#1](https://x/1)) by @test\n'): string {
    const file = path.join(dir, 'notes.md');
    fs.writeFileSync(file, content);
    return file;
  }

  test('new release — calls POST, not PATCH', async () => {
    const dir = makeTempDir();
    tmpDirs.push(dir);

    const result = run('create-or-update-release.sh', {
      TAG: 'test/new',
      NOTES_FILE: writeNotes(dir),
    });

    expect(result.status).toBe(0);
    expect(await bamboozle.assert(CREATE_KEY.verb, CREATE_KEY.pattern, { calledExactly: 1 })).toBeTruthy();
    expect(await bamboozle.assert(EDIT_KEY.verb, EDIT_KEY.pattern, { neverCalled: true })).toBeTruthy();
  });

  test('existing release — calls PATCH, not POST', async () => {
    const dir = makeTempDir();
    tmpDirs.push(dir);

    const result = run('create-or-update-release.sh', {
      TAG: 'test/existing',
      NOTES_FILE: writeNotes(dir),
    });

    expect(result.status).toBe(0);
    expect(await bamboozle.assert(EDIT_KEY.verb, EDIT_KEY.pattern, { calledExactly: 1 })).toBeTruthy();
    expect(await bamboozle.assert(CREATE_KEY.verb, CREATE_KEY.pattern, { neverCalled: true })).toBeTruthy();
  });

  test('sends correct tag_name, name, and notes body in request', async () => {
    const dir = makeTempDir();
    tmpDirs.push(dir);
    const notesContent = '## What\'s Changed\n\n- my PR ([#7](https://x/7)) by @dev\n';
    writeNotes(dir, notesContent);

    run('create-or-update-release.sh', {
      TAG: 'test/new',
      RELEASE_NAME: 'My Release',
      NOTES_FILE: path.join(dir, 'notes.md'),
    });

    expect(await bamboozle.assert(CREATE_KEY.verb, CREATE_KEY.pattern, {
      expression: new BamboozleAssertBuilder()
        .with(({ body }) => body.tag_name.equals('test/new'))
        .and()
        .with(({ body }) => body.name.equals('My Release'))
        .and()
        .with(({ body }) => body.body.contains('my PR')),
    })).toBeTruthy();
  });

  test('RELEASE_NAME defaults to TAG when not provided', async () => {
    const dir = makeTempDir();
    tmpDirs.push(dir);

    run('create-or-update-release.sh', {
      TAG: 'test/new',
      NOTES_FILE: writeNotes(dir),
      // deliberately omitting RELEASE_NAME
    });

    expect(await bamboozle.assert(CREATE_KEY.verb, CREATE_KEY.pattern, {
      expression: new BamboozleAssertBuilder()
        .with(({ body }) => body.name.equals('test/new')),
    })).toBeTruthy();
  });

  test('prerelease=true — request body has prerelease: true', async () => {
    const dir = makeTempDir();
    tmpDirs.push(dir);

    run('create-or-update-release.sh', {
      TAG: 'test/new',
      NOTES_FILE: writeNotes(dir),
      PRERELEASE: 'true',
    });

    expect(await bamboozle.assert(CREATE_KEY.verb, CREATE_KEY.pattern, {
      expression: new BamboozleAssertBuilder()
        .with(({ body }) => body.prerelease.equals(true)),
    })).toBeTruthy();
  });

  test('prerelease=false — request body has prerelease: false', async () => {
    const dir = makeTempDir();
    tmpDirs.push(dir);

    run('create-or-update-release.sh', {
      TAG: 'test/new',
      NOTES_FILE: writeNotes(dir),
      PRERELEASE: 'false',
    });

    expect(await bamboozle.assert(CREATE_KEY.verb, CREATE_KEY.pattern, {
      expression: new BamboozleAssertBuilder()
        .with(({ body }) => body.prerelease.equals(false)),
    })).toBeTruthy();
  });

  test('make_latest=true — request body has make_latest: "true"', async () => {
    const dir = makeTempDir();
    tmpDirs.push(dir);

    run('create-or-update-release.sh', {
      TAG: 'test/new',
      NOTES_FILE: writeNotes(dir),
      MAKE_LATEST: 'true',
    });

    expect(await bamboozle.assert(CREATE_KEY.verb, CREATE_KEY.pattern, {
      expression: new BamboozleAssertBuilder()
        .with(({ body }) => body.make_latest.equals('true')),
    })).toBeTruthy();
  });

  test('make_latest=false — request body has make_latest: "false"', async () => {
    const dir = makeTempDir();
    tmpDirs.push(dir);

    run('create-or-update-release.sh', {
      TAG: 'test/new',
      NOTES_FILE: writeNotes(dir),
      MAKE_LATEST: 'false',
    });

    expect(await bamboozle.assert(CREATE_KEY.verb, CREATE_KEY.pattern, {
      expression: new BamboozleAssertBuilder()
        .with(({ body }) => body.make_latest.equals('false')),
    })).toBeTruthy();
  });
});
