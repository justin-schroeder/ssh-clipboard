import { accessSync, constants } from "node:fs";
import { join } from "node:path";

const PLATFORM_NAMES = new Map([
  ["darwin", "darwin"],
  ["linux", "linux"],
]);

const ARCH_NAMES = new Map([
  ["arm64", "arm64"],
  ["x64", "amd64"],
]);

export function nativeTarget(platform = process.platform, arch = process.arch) {
  const osName = PLATFORM_NAMES.get(platform);
  const archName = ARCH_NAMES.get(arch);
  if (!osName || !archName) {
    throw new Error(
      `ssh-clipboard does not support ${platform}/${arch}; supported targets are macOS and Linux on arm64 or x64`,
    );
  }
  return `${osName}-${archName}`;
}

export function resolveNativeBinary(vendorRoot, platform = process.platform, arch = process.arch) {
  const target = nativeTarget(platform, arch);
  const binary = join(vendorRoot, target, "ssh-clipboard");
  try {
    accessSync(binary, constants.X_OK);
  } catch {
    throw new Error(
      `the ssh-clipboard npm package is missing its ${target} native binary; reinstall the package or report a broken release`,
    );
  }
  return binary;
}
