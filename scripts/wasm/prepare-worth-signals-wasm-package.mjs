import { execFile } from "node:child_process";
import { access, copyFile, mkdir, mkdtemp, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire, stripTypeScriptTypes } from "node:module";
import { tmpdir } from "node:os";
import { promisify } from "node:util";
import path from "node:path";
import process from "node:process";

import { bundleWorthSignalsWasmProduct } from "./bundle-worth-signals-wasm-product.mjs";
import { writeBundlerCompatibleWasmEntrypoint } from "./write-worth-signals-wasm-entrypoint.mjs";

const execFileAsync = promisify(execFile);
const crateRequire = createRequire(
  path.resolve("crates/worth-signal-wasm/package.json"),
);
const scope = process.env.WORTH_SIGNAL_WASM_SCOPE ?? null;
const packageNameOverride = process.env.WORTH_SIGNAL_WASM_PACKAGE_NAME ?? null;
const publishRegistry = process.env.WORTH_SIGNAL_WASM_REGISTRY
  ?? "https://registry.npmjs.org";
const publishAccess = process.env.WORTH_SIGNAL_WASM_PUBLISH_ACCESS ?? "public";
const publishNoticeMode = process.env.WORTH_SIGNAL_WASM_NOTICE_MODE ?? "none";

const normalizedScope = scope ? scope.toLowerCase() : null;
const repoUrl = process.env.WORTH_SIGNAL_WASM_REPOSITORY_URL
  ?? "https://github.com/recnepspencer/forge.git";
const pkgDir = path.resolve(
  process.argv[2] ?? "crates/worth-signal-wasm/pkg",
);
const packageIndexDeclarationsPath = path.resolve(
  "crates/worth-signal-wasm/package/index.d.ts",
);
const rawSurfaceDeclarationsPath = path.resolve(
  "crates/worth-signal-wasm/package/raw_surface.d.ts",
);
const packageSourceDirPath = path.resolve("crates/worth-signal-wasm/package-src");
const typesDirPath = path.resolve("crates/worth-signal-wasm/package/types");
const readmePath = path.resolve("crates/worth-signal-wasm/README.md");
const licensePath = path.resolve("crates/worth-signal-wasm/LICENSE");
const docsDirPath = path.resolve("crates/worth-signal-wasm/docs");
const cargoManifestPath = path.resolve("crates/worth-signal-wasm/Cargo.toml");
const reactDeclarationsDirPath = path.resolve("crates/worth-signal-wasm/react");
const reactTsConfigPath = path.resolve("crates/worth-signal-wasm/tsconfig.react.json");
const reactCrateDir = path.resolve("crates/worth-signal-wasm");
const reactTscBinaryPath = path.resolve(
  "crates/worth-signal-wasm/node_modules/typescript/lib/tsc.js",
);
const preservedStageEntries = new Set([
  ".gitignore",
  "package.json",
  "worth_signal_wasm.js",
  "worth_signal_wasm_bg.js",
  "worth_signal_wasm_bg.wasm",
  "worth_signal_wasm_bg.wasm.d.ts",
  "snippets",
]);
const packageJsonPath = path.join(pkgDir, "package.json");
const packageJson = JSON.parse(await readFile(packageJsonPath, "utf8"));
const cargoManifest = await readFile(cargoManifestPath, "utf8");
const crateVersionMatch = cargoManifest.match(/^version\s*=\s*"([^"]+)"\s*$/mu);

if (!crateVersionMatch) {
  throw new Error(`Could not determine worth-signals-wasm version from ${cargoManifestPath}`);
}

const crateVersion = crateVersionMatch[1];

async function copyDirectoryRecursive(sourceDir, destinationDir) {
  await mkdir(destinationDir, { recursive: true });
  const entries = await readdir(sourceDir, { withFileTypes: true });
  for (const entry of entries) {
    const sourcePath = path.join(sourceDir, entry.name);
    const destinationPath = path.join(destinationDir, entry.name);
    if (entry.isDirectory()) {
      await copyDirectoryRecursive(sourcePath, destinationPath);
      continue;
    }
    await copyFile(sourcePath, destinationPath);
  }
}

async function ensureCratePublishToolingInstalled() {
  // TypeScript is vendored in-tree for React emit; esbuild is lockfile-pinned and
  // platform-native, so install on demand when missing from a clean checkout.
  try {
    crateRequire.resolve("esbuild");
    return;
  } catch {
    // continue to install
  }
  const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
  await execFileAsync(
    npmCommand,
    ["install", "--no-fund", "--no-audit"],
    { cwd: reactCrateDir },
  );
  crateRequire.resolve("esbuild");
}

async function transpilePackageSourcesRecursive(sourceDir, destinationDir) {
  await mkdir(destinationDir, { recursive: true });
  const entries = await readdir(sourceDir, { withFileTypes: true });
  for (const entry of entries) {
    const sourcePath = path.join(sourceDir, entry.name);
    const destinationPath = path.join(destinationDir, entry.name);
    if (entry.isDirectory()) {
      await transpilePackageSourcesRecursive(sourcePath, destinationPath);
      continue;
    }
    if (entry.name.endsWith(".d.ts")) {
      continue;
    }
    if (!entry.name.endsWith(".ts")) {
      continue;
    }
    if (entry.name === "types-smoke.ts" || entry.name.endsWith(".test.ts")) {
      continue;
    }
    const outputPath = destinationPath.replace(/\.ts$/u, ".js");
    const source = await readFile(sourcePath, "utf8");
    const transformed = stripTypeScriptTypes(source, { mode: "transform" });
    await mkdir(path.dirname(outputPath), { recursive: true });
    await writeFile(outputPath, transformed, "utf8");
  }
}

async function resetPackageStage() {
  const entries = await readdir(pkgDir, { withFileTypes: true });
  for (const entry of entries) {
    if (preservedStageEntries.has(entry.name)) {
      continue;
    }
    await rm(path.join(pkgDir, entry.name), { recursive: true, force: true });
  }
}

async function copyReactDeclarationsRecursive(sourceDir, destinationDir) {
  await mkdir(destinationDir, { recursive: true });
  const entries = await readdir(sourceDir, { withFileTypes: true });
  for (const entry of entries) {
    const sourcePath = path.join(sourceDir, entry.name);
    const destinationPath = path.join(destinationDir, entry.name);
    if (entry.isDirectory()) {
      await copyReactDeclarationsRecursive(sourcePath, destinationPath);
      continue;
    }
    if (!entry.name.endsWith(".d.ts")) {
      continue;
    }
    const source = await readFile(sourcePath, "utf8");
    const rewritten = source.replaceAll("../package/types/", "../types/");
    await writeFile(destinationPath, rewritten, "utf8");
  }
}

async function compileReactEntryPoints(reactOutDir) {
  const reactTsConfigArg = path.relative(reactCrateDir, reactTsConfigPath) || "tsconfig.react.json";
  // tsc emits to pkg/react per tsconfig; override by compiling then moving when needed.
  const defaultOutDir = path.join(pkgDir, "react");
  await mkdir(defaultOutDir, { recursive: true });
  try {
    await access(reactTscBinaryPath);
    await execFileAsync(
      process.execPath,
      [reactTscBinaryPath, "-p", reactTsConfigArg],
      { cwd: reactCrateDir },
    );
  } catch (error) {
    if (error?.code !== "ENOENT") {
      throw error;
    }
    if (process.platform === "win32") {
      await execFileAsync(
        "cmd.exe",
        [
          "/d",
          "/s",
          "/c",
          `npx --yes -p typescript -p react -p @types/react tsc -p ${reactTsConfigArg}`,
        ],
        { cwd: reactCrateDir },
      );
    } else {
      await execFileAsync(
        "npx",
        [
          "--yes",
          "-p",
          "typescript",
          "-p",
          "react",
          "-p",
          "@types/react",
          "tsc",
          "-p",
          reactTsConfigArg,
        ],
        { cwd: reactCrateDir },
      );
    }
  }
  if (path.resolve(reactOutDir) !== path.resolve(defaultOutDir)) {
    await copyDirectoryRecursive(defaultOutDir, reactOutDir);
  }
}

packageJson.name = packageNameOverride
  ?? (normalizedScope ? `@${normalizedScope}/worth-signals-wasm` : "worth-signals-wasm");
packageJson.version = crateVersion;
packageJson.license = "BUSL-1.1";
packageJson.repository = {
  type: "git",
  url: repoUrl,
};
packageJson.publishConfig = {
  registry: publishRegistry,
};
if (publishAccess) {
  packageJson.publishConfig.access = publishAccess;
}
packageJson.main = "./index.js";
packageJson.module = "./index.js";
packageJson.types = "./index.d.ts";
packageJson.files = [
  "worth_signal_wasm.js",
  "worth_signal_wasm_bg.js",
  "worth_signal_wasm_bg.wasm",
  "worth_signal_wasm_bg.wasm.d.ts",
  "index.js",
  "index.d.ts",
  "raw_surface.js",
  "raw_surface.d.ts",
  "product",
  "chunks",
  "types",
  "README.md",
  "LICENSE",
  "docs",
  "react",
  "snippets",
];
packageJson.exports = {
  ".": {
    types: "./index.d.ts",
    import: "./index.js",
  },
  "./wasm": "./worth_signal_wasm_bg.wasm",
  "./worker": "./product/entrypoint/bridge/worker_runtime_bridge_worker.js",
  "./raw": {
    types: "./raw_surface.d.ts",
    import: "./raw_surface.js",
  },
  "./raw_surface.js": {
    types: "./raw_surface.d.ts",
    import: "./raw_surface.js",
  },
  "./react": {
    types: "./react/index.d.ts",
    import: "./react/index.js",
  },
};
packageJson.peerDependencies = {
  react: "^18.0.0 || ^19.0.0",
};
packageJson.peerDependenciesMeta = {
  react: {
    optional: true,
  },
};
packageJson.sideEffects = [
  "./worth_signal_wasm.js",
  "./product/entrypoint/bridge/worker_runtime_bridge_worker.js",
  "./snippets/*",
];

await resetPackageStage();
await writeBundlerCompatibleWasmEntrypoint(pkgDir);

const noticePath = path.join(pkgDir, "PROPRIETARY.md");
if (publishNoticeMode === "proprietary") {
  const notice = `Proprietary Software Notice

This package is unpublished for general public use and is distributed only through private agreement.

No license is granted except as expressly provided in a separate written agreement with the rights holder.
`;
  await writeFile(noticePath, notice, "utf8");
} else {
  await rm(noticePath, { force: true });
}

const npmrcPath = path.join(pkgDir, ".npmrc");
const registryUrl = new URL(publishRegistry);
const authHost = registryUrl.host;
const normalizedRegistry = publishRegistry.endsWith("/")
  ? publishRegistry.slice(0, -1)
  : publishRegistry;
const scopeRegistryLine = !normalizedScope || publishRegistry.includes("registry.npmjs.org")
  ? ""
  : `@${normalizedScope}:registry=${normalizedRegistry}\n`;
const npmrc = `${scopeRegistryLine}//${authHost}/:_authToken=\${NODE_AUTH_TOKEN}
`;
await writeFile(npmrcPath, npmrc, "utf8");

await copyFile(
  packageIndexDeclarationsPath,
  path.join(pkgDir, "index.d.ts"),
);
await copyFile(
  rawSurfaceDeclarationsPath,
  path.join(pkgDir, "raw_surface.d.ts"),
);

await ensureCratePublishToolingInstalled();

const transpileRoot = await mkdtemp(path.join(tmpdir(), "worth-signals-transpile-"));
const reactEmitRoot = await mkdtemp(path.join(tmpdir(), "worth-signals-react-emit-"));
try {
  await transpilePackageSourcesRecursive(packageSourceDirPath, transpileRoot);
  // Wasm glue must be resolvable while bundling even though it stays external.
  await copyFile(
    path.join(pkgDir, "worth_signal_wasm.js"),
    path.join(transpileRoot, "worth_signal_wasm.js"),
  );
  await copyFile(
    path.join(pkgDir, "worth_signal_wasm_bg.js"),
    path.join(transpileRoot, "worth_signal_wasm_bg.js"),
  );

  await compileReactEntryPoints(path.join(pkgDir, "react"));
  await copyDirectoryRecursive(path.join(pkgDir, "react"), reactEmitRoot);
  // Clear tsc forest before writing the react bundle entry.
  await rm(path.join(pkgDir, "react"), { recursive: true, force: true });

  await bundleWorthSignalsWasmProduct({
    transpileRoot,
    reactEmitRoot,
    pkgDir,
  });
} finally {
  await rm(transpileRoot, { recursive: true, force: true });
  await rm(reactEmitRoot, { recursive: true, force: true });
}

await copyFile(readmePath, path.join(pkgDir, "README.md"));
await copyFile(licensePath, path.join(pkgDir, "LICENSE"));
await copyDirectoryRecursive(docsDirPath, path.join(pkgDir, "docs"));
await copyDirectoryRecursive(typesDirPath, path.join(pkgDir, "types"));
await mkdir(path.join(pkgDir, "react"), { recursive: true });
await copyReactDeclarationsRecursive(
  reactDeclarationsDirPath,
  path.join(pkgDir, "react"),
);
await writeFile(
  packageJsonPath,
  `${JSON.stringify(packageJson, null, 2)}\n`,
  "utf8",
);

console.log(`Prepared ${packageJson.name} in ${pkgDir}`);
