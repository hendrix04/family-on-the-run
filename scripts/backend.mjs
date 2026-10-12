import { spawnSync } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const state = resolve('.spacetime');
const server = 'http://127.0.0.1:3000';
const database = 'family-on-the-run';
mkdirSync(state, { recursive: true, mode: 0o700 });

function run(args) {
  const result = spawnSync('spacetime', ['--config-path', `${state}/cli.toml`, ...args], {
    stdio: 'inherit',
    env: process.env,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

function buildFrontend() {
  const frontend = spawnSync('npm', ['run', 'build'], { stdio: 'inherit', env: process.env });
  if (frontend.status !== 0) process.exit(frontend.status ?? 1);
}

switch (process.argv[2]) {
  case 'build':
    buildFrontend();
    run(['build', '--module-path', 'spacetimedb']);
    console.log('Deploy artifact: spacetimedb/target/wasm32-unknown-unknown/release/family_on_the_run.wasm');
    break;
  case 'start':
    run(['start', '--listen-addr', '127.0.0.1:3000', '--data-dir', `${state}/data`, '--non-interactive']);
    break;
  case 'publish': {
    buildFrontend();
    run(['publish', '--server', server, '--module-path', 'spacetimedb', '--no-config', '--yes=skip-login', '--break-clients', database]);
    run(['call', '--server', server, '--no-config', database, 'initialize_calendar_schedules']);
    const codeFile = `${state}/access-code`;
    if (!existsSync(codeFile)) {
      const code = randomBytes(24).toString('hex');
      run(['call', '--server', server, '--no-config', database, 'configure_access', JSON.stringify(code)]);
      writeFileSync(codeFile, code + '\n', { mode: 0o600 });
      console.log('Household access code saved to .spacetime/access-code. Enter it once on each screen.');
    }
    const adminCodeFile = `${state}/admin-code`;
    if (!existsSync(adminCodeFile)) {
      const code = randomBytes(16).toString('hex');
      run(['call', '--server', server, '--no-config', database, 'configure_admin_access', JSON.stringify(code)]);
      writeFileSync(adminCodeFile, code + '\n', { mode: 0o600 });
      console.log('Admin access code saved to .spacetime/admin-code.');
    }
    run(['generate', '--module-path', 'spacetimedb', '--lang', 'typescript', '--out-dir', 'src/module_bindings', '--yes', '--no-config']);
    break;
  }
  case 'generate':
    run(['generate', '--module-path', 'spacetimedb', '--lang', 'typescript', '--out-dir', 'src/module_bindings', '--yes', '--no-config']);
    break;
  default:
    throw new Error('Usage: node scripts/backend.mjs start|build|publish|generate');
}
