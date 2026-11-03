import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import process from "node:process";
import { runRuntimeSmoke } from "./verify-worth-signals-wasm-runtime-smoke.mjs";
import { buildTypeSmokeSource } from "./verify-worth-signals-wasm-type-smoke-source.mjs";
import {
  assertDocsStayOnCurrentPackageStory,
  execFileAsync,
  installSmokeDependencies,
  normalizeTarEntries,
  runNpm,
  tarballFileName,
} from "./verify-worth-signals-wasm-package-support.mjs";
import { BUNDLED_JS_FILE_CAP } from "./bundle_worth_signals_wasm_entries.mjs";
import { measureJsFootprint } from "./measure-worth-signals-wasm-js-footprint.mjs";
import { verifyAdditionalConsumers } from "./verify-worth-signals-wasm-consumers.mjs";
import { buildAbortSmokeSource } from "./verify-worth-signals-wasm-abort-smoke-source.mjs";
import { buildEmptyRootSmokeSource } from "./verify-worth-signals-wasm-empty-root-smoke-source.mjs";
import { assertPublishedWasmSizeContract } from "./verify-worth-signals-wasm-size-asserts.mjs";

const pkgDir = path.resolve(process.argv[2] ?? "crates/worth-signal-wasm/pkg");
const packageJsonPath = path.join(pkgDir, "package.json");

async function runAbortSmoke(tempDir, packageName) {
  const smokePath = path.join(tempDir, "abort-smoke.mjs");
  await writeFile(smokePath, buildAbortSmokeSource(packageName), "utf8");
  const { stdout } = await execFileAsync("node", [smokePath], { cwd: tempDir });
  const result = JSON.parse(stdout.trim());
  assert.equal(
    result.rawDenialKind,
    "js-error",
    "raw createRawSignals().read denial must be a JS/wasm-bindgen error, not a trap",
  );
  assert.equal(
    result.productDenialKind,
    "js-error",
    "product signals.read denial must be a JS/wasm-bindgen error, not a trap",
  );
  assert.equal(result.rawDenialLooksLikeBoundaryError, true);
  assert.equal(result.stillReadable, true);
}

async function runEmptyRootSmoke(tempDir, packageName) {
  const smokePath = path.join(tempDir, "empty-root-smoke.mjs");
  await writeFile(smokePath, buildEmptyRootSmokeSource(packageName), "utf8");
  const { stdout } = await execFileAsync("node", [smokePath], { cwd: tempDir });
  const result = JSON.parse(stdout.trim());
  assert.equal(result.emptyRootSpecAuthored, true);
  assert.equal(result.emptyRootSpecMutated, true);
  assert.equal(result.emptyRootAspectReads, true);
  assert.equal(result.emptyRootFormSyncReceipt, true);
  assert.equal(result.emptyRootReactSnapshot, true);
  assert.equal(result.emptyRootGraphDenied, true);
}

async function runTypeSmoke(tempDir, packageName) {
  const smokeTypePath = path.join(tempDir, "smoke.ts");
  const tscJsPath = path.join(tempDir, "node_modules", "typescript", "bin", "tsc");
  const source = buildTypeSmokeSource(packageName);
  await writeFile(smokeTypePath, source, "utf8");
  const observationTypePath = path.join(tempDir, "observation-target-smoke.ts");
  const observationSource = await readFile(new URL(
    "../../crates/worth-signal-wasm/package/observation-target-types-smoke.ts",
    import.meta.url,
  ), "utf8");
  await writeFile(observationTypePath,
    observationSource.replace('from "./index.js"', `from "${packageName}"`), "utf8");
  const args = [
    tscJsPath,
    "--noEmit",
    "--strict",
    "--target", "ES2022",
    "--module", "ESNext",
    "--moduleResolution", "Bundler",
    "--skipLibCheck", "false",
    "--lib", "ESNext,DOM,ESNext.Disposable",
    "--jsx", "react-jsx",
    smokeTypePath,
    observationTypePath,
  ];
  await execFileAsync(process.execPath, args, { cwd: tempDir });
}

async function main() {
  const packageJson = JSON.parse(await readFile(packageJsonPath, "utf8"));
  const expectedTarballName = tarballFileName(packageJson.name, packageJson.version);
  const tarballPath = path.join(pkgDir, expectedTarballName);

  assert.equal(packageJson.main, "./index.js");
  assert.equal(packageJson.module, "./index.js");
  assert.equal(packageJson.types, "./index.d.ts");
  assert.equal(packageJson.exports["."].import, "./index.js");
  assert.equal(packageJson.exports["."].types, "./index.d.ts");
  assert.equal(packageJson.exports["./wasm"], "./worth_signal_wasm_bg.wasm");
  assert.equal(
    packageJson.exports["./worker"],
    "./product/entrypoint/bridge/worker_runtime_bridge_worker.js",
  );
  assert.equal(packageJson.exports["./raw"].import, "./raw_surface.js");
  assert.equal(packageJson.exports["./raw"].types, "./raw_surface.d.ts");
  assert.equal(packageJson.exports["./raw_surface.js"].import, "./raw_surface.js");
  assert.equal(packageJson.exports["./raw_surface.js"].types, "./raw_surface.d.ts");
  assert.equal(packageJson.exports["./react"].import, "./react/index.js");
  assert.equal(packageJson.exports["./react"].types, "./react/index.d.ts");
  assert.equal(packageJson.peerDependencies.react, "^18.0.0 || ^19.0.0");
  assert.equal(
    packageJson.peerDependencies["@types/react"],
    undefined,
    "the package must not install a React type version into the consumer",
  );
  assert.equal(packageJson.peerDependenciesMeta.react.optional, true);

  const wasmEntrypointSource = await readFile(
    path.join(pkgDir, "worth_signal_wasm.js"),
    "utf8",
  );
  assert.match(
    wasmEntrypointSource,
    /function assertWasmMagic\(/u,
    "prepared WASM entry must reject non-WASM magic before instantiate",
  );
  assert.match(
    wasmEntrypointSource,
    /received HTML \(prefix/u,
    "prepared WASM entry must diagnose HTML-as-WASM bodies",
  );
  assert.match(
    wasmEntrypointSource,
    /createSignals\(\{\s*assets:/u,
    "HTML-as-WASM diagnostic must remediate via createSignals({ assets })",
  );
  assert.match(
    wasmEntrypointSource,
    /wasmInitPromise = null/u,
    "failed WASM init must clear the in-flight promise so callers can retry",
  );

  const reactEntrySource = await readFile(
    path.join(pkgDir, "react", "index.js"),
    "utf8",
  );
  assert.doesNotMatch(
    reactEntrySource,
    /import\s+(?!\{)[A-Za-z_$][\w$]*(?:\s*,\s*\{[^}]*\})?\s+from\s*["']react["']/u,
    "the ESM adapter must not depend on synthetic React default-import interop",
  );
  assert.match(
    reactEntrySource,
    /from\s*["']react["']/u,
    "the bundled React adapter must import the react peer",
  );

  await assertPublishedWasmSizeContract(pkgDir);

  const footprint = await measureJsFootprint(pkgDir, { pack: false });
  assert.ok(
    footprint.counts.jsFiles <= BUNDLED_JS_FILE_CAP,
    `published JS file count ${footprint.counts.jsFiles} exceeds Track 5 cap ${BUNDLED_JS_FILE_CAP}`,
  );
  assert.ok(
    footprint.counts.productJsFiles <= 8,
    `product JS forest must stay collapsed (got ${footprint.counts.productJsFiles})`,
  );
  const hashedChunks = (footprint.files ?? [])
    .map((file) => file.path)
    .filter((relativePath) =>
      relativePath.startsWith("chunks/") &&
      /(?:^|[-_.])[a-f0-9]{8,}(?:\.js)?$/iu.test(path.basename(relativePath))
    );
  assert.deepEqual(
    hashedChunks,
    [],
    "Track 5 forbids content-hashed chunk filenames (use chunks/[name])",
  );

  const bridgeSource = await readFile(
    path.join(pkgDir, "product/entrypoint/bridge/worker_runtime_bridge.js"),
    "utf8",
  );
  assert.match(
    bridgeSource,
    /new URL\(\s*["']\.\/worker_runtime_bridge_worker\.js["']\s*,\s*import\.meta\.url\s*\)/u,
    "bridge shell must keep colocated worker URL on import.meta.url",
  );
  assert.doesNotMatch(
    bridgeSource,
    /from\s+["']\.\/chunks\//u,
    "bridge shell must not depend on shared chunks (colocated worker URL authority)",
  );

  await rm(tarballPath, { force: true });
  const { stdout: packStdout } = await runNpm(["pack", "--json"], { cwd: pkgDir });
  const packOutput = JSON.parse(packStdout);
  const packResults = Array.isArray(packOutput) ? packOutput : Object.values(packOutput);
  const packedFiles = Array.isArray(packResults) && packResults.length > 0
    ? packResults[0].files
    : [];
  const entries = packedFiles.map((file) =>
    `package/${String(file.path).replaceAll("\\", "/")}`
  );
  const requiredEntries = [
    "package/index.js",
    "package/index.d.ts",
    "package/worth_signal_wasm.js",
    "package/worth_signal_wasm_bg.js",
    "package/worth_signal_wasm_bg.wasm",
    "package/product/entrypoint/bridge/worker_runtime_bridge.js",
    "package/product/entrypoint/bridge/worker_runtime_bridge_worker.js",
    "package/raw_surface.d.ts",
    "package/raw_surface.js",
    "package/types/callable_surface.d.ts",
    "package/types/controller_surface.d.ts",
    "package/types/diagnostics.d.ts",
    "package/types/graph_surface.d.ts",
    "package/types/model.d.ts",
    "package/types/raw_surface.d.ts",
    "package/types/resource/api_namespace.d.ts",
    "package/types/resource/api_route_builder.d.ts",
    "package/types/resource/resource_namespace.d.ts",
    "package/react/index.js",
    "package/react/index.d.ts",
    "package/react/model.d.ts",
    "package/README.md",
    "package/docs/start_here.md",
    "package/docs/core/README.md",
    "package/docs/core/aspects.md",
    "package/docs/core/diagnostics.md",
    "package/docs/forms/index.md",
    "package/docs/forms/getting-started/your-first-form.md",
    "package/docs/package/install-and-publish.md",
    "package/docs/local-truth/README.md",
    "package/docs/local-truth/branch-merge.md",
    "package/docs/reference/support-status.md",
    "package/docs/resources/overview.md",
    "package/docs/resources/branch-native-effects.md",
    "package/docs/resources/merge-and-rebase.md",
    "package/docs/resources/json-effects.md",
    "package/docs/resources/effects/effect-envelopes-and-closeout.md",
    "package/docs/resources/verification/response-topology-proof.md",
    "package/docs/router/index.md",
    "package/docs/router/runtime_placement/worker_first_default.md",
    "package/docs/api-reference/route-authoring.md",
  ];
  const forbiddenEntries = [
    "package/worth_signal_wasm.d.ts",
    "package/product/signals.js",
    "package/product/host_capabilities.js",
    "package/product/handles.js",
    "package/react/context.js",
  ];
  for (const requiredEntry of requiredEntries) {
    assert.equal(
      entries.includes(requiredEntry),
      true,
      `expected tarball to contain ${requiredEntry}`,
    );
  }
  for (const forbiddenEntry of forbiddenEntries) {
    assert.equal(
      entries.includes(forbiddenEntry),
      false,
      `expected tarball to exclude ${forbiddenEntry}`,
    );
  }
  const staleProductModules = entries.filter(
    (entry) => entry.startsWith("package/product/") && entry.endsWith(".mjs"),
  );
  assert.deepEqual(
    staleProductModules,
    [],
    "expected tarball to exclude stale product-side .mjs artifacts",
  );
  const deepProductJs = entries.filter(
    (entry) =>
      entry.startsWith("package/product/") &&
      entry.endsWith(".js") &&
      entry !== "package/product/entrypoint/bridge/worker_runtime_bridge.js" &&
      entry !== "package/product/entrypoint/bridge/worker_runtime_bridge_worker.js",
  );
  assert.deepEqual(
    deepProductJs,
    [],
    "unbundled product forest must not ship beside the Track 5 entry shells",
  );

  const tempDir = await mkdtemp(path.join(tmpdir(), "worth-signals-wasm-package-verify-"));
  try {
    await installSmokeDependencies(tempDir, tarballPath);
    await runAbortSmoke(tempDir, packageJson.name);
    await runEmptyRootSmoke(tempDir, packageJson.name);
    await runRuntimeSmoke(tempDir, packageJson.name);
    await runTypeSmoke(tempDir, packageJson.name);
    await assertDocsStayOnCurrentPackageStory(pkgDir, packageJson.name);
    await verifyAdditionalConsumers(tarballPath, packageJson.name);
  } finally {
    await rm(tempDir, { recursive: true, force: true });
    await rm(tarballPath, { force: true });
  }
}

await main();
