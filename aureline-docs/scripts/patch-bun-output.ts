import { cpSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';

const source = resolve('node_modules/tslib');
const target = resolve('.output/server/node_modules/tslib');

if (existsSync(source)) {
  cpSync(source, target, { recursive: true });
}
