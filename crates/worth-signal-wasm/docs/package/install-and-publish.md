# Consuming worth-signals-wasm

## What This Guide Is

This guide covers how to install, build, verify, and consume the public
`worth-signals-wasm` package.

Use this when you need the package entrypoints, local package workflow, or the
smallest useful examples for the shipped surface.

The root package is intentionally mixed:

- `import init from "worth-signals-wasm"` gives the raw wasm init entry
- named imports like `createSignals` and `hostCapabilityPlan` give the modern
  app-facing surface

## Why You Use It

- install the npm package cleanly
- consume a locally prepared package during workspace development
- understand the main callable surface before moving into the deeper reference
  docs
- verify that the tarball you are about to publish is internally consistent

If you just want to start coding, you only need:

1. `npm install worth-signals-wasm`
2. `import { createSignals } from "worth-signals-wasm"`
3. the small example below

The rest of this guide is for local package work and publishing from this repo.

Release tooling uses Node 24.22.2 or later on the 24.x line and npm 12.2.1 or
later on the 12.x line. The exact starting versions are recorded in
`.node-version` and `package.json` next to the crate. Before building, run:

```powershell
node scripts/wasm/verify-worth-signals-release-toolchain.mjs
```

Node 24.22.2 includes the October 2026 Brotli decoder security backport
(CVE-2026-111497), OpenSSL 3.5.9, and the other Node security updates. Its
reported Brotli version remains 1.2.0; the release check uses the patched Node
version and decoder boundary rather than treating that string as patch status.
See the [Node security release](https://nodejs.org/en/blog/release/v24.22.1).
Updating a standalone npm package named `brotli` does not patch Node's decoder.

Package contract note:

- the published package is ESM-first
- `import` and bundler resolution are the supported consumer paths
- CommonJS callers should use dynamic `import(...)` instead of `require(...)`
- **1.5.0 breaking:** default `awaitSettlement()` is tip-status only — see
  [migration-1.5](./migration-1.5.md)

## Stable Entry Points

- `createSignals(...)`
- `createReactSignalsStore(...)`
- `signals.spec.*`
- `signals.graph(...)`
- `signals.importGraph(...)`
- `signals.resource.detail(...)`
- `signals.resource.collection(...)`
- `signals.resource.paged(...)`

Package-preparation and proof entrypoints:

- `scripts/wasm/publish-worth-signals-wasm.ps1 -SkipPublish`
- `scripts/wasm/verify-worth-signals-wasm-package.mjs`
- `scripts/wasm/verify-worth-signals-demo-packed.mjs` (demo builds from `npm pack`)
- `scripts/wasm/measure-worth-signals-wasm-js-footprint.mjs` (published JS count oracle)
- `scripts/wasm/measure-worth-signals-wasm-size.mjs` (published WASM byte oracle)
- `scripts/wasm/optimize-worth-signals-wasm.mjs` (Binaryen `-Oz` size pass; required on publish)

The published package is a **multi-entry ESM bundle**: root, raw, and React
entries plus colocated worker/bridge shells. The `.wasm` binary and prepared
wasm-bindgen glue remain separate assets (Track 6 size-optimized via
`release-wasm` + `wasm-opt`). Publish JS entries are minified; deep
`product/**` modules are not part of the published layout — import only the
documented export map.

## Install Shapes

### Public npm package

```bash
npm install worth-signals-wasm
```

### Public npm package with React adapter

```bash
npm install worth-signals-wasm react
```

### Local prepared package during development

Build from the Worth workspace root:

```powershell
wasm-pack build crates/worth-signal-wasm --target bundler --profile release-wasm --no-opt --out-dir pkg
node scripts/wasm/optimize-worth-signals-wasm.mjs crates/worth-signal-wasm/pkg
```

Publish builds use the workspace `release-wasm` Cargo profile, skip wasm-pack's
implicit Binaryen pass (`--no-opt`), and require Binaryen `wasm-opt` on PATH for
the Track 6 size pass.

Prepare the package:

```powershell
node scripts/wasm/prepare-worth-signals-wasm-package.mjs crates/worth-signal-wasm/pkg
```

Verify the package:

```powershell
node scripts/wasm/verify-worth-signals-wasm-package.mjs crates/worth-signal-wasm/pkg
```

Then consume the local folder:

```json
{
  "dependencies": {
    "worth-signals-wasm": "file:../path/to/worth/crates/worth-signal-wasm/pkg"
  }
}
```

## Public Publish Flow

If you are publishing from this workspace, use this flow:

```powershell
wasm-pack build crates/worth-signal-wasm --target bundler --profile release-wasm --no-opt --out-dir pkg
node scripts/wasm/optimize-worth-signals-wasm.mjs crates/worth-signal-wasm/pkg
node scripts/wasm/prepare-worth-signals-wasm-package.mjs crates/worth-signal-wasm/pkg
node scripts/wasm/verify-worth-signals-wasm-package.mjs crates/worth-signal-wasm/pkg
cd crates/worth-signal-wasm/pkg
npm publish --access public
```

Or use the one-command release gate:

```powershell
scripts/wasm/publish-worth-signals-wasm.ps1 -SkipPublish
```

Good to know:

- the verifier is not just a packaging nicety
- it checks that the built tarball contains the files the public entrypoints
  actually need
- it also checks that a clean consumer can import and type-check the package

## Core Imports

### Main callable surface

```ts
import { createSignals } from "worth-signals-wasm";
```

### Bundler asset URLs (Vite / webpack)

```ts
import { createSignals } from "worth-signals-wasm";
import wasmUrl from "worth-signals-wasm/wasm?url";
import workerUrl from "worth-signals-wasm/worker?worker&url";

const signals = await createSignals({
  assets: { wasmUrl, workerUrl },
});
```

Vite consumers must set `worker: { format: "es" }`. Prefer this explicit
`assets` path on Vite 7 and other bundlers that prebundle dependencies away from
the package WASM file. Vite 8 often works without it when the package files
remain fetchable.

Host rule: missing `.wasm` or worker script routes must return **404**, never
SPA `index.html`. An HTML body for a WASM URL produces the fingerprint
`3c 21 64 6f` (`<!do`) and fails construction with a package remediation error.

`optimizeDeps.exclude: ["worth-signals-wasm"]` is a legacy Vite workaround only.
It is not the supported long-term recipe once `assets` or a modern Vite rebase
is available.

Workspace `file:…/pkg` installs (and `preserveSymlinks`) are convenient for
local demo work. They are not proof that a real npm consumer works. Use
`npm pack` / the packed demo verify path for that claim.

### Host capability helpers

```ts
import {
  createSignals,
  hostCapabilityPlan,
  visibilityCapability,
} from "worth-signals-wasm";
```

### Resource helpers

```ts
import {
  createSignals,
  resourceParamIdentity,
  resourceParams,
} from "worth-signals-wasm";
```

### React adapter

```ts
import {
  createReactSignalsStore,
  useOutputValue,
  useSignalValue,
  useSignalsDiagnostics,
} from "worth-signals-wasm/react";
```

## Small Example

This is the simplest useful example for the current app lane:

```ts
import { createSignals } from "worth-signals-wasm";

const signals = await createSignals();

const count = signals.input(1);
const doubled = signals.computed(() => count() * 2);

signals.transaction((tx) => {
  tx.set(count, 2);
});

console.log(doubled());
```

Why this is the best starting example:

- it uses handle-based local authoring
- it does not rely on explicit ids
- it still shows the real runtime mutation path

## Real Example

This is a more realistic consumer shape that uses local state, linked state,
controller composition, and graph publication:

```ts
import { createSignals } from "worth-signals-wasm";

const signals = await createSignals();

const itemWorkspace = signals.graph("itemWorkspace", (graph) => {
  const editor = graph.controller("editor", ({ input, computed, linked }) => {
    const serverItem = input({
      id: "task-7",
      title: "Ship docs",
      workflowTargetStateId: "ready",
    });

    const draft = input({});

    const effectiveItem = computed(() => ({
      ...serverItem(),
      ...draft(),
    }));

    const selectedWorkflowTarget = linked({
      source: () => [
        { id: "draft", label: "Draft" },
        { id: "ready", label: "Ready" },
      ],
      computation: (options, previous) => (
        options.find((option) => option.id === previous?.value?.id) ?? options[0]
      ),
    });

    const dirtyState = computed(() => Object.keys(draft()).length > 0);

    return {
      inputs: { serverItem, draft, selectedWorkflowTarget },
      outputs: { effectiveItem, dirtyState },
    };
  });

  return graph.expose({
    inputs: {
      serverItem: graph.input.required(editor.inputs.serverItem, {
        authority: "readOnly",
      }),
      draft: graph.input.optional(editor.inputs.draft),
      selectedWorkflowTarget: graph.input.optional(
        editor.inputs.selectedWorkflowTarget,
      ),
    },
    outputs: {
      effectiveItem: editor.outputs.effectiveItem,
      dirtyState: editor.outputs.dirtyState,
    },
  });
});

itemWorkspace.patchInput("draft", {
  title: "Ready to ship",
});

console.log(itemWorkspace.read());
```

## Main Lane vs Explicit Named Lane

The normal app lane is:

- `signals.input(value)`
- `signals.computed(() => ...)`
- `signals.output(() => ...)`

The explicit named lane is:

- `signals.spec.input("name", value)`
- `signals.spec.computedCallback("name", () => ...)`
- `signals.spec.outputCallback("name", () => ...)`

Use the app lane for ordinary application code. Use `signals.spec` when you
need structural names because names are part of the contract.

Add `debugName` only when you want friendlier diagnostics or clearer inspection
output. It is optional metadata, not part of local identity.

If you are unsure which lane to use, use the normal app lane.

## Graph Boundaries

When you publish a graph, that is where explicit public names become real:

```ts
const graph = signals.graph("counter", {
  inputs: {
    count,
  },
  outputs: {
    doubled,
  },
});
```

Graph inputs can also carry public input posture:

```ts
graph.expose({
  inputs: {
    serverItem: graph.input.required(editor.inputs.serverItem, {
      authority: "readOnly",
    }),
    draft: graph.input.optional(editor.inputs.draft),
  },
  outputs: {
    effectiveItem: editor.outputs.effectiveItem,
  },
});
```

That is where:

- required vs optional becomes explicit
- authority classes become explicit
- public contract names become explicit

## Mutation Helpers

Local input helpers:

```ts
draft.patch({ done: true });
draft.assign({ title: "Ready to ship" });
draft.reset();
```

Graph boundary helpers:

```ts
graph.writeInput("draft", { title: "Queued" });
graph.patchInput("draft", { reviewer: "Avery" });
graph.resetInput("draft");
```

These helpers still lower through the same runtime mutation model as
`transaction(...)`.

## Import And Restore

Published graphs can export exact same-runtime restore artifacts:

```ts
const definition = graph.exportDefinition();
const snapshot = graph.exportSnapshot();
const restoredGraph = signals.importGraph(definition, snapshot);
```

Good to know:

- this is for exact graph restore
- portable graph import is still denied on this surface
- `importPosture()` tells you what kind of restore is actually admitted

## Explicit Compatibility Recovery

The normal package lane is:

```ts
const signals = await createSignals();
```

If worker-first construction is unavailable, recover explicitly:

```ts
import { createSignals } from "worth-signals-wasm";

let signals;

try {
  signals = await createSignals();
} catch (error) {
  if (error?.artifactFamily !== "workerUnavailableConstruction") {
    throw error;
  }
  signals = await createSignals({
    deployment: "mainThreadCompatibility",
  });
}
```

Do not assume `createSignals()` silently falls back. The package keeps the
worker-unavailable lane typed so callers can decide whether compatibility is
acceptable.

## What To Read Next

- [App Surface Overview](../getting-started/choosing-a-surface.md)
- [Resource Overview](../resources/overview.md)
- [Resource Family Authoring Reference](../api-reference/resource-family-authoring.md)
- [Resource Line Reference](../api-reference/resource-line.md)
- [Resource Recipes](../start_here.md)
- [Host Capabilities](../integrations/host-capabilities.md)
- [Diagnostics And History](../core/diagnostics.md)
- [React Adapter](../integrations/react.md)
