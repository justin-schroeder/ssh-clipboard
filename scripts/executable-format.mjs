const MACHO_64_LE = "cffaedfe";
const ELF = "7f454c46";

export function executableTarget(contents) {
  if (contents.length < 20) {
    throw new Error("executable header is truncated");
  }
  const magic = contents.subarray(0, 4).toString("hex");
  if (magic === MACHO_64_LE) {
    const cpuType = contents.readUInt32LE(4);
    if (cpuType === 0x0100000c) return "darwin-arm64";
    if (cpuType === 0x01000007) return "darwin-amd64";
    throw new Error(`unsupported Mach-O CPU type 0x${cpuType.toString(16)}`);
  }
  if (magic === ELF) {
    if (contents[4] !== 2 || contents[5] !== 1) {
      throw new Error("Linux binary must be a little-endian 64-bit ELF executable");
    }
    const machine = contents.readUInt16LE(18);
    if (machine === 183) return "linux-arm64";
    if (machine === 62) return "linux-amd64";
    throw new Error(`unsupported ELF machine ${machine}`);
  }
  throw new Error(`unexpected executable magic ${magic}`);
}
