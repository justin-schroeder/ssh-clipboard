import { createHash } from "node:crypto";
import { chmod, copyFile, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { executableTarget } from "./executable-format.mjs";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const sourceRoot = resolve(process.argv[2] ?? join(projectRoot, "dist"));
const vendorRoot = join(projectRoot, "vendor");
const targets = ["darwin-arm64", "darwin-amd64", "linux-arm64", "linux-amd64"];

const sources = new Map();
for (const target of targets) {
  const source = join(sourceRoot, `ssh-clipboard-${target}`);
  const contents = await readFile(source);
  const detectedTarget = executableTarget(contents);
  if (detectedTarget !== target) {
    throw new Error(`${source} contains a ${detectedTarget} executable`);
  }
  sources.set(target, { source, contents });
}

await mkdir(vendorRoot, { recursive: true });
for (const target of targets) {
  await rm(join(vendorRoot, target), { recursive: true, force: true });
}
await rm(join(vendorRoot, "manifest.json"), { force: true });

const manifest = {};
for (const target of targets) {
  const { source, contents } = sources.get(target);
  const destination = join(vendorRoot, target, "ssh-clipboard");
  await mkdir(dirname(destination), { recursive: true });
  await copyFile(source, destination);
  await chmod(destination, 0o755);
  manifest[target] = {
    bytes: contents.byteLength,
    sha256: createHash("sha256").update(contents).digest("hex"),
  };
}

await writeFile(join(vendorRoot, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
process.stdout.write(`Staged ${targets.length} native binaries from ${sourceRoot}\n`);
