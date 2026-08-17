import { chmod, mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import assert from "node:assert/strict";

import { nativeTarget, resolveNativeBinary } from "../bin/platform.mjs";
import { executableTarget } from "../scripts/executable-format.mjs";

test("maps Node platform names to release target names", () => {
  assert.equal(nativeTarget("darwin", "arm64"), "darwin-arm64");
  assert.equal(nativeTarget("darwin", "x64"), "darwin-amd64");
  assert.equal(nativeTarget("linux", "arm64"), "linux-arm64");
  assert.equal(nativeTarget("linux", "x64"), "linux-amd64");
});

test("rejects unsupported platforms and architectures", () => {
  assert.throws(() => nativeTarget("win32", "x64"), /does not support win32\/x64/);
  assert.throws(() => nativeTarget("linux", "riscv64"), /does not support linux\/riscv64/);
});

test("resolves an executable native binary", async () => {
  const root = await mkdtemp(join(tmpdir(), "ssh-clipboard-npm-test-"));
  const directory = join(root, "darwin-arm64");
  const binary = join(directory, "ssh-clipboard");
  await mkdir(directory);
  await writeFile(binary, "test");
  await chmod(binary, 0o755);
  assert.equal(resolveNativeBinary(root, "darwin", "arm64"), binary);
});

test("reports a broken package without falling back to a download", () => {
  assert.throws(
    () => resolveNativeBinary("/definitely/missing", "linux", "x64"),
    /missing its linux-amd64 native binary/,
  );
});

test("identifies every native executable architecture", () => {
  const machoArm = Buffer.alloc(20);
  machoArm.writeUInt32BE(0xcffaedfe, 0);
  machoArm.writeUInt32LE(0x0100000c, 4);
  assert.equal(executableTarget(machoArm), "darwin-arm64");

  const machoIntel = Buffer.from(machoArm);
  machoIntel.writeUInt32LE(0x01000007, 4);
  assert.equal(executableTarget(machoIntel), "darwin-amd64");

  const elfArm = Buffer.alloc(20);
  elfArm.writeUInt32BE(0x7f454c46, 0);
  elfArm[4] = 2;
  elfArm[5] = 1;
  elfArm.writeUInt16LE(183, 18);
  assert.equal(executableTarget(elfArm), "linux-arm64");

  const elfIntel = Buffer.from(elfArm);
  elfIntel.writeUInt16LE(62, 18);
  assert.equal(executableTarget(elfIntel), "linux-amd64");
});

test("rejects a binary staged under the wrong architecture", () => {
  const elf = Buffer.alloc(20);
  elf.writeUInt32BE(0x7f454c46, 0);
  elf[4] = 2;
  elf[5] = 1;
  elf.writeUInt16LE(62, 18);
  assert.notEqual(executableTarget(elf), "linux-arm64");
});

test("Cargo and npm package versions stay aligned", async () => {
  const packageJson = JSON.parse(
    await readFile(new URL("../package.json", import.meta.url), "utf8"),
  );
  const cargoToml = await readFile(new URL("../Cargo.toml", import.meta.url), "utf8");
  const cargoVersion = cargoToml.match(/^version = "([^"]+)"$/m)?.[1];
  assert.equal(packageJson.version, cargoVersion);
});
