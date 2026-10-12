import { build } from 'esbuild';
import { mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { spawnSync } from 'node:child_process';
import { randomBytes } from 'node:crypto';

const output = resolve('.spacetime/integration-client.mjs');
await mkdir(resolve('.spacetime'), { recursive: true });
await build({
  entryPoints: ['scripts/integration-client.ts'],
  outfile: output,
  bundle: true,
  platform: 'node',
  format: 'esm',
  packages: 'external',
});
let testDatabase;
function command(args) {
  const result = spawnSync('spacetime', ['--config-path', resolve('.spacetime/cli.toml'), ...args], { encoding: 'utf8' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`SpacetimeDB test command failed: ${args[0]}\n${result.stderr}`);
}

try {
  if (process.argv.includes('--admin')) {
    testDatabase = `family-admin-test-${randomBytes(6).toString('hex')}`;
    const code = randomBytes(24).toString('hex');
    const adminCode = randomBytes(16).toString('hex');
    command(['publish', '--server', 'http://127.0.0.1:3000', '--module-path', 'spacetimedb', '--no-config', '--yes=skip-login', testDatabase]);
    command(['call', '--server', 'http://127.0.0.1:3000', '--no-config', testDatabase, 'configure_access', JSON.stringify(code)]);
    command(['call', '--server', 'http://127.0.0.1:3000', '--no-config', testDatabase, 'configure_admin_access', JSON.stringify(adminCode)]);
    process.env.TEST_SPACETIMEDB_DB_NAME = testDatabase;
    process.env.TEST_HOUSEHOLD_CODE = code;
    process.env.TEST_ADMIN_CODE = adminCode;
  }
  process.env.TEST_BROWSER = process.argv.includes('--browser') ? 'true' : '';
  await import(pathToFileURL(output).href);
  if (process.argv.includes('--browser') && testDatabase) {
    process.env.TEST_APP_URL = `http://127.0.0.1:3000/v1/database/${testDatabase}/route/`;
    await import('./admin-browser-check.mjs');
  }
} finally {
  if (testDatabase) command(['delete', '--server', 'http://127.0.0.1:3000', '--no-config', '--yes', testDatabase]);
}
