import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const wasmBindgenGlueFileName = "worth_signal_wasm_bg.js";

/**
 * Reads the public export names of the wasm-bindgen glue so the entry re-exports
 * exactly what the crate exposes. A hard-coded list silently went stale when the
 * crate gained `discardRestoreToken`, which made `raw_surface.js` fail to link.
 */
export async function readWasmBindgenPublicExportNames(pkgDir) {
  const glueSource = await readFile(path.join(pkgDir, wasmBindgenGlueFileName), "utf8");
  const names = new Set();
  for (const match of glueSource.matchAll(
    /^export\s+(?:async\s+)?(?:function|class)\s+([A-Za-z_$][\w$]*)/gmu,
  )) {
    const name = match[1];
    // `__wbg_*` / `__wbindgen_*` are wasm import shims, never part of the surface.
    if (!name.startsWith("__")) {
      names.add(name);
    }
  }
  if (names.size === 0) {
    throw new Error(
      `worth-signals-wasm: found no public exports in ${wasmBindgenGlueFileName}; was wasm-pack run with --target bundler?`,
    );
  }
  return [...names].sort((left, right) => left.localeCompare(right, "en"));
}

/** Writes the bundler-compatible WASM JS entry with HTML-as-WASM diagnostics. */
export async function writeBundlerCompatibleWasmEntrypoint(pkgDir) {
  const publicExportNames = await readWasmBindgenPublicExportNames(pkgDir);
  const source = `/* @ts-self-types="./worth_signal_wasm.d.ts" */

import * as imports from "./worth_signal_wasm_bg.js";
import { __wbg_set_wasm } from "./worth_signal_wasm_bg.js";

let wasmInitialized = false;
let wasmInitPromise = null;

async function init(input) {
  if (wasmInitialized) {
    return imports;
  }
  if (wasmInitPromise !== null) {
    return wasmInitPromise;
  }
  wasmInitPromise = initializeWasm(input).catch((error) => {
    wasmInitPromise = null;
    throw error;
  });
  return wasmInitPromise;
}

async function initializeWasm(input) {
  const importObject = { "./worth_signal_wasm_bg.js": imports };
  const wasm = input === undefined
    ? await instantiateDefaultWasm(importObject)
    : (await instantiateWasm(input, importObject)).exports;
  __wbg_set_wasm(wasm);
  wasm.__wbindgen_start();
  wasmInitialized = true;
  return imports;
}

async function instantiateDefaultWasm(importObject) {
  return (await instantiateWasm(
    new URL("./worth_signal_wasm_bg.wasm", import.meta.url),
    importObject,
  )).exports;
}

async function instantiateWasm(source, importObject) {
  if (source instanceof WebAssembly.Module) {
    return new WebAssembly.Instance(source, importObject);
  }
  if (source instanceof WebAssembly.Instance) {
    return source;
  }
  if (source instanceof Response) {
    return instantiateResponse(source, importObject);
  }
  if (source instanceof URL && source.protocol === "file:") {
    return instantiateFileUrl(source, importObject);
  }
  if (source instanceof URL || typeof source === "string" || source instanceof Request) {
    return instantiateResponse(fetch(source), importObject);
  }
  const result = await WebAssembly.instantiate(source, importObject);
  return result instanceof WebAssembly.Instance ? result : result.instance;
}

async function instantiateFileUrl(url, importObject) {
  const nodeFsPromises = "node:fs/promises";
  const { readFile } = await import(/* @vite-ignore */ nodeFsPromises);
  const bytes = await readFile(url);
  assertWasmMagic(bytes, url.href);
  const result = await WebAssembly.instantiate(bytes, importObject);
  return result instanceof WebAssembly.Instance ? result : result.instance;
}

async function instantiateResponse(responseOrPromise, importObject) {
  const response = await responseOrPromise;
  if (WebAssembly.instantiateStreaming && response.headers.get("Content-Type") === "application/wasm") {
    try {
      const result = await WebAssembly.instantiateStreaming(response.clone(), importObject);
      return result.instance;
    } catch {
      // Fall through to buffered instantiate with magic-byte diagnostics.
    }
  }
  const bytes = await response.arrayBuffer();
  assertWasmMagic(bytes, response.url || "<response>");
  const result = await WebAssembly.instantiate(bytes, importObject);
  return result.instance;
}

function assertWasmMagic(bytes, sourceLabel) {
  const prefix = bytes instanceof ArrayBuffer
    ? new Uint8Array(bytes, 0, Math.min(4, bytes.byteLength))
    : Uint8Array.from(bytes).subarray(0, 4);
  if (prefix.length >= 4 && prefix[0] === 0x00 && prefix[1] === 0x61 && prefix[2] === 0x73 && prefix[3] === 0x6d) {
    return;
  }
  const hex = [...prefix].map((byte) => byte.toString(16).padStart(2, "0")).join(" ");
  const looksLikeHtml = prefix.length >= 1 && prefix[0] === 0x3c;
  throw new Error(
    looksLikeHtml
      ? \`worth-signals-wasm: expected WASM bytes from \${sourceLabel}, but received HTML (prefix \${hex}). This usually means a bundler relocated the module (for example Vite prebundling into .vite/deps) or the host SPA-fell back missing asset routes to index.html. Fix: pass createSignals({ assets: { wasmUrl, workerUrl } }) with bundler-emitted URLs (Vite: import wasmUrl from "worth-signals-wasm/wasm?url"), ensure missing .wasm routes return 404 not HTML, and on Vite set worker.format = "es".\`
      : \`worth-signals-wasm: expected WASM magic \\\\0asm from \${sourceLabel}, got prefix \${hex}.\`,
  );
}

export default init;
export {
    ${publicExportNames.join(", ")}
} from "./worth_signal_wasm_bg.js";
`;
  await writeFile(path.join(pkgDir, "worth_signal_wasm.js"), source, "utf8");
}
