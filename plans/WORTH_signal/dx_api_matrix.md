# WORTH Signal DX API Matrix

## Intent

This matrix is for **public API hardening**, not for documenting every internal
subsystem as a user-facing feature.

The product goal is:

- make `worth-signal` feel powerful immediately
- make the first 15 minutes smooth
- make expert workflows available without making the product look complicated
- support `worth-runtime-bridge` cleanly
- avoid exposing architecture for its own sake

This means the question is not "what exists?" It is:

> what should a user see first, what should remain available for expert use,
> what should exist mainly for bridge/integration authors, and what should stay
> internal or test-only?

---

## Exposure Tiers

### `P0` Primary public product surface

This is what should dominate docs, examples, and autocomplete.

Properties:

- easy to discover
- safe defaults
- short path to first success
- reflects the mental model we want users to adopt

### `P1` Advanced public surface

This remains public, but should not compete with `P0` in tutorials or landing
docs.

Properties:

- for power users
- explicit control
- performance or policy tuning
- advanced runtime operations

### `P2` Bridge-facing / integration-author surface

This may remain public, but is not a general entrypoint. It exists primarily
for runtime bridge, host integrations, and framework-style adapters.

Properties:

- specialized
- contract-heavy
- often infrastructure-facing rather than app-facing

### `P3` Internal / certification / test-only

This should not be presented as part of the public product story. If possible,
it should move out of the main public facade or become feature-gated /
crate-private / separately packaged.

Properties:

- harness or parity support
- certification scaffolding
- internal proofs or low-level maintenance shapes
- useful to us, not to most library consumers

---

## Product Principles

These should govern every exposure decision:

1. Users buy outcomes, not architecture.
2. The primary story should be "build reactive derived computation quickly,"
   not "learn our subsystem taxonomy."
3. The bridge should extend the product, not redefine the main API.
4. Bulk semantics should feel natural where the runtime is truly batch-oriented.
5. Diagnostics should feel premium and trustworthy, not overwhelming.
6. Power should appear progressively.

---

## Current Surface Assessment

Current public exposure is broader than the likely product surface.

Notable current conditions:

- [`crates/worth-signal/src/facade.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/facade.rs)
  already groups exports by namespace, which is directionally correct.
- The grouped facade still exposes a very large amount of machinery, including
  proof-bearing forms, merge surfaces, harness surfaces, and certification-ish
  utilities.
- [`crates/worth-signal/src/easy/mod.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/easy/mod.rs)
  is explicitly convenience-only, but it is public and could become the wrong
  default story if not positioned carefully.
- Harness and deployment surfaces are currently exported via
  [`crates/worth-signal/src/facade.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/facade.rs)
  even though they are unlikely to be part of the main product promise.

Working hypothesis:

- `S7` exists structurally
- `S7` has likely regressed at the product-boundary level
- the main problem is not lack of API, but lack of exposure discipline

---

## Exposure Matrix

| Category | Current examples | Primary audience | Target tier | Recommendation |
| --- | --- | --- | --- | --- |
| Quick start / typed convenience | `easy::ReactiveGraph`, `Signal`, `InputSignal`, `ComputedSignal` | new users, demos, evaluation | `P0` | Keep public, but message clearly as the shortest path, not the full runtime story |
| Core graph construction | `SignalGraph`, `NodeBuilder`, `DependencyEdge`, `NodeContract` | most serious users | `P0` | Keep public and central |
| Basic invalidation | `mark_dirty`, `mark_dirty_with_regions` | most users | `P0` | Keep public and central |
| Batch invalidation | `DirtyBatch`, `mark_dirty_batch` | serious users, bridge authors | `P0` | Keep public and push hard in docs because S9 makes batch-first normative |
| Core evaluation results | `NodeEvaluationResult`, `OutputChange`, `OutputIdentity`, `ChangedRegion` | most users building computed nodes | `P0` | Keep public and central |
| Prepared execution | `build_evaluation_plan`, `execute_prepared_plan`, `ExecutionReadView` | advanced users | `P1` | Keep public, but move below primary getting-started story |
| Runtime builder and runtime | `SignalRuntime`, `SignalRuntimeBuilder`, `TransactionResult` | serious users, production adopters | `P0` | Keep public and central |
| Runtime transactions | `SignalTransaction`, commit/rollback result types | production users | `P0` | Keep public and central |
| Runtime policy presets | `SignalRuntimePolicy::{operational,development,forensic,fintech,kernel}` | production users | `P0` | Keep public, but present as named presets before low-level retention knobs |
| Conditions and comparators | `EvaluationCondition`, comparator policies, tolerance hooks | advanced users | `P1` | Keep public, but progressive disclosure only |
| Tiers, checkpoints, keyed runtime | tier policy, checkpoint policy, keyed computation surfaces | expert runtime users | `P1` | Keep public; likely critical to product differentiation |
| Context and bridge contracts | `EvaluationContext`, `ContextRequirement`, bridge-specific context flows | bridge/integration authors | `P2` | Keep public but frame as integration authoring surface |
| Comparator resolver and policy plumbing | `VersionComparatorPolicy`, `DefaultComparatorPolicyResolver`, `TierPolicyResolver` | expert users, framework authors | `P1` | Keep public; these are real extension/config points |
| Branches, snapshots, replay | branch handles, snapshot types, restore plans, lineage | advanced users, bridge, forensic workflows | `P1` | Keep public, but do not lead with this in core docs |
| Snapshot storage and restore policy details | snapshot retention, restore modes, storage strategy, restore plans | advanced users, bridge authors | `P1` | Keep public if snapshot semantics are a product capability, but group tightly under history/state APIs |
| Explain / inspect / compare diagnostics | explanation, replay, summary, compare, render functions | advanced users, debugging, enterprise buyers | `P1` | Keep public; package as premium observability, not baseline complexity |
| Observers, materializers, and presentation outputs | `GraphObserver`, `RuntimeObserver`, `GraphMaterializer`, `RuntimeMaterializer`, DOT/metrics renderers | advanced users, tooling authors | `P1` | Keep public, but separate from the day-one API story |
| Event bus and subscriber integration | `EventBus`, `EventSubscriber`, subscriber context and errors | integration authors | `P2` | Keep public for runtime bridge and host integrations |
| Merge surfaces | branch merge plans, conflict records, reconciliation policy | advanced users, bridge authors | `P2` | Keep public if runtime-bridge needs it, but do not treat as general API |
| Proof-bearing performance forms | `DirtyDelta`, `LocalityFootprint`, `PatchPlan`, lowered/canonical forms | expert users, bridge, internal perf work | `P2` | Likely public-but-specialized; hide from primary docs and consider feature or submodule containment |
| Reuse and equivalence contracts | artifact equivalence, reuse proofs, semantic boundaries | expert users, bridge authors, performance-sensitive integrators | `P2` | Keep public if reuse is part of the differentiated product, but isolate under a specialist namespace |
| Runtime reconstructability proofs | journal/checkpoint rebuild proofs, reconstructability records | bridge authors, forensic tooling | `P2` | Keep public only if external tooling really consumes them; otherwise consider narrowing |
| Storage/core-profile tuning | `SignalCoreStorageProfile`, stable hash width, storage profile constants | extreme power users, maintainers | `P2` | Inventory explicitly; likely too low-level for the main facade unless a concrete external use case exists |
| Telemetry and node metadata plumbing | `RuntimeTelemetry`, `NodeMetaStore`, effect mapping | tooling authors, bridge authors | `P2` | Keep public only where needed for tooling or adapters |
| Harness runtime and scenarios | `SignalHarnessRuntime`, `SignalScenario`, parity helpers | internal team, maybe select integrators | `P3` | Remove from main public product story; likely separate crate/module boundary |
| Certification/deployment presets | `SignalDeploymentPlan`, `SignalDeploymentPreset`, profile catalogs | internal team, certification flows | `P3` | Do not expose as part of default public API story |
| Boundary contracts for internal review | `DependencyGraphContract`, `StructuralStateBoundaryContract`, transaction contracts | internal architecture enforcement | `P3` | Keep out of product-facing surface unless a real external use case appears |
| Test support helpers | graph dependency batch test extensions, support fixtures | internal | `P3` | Keep test-only |

---

## Recommended Public Story

This is the product sequence users should feel:

### Story 1: "I want derived computation working in minutes"

Expose first:

- `easy::ReactiveGraph`
- inputs
- computed signals
- reads
- batched writes

This is the emotional conversion surface.

### Story 2: "I need a production runtime"

Expose next:

- `SignalGraph`
- `NodeBuilder`
- `mark_dirty` and `DirtyBatch`
- `SignalRuntime::builder(...)`
- `SignalRuntimePolicy`
- `SignalTransaction`
- `TransactionResult`

This is the main product surface.

### Story 3: "I need precision and control"

Expose after that:

- prepared plans
- executors
- conditions
- comparators
- tiers
- checkpoints
- keyed computation

This is advanced public API.

### Story 4: "Iâ€™m building infrastructure on top"

Expose last:

- context-heavy integration surfaces
- bridge-targeted contracts
- merge/replay/restore internals that genuinely need to be public
- proof-bearing batch and locality forms

This is integration-author API, not app-developer API.

---

## What Should Probably Not Be Exposed Prominently

These are the strongest current candidates for de-emphasis or removal from the
main facade:

- harness runtime and scenario surfaces
- parity/certification helpers
- deployment presets
- internal architectural contract marker types
- low-level proof-bearing forms that are only meaningful inside performance
  enforcement pipelines

This does **not** automatically mean "make private tomorrow." It means:

- do not sell them as the library
- do not make them autocomplete-adjacent to everyday workflows
- consider separate modules, feature gates, or a companion crate if needed

---

## Search Matrix For Audit Passes

This is the practical search plan for the code audit.

| Pass | Goal | Search focus |
| --- | --- | --- |
| 1 | Find all public entrypoints | `pub use`, `pub fn`, `pub struct`, `pub enum`, `pub trait`, facade groups |
| 2 | Find beginner-facing stories | `easy`, docs examples, builder docs, top-level rustdoc |
| 3 | Find primary runtime workflows | `SignalGraph`, `NodeBuilder`, `mark_dirty`, `SignalRuntime`, `SignalTransaction` |
| 4 | Find advanced-but-valid public APIs | plan/executor/comparator/tier/checkpoint/keyed/context surfaces |
| 5 | Find bridge-facing APIs | merge, replay, restore, context contracts, proof-bearing forms |
| 6 | Find exposure leaks | harness, deployment, boundary contracts, test helpers exported publicly |
| 7 | Find DX hazards | `panic!`, `expect`, confusing names, duplicate concepts, flat overload |
| 8 | Find doc-story mismatch | compare public APIs against docs hierarchy and examples |

---

## Priority Heuristic

After the matrix, prioritize with this order:

1. Remove or hide obvious `P3` surfaces from the main product story.
2. Tighten and polish `P0` so the primary journey is elegant.
3. Repackage `P1` so advanced control feels powerful, not noisy.
4. Isolate `P2` so bridge work has room without contaminating the main UX.

This keeps the product optimized for adoption while still supporting the
runtime bridge and expert integrations.

---

## Immediate Decisions Suggested By This Matrix

These are the most likely next decisions to validate:

1. Whether `harness` should remain in the main facade at all.
2. Whether deployment/certification surfaces belong in `worth-signal` or in a
   separate support crate.
3. Whether proof-bearing forms need a narrower public namespace.
4. Whether `easy` should be the documented first step or a secondary convenience
   module.
5. Whether the main docs should be reorganized into `quick start`, `production
   runtime`, `advanced control`, and `integration authoring`.

---

## Bottom Line

The right DX strategy is not "show the whole machine elegantly."

The right strategy is:

- present a sharp primary story
- progressively disclose power
- keep bridge-grade integration surfaces available
- keep certification and internal enforcement tooling out of the main product
  identity
