import fs from 'node:fs';
import path from 'node:path';

export function runtimeBinary(cwd = process.cwd(), env: NodeJS.ProcessEnv = process.env): string {
  if (env.FIKUTHY_RUNTIME_BIN) return env.FIKUTHY_RUNTIME_BIN;
  const executable = process.platform === 'win32' ? 'fikuthy.exe' : 'fikuthy';
  const candidates = [path.resolve(cwd, 'target/release', executable), path.resolve(cwd, '../target/release', executable), path.resolve(cwd, '../../target/release', executable)];
  return candidates.find(candidate => fs.existsSync(candidate)) ?? candidates[0]!;
}
