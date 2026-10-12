import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { randomUUID } from 'node:crypto';

const image = process.env.TEST_CONTAINER_IMAGE ?? 'family-on-the-run:coolify';
const name = `family-check-${randomUUID()}`;
const volume = `${name}-data`;
function docker(args) {
  const result = spawnSync('docker', args, { encoding: 'utf8' });
  if (result.status !== 0) throw new Error(result.stderr || 'Docker command failed');
  return result.stdout.trim();
}
const cli = ['/opt/spacetime/spacetimedb-cli', '--config-path', '/data/cli.toml'];
async function start() {
  docker(['run', '--detach', '--name', name, '--publish', '127.0.0.1::3000', '--mount', `source=${volume},target=/data`, image]);
  const port = JSON.parse(docker(['inspect', name]))[0].NetworkSettings.Ports['3000/tcp'][0].HostPort;
  const url = `http://127.0.0.1:${port}/v1/database/family-on-the-run/route/`;
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url, { signal: AbortSignal.timeout(2000) });
      if (response.ok && docker(['exec', name, 'test', '-f', '/tmp/family-ready']) === '') return url;
    } catch { /* The server is still starting. */ }
    const state = JSON.parse(docker(['inspect', name]))[0].State;
    assert.ok(state.Running, 'Container stays running during initialization');
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  throw new Error('Container did not become ready within 60 seconds');
}
function fingerprints() {
  return docker(['exec', name, 'sha256sum', '/data/identity.private.pem', '/data/identity.public.pem', '/data/cli.toml', '/data/access-code', '/data/admin-code']);
}
const taskTitle = 'Container persistence check';
try {
  docker(['volume', 'create', volume]);
  await start();
  // Use the private code inside the container; never return it to the test logs.
  docker(['exec', name, 'sh', '-c', '/opt/spacetime/spacetimedb-cli --config-path /data/cli.toml call --server http://127.0.0.1:3000 --no-config family-on-the-run join_household "$(jq -Rs \'rtrimstr("\\n")\' /data/access-code)"']);
  docker(['exec', name, ...cli, 'call', '--server', 'http://127.0.0.1:3000', '--no-config', 'family-on-the-run', 'add_task', '1', JSON.stringify(taskTitle)]);
  const before = fingerprints();
  docker(['stop', '--time', '30', name]);
  docker(['rm', name]);
  await start();
  assert.equal(fingerprints(), before, 'Server keys, owner identity, and codes survive container replacement');
  const tasks = docker(['exec', name, ...cli, 'sql', '--server', 'http://127.0.0.1:3000', '--no-config', 'family-on-the-run', 'SELECT title FROM task']);
  assert.ok(tasks.includes(taskTitle), 'Database records survive replacement and module republishing');
  console.log('PASS: container boot, embedded dashboard, initial access, task writes, graceful stop, replacement, republish, and persistent records/keys/codes/owner identity.');
} finally {
  // Only the isolated test container and its uniquely named volume are removed.
  spawnSync('docker', ['rm', '--force', name], { stdio: 'ignore' });
  spawnSync('docker', ['volume', 'rm', volume], { stdio: 'ignore' });
}
