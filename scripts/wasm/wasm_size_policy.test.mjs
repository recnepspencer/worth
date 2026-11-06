import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  BASELINE_WASM_BYTES,
  MAX_WASM_BYTES,
  assertWasmMagicPrefix,
  computeMaxWasmBytesFromMeasured,
  pathLeakNeedles,
} from "./wasm_size_policy.mjs";

describe("worth-signals-wasm Track 6 size policy", () => {
  it("keeps the hard ceiling below the Gate 6.0 baseline", () => {
    assert.equal(BASELINE_WASM_BYTES, 13_092_002);
    assert.ok(MAX_WASM_BYTES < BASELINE_WASM_BYTES);
    assert.equal(MAX_WASM_BYTES, computeMaxWasmBytesFromMeasured(11_707_918));
  });

  it("rejects measurements that do not beat baseline", () => {
    assert.throws(
      () => computeMaxWasmBytesFromMeasured(BASELINE_WASM_BYTES),
      /did not beat baseline/u,
    );
  });

  it("accepts WASM magic and scans path-leak needles", () => {
    assertWasmMagicPrefix(Buffer.from([0x00, 0x61, 0x73, 0x6d, 0x01]));
    assert.throws(
      () => assertWasmMagicPrefix(Buffer.from([0x3c, 0x21, 0x64, 0x6f])),
      /expected WASM magic/u,
    );
    const needles = pathLeakNeedles("C:/forge");
    assert.ok(needles.includes("C:/forge"));
    assert.ok(needles.includes("/Users/"));
  });
});
