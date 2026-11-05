# Program S1: Agent-Grade Signals Runtime

Governing roadmap: [wasm_product_roadmap.md](wasm_product_roadmap.md).
Amends: [worker_runtime_placement_plan.md](worker_runtime_placement_plan.md)
§10.10 and §18 (D19 below).

`worth-signals-wasm` becomes the state runtime that coding agents get right
on the first attempt and can inspect when it misbehaves. Ordinary operations
cost what their semantic delta costs, never what the whole form, store, graph
or history costs, so no representative page waits seconds. The package builds
with the native TypeScript 7 compiler. No npm release ships until this program
closes. The closing release is 2.0.0, because D10–D12 are breaking cutovers.

## What already landed

| Commit | Change |
|---|---|
| `bac73209d1` | Release chain 1.6.0–1.6.3 merged into master |
| `1332187656` | Guided bulk reads skip current targets without hiding dirty ancestors |
| `4e2dbdfe4e` | 1.6.4 published from master |
| `9e43ea42b2` | Single-pass `SignalValue` decoder: a 100-row value went from 13,555 Errors to 0, with a real-artifact regression |
| `3c08acaa20` | The decoder reads serde_json `arbitrary_precision` number tokens as finite numbers, so its answer no longer depends on feature unification |

Two pieces are in flight in the original session, `local_2ebb73b1`:
- The TypeScript 7 migration (`b0804b0d33`).
- Single-read currentness (single reads routed through settlement).

Both land before any S1 phase touches the same files.

## Adversarial constraint

The NCR reference workload is an item page carrying three things at once:
- **Form:** 17 sections, 92 schema fields and 3 repeatable sections, today lowered to 1,143 native form fields.
- **Pinned configuration:** a 93 KB JSON config.
- **Resource:** a 1,000-row collection.

Nine conditions apply to it simultaneously:
- worker-first deployment;
- React 18 and 19, in production and in Strict Mode;
- a poll every 5 s with unchanged data;
- focus, online and reconnect storms that restore every active line;
- typing at 60 keystrokes per second into any field;
- autosave racing typing;
- route switches while requests are pending;
- 10,000 unrelated nodes in the same runtime;
- unbounded session length.

Under all of these, every ordinary operation must meet the D9 contract:
- Its work is bounded by its semantic delta plus the stated granule.
- The edited field, its declared dependents and the changed rows are the delta.
- Work never grows with total fields, total rows, unrelated graph size or history length.

Every surface that observes the value must also hold four properties:
- It never observes a stale, torn or regressed value.
- It never loses a committed write.
- It never adopts a result computed from an older basis.
- When it is wrong, it fails with a typed refusal that names the cause.

The naive implementation breaks this today in seven places. All were confirmed by reading the code on 2026-11-04.

| # | Amplification | Growth per ordinary operation |
|---|---|---|
| A1 | Per keystroke, whole-form validation runs 3–5 times (admission, history snapshot, summary). Each run clones the form per field. | O(fields²) |
| A2 | Every mutation reads back every authored readable. The worker also SHA-hashes them, digests branch state, serializes them to measure size, and the host clones and freezes them. | O(readables × value + state) |
| A3 | Every transaction clones the whole store twice (rollback copy plus snapshot). | O(store) |
| A4 | An equal-value `set` bumps the version and invalidates; dependents re-run their callbacks. | O(dependents × value) per no-op |
| A5 | An unchanged resource reload commits a new value and `canonicalRevision`, makes about ten whole-value copies, publishes twice and force-notifies. | O(rows), and the full A2/A3 cost, every poll |
| A6 | A single read builds one plan per ancestor, each sized to the whole arena. | O(ancestors × graph capacity) |
| A7 | The form summary carries source, draft, effective and patch (about 4× the form) back into Rust on every edit. | O(form) per edit |

## Decisions

**D1. The value domain is JSON, enforced at every boundary, with typed refusal.**

- **The domain:** `null`, booleans, finite numbers, strings, arrays, and plain objects with string keys.
- **One rule for `undefined`:** an object member whose value is `undefined` is absent. That is JavaScript's own object semantics, and it is how TypeScript's `{ a?: T }` already reads such a member.
- **`-0`** is a finite number, equal to `0` under the D2 comparator, and canonicalizes to `0`, as JSON serialization does.
- **Refused:** everything else. That covers top-level or element `undefined`, `NaN`, `±Infinity`, `BigInt`, `Date` and any other `toJSON` holder, `Map`, `Set`, typed arrays, functions, symbols, cycles and non-plain prototypes.
  - Each refusal is a `WorthError` with `code: "valueOutsideJsonDomain"`, a JSON-pointer `path` and the offending `kind`.
  - Nothing commits from a refused write.
- **Where it applies:** inputs, transactions, resource loader results and mutation responses, form values, host-capability payloads and imported graphs, through one validator in `package-src/product/values/json_domain.ts`. The Rust decoder rejects the same set.
- **Types:** the public `SignalValue` type becomes `JsonValue`, so `Date`, `bigint`, functions and `Map` fail to compile.
- **Cutover:** this replaces DECISIONS.md #1, where resource values were canonicalized through `toJSON`. Under clean cutover the coercion is removed, not kept. Hearth's `Date` mutation responses must serialize explicitly; the package docs and the error hint say how.

**D2. Equality cuts off work at the authority boundary.**

- **Comparator:** two values are equal when their canonical JSON is equal. Objects compare after D1 normalization with sorted keys, which is already the order of `SignalValue::Object`. Numbers compare by IEEE equality within the D1 domain.
- **Input `set` / `transaction`:** writing a value equal to the committed one bumps no version, invalidates nothing, runs no dependent, and returns a receipt with `outcome: "unchanged"`.
- **Resource reload:** a canonically equal result keeps the previous value object, its value version and `canonicalRevision`. It still advances `freshness`, `status`, `fetchedAt` and diagnostics.
  - These are separate fields of line state, so a metadata-only change, such as a new server version stamp, an `updatedAt` or an authorization posture, is published honestly without republishing the value.
  - The pending and fulfilled publishes of one reload coalesce into one worker round trip when no observer can see the pending state, which is the case when the value is already visible.
- **Outputs:** the existing comparison of computed outputs (`evaluation.rs` `Unchanged`) stays as the second line of defence.
- **Observation:** notification fires only for a changed value or changed metadata the subscriber selected. Force-notify is removed.

**D3. Acknowledgements carry deltas; certification work leaves the ordinary path.**

- **Delta acknowledgement:** a worker acknowledgement carries only the outputs whose value changed in that transaction. It carries their id, value and version, plus the committed revision.
  - The host projection applies those deltas.
  - "Refresh every authored readable" (DECISIONS.md #13) is deleted.
- **Diagnostic lane:** truth digests, SHA-256 readback hashes, `serde_json::to_vec` byte counts and imported-graph definition, snapshot and replay export move to that lane (perf laws, "path conflation"). They are computed only when a caller asks for evidence. The committed-truth digest becomes incremental per changed source when it is requested.
- **Imported graphs:** an ordinary write applies deltas. Reconstruction happens only on import, restore and explicit inspection.

**D4. Transactions roll back through an undo log, not a store clone.**

- `apply.rs` stops cloning the store for rollback and stops snapshotting branch state after every mutation.
- A transaction records the prior value and version of each source it touches. Rollback replays that log in reverse.
- Branch snapshots and restore tokens are minted lazily, when history capture is requested.
- This removes the 64-token eviction trap of DECISIONS.md #5: after this change, holding a snapshot is never revoked by later writes.
- Growth: O(touched sources) per transaction.

**D5. Reads plan once, sized to their affected closure.**

- Single and bulk reads settle through one plan, built by the currentness planning that Phase 0 lands: guided reads plan by currentness, never force. S1 builds on it, not beside it.
- Planner scratch (`unsettled_paths`, `stage_formation`) uses epoch-stamped sparse sets reused across requests, not fresh arena-sized vectors.
- Growth: O(affected closure) per read, and O(1) for a clean read.

**D6. Forms cost per edit what the edit touches.**

1. **One memoized snapshot** per (draft revision, source version) in `form_controller.ts`. Write admission, history and summary share it.
2. **Validation is indexed by dependency:**
   - A rule declares the fields it reads, through the existing `dependency_graph.ts`.
   - An edit re-validates only rules that read the edited fields.
   - The whole-form report is assembled from cached per-rule results.
3. **Write admission evaluates only declarations covering the edited path.**
4. **Interaction events don't bump the value revision:** `focus`, `blur`, `touch` and `visit` update interaction state only.
5. **The summary carries revisions and patch deltas, not whole values,** and crosses into Rust once per edit. The full value is read on demand.
6. **`setFields` admits and snapshots once per batch** (perf laws, "boundary cardinality").
7. **Histories are bounded rings** with a declared capacity. Reading the interaction report is O(capacity).
8. **Repeatable sections are `repeated` collections with live instances.**
   - A declared storage lens lowers an instance field to its storage key. For Pulse that is a flat suffixed key: `name`, `name_2` … `name_20`.
   - Declared native fields scale with live instances, not with the maximum.
   - The lens is part of the form declaration and is checked against the schema at construction.

**D7. One conversion per boundary crossing.**

- A value crosses JS→WASM once (`from_js`) and WASM→JS once (`to_js` or a postMessage structured clone, never both).
- `materializeWorkerCachedValue` runs once per value, not separately for `value` and `canonicalValue`.
- Resource values are not re-canonicalized after D1 validation, because validation and canonicalization are one pass.
- Clean reads return the cached frozen projection without crossing the boundary.
- `graph.read()` returns a cached output record whose identity changes only when an output changes.

**D8. Observation lookups are indexed.**

- Observer routing uses maps from event to subscribers, not linear scans.
- Delivery cost is O(changed outputs + matched subscribers).

**D9. The latency guarantee is enforced mechanically on three layers.**

1. **Growth-class gates (CI, every signals change).**
   - These run against the built package in Node and use structural observations the harness already sees:
     - recipe executions from the existing executor usage summary;
     - worker messages and bytes posted, observed by wrapping `Worker.prototype.postMessage` in the test;
     - Error constructions;
     - React commits counted by the test component.
   - **Sweeps.** Each gate sweeps one independent axis while holding the others:

     | Axis | Values |
     |---|---|
     | Form fields | 100, 1,143, 4,000 |
     | Resource rows | 10, 1,000, 10,000 |
     | Unrelated nodes | 0, 10,000 |
     | History length | 10, 10,000 |
   - **Assertion.** It checks the growth class with headroom, never an exact count (MENTALITY §7). Raising an unrelated axis 10× may raise per-operation work at most 1.5×.
   - **Equalities.** Some properties are asserted as equalities, not ranges:
     - a canonically equal write runs zero dependents;
     - valid JSON constructs zero Errors;
     - an unchanged poll commits zero React renders of value consumers.
2. **Order-of-magnitude ceilings (CI, headless Chromium, production build).**
   - Measured on the NCR reference fixture through the packed package:

     | Measurement | CI ceiling |
     |---|---|
     | Warm open to interactive form | 3 s |
     | Create to interactive | 6 s |
     | Keystroke to painted value | 150 ms |
     | Longest main-thread task caused by an unchanged poll | 100 ms |
   - "Interactive" means a typed character appears in the field through its real handler. A header or spinner is not readiness.
   - The ceilings are 3× the D9.3 targets, so they catch an order-of-magnitude regression on shared CI hardware without flaking.
3. **Reference targets (scheduled lane, named reference machine).**
   - **Targets (p95):**

     | Measurement | Target |
     |---|---|
     | Warm open | ≤ 1 s |
     | Create | ≤ 2 s |
     | Input to visible | ≤ 50 ms |
     | Refresh-induced main-thread tasks | none > 100 ms |
   - **Run discipline:** at least 30 samples per point, with cold and warm postures recorded separately, and median, p95 and worst case reported.
   - **Regression rule:** a regression of more than 25% against the committed reference blocks the next release.

Latency is never bought by weakening D2's freshness honesty, authorization, pending state, recovery, atomicity or provenance. Each such invariant travels with the gate that measures it.

**D10. One typed error model.**

- **Base:** every refusal the package throws or returns is a `WorthError` (`code`, `message`, `hint`, `path?`, `docs`).
  - `code` is a closed string-literal union exported as `WorthErrorCode`.
  - `isWorthError(value, code?)` narrows it.
- **One discriminator:**
  - `artifactFamily` remains on artifacts only.
  - Result unions use `kind` everywhere.
  - `posture`/`resultKind` discriminators are renamed to `kind`.
- **No bare throws:** no `TypeError` or `Error` is thrown from `package-src` for a caller mistake.
- **Enforcement:** a lint in `scripts/wasm` refuses `throw new (Type)?Error(` under `package-src/product`. Each refusal site moves to a typed constructor in `package-src/product/errors/`.

**D11. Deployment decides the type, not a union.**

- `createSignals()` resolves to `WorkerSignals`; every write returns `Promise<WriteReceipt>`.
- `createSignals({ deployment: "mainThreadCompatibility" })` resolves to `CompatSignals`; writes return `WriteReceipt` synchronously.
- Both share the read surface. No public signature returns `T | Promise<T>`.
- `createCallableSignals` is deleted: it silently forced compatibility.

**D12. One way to do each thing (clean cutover).**

- **Authoring:**
  - `input`, `computed` and `output` take a callback.
  - Declarative and explicit-id forms live under `signals.spec`.
  - `computedAsync`, `computedSpec`, `computedCallback` and their `output*` twins are deleted.
- **Writes:** `transaction` only. `batch` and the `*Async` variants are deleted.
- **Disposal:** `dispose()` and `[Symbol.dispose]` only. `free`, `terminate` and `nuke` are deleted from public types.
- **Public types:**
  - Host-tip plumbing (`commitHostTipAndNotify`, `applyCommittedTipWorkerBatch`, `authoredSettleInvocationCount`) and specialist doors leave the public root type, behind the `/internal` export used by the adapters.
  - One casing throughout: camelCase. History's snake_case methods are renamed.
- **Aspects** are declared and bound by string id.
- **Enforcement:** a public-surface snapshot test in `package/product/documentation/` fails on any unreviewed public addition. Lowering the snapshot is the cutover record.

**D13. JSON-safe introspection and a devtools bridge.**

- **`signals.inspect()`** returns JSON-safe data:
  - `graph()` lists `{ id, debugName, kind, value, version, dependencies, subscribers, state }` for every node.
  - `why(handle)` returns the existing `WhySummary`.
  - `whyEffect(handle)` explains why an effect fired.
  - `history()` and `branches()` return those records.
  - Branch ids are strings, durations are numbers of milliseconds, and nothing is a `bigint`.
- **Dev bridge:** `worth-signals-wasm/devtools` exposes the same data over a dev-only `BroadcastChannel` + WebSocket bridge. It can also fork a local-truth branch, apply trial writes, diff the result and discard the branch.
  - It is never present in a production build: the export condition is `development`, and the production bundle is verified not to contain it.
- **MCP server:** `worth-signals-mcp` (new package, `apps/worth-signals-mcp/`) attaches an agent to that bridge.

**D14. Agents learn the package from the package.**

- **Shipped guidance:** the package ships `llms.txt` (the happy path, when to await, a do/don't list, error codes with fixes) and `AGENTS.md`.
- **Doc snippets:** every TypeScript snippet in `docs/` is extracted and type-checked in CI with TypeScript 7. A snippet that doesn't compile fails the lane.
- **Glossary:** a package-wide glossary replaces jargon in the first two docs layers.

**D15. The implementation is typed and generates the public declarations.**

- **Converted:** `package-src` (about 79,000 lines of untyped JavaScript in `.ts` files) becomes strict TypeScript, checked by TypeScript 7 (`noImplicitAny`, `strict`, `exactOptionalPropertyTypes`).
- **Generated:** public `.d.ts` files are generated from the implementation's exported types, and the hand-written `package/types` tree is deleted.
- **Drift check:** a declaration-drift check compares the generated public surface with the D12 snapshot.
- **Consumer compatibility:** the published declarations are verified against TypeScript 5.x and 7.x consumers.

**D16. A deterministic test runtime for consumers.**

- `worth-signals-wasm/testing` exports `createTestSignals({ clock, resources })`. Its parts:
  - a manual clock;
  - scripted resource transports that resolve on command;
  - `settle()`, which drains all authored work deterministically.
- It uses the main-thread deployment and the same runtime, so it is not a fake engine.

**D17. Correctness is certified by generated adversity, not by examples.**

1. **Deployment differential.**
   - A seeded generator of operation sequences covers inputs, computed values, keyed families, transactions, resources (fulfil, reject, refresh, equal refresh), form edits, subscribe/unsubscribe, branch fork/merge and dispose.
   - Each sequence runs against main-thread, worker-first and a naive reference evaluator that recomputes everything.
   - All observable traces must match.
   - Fixed seeds run in CI; a seed sweep runs in the scheduled lane, and a failing seed becomes a fixed regression.
2. **Schedule adversity.** A seeded bridge shim delays, reorders (where the protocol permits) and holds worker replies. No observer may see a stale, torn or regressed value.
3. **Worker loss.**
   - A worker crash or termination makes every pending and later call reject with `WorthError` `workerLost`, and subscribers receive a terminal notice.
   - The host never silently re-creates a worker, because authority would be lost.
   - `signals.recover(snapshot)` rebuilds from an exported snapshot when the app chose to keep one.
4. **Core properties.** Proptest in `crates/worth-signal` generates graphs, edits, dynamic rewires and on-demand gating, and compares them against a naive evaluator. It covers glitch-freedom, evaluate-once, equality cut-off and currentness.
5. **Fuzzing.** cargo-fuzz targets cover the portable wire format, JSON-wire values, snapshot restore and recipe specs.
6. **Seeds.** Adversarial workflow seeds sweep in the scheduled lane, and the inert `--long` flag is fixed.
7. **Retention soak.** Ten thousand subscribe/write/dispose cycles must leave handle tables, worker registries and the JS heap flat (growth-class assertion).

**D18. CI gates signals, and releases are reproducible.**

- **CI lane.**
  - A `signals` job in `ci.yml` is selected by path and by cargo reach over the dependency closure of `worth-signal`, `worth-signal-wasm` and `worth-court-signal-*`. It runs:
    - native nextest (default and integration profiles);
    - compile-fail courts with their prebuild;
    - the WASM release build;
    - the six runtime suites;
    - docs and type smoke;
    - the packed consumer checks;
    - the demo;
    - the headless React matrix (2 React versions × prod/StrictMode, both deployments, driven by Playwright and asserting the posted `pass`);
    - the D9.1–D9.2 gates.
- **Release orchestrator.**
  - One Node script, `scripts/wasm/release-worth-signals-wasm.mjs`, replaces both the `.ps1` and the duplicated workflow steps.
  - It pins `wasm-pack` and the Binaryen 132 release tarball. It invokes `wasm-opt` directly on every OS and resolves `npm` through `npm_execpath`.
  - It refuses a version that already exists on the registry.
  - It keeps the verified `.tgz`, publishes that exact file, and compares registry integrity afterwards.
- **Publishing.** The workflow uses npm trusted publishing: `id-token: write` and `npm publish --provenance`, with no token secret. The repository default is corrected from `forge` to `worth`.

**D19. Partitioned execution with a bounded compute pool (amends the placement plan).**

- **One authority stays.** The runtime keeps one authoritative graph owner per runtime.
- **Amendment.** §10.10's "no helper workers" is amended. A bounded, reusable compute-worker pool under `worth-execution` runs admitted portable kernels and returns candidates; only the owner commits.
- **Partition-local recompute and storage.**
  - A partitioned producer recomputes and stores per partition.
  - A partition's value is addressable without materializing the whole collection.
  - Resource collections declare item identity (`key`), and keyed lines lower onto partitions.
- **Collection kernel IR.** `Expr` gains bound variables and `map`, `filter`, `reduce`, `groupBy` and `sortBy` over keyed collections. Results are incremental per key. Kernels are serializable and pure; arbitrary closures keep their host placement.
- **Async execution seam.**
  - `worth-execution` gains a message-passing backend keyed by kernel id plus payload.
  - Settlement is asynchronous.
  - Admission checks partition identity, input basis versions and assignment generation, mirroring `admit_resource_completion`.
  - Wasm transactions wire a real `CancellationToken`.
- **Placement.**
  - Automatic: serial in the authority worker unless measured kernel cost exceeds transfer cost.
  - Declarative: `signals.partitioned({ source, key, partitionBy?, kernel, placement })`.
- **Acceptance.** The extreme-scale fixture has 1,000,000 records across 10,000 partitions, with a virtualized 200-row view. The gates:
  - **Locality:** a keyed edit's work and transfer do not grow from 10k to 1M records.
  - **Sparse edits:** ≥10× lower warm sparse-edit latency than whole-collection recompute.
  - **Throughput:** ≥2× with four workers against partitioned serial on the CPU-heavy kernel.
  - **Small workloads:** auto-placement stays within 10% or 0.5 ms p95 of serial.
  - **Soak:** a 30-minute soak with no retained growth.
  - **Correctness:** stale results and partial publication are release blockers.

**D20. The bundle is attributed, split and budgeted.**

- **Attribution.** A `twiggy` attribution report is produced by the release build and kept as a CI artifact. JS attribution uses esbuild's metafile.
- **Splitting.**
  - JS code splitting is enabled, which requires fixing the package-root-relative externals that forced `splitting:false`.
  - Resources, forms, router, local truth and devtools become lazy entries.
  - Rust subsystems not needed by core signals (branch merge, resources, forms, router) are separated into lazily instantiated modules, or gated by entry-specific builds, whichever the attribution shows is smaller.
- **Budgets.** `wasm_size_policy.mjs` sets a budget per entry, fixed from the attribution at phase close. Core signals must not pay for forms or resources.

**D21. TypeScript 7 everywhere.**

- **Toolchain:** the native TypeScript 7 compiler is the only TypeScript toolchain for builds, emit, type smoke, doc snippets (D14) and the D15 implementation check.
- **Consumers:** compatibility with TypeScript 5.x and 7.x is verified.

**D22. Pulse removes its own amplification (Pulse repository).**

- **Repeatable sections:** they become D6.8 `repeated` collections with a flat-key storage lens. Declared native fields drop from 1,143 toward 2 × live fields.
- **Projections:** the 13 overlapping projections plus 3 aggregates in `workspaceItemDetail.resourceViews.ts`, `authoritativeProof` and `createItemDetailPageModel` collapse to one projection per consumer concern. Each reads only its keys, and none copies the 93 KB configuration.
- **Restore:** restore on focus or online does not refresh pinned lines (`configVersion`, `workflowVersion`), which are immutable by identity.
- **Fixture:** the NCR fixture is generated from `testdata/qms/lookup-placement/v2/ncr_v400.json` `loaded_sections`, reconciled to the released 17/92/3 shape. It is vendored into WORTH as the D9 reference fixture (`crates/worth-signal-wasm/package/product/perf_fixtures/ncr_reference.json`).
  - The generator keeps only shape and rules: field types, counts, nesting, repeat flags, dependencies, value widths and option counts.
  - Every label, help text, option text and identifier becomes a generated placeholder of equal length, so no NCR content enters WORTH and no manual privacy review is needed.
- **E2E:** Pulse's journey adds the D9.2 create-to-interactive and keystroke checks on its isolated world. The 60 s functional timeouts stay functional only.

## Phases

| Phase | Name | Depends on | Owner |
|---|---|---|---|
| 0 | TypeScript 7 + single-read currentness | — | original session (in flight) |
| 1 | Gates: CI lane, release orchestrator, D9 harness and fixture | 0 | S1 |
| 2 | Ordinary-path cost: D1–D8 | 1 | S1 |
| 3 | Correctness certification: D17 | 2 | S1 |
| 4 | Agent-grade surface: D10–D16 | 2 | S1 |
| 5 | Pulse integration: D22 | 2 (consumes a packed tarball) | S1, Pulse repo |
| 6 | Partitioned execution: D19 | 2, 3 | S1 |
| 7 | Bundle: D20 | 4 | S1 |
| 8 | Close: 2.0.0 release | 1–7 | S1 |

**Landings on master:**
1. Phase 1 lands alone. The gates come first, so that Phase 2 is measured by them. Its growth gates for A1–A7 land as expected-red cases listed in `perf_fixtures/expected_red.json`, which Phase 2 must empty.
2. Phases 2 + 3.
3. Phases 4 + 7, one public cutover.
4. Phase 6.
5. Phase 5 lands in Pulse against the 2.0.0 package.
6. Phase 8 releases.

No phase lands with a non-empty `expected_red.json` except Phase 1.

### Phase 1 exit
- The `signals` CI job is green on master.
- The release orchestrator produces a verified tarball on Linux and Windows. It has not published.
- The D9.1 and D9.2 gates run, and their red set equals A1–A7.
- The NCR reference fixture is vendored, generated shape-only.

### Phase 2 exit
- `expected_red.json` is empty.
- The D9.2 ceilings are green.
- The D9.3 reference run is recorded at or under target.
- The decoder, the ack and React matrix, and the 1,383 runtime tests stay green.

### Phase 3 exit
- The differential, schedule-adversity, worker-loss, proptest, fuzz (60 minutes clean per target) and soak suites are green, with fixed seeds in CI.
- Every seed failure found was fixed in product code.

### Phase 4 exit
- The public-surface snapshot equals the D12 surface.
- No bare throws remain.
- Doc snippets compile under TypeScript 7.
- The generated declarations replace `package/types`.
- `llms.txt` ships.
- `inspect()`, the devtools bridge and the MCP server pass their end-to-end journey: an agent attaches, explains a value, trials a write on a branch and discards it.

### Phase 6 exit
- The D19 acceptance gates are green on the reference machine.
- Auto-placement is on by default only after that.

### Phase 7 exit
- Per-entry budgets are set from the attribution and enforced.
- Core signals do not load the forms or resources code.

### Phase 8
- Version 2.0.0 is published through D18 trusted publishing, with registry integrity verified.
- The migration guide covers every D10–D12 cutover.

## Verification tiers
- **Tier 1, per slice:** `cargo check` on touched crates, plus the tests the slice adds or changes, selected by name.
- **Tier 2, per landing:** the lanes the diff reaches (`orchestration/codex/affected_lanes.py`), plus the `signals` job and the D9.1–D9.2 gates.
- **Tier 3, at program close:** full lanes plus the D9.3 reference run and the D19 extreme-scale run. Ignored tests are never run.
