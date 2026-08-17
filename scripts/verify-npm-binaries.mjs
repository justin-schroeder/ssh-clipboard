import { createHash } from "node:crypto";
import { access, readFile, stat } from "node:fs/promises";
import { constants } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { executableTarget } from "./executable-format.mjs";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const vendorRoot = join(projectRoot, "vendor");
const targets = ["darwin-arm64", "darwin-amd64", "linux-arm64", "linux-amd64"];
const manifest = JSON.parse(await readFile(join(vendorRoot, "manifest.json"), "utf8"));

for (const target of targets) {
  const binary = join(vendorRoot, target, "ssh-clipboard");
  await access(binary, constants.X_OK);
  const metadata = await stat(binary);
  if (!metadata.isFile() || metadata.size === 0) {
    throw new Error(`${target} is not a non-empty regular file`);
  }
  const contents = await readFile(binary);
  const detectedTarget = executableTarget(contents);
  if (detectedTarget !== target) {
    throw new Error(`${target} contains a ${detectedTarget} executable`);
  }
  const sha256 = createHash("sha256").update(contents).digest("hex");
  if (manifest[target]?.bytes !== metadata.size || manifest[target]?.sha256 !== sha256) {
    throw new Error(`${target} does not match vendor/manifest.json`);
  }
}

process.stdout.write(`Verified ${targets.length} native binaries and checksums\n`);
