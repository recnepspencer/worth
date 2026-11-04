/**
 * Track 6 WASM size policy: baseline, ceiling, and path-leak needles.
 *
 * MAX_WASM_BYTES includes 5% headroom on the accepted release measurement
 * and must stay strictly below BASELINE_WASM_BYTES.
 */
export const BASELINE_WASM_BYTES = 13_092_002;

/**
 * Release 1.6 ceiling: ceil(11_707_918 * 1.05). The larger runtime artifact
 * was accepted for this release; retain the original pre-Track-6 baseline.
 * Must remain strictly below BASELINE_WASM_BYTES.
 */
export const MAX_WASM_BYTES = 12_293_314;

export const WASM_BINARY_RELATIVE_PATH = "worth_signal_wasm_bg.wasm";
export const WASM_GLUE_RELATIVE_PATH = "worth_signal_wasm_bg.js";

export const WASM_MAGIC = Object.freeze([0x00, 0x61, 0x73, 0x6d]);

export function computeMaxWasmBytesFromMeasured(measuredBytes) {
  if (!Number.isInteger(measuredBytes) || measuredBytes <= 0) {
    throw new Error(`measured WASM bytes must be a positive integer, got ${measuredBytes}`);
  }
  if (measuredBytes >= BASELINE_WASM_BYTES) {
    throw new Error(
      `optimized WASM (${measuredBytes}) did not beat baseline ${BASELINE_WASM_BYTES}`,
    );
  }
  const withSlack = Math.ceil(measuredBytes * 1.05);
  return Math.min(withSlack, BASELINE_WASM_BYTES - 1);
}

export function pathLeakNeedles(repoRootPosix) {
  const normalized = String(repoRootPosix).replaceAll("\\", "/");
  const needles = [
    normalized,
    "C:/forge",
    "C:\\forge",
    "/Users/",
  ];
  return [...new Set(needles.filter((needle) => needle.length > 0))];
}

export function assertWasmMagicPrefix(bytes) {
  if (!(bytes instanceof Uint8Array) && !Buffer.isBuffer(bytes)) {
    throw new Error("assertWasmMagicPrefix expects Buffer or Uint8Array");
  }
  for (let index = 0; index < WASM_MAGIC.length; index += 1) {
    if (bytes[index] !== WASM_MAGIC[index]) {
      const prefix = [...bytes.slice(0, 4)]
        .map((value) => value.toString(16).padStart(2, "0"))
        .join(" ");
      throw new Error(
        `expected WASM magic 00 61 73 6d, got ${prefix || "<empty>"}`,
      );
    }
  }
}
