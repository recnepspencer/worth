# worth-signals-wasm

Worth Signals is a browser state runtime for applications that need explicit
behavior, inspectable decisions, and reproducible execution.

Start with local inputs and computed values. Add resources, forms, routing, or
browser-local branches when the application actually has those problems.
The package keeps those responsibilities separate so React components do not
become an accidental state engine, cache, workflow system, and audit log at the
same time.

## Install

```bash
npm install worth-signals-wasm
```

```ts
import { createSignals } from "worth-signals-wasm";

const signals = await createSignals();
```

The default deployment is worker-first. Construction is asynchronous and does
not silently fall back to the main thread.

### Vite / bundler assets

On Vite, set `worker.format: "es"` (the worker uses top-level await). Vite 8 can
usually resolve the default relative WASM/worker URLs after optimizeDeps. For
older Vite, webpack, or CDN-hashed assets, inject explicit URLs:

```ts
import { createSignals } from "worth-signals-wasm";
import wasmUrl from "worth-signals-wasm/wasm?url";
import workerUrl from "worth-signals-wasm/worker?worker&url";

const signals = await createSignals({
  assets: { wasmUrl, workerUrl },
});
```

Missing `.wasm` / worker routes must return 404, not SPA `index.html`.

## Your First Signal

```ts
const quantity = signals.input(2, { debugName: "quantity" });
const unitPrice = signals.input(18, { debugName: "unitPrice" });
const total = signals.computed(() => quantity() * unitPrice(), {
  debugName: "total",
});

quantity.set(3);
console.log(total()); // 54
```

Inputs own writable runtime state. Computed values derive from the handles they
read. The runtime tracks that dependency relationship; you do not maintain a
second dependency list beside the callback.

`debugName` exists for people reading diagnostics. It is not stable identity
and must never become a lookup key.

## Coordinated Writes

```ts
await signals.transaction((tx) => {
  tx.set(quantity, 4);
  tx.set(unitPrice, 20);
});

console.log(total()); // 80
```

A Signals transaction coordinates one browser-runtime commit. It is not a
server database transaction and does not authenticate an actor.

## When The Feature Needs A Boundary

Publish a graph when inputs, outputs, and write authority become application
contract:

```ts
const pricing = signals.graph("pricing", (graph) => {
  const state = graph.scope("state");
  const quantity = state.input(2);
  const unitPrice = state.input(18);
  const total = state.computed(() => quantity() * unitPrice());

  return graph.expose({
    inputs: { quantity, unitPrice },
    outputs: { total },
  });
});

await pricing.writeInput("quantity", 4);
console.log(pricing.read().total); // 72
```

Local handles remain runtime-owned. The graph is the named public boundary.

## Choose The Right Product Surface

- Use **Core Signals** for local writable and derived state.
- Use **Resources** for request identity, loading, freshness, retries,
  optimistic effects, and server reconciliation.
- Use **Forms** for source values, drafts, validation, readiness, and actions.
- Use **Router** for route projection, admission, history, and recovery.
- Use **Local Truth** for process-local application branches, aspect-aware
  conflict review, and manual merge resolution.

Do not rebuild a resource cache or form lifecycle from ordinary inputs merely
because generic state feels familiar.

## Explicit Compatibility Recovery

Some hosts cannot construct a dedicated worker. Recover only from the specific
construction artifact:

```ts
import { createSignals } from "worth-signals-wasm";

let signals;

try {
  signals = await createSignals();
} catch (error) {
  if (
    !error ||
    typeof error !== "object" ||
    !("artifactFamily" in error) ||
    error.artifactFamily !== "workerUnavailableConstruction"
  ) {
    throw error;
  }

  signals = await createSignals({
    deployment: "mainThreadCompatibility",
  });
}
```

Do not assume `createSignals()` silently falls back. Compatibility is supported
for unsupported worker hosts, migration, and specialist lower-level work. It is
not the default architecture.

## React

The adapter supports React 18 and React 19. Your application owns the React
runtime and its type package; Worth does not install a second version.

```tsx
import {
  createReactSignalsStore,
  ReactSignalsStoreProvider,
  useSignalValue,
} from "worth-signals-wasm/react";
```

The adapter subscribes React to Worth Signals state. It does not copy that state
into a second React-owned store.

## Documentation

- [Start Here](./docs/start_here.md)
- [How Worth Signals Thinks About State](./docs/getting-started/mental-model.md)
- [Core Signals](./docs/core/README.md)
- [Resources](./docs/resources/index.md)
- [Forms](./docs/forms/index.md)
- [Router](./docs/router/index.md)
- [Local Truth](./docs/local-truth/README.md)
- [Support Status](./docs/reference/support-status.md)

The public documentation navigation is curated. Engineering crosswalks,
closeout matrices, and verification evidence remain in the repository without
being presented as the normal learning path.

## Package Development

Before publishing from this repository, run the package verification workflow:

```powershell
scripts/wasm/publish-worth-signals-wasm.ps1 -SkipPublish
```

The published package is ESM-first. Use `import` or dynamic `import(...)` from
CommonJS callers.
