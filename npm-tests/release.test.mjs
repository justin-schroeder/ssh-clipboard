import assert from "node:assert/strict";
import { cp, mkdtemp, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import test from "node:test";

import { assertVersion, bumpVersion, readVersions, releaseVersion, writeVersion } from "../scripts/version.mjs";

test("bumps stable semantic versions", () => {
  assert.equal(bumpVersion("1.2.3", "patch"), "1.2.4");
  assert.equal(bumpVersion("1.2.3-dev.abcdef0", "minor"), "1.3.0");
  assert.equal(bumpVersion("1.2.3", "major"), "2.0.0");
});

test("creates stable and named prerelease versions", () => {
  assert.equal(releaseVersion("1.2.4", "latest", "abcdef0"), "1.2.4");
  assert.equal(releaseVersion("1.2.4", "dev", "abcdef0"), "1.2.4-dev.abcdef0");
  assert.equal(releaseVersion("1.2.4", "nightly", "1234abc"), "1.2.4-nightly.1234abc");
  assert.throws(() => releaseVersion("1.2.4", "Bad Tag", "abcdef0"), /invalid npm dist-tag/);
});

test("writes Rust and npm versions in lockstep", async () => {
  const root = await mkdtemp(join(tmpdir(), "ssh-clipboard-version-test-"));
  for (const file of ["package.json", "package-lock.json", "Cargo.toml", "Cargo.lock"]) {
    await cp(new URL(`../${file}`, import.meta.url), join(root, basename(file)));
  }
  writeVersion("9.8.7-dev.abcdef0", root);
  assertVersion("9.8.7-dev.abcdef0", root);
  assert.deepEqual(new Set(Object.values(readVersions(root))), new Set(["9.8.7-dev.abcdef0"]));
  assert.match(await readFile(join(root, "Cargo.toml"), "utf8"), /version = "9\.8\.7-dev\.abcdef0"/);
});
