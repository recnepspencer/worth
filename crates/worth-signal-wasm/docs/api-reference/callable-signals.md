# Callable Signals API

`CallableSignals` is the stable root facade returned by `await createSignals()`.
It owns one runtime and gives application code callable handles, published
graphs, domain facades, inspection, and lifecycle control.

## Root Namespaces

The root declaration and its public module augmentations expose:

| Property | Purpose |
| --- | --- |
| `host` | Declared browser host capabilities. |
| `spec` | Explicit structural signal authoring. |
| `resource` | Raw resource-family authoring and resource helpers. |
| `api` / `apiScope` | Route-first resource authoring. |
| `form` | Form declarations and source factories. |
| `router` | Route declaration, projection, admission, and navigation. |
| `localTruth` | Process-local application truth schemas and stores. |
| `local` | Higher-level process-local application surfaces. |
| `featureStore` | Published feature-store facade. |

The root facade is assembled from the package's exported declaration modules.
Import from `worth-signals-wasm` so those public augmentations are present.

## Signal Handles

All signal handles are callable and expose:

```ts
signal(): T
signal.get(): T
signal.value(): T
signal.id: string
signal.debugName: string | null
signal.free(): void
```

`InputSignalHandle<T>` additionally exposes:

```ts
set(value: T)
reset()
patch(partial)
assign(fields)
```

Each mutation returns `RunSummary | Promise<RunSummary>`. Await it in portable
code.

`LinkedSignalHandle<TValue, TSource>` adds `relink()`. Computed and output
handles are read-only.

## Authoring Methods

| Method | Result |
| --- | --- |
| `input(initial, options?)` | Writable input handle. |
| `linked(sourceOrDefinition, options?)` | Writable value with an explicit source baseline. |
| `computed(computeOrSpec, options?)` | Derived callable handle. |
| `output(computeOrSpec, options?)` | Published derived output handle. |
| `computedCallback(id, compute, options?)` | Explicitly named callback-computed handle. |
| `outputCallback(id, compute, options?)` | Explicitly named callback output. |
| `computedSpec(id, spec, options?)` | Explicit structural computed definition. |
| `outputSpec(id, spec, options?)` | Explicit structural output definition. |

Async variants exist for input, linked, computed, and output authoring. The
ordinary callback methods already work through the worker-first facade; async
variants are useful when a caller needs an always-Promise construction shape.

`debugName` is diagnostic metadata. Use a published graph when a name must be a
portable application contract.

## Coordinated Mutation

```ts
const summary = await signals.transaction((tx) => {
  tx.set(quantity, 4);
  tx.set(unitPrice, 20);
});
```

`transaction(...)` and `batch(...)` accept a synchronous callback and return
`RunSummary | Promise<RunSummary>`. `transactionAsync(...)` and
`batchAsync(...)` always return a promise, but their callback is still a
synchronous mutation description.

Transaction methods:

- `set(input, value)`;
- `patch(input, partial)`;
- `setWithAspects(input, value, aspects)`;
- `setWithRegions(input, value, changedRegions)`;
- `setWithRegionsAndAspects(input, value, changedRegions, aspects)`.

All handles in one transaction must belong to the same runtime. This boundary
coordinates signal writes; it is not a remote database transaction.

Host tip ingress (`commitHostTipAndNotify(tipWrites)`,
`applyCommittedTipWorkerBatch(tipWrites)`, `publishAuthoredTipProjection(ids)`,
`settleAuthoredWork()`) is present on the root and on scoped namespaces of
every deployment. Worker-first advances the tips, notifies observers once,
and follows with exactly one worker batch. `mainThreadCompatibility` applies
the same writes in one transaction and projects dependents synchronously,
so its `HostTipCommitResult.projectedReadableIds` is empty, the worker batch
only validates epochs (a stale batch throws), and `settleAuthoredWork()`
resolves immediately. Line bindings and forms use this one path on both
deployments instead of N independent `signal.set()` calls.

## Scopes And Published Graphs

```ts
const billing = signals.scope("billing");
const invoice = billing.graph("invoice", (graph) => {
  const state = graph.scope("state");
  const amount = state.input(0);
  const total = state.output(() => amount());
  return graph.expose({ inputs: { amount }, outputs: { total } });
});
```

The scoped namespace carries `scopeId`, local and parent identity, canonical ID
helpers, the same core authoring methods, and scoped resource, form, router,
local, and feature-store namespaces. `localTruth` is a root-only authority
factory.

Use `graph(...)` to publish named inputs and outputs. `importGraph(...)` exists
on the root for a definition and snapshot exported by another graph boundary.
Imported worker-first graphs expose their own readiness and termination
lifecycle.

## Read, Watch, And Effect

```ts
signals.read(target)
signals.watch(target, listener)
signals.effect(target, callback)
signals.nuke(disposable)
```

Signals are normally read by calling the handle. `read(...)` is useful for a
generic target. `watch(...)` and `effect(...)` return disposable handles;
`nuke(...)` removes a disposable through the owning runtime.

Prefer framework adapters such as `worth-signals-wasm/react` at UI boundaries
instead of creating ad hoc watchers per component.

## Diagnostics And History

`signals.diagnostics()` exposes runtime diagnostics plus
`hostCapabilityReport()`.

`signals.history()` exposes:

- per-signal replay and lineage summaries;
- runtime and branch snapshots;
- exact same-runtime restore artifacts;
- branch creation, switching, planning, and merge evidence;
- branch-state, replay-parity, and replay-artifact proof reports;
- branch-native transaction and retirement commands used by resource effects;
- subscription and cleanup.

Runtime history describes the active process-local execution engine. It is not
durable application history or a shared collaboration branch.

## Runtime Contract

```ts
signals.contract(): SignalsRuntimeContract
signals.assertCompatibility(options): SignalsRuntimeContract
```

The contract reports surface family, version, deployment, scope, and declared
capabilities. The assertion throws with the exact missing capability names.

## Specialist And Compatibility Doors

| Method | Status | Purpose |
| --- | --- | --- |
| `specialist()` | Compatibility-only | Dirty evaluation, graph summaries, and version reads. |
| `adapters()` | Mixed specialist boundary | Runtime envelope export/replace/restore (`exportRuntimeEnvelope()` is root-branch only; release an abandoned export with `discardExactRuntimeEnvelope(envelope)`) and transport proof reports. |
| `compatibilityApp()` | Compatibility-only | Lower-level `SignalApp`. |
| `compatibilityRuntime()` | Compatibility-only | Lower-level `SignalRuntime`. |

These methods exist on the facade so migration and platform tooling can share
the runtime. Do not make them the default application architecture.

## Lifecycle

```ts
signals.terminate(): void | Promise<void>
signals.free(): void
signals[Symbol.dispose](): void
```

The construction owner releases the root. Individual retained handles,
histories, diagnostics views, adapters, and subscriptions should be released at
the boundary that created them. Do not continue reading or mutating after the
owning runtime is terminated.

## Common Denials

- a handle belongs to another runtime;
- a requested compatibility capability is absent;
- a graph import or exact restore artifact belongs to another runtime or basis;
- a worker callback attempts an undeclared ambient host read;
- an operation needs retained history or identity proof that is unavailable.

Use the typed result or structured error. Do not retry against a different
runtime unless the application explicitly performs that handoff.

## Related Reference

- [Construction API](./construction.md)
- [Core Signals Overview](../core/README.md)
- [Typed Results, Denials, And Unavailability](../reference/typed-results-and-unavailability.md)
- [Lower-Level Compatibility Surface](./compatibility-surface.md)
