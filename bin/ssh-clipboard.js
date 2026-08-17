#!/usr/bin/env node

import { spawn } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { resolveNativeBinary } from "./platform.mjs";

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const vendorRoot = resolve(packageRoot, "vendor");

let binary;
try {
  binary = resolveNativeBinary(vendorRoot);
} catch (error) {
  process.stderr.write(`ssh-clipboard: ${error.message}\n`);
  process.exit(1);
}

const child = spawn(binary, process.argv.slice(2), {
  stdio: "inherit",
  env: {
    ...process.env,
    SSH_CLIPBOARD_BINARIES_DIR: vendorRoot,
  },
});

child.once("error", (error) => {
  process.stderr.write(`ssh-clipboard: could not start the native binary: ${error.message}\n`);
  process.exit(1);
});

child.once("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 1);
});
