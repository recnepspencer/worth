import { readFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { constants } from "node:zlib";
import { runNpm } from "./verify-worth-signals-wasm-package-support.mjs";

function supportedVersion(actual, minimum) {
  const actualParts = actual.replace(/^v/u, "").split(".").map(Number);
  const minimumParts = minimum.split(".").map(Number);
  return actualParts.length === 3 && actualParts.every(Number.isInteger)
    && actualParts[0] === minimumParts[0]
    && (actualParts[1] > minimumParts[1]
      || (actualParts[1] === minimumParts[1] && actualParts[2] >= minimumParts[2]));
}

export function assertPatchedReleaseToolchain(actual, minimum) {
  if (!supportedVersion(actual.node, minimum.node)) {
    throw new Error(`Release requires Node ${minimum.node} or newer on the ${minimum.node.split(".")[0]}.x line; found ${actual.node}`);
  }
  if (!supportedVersion(actual.npm, minimum.npm)) {
    throw new Error(`Release requires npm ${minimum.npm} or newer on the ${minimum.npm.split(".")[0]}.x line; found ${actual.npm}`);
  }
  // Node backports the Brotli fix without changing process.versions.brotli.
  // The security release also exposes this decoder error constant.
  if (!Number.isInteger(actual.brotliBlockSwitchError)) {
    throw new Error("Release Node lacks the patched Brotli block-switch decoder boundary (CVE-2026-111497)");
  }
}

async function main() {
  const crateDir = new URL("../../crates/worth-signal-wasm/", import.meta.url);
  const manifest = JSON.parse(await readFile(new URL("package.json", crateDir), "utf8"));
  const minimum = {
    node: (await readFile(new URL(".node-version", crateDir), "utf8")).trim(),
    npm: manifest.packageManager.replace(/^npm@/u, ""),
  };
  const { stdout } = await runNpm(["--version"]);
  const actual = {
    node: process.versions.node,
    npm: stdout.trim(),
    brotliBlockSwitchError: constants.BROTLI_DECODER_ERROR_FORMAT_BLOCK_SWITCH,
  };
  assertPatchedReleaseToolchain(actual, minimum);
  console.log(`Release toolchain verified: Node ${actual.node}, npm ${actual.npm}, Brotli security backport present (${process.versions.brotli})`);
}

if (process.argv[1] && pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url) {
  await main();
}
