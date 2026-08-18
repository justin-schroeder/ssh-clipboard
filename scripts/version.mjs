import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const VERSION_PATTERN = /^\d+\.\d+\.\d+(?:-[a-z][a-z0-9-]*\.[0-9a-f]+)?$/;

export function bumpVersion(version, bump) {
  const match = /^(\d+)\.(\d+)\.(\d+)(?:-.+)?$/.exec(version);
  if (!match) throw new Error(`unsupported version: ${version}`);
  const [, majorText, minorText, patchText] = match;
  const major = Number(majorText);
  const minor = Number(minorText);
  const patch = Number(patchText);
  if (bump === "major") return `${major + 1}.0.0`;
  if (bump === "minor") return `${major}.${minor + 1}.0`;
  if (bump === "patch") return `${major}.${minor}.${patch + 1}`;
  throw new Error(`unsupported bump: ${bump}`);
}

export function releaseVersion(baseVersion, distTag, commitHash) {
  if (distTag === "latest") return baseVersion;
  if (!/^[a-z][a-z0-9-]*$/.test(distTag)) {
    throw new Error(`invalid npm dist-tag: ${distTag}`);
  }
  if (!/^[0-9a-f]+$/.test(commitHash)) {
    throw new Error(`invalid commit hash: ${commitHash}`);
  }
  return `${baseVersion}-${distTag}.${commitHash}`;
}

export function readVersions(root = process.cwd()) {
  const packageJson = JSON.parse(readFileSync(resolve(root, "package.json"), "utf8"));
  const packageLock = JSON.parse(readFileSync(resolve(root, "package-lock.json"), "utf8"));
  const cargoToml = readFileSync(resolve(root, "Cargo.toml"), "utf8");
  const cargoLock = readFileSync(resolve(root, "Cargo.lock"), "utf8");
  const cargoManifestVersion = cargoToml.match(
    /^\[package\]\n(?:.*\n)*?version = "([^"]+)"$/m,
  )?.[1];
  const cargoLockVersion = cargoLock.match(
    /^\[\[package\]\]\nname = "ssh-clipboard"\nversion = "([^"]+)"$/m,
  )?.[1];
  return {
    packageJson: packageJson.version,
    packageLock: packageLock.version,
    packageLockRoot: packageLock.packages?.[""]?.version,
    cargoToml: cargoManifestVersion,
    cargoLock: cargoLockVersion,
  };
}

export function assertVersion(expected, root = process.cwd()) {
  if (!VERSION_PATTERN.test(expected)) {
    throw new Error(`unsupported release version: ${expected}`);
  }
  const versions = readVersions(root);
  const mismatches = Object.entries(versions).filter(([, version]) => version !== expected);
  if (mismatches.length > 0) {
    const details = mismatches.map(([file, version]) => `${file}=${version ?? "missing"}`).join(", ");
    throw new Error(`release version must be ${expected}; ${details}`);
  }
  return versions;
}

export function writeVersion(version, root = process.cwd()) {
  if (!VERSION_PATTERN.test(version)) {
    throw new Error(`unsupported release version: ${version}`);
  }
  const packagePath = resolve(root, "package.json");
  const packageLockPath = resolve(root, "package-lock.json");
  const cargoTomlPath = resolve(root, "Cargo.toml");
  const cargoLockPath = resolve(root, "Cargo.lock");

  const packageJson = JSON.parse(readFileSync(packagePath, "utf8"));
  packageJson.version = version;
  writeFileSync(packagePath, `${JSON.stringify(packageJson, null, 2)}\n`);

  const packageLock = JSON.parse(readFileSync(packageLockPath, "utf8"));
  packageLock.version = version;
  packageLock.packages[""].version = version;
  writeFileSync(packageLockPath, `${JSON.stringify(packageLock, null, 2)}\n`);

  const cargoToml = readFileSync(cargoTomlPath, "utf8").replace(
    /(^\[package\]\n(?:.*\n)*?version = ")[^"]+("$)/m,
    `$1${version}$2`,
  );
  writeFileSync(cargoTomlPath, cargoToml);

  const cargoLock = readFileSync(cargoLockPath, "utf8").replace(
    /(^\[\[package\]\]\nname = "ssh-clipboard"\nversion = ")[^"]+("$)/m,
    `$1${version}$2`,
  );
  writeFileSync(cargoLockPath, cargoLock);
  assertVersion(version, root);
}
