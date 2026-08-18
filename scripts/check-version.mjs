#!/usr/bin/env node

import { assertVersion, readVersions } from "./version.mjs";

const expected = process.argv[2] ?? readVersions().packageJson;

try {
  assertVersion(expected);
  process.stdout.write(`Rust and npm versions agree on ${expected}\n`);
} catch (error) {
  process.stderr.write(`${error.message}\n`);
  process.exit(1);
}
