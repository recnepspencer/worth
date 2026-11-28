# Support Status

This is the public support matrix for `worth-signals-wasm`. It answers a
different question from the type declarations: not "can TypeScript name this
shape?", but "what does the published package actually promise to perform?"

## Status Vocabulary

| Status | Meaning |
| --- | --- |
| Stable | Shipped through the public facade with executable evidence. |
| Mixed | Shipped, but a deployment or host boundary changes part of the behavior. |
| Deferred | Public vocabulary exists for a future or conditionally admitted lane; do not build an ordinary path that requires it. |
| Unsupported | The package deliberately makes no such promise. |
| Compatibility-only | Supported for migration or specialist use, but not the recommended application surface. |

`unavailable` is a runtime outcome, not a product status. A stable operation may
return an unavailable result when the runtime lacks the authority or retained
evidence needed to perform it honestly.

## Package Entrypoints

| Import | Status | Use |
| --- | --- | --- |
| `worth-signals-wasm` named exports | Stable | `createSignals`, callable signals, resources, forms, router, Local Truth, and shared public types. Published as a bundled ESM entry (not a deep `product/**` forest). |
| `worth-signals-wasm/react` | Stable | React subscriptions and form bindings over Worth-owned state (bundled adapter entry). |
| `worth-signals-wasm/wasm` | Stable | Package WASM asset for bundler `?url` imports. |
| `worth-signals-wasm/worker` | Stable | Worker-first runtime worker entry for bundler `?worker&url` imports (colocated bridge/worker shells). |
| `worth-signals-wasm` default export | Compatibility-only | Initialize the lower-level Wasm module. It is not `createSignals()`. |
| `worth-signals-wasm/raw` | Compatibility-only | Raw runtime and structural authoring for migration or specialist tooling. |
| `worth-signals-wasm/raw_surface.js` | Compatibility-only | Alias of the published raw entrypoint. |

## Construction And Deployment

| Surface | Status | Important boundary |
| --- | --- | --- |
| `await createSignals()` | Stable | Selects worker-first and never silently falls back to the main thread. |
| `createSignals({ assets: { wasmUrl, workerUrl } })` | Stable portable bundler path | Injects bundler-emitted asset URLs. Required on hosts that relocate package modules away from the `.wasm` / worker files (for example Vite 7 optimizeDeps). Optional on Vite 8+ when defaults work. |
| `createSignals({ deployment: "mainThreadCompatibility" })` | Stable deployment choice | Explicitly constructs the callable facade on the main thread. The deployment is compatibility posture even though construction is supported. |
| `createCallableSignals()` | Compatibility-only | Always selects `mainThreadCompatibility`, including when its options request another deployment. |
| `wrapSignals(rawSignals)` | Compatibility-only | Wraps an already-created raw runtime; it does not create worker ownership. |
| `planCreateSignalsDeployment()` / `explainCreateSignalsConstruction()` | Stable | Inspects selection and recovery without constructing a runtime. |
| `signals.contract()` / `signals.assertCompatibility(...)` | Stable | Inspects or enforces the active facade's declared capabilities. |

If the environment has no `Worker` constructor, default construction rejects
with `artifactFamily: "workerUnavailableConstruction"`. The error includes an
explicit `mainThreadCompatibility` recovery, but the package does not take that
recovery on your behalf.

## Host And Bundler Asset Loading

Measured against a packed npm tarball install (not a workspace `file:` symlink).
Missing `.wasm` or worker routes must return **404**, never SPA `index.html`
(`3c 21 64 6f` / `<!do`). HTML bodies are rejected with a package diagnostic.

| Host / bundler | Status | Recipe |
| --- | --- | --- |
| Node ESM / no bundler rewrite | Stable | Default relative URLs beside the package files. No `assets` required. |
| Vite 8+ with `worker.format: "es"` | Stable zero-config | Default relative URLs after optimizeDeps when package files remain fetchable. |
| Vite 7 with forced optimizeDeps | Broken without assets | Default relative URLs resolve beside `.vite/deps` and often receive SPA HTML. |
| Vite 7 + `createSignals({ assets })` | Stable portable path | `import wasmUrl from "worth-signals-wasm/wasm?url"` and `worker?worker&url`. |
| Vite / webpack / CDN-hashed assets | Stable portable path | Always prefer explicit `assets` when the bundler emits hashed or relocated URLs. |
| `optimizeDeps.exclude: ["worth-signals-wasm"]` | Compatibility-only workaround | Legacy escape hatch for older Vite; not the supported long-term recipe. |

Vite consumers must set `worker: { format: "es" }` because the worker entry uses
top-level await.

## Product Surfaces

| Surface | Status | Current promise |
| --- | --- | --- |
| Inputs, computed values, and outputs | Stable | Runtime-owned local state and captured-read derivation. |
| Transactions and batches | Stable | One runtime mutation boundary; not a database transaction. |
| Linked writable state | Stable | Writable derived state with explicit relinking. |
| Graphs, scopes, and controllers | Stable | Named publication boundaries over runtime-owned handles. |
| Aspects and aspect-aware writes | Stable | Numeric invalidation lanes; the active native profile supports 32 aspects. |
| Diagnostics | Mixed | Runtime evidence is stable; worker-first host-event replay is not promised. |
| Runtime history, snapshots, and branches | Stable | Exact process-local execution history retained by the active runtime. |
| Resources | Stable | Browser-local request, lifecycle, optimistic effect, reconciliation, and history state. |
| Forms | Stable | Source, draft, validation, readiness, and action lifecycle remain distinct. |
| Router | Stable | Admitted visible route state remains distinct from raw browser location. |
| TypeScript Local Truth | Stable | Process-local application values, commits, branches, and manual resolution. |
| React adapter | Stable | Subscription and form bindings without a second React-owned store. |
| Browser host capabilities | Mixed | Declared capabilities are stable; ambient worker reads and unlimited event replay are not. |

## Stable Typed Failure And Unavailability

These are part of the supported contract, not accidental error cases:

- construction can reject with a structured construction artifact;
- runtime compatibility assertions throw with required and missing capability
  names;
- resource settlement, delivery, replay, restore, rollback,
  download, and transfer surfaces use discriminated outcomes;
- form readiness, admission, action, reset, and replay/restore preserve blockers and unavailable outcomes;
- router projection, admission, recovery, navigation, replay, and restore expose
  typed non-admitted or unavailable states;
- Local Truth mutations return explicit outcome unions.

Do not replace these outcomes with booleans or parse their human-readable
messages. Narrow on the documented discriminator and retain the supplied
reason or evidence for diagnostics.

## Deferred Vocabulary

- `SignalsConstructionArtifact` includes the
  `"signalsConstructionDenied"` family. The current published worker-first host
  capability plan admits the shipped capability families; ordinary code should
  not invent a denied-construction trigger.
- Some domain declaration types describe capabilities that are admitted only
  when their required identity, shape, history, host fact, or route authority
  is present. The declaration is stable; a successful execution is conditional.

Deferred does not mean "probably works." It means the public vocabulary is
retained without promising an ordinary executable path today.

## Process-Local By Design

- signal values and runtime execution history;
- runtime snapshots and branches;
- TypeScript Local Truth values and commits;
- browser host persistence.

These surfaces can be exact and inspectable without being durable or shared.
Use your platform truth layer for durable multi-process authority.

## Unsupported

- silent worker-to-main-thread fallback;
- durable database transactions from `signals.transaction(...)`;
- authenticated identity from host-supplied actor metadata;
- cross-process Local Truth collaboration;
- addressability or durable identity derived from `debugName`;
- unlimited diagnostics or history retention;
- synchronous access to worker-first operations merely because compatibility
  deployment can complete the equivalent operation synchronously.

## Compatibility-Only Surface

- `createCallableSignals(...)`;
- `wrapSignals(...)`;
- `signals.compatibilityApp()` and `signals.compatibilityRuntime()`;
- the package default Wasm initializer;
- the `./raw` and `./raw_surface.js` entrypoints;
- explicit structural `signals.spec` authoring when names are the portable
  contract;
- specialist keyed, packed, and lower-level runtime operations.

Compatibility-only means supported with more caller responsibility. It does not
mean deprecated, and it does not mean "more real" than the callable facade.

## Related Reference

- [Reference Overview](./README.md)
- [Package Entrypoints And Runtime Contracts](./package-entrypoints-and-contracts.md)
- [Typed Results, Denials, And Unavailability](./typed-results-and-unavailability.md)
- [Construction API](../api-reference/construction.md)
- [Callable Signals API](../api-reference/callable-signals.md)
- [Lower-Level Compatibility Surface](../api-reference/compatibility-surface.md)
