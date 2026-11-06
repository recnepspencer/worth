import assert from "node:assert/strict";
import test from "node:test";
import { assertPatchedReleaseToolchain } from "./verify-worth-signals-release-toolchain.mjs";

const minimum = { node: "24.22.2", npm: "12.2.1" };
const patched = { ...minimum, brotliBlockSwitchError: -17 };

test("release accepts the patched Node backport and supported later patches", () => {
  assert.doesNotThrow(() => assertPatchedReleaseToolchain(patched, minimum));
  assert.doesNotThrow(() => assertPatchedReleaseToolchain({ ...patched, node: "24.22.3", npm: "12.3.0" }, minimum));
});

test("release refuses vulnerable or unsupported toolchains before building", () => {
  for (const node of ["24.20.0", "24.22.1", "22.23.4", "24.22.3-rc.1"]) {
    assert.throws(() => assertPatchedReleaseToolchain({ ...patched, node }, minimum), /Release requires Node/u);
  }
  assert.throws(() => assertPatchedReleaseToolchain({ ...patched, npm: "11.19.0" }, minimum), /Release requires npm/u);
  assert.throws(() => assertPatchedReleaseToolchain({ ...patched, brotliBlockSwitchError: undefined }, minimum), /patched Brotli/u);
});
