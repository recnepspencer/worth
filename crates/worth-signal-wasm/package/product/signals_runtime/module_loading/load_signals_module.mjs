import { rmSync } from "node:fs";
import { mkdir, mkdtemp, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { stripTypeScriptTypes } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import { after } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  productFilesToCopy,
  productSourceTreeNames,
} from "./load_signals_module_manifest.mjs";

const moduleDir = path.dirname(fileURLToPath(import.meta.url));
const packageDir = path.join(moduleDir, "..", "..", "..");
const packageSourceDir = path.join(packageDir, "..", "package-src");
const signalsModuleGlobal = globalThis;
const cachedSignalsModuleLoads =
  signalsModuleGlobal.__WorthCachedSignalsModuleLoads ?? new Map();
signalsModuleGlobal.__WorthCachedSignalsModuleLoads = cachedSignalsModuleLoads;
const cachedSignalsModuleTempDirs =
  signalsModuleGlobal.__WorthCachedSignalsModuleTempDirs ?? new Set();
signalsModuleGlobal.__WorthCachedSignalsModuleTempDirs = cachedSignalsModuleTempDirs;

// Every root created through the loaded module. A worker-first root owns a
// worker thread; a test that fails before terminating its root would keep
// the test runner's child process alive forever, which stalls the whole run
// instead of reporting the failure. Roots still alive when the file's tests
// are done are terminated here (termination is idempotent on every root).
const liveSignalsRoots =
  signalsModuleGlobal.__WorthLiveSignalsRoots ?? new Set();
signalsModuleGlobal.__WorthLiveSignalsRoots = liveSignalsRoots;

if (!signalsModuleGlobal.__WorthCachedSignalsModuleCleanupInstalled) {
  process.once("exit", () => {
    for (const tempDir of cachedSignalsModuleTempDirs) {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });
  after(async () => {
    const roots = [...liveSignalsRoots];
    liveSignalsRoots.clear();
    for (const root of roots) {
      await root.terminate();
    }
  });
  signalsModuleGlobal.__WorthCachedSignalsModuleCleanupInstalled = true;
}

function trackSignalsRoots(createRoot) {
  return async function createTrackedSignalsRoot(...args) {
    const root = await createRoot(...args);
    liveSignalsRoots.add(root);
    return root;
  };
}

export async function loadSignalsModule(options = {}) {
  const cacheKey = options.rawSurface === "real" ? "real" : "stub";
  const cachedLoad = cachedSignalsModuleLoads.get(cacheKey);
  if (cachedLoad !== undefined) {
    return cachedLoad;
  }
  const loadPromise = loadSignalsModuleIntoCachedTempDir(options, cacheKey);
  cachedSignalsModuleLoads.set(cacheKey, loadPromise);
  return loadPromise;
}

async function loadSignalsModuleIntoCachedTempDir(options, cacheKey) {
  const tempDir = await mkdtemp(path.join(tmpdir(), "worth-signal-product-"));
  cachedSignalsModuleTempDirs.add(tempDir);
  try {
    for (const [sourceRelativePath, outputRelativePath] of productFilesToCopy) {
      const sourcePath = path.join(packageSourceDir, sourceRelativePath);
      const targetPath = path.join(tempDir, outputRelativePath);
      await mkdir(path.dirname(targetPath), { recursive: true });
      const source = await readFile(sourcePath, "utf8");
      await writeFile(
        targetPath,
        stripTypeScriptTypes(source, { mode: "transform" }),
        "utf8",
      );
    }

    await writeConvertedProductSourceTrees(packageSourceDir, tempDir);

    const rawSurfacePath = path.join(tempDir, "raw_surface.js");
    if (options.rawSurface === "real") {
      const realRawSurfaceUrl = pathToFileURL(
        path.join(packageDir, "..", "pkg", "raw_surface.js"),
      ).href;
      await writeFile(
        rawSurfacePath,
        `export { default } from ${JSON.stringify(realRawSurfaceUrl)};\nexport * from ${JSON.stringify(realRawSurfaceUrl)};\n`,
        "utf8",
      );
    } else {
      await writeFile(
        rawSurfacePath,
        "export function createRawSignals() { throw new Error('createRawSignals should not be used in signals product runtime tests'); }\n",
        "utf8",
      );
    }

    const moduleUrl = new URL(
      `file:///${path.join(tempDir, "product", "signals.js").replace(/\\/g, "/")}`,
    );
    const [loadedSignals, loadedEntrypointConstruction, loadedWorkerRuntimeBridge] =
      await Promise.all([
        import(moduleUrl.href),
        import(
          pathToFileURL(
            path.join(
              tempDir,
              "product",
              "entrypoint",
              "construction",
              "entrypoint_construction.js",
            ),
          ).href
        ),
        import(
          pathToFileURL(
            path.join(
              tempDir,
              "product",
              "entrypoint",
              "bridge",
              "worker_runtime_bridge.js",
            ),
          ).href
        ),
      ]);
    const loaded = {
      ...loadedSignals,
      ...loadedEntrypointConstruction,
      ...loadedWorkerRuntimeBridge,
    };
    return {
      ...loaded,
      createSignals: trackSignalsRoots(loaded.createSignals),
      importProductModule(relativePath) {
        return import(
          pathToFileURL(
            path.join(tempDir, "product", relativePath),
          ).href
        );
      },
      cleanup: async () => {},
    };
  } catch (error) {
    cachedSignalsModuleLoads.delete(cacheKey);
    cachedSignalsModuleTempDirs.delete(tempDir);
    await rm(tempDir, { recursive: true, force: true });
    throw error;
  }
}

async function writeConvertedTree(sourceDir, outputDir) {
  const entries = await readdir(sourceDir, { withFileTypes: true });
  await mkdir(outputDir, { recursive: true });
  for (const entry of entries) {
    const sourcePath = path.join(sourceDir, entry.name);
    const outputPath = path.join(outputDir, replaceTsWithJs(entry.name));
    if (entry.isDirectory()) {
      await writeConvertedTree(sourcePath, outputPath);
      continue;
    }
    const source = await readFile(sourcePath, "utf8");
    await writeFile(
      outputPath,
      stripTypeScriptTypes(source, { mode: "transform" }),
      "utf8",
    );
  }
}

function replaceTsWithJs(name) {
  return name.endsWith(".ts") ? `${name.slice(0, -3)}.js` : name;
}

async function writeConvertedProductSourceTrees(packageSourceDir, tempDir) {
  for (const treeName of productSourceTreeNames) {
    await writeConvertedTree(
      path.join(packageSourceDir, "product", treeName),
      path.join(tempDir, "product", treeName),
    );
  }
}
