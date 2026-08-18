#!/usr/bin/env node

import { execSync } from "node:child_process";
import process from "node:process";
import * as prompts from "@clack/prompts";

import { assertVersion, bumpVersion, readVersions, releaseVersion, writeVersion } from "./version.mjs";

const VALID_BUMPS = ["patch", "minor", "major"];
const args = process.argv.slice(2);
const dryRun = args.includes("--dry-run");
const yes = args.includes("--yes");
const cliTag = flagValue("--tag") ?? "latest";
const cliBump = flagValue("--bump") ?? args.find((argument) => VALID_BUMPS.includes(argument));

function flagValue(name) {
  const inline = args.find((argument) => argument.startsWith(`${name}=`));
  if (inline) return inline.slice(name.length + 1);
  const index = args.indexOf(name);
  return index >= 0 ? args[index + 1] : null;
}

function quote(value) {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

function run(command, { inherit = false } = {}) {
  return execSync(command, {
    encoding: "utf8",
    stdio: inherit ? "inherit" : "pipe",
  }).trim();
}

function mutate(command, options = {}) {
  if (dryRun) {
    prompts.log.info(`[dry-run] ${command}`);
    return "";
  }
  return run(command, options);
}

function cancel(message) {
  prompts.cancel(message);
  process.exit(1);
}

function requireCleanMain() {
  if (run("git status --porcelain") !== "") cancel("Working tree must be clean.");
  const branch = run("git branch --show-current");
  if (branch !== "main") cancel(`Releases must start on main, not ${branch || "detached HEAD"}.`);
  run("git fetch origin main --quiet");
  const local = run("git rev-parse main");
  const remote = run("git rev-parse origin/main");
  if (local !== remote) cancel("Local main must exactly match origin/main.");
}

function requireAvailableTag(tag) {
  const reference = `refs/tags/${tag}`;
  try {
    run(`git rev-parse --verify --quiet ${quote(reference)}`);
    cancel(`Tag ${tag} already exists locally.`);
  } catch (error) {
    if (error?.status === 1) {
      // A missing local ref is expected.
    } else {
      throw error;
    }
  }
  if (run(`git ls-remote --tags origin ${quote(reference)}`) !== "") {
    cancel(`Tag ${tag} already exists on origin.`);
  }
}

function verify() {
  const spinner = prompts.spinner();
  spinner.start("Running Rust and npm release gates");
  try {
    run("npm test");
    run("cargo fmt --all -- --check");
    run("cargo clippy --all-targets -- -D warnings");
    run("cargo test --all-targets");
    spinner.stop("Release gates passed");
  } catch (error) {
    spinner.stop("Release gates failed");
    throw error;
  }
}

function commitAndTag(version, tag) {
  if (dryRun) {
    prompts.log.info(`[dry-run] Rust and npm versions → ${version}`);
  } else {
    writeVersion(version);
    assertVersion(version);
  }
  mutate("git add package.json package-lock.json Cargo.toml Cargo.lock", { inherit: true });
  mutate(`git commit -m ${quote(`chore: release ${tag}`)}`, { inherit: true });
  mutate(`git tag -a ${quote(tag)} -m ${quote(tag)}`, { inherit: true });
}

async function chooseBump(currentVersion) {
  if (cliBump) {
    if (!VALID_BUMPS.includes(cliBump)) cancel(`Bump must be one of: ${VALID_BUMPS.join(", ")}.`);
    return cliBump;
  }
  const bump = await prompts.select({
    message: "Version bump",
    options: VALID_BUMPS.map((value) => ({
      value,
      label: value,
      hint: `${currentVersion} → ${bumpVersion(currentVersion, value)}`,
    })),
  });
  if (prompts.isCancel(bump)) {
    prompts.cancel("Release cancelled.");
    process.exit(0);
  }
  return bump;
}

async function main() {
  prompts.intro("ssh-clipboard release");
  requireCleanMain();
  const currentVersion = readVersions().packageJson;
  assertVersion(currentVersion);
  const bump = await chooseBump(currentVersion);
  const baseVersion = bumpVersion(currentVersion, bump);
  const shortHash = run("git rev-parse --short=7 HEAD");
  const version = releaseVersion(baseVersion, cliTag, shortHash);
  const tag = `v${version}`;
  requireAvailableTag(tag);

  if (!yes) {
    const shouldVerify = await prompts.confirm({ message: "Run the full release gates?", initialValue: true });
    if (prompts.isCancel(shouldVerify)) process.exit(0);
    if (shouldVerify) verify();
  } else {
    verify();
  }

  prompts.note(
    [`Version:  ${currentVersion} → ${version}`, `Git tag:  ${tag}`, `npm tag:  ${cliTag}`].join("\n"),
    "Release summary",
  );
  if (!yes) {
    const confirmed = await prompts.confirm({
      message: dryRun ? "Complete this dry run?" : "Commit, tag, and publish through GitHub Actions?",
    });
    if (prompts.isCancel(confirmed) || !confirmed) {
      prompts.cancel("Release cancelled.");
      process.exit(0);
    }
  }

  if (cliTag === "latest") {
    commitAndTag(version, tag);
    mutate("git push origin main --follow-tags");
    prompts.outro(`${dryRun ? "Would release" : "Released"} ${tag} through GitHub Actions`);
    return;
  }

  const temporaryBranch = `release/${tag}`;
  let switched = false;
  try {
    mutate(`git switch -c ${quote(temporaryBranch)}`);
    switched = !dryRun;
    commitAndTag(version, tag);
    mutate(`git push origin ${quote(tag)}`);
    prompts.outro(`${dryRun ? "Would publish" : "Published tag"} ${tag} as npm @${cliTag}`);
  } finally {
    if (switched) {
      run("git switch main");
      run(`git branch -D ${quote(temporaryBranch)}`);
    }
  }
}

main().catch((error) => {
  prompts.cancel(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
