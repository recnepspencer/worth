# Milestone 3.1: Runtime Boundary Closure and Inspection Authority

## Goal

Establish the first production-grade hot-composition runtime boundary for Worth
UI by splitting runtime ownership into explicit crates, closing the public
facade/lifecycle contract, and making inspection authority a first-class
runtime subsystem from the start rather than a later debug add-on.

## Why This Milestone Exists

Milestone 3.1 exists to stop the two most dangerous forms of drift before the
rest of the 3.x series broadens:

- runtime work collapsing back into one `worth-ui` crate with ad hoc deep
  imports, renderer-owned meaning, and blurry lifecycle boundaries
- AI/human inspection arriving later as screenshots, logs, panel-local state,
  or renderer-local helpers instead of a runtime-owned evidence lane

Without this milestone, later work on declarations, graph authority,
measurement, rebind, Query binding, services, diagnostics, and hot reload would
have no mechanically enforced architectural home. The result would be a runtime
that works just enough to demo but not enough to survive the 3.x adversarial
pressure honestly.

## Governing Summaries

- `MENTALITY.md`: protects this milestone from becoming a thin crate renaming
  pass by forcing the hard problem first: runtime authority and inspection
  authority must be closed before feature breadth lands on top of them.
- `arch_laws.md`: protects facade/lifecycle/inspection boundaries by requiring
  autonomous subsystems, compile-time contracts, proof-bearing types, typed
  unsupported posture, and no second truth or explanation runtime.
- `composition_laws.md`: protects the spec from broad “runtime stuff” buckets;
  each phase must own one real boundary and map cleanly to narrow modules and
  anti-bypass tests.
- `domain_structure_laws.md`: protects the tree from decorative layering; the
  split must encode authority, lifecycle, truth source, adapter boundaries, and
  inspection ownership in the physical structure.
- `perf_laws.md`: protects future hot paths by forcing inspection to be a
  formal indexed runtime surface rather than broad scans, logs, or renderer
  reconstruction.
- `worth_ui_roadmap.md`: protects the milestone sequence by requiring 3.1 to
  establish runtime boundary closure and the formal inspection harness contract
  before later 3.x slices enrich evidence, mounting, snapshots, replay, and the
  human inspector.

## Adversarial Constraint

A running Worth UI product must be able to accept future declaration, graph,
measurement, Query-binding, service, and diagnostics work from later 3.x slices
without any engineer needing to:

- add new public deep imports into runtime internals
- teach the host adapter what UI meaning is
- bolt AI inspection onto logs or screenshots
- create panel-local diagnostic truth
- bypass the facade to reach not-yet-stable runtime state

Concretely, after this milestone:

- adding subsystem `N+1` to the hot-composition runtime must produce compiler
  failures at every lifecycle propagation boundary that failed to initialize or
  forward it
- AI and human inspection requests must have exactly one public runtime entry
  point with typed unsupported posture for not-yet-admitted scopes
- host adapters must be unable to depend on runtime internals except through
  admitted host contracts
- unsupported or premature inspection must fail typed, not through missing APIs
  or string errors

If any of those conditions fail, later milestones will build on folklore rather
than architecture.

## Product Decision Lock

- `worth-ui` remains the public product facade, not the implementation bucket.
- runtime truth, host mechanics, inspection authority, Query binding, and
  certification each own separate lifecycle boundaries.
- `worth-ui-host-contract` is the stable host boundary; `worth-ui-host-egui`
  is only the first adapter implementation and must remain replaceable.
- inspection authority begins in Milestone 3.1; later milestones enrich it, but
  do not invent it.
- AI and human inspection are separate consumers over one runtime-owned
  inspection substrate.
- unsupported inspection scopes are valid runtime outcomes, not excuses for
  ad hoc APIs.
- this milestone closes boundaries and contract shapes only; it does not pull
  later evidence families, visual snapshots, replay richness, or inspector UI
  polish forward out of sequence.

## Non-Goals

Milestone 3.1 does not implement:

- DSL parsing
- canonical UI declarations
- authority graph topology
- aspect contracts
- measurement/allocation
- mounted receipts
- visual snapshots
- replay
- AI screenshot tools
- human inspector UI
- Query projection binding
- portal/focus/motion services

It only creates the enforced homes, public lifecycle, inspection contract
shape, support posture, unsupported posture, and anti-bypass proof that later
milestones must use.

## Boundary Matrix

| Crate | Owns | Must Not Own |
| --- | --- | --- |
| `worth-ui` | product facade | runtime internals |
| `worth-ui-dsl` | source/semantic DSL boundary | graph truth |
| `worth-ui-runtime` | hot-composition truth | host mechanics |
| `worth-ui-inspection` | inspection contracts/evidence | panel UI truth |
| `worth-ui-query-binding` | Query consumption boundary | Query authority |
| `worth-ui-host-contract` | native host boundary facts | egui mechanics |
| `worth-ui-host-egui` | egui translation | UI meaning |
| `worth-ui-certification` | anti-bypass proof | production runtime truth |

## Runtime Boundary Rule

The governing boundary for this milestone is:

```text
worth-ui facade may expose owned runtime contracts
worth-ui-dsl may own DSL parsing, semantic lowering contracts, and authored
source-boundary types
worth-ui-runtime may own active hot-composition truth
worth-ui-inspection may own typed inspection contracts and later evidence
worth-ui-host-contract may define the native host boundary
worth-ui-host-* may implement that contract and emit observations
no host, panel, or app code may recreate runtime meaning or inspection truth
```

The inspection-specific corollary is:

```text
all AI-facing and human-facing inspection must enter through the same runtime
inspection facade, even when a requested scope is not admitted yet
```

## Query And Foundational Ownership Lock

Worth UI inspection must not create a second Query runtime or prematurely lower
runtime-local truth into Foundational vocabulary.

Rules:

- Query-backed facts must be consumed through admitted Query
  projection-consumption receipts, not reconstructed from retained rows, bridge
  internals, or local caches.
- Cross-runtime causal explanation must route through Query's causal-inspection
  lane when the cause crosses Worth UI / Query / bridge boundaries.
- Query async/result posture must be preserved through typed Query artifacts
  rather than rewritten into Worth-UI-local loading/error enums.
- runtime-local Worth UI receipts, support rows, and closure posture remain
  Worth-owned artifacts until they cross a real support, export, or reporting
  boundary.
- Foundational types are allowed only when the meaning is now shared boundary
  meaning, not while the semantics remain Query-owned or Worth-runtime-owned.

## Phase Plan

### Required Implementation Surfaces By Phase

These are the intended implementation and proof surfaces for the milestone.
Exact file names may shift, but responsibility boundaries must remain intact.

- Phase 1 uses `workspaces/worth-ui/crates/worth-ui/src/lib.rs`,
  `workspaces/worth-ui/crates/worth-ui-dsl/src/lib.rs`,
  `workspaces/worth-ui/crates/worth-ui-runtime/src/lib.rs`,
  `workspaces/worth-ui/crates/worth-ui-inspection/src/lib.rs`,
  `workspaces/worth-ui/crates/worth-ui-host-contract/src/lib.rs`,
  `workspaces/worth-ui/crates/worth-ui-host-egui/src/lib.rs`, and workspace
  manifest propagation tests.
- Phase 2 uses workspace crate-disposition audits, migration shims where
  necessary, and parallel-authority regression tests.
- Phase 3 uses `workspaces/worth-ui/crates/worth-ui/src/facade/`,
  public builder/app entry points, and compile-fail lifecycle propagation
  tests.
- Phase 4 uses `workspaces/worth-ui/crates/worth-ui-inspection/src/query/`,
  `.../receipt/`, `.../target/`, and inspection contract compile tests.
- Phase 5 uses `workspaces/worth-ui/crates/worth-ui-inspection/src/posture/`
  and typed unsupported/admission tests.
- Phase 6 uses crate visibility boundaries, dependency audit tests, and
  structural forbidden-import checks.
- Phase 7 uses `workspaces/worth-ui/crates/worth-ui-inspection/src/facade/`,
  `.../scopes/`, `.../budgets/`, and no-evidence-yet contract tests.
- Phase 8 uses `workspaces/worth-ui/crates/worth-ui-certification/`,
  compile-fail anti-bypass tests, and hostile boundary regression tests.

### Phase 1: Crate Topology and Ownership Split

This phase freezes the physical ownership map so later 3.x work cannot drift
back into a decorative “one crate plus helpers” layout.

**Relevant subsystems**
- public Worth UI facade
- DSL/source-boundary crate
- hot-composition runtime
- inspection substrate
- host contract
- egui host adapter
- certification

**Relevant APIs**
- workspace `Cargo.toml` members for the Worth UI crates
- `worth-ui::lib`
- `worth-ui-dsl::lib`
- `worth-ui-runtime::lib`
- `worth-ui-inspection::lib`
- `worth-ui-host-contract::lib`
- `worth-ui-host-egui::lib`
- `worth-ui-certification::lib`

**Warnings**
- Do not let `worth-ui-inspection` become a panel crate; it is the runtime
  inspection substrate, not the human inspector UI.
- Do not let `worth-ui-dsl` become a thin forwarding crate while real DSL
  parsing/lowering types stay hidden in facade or runtime modules.
- Do not let `worth-ui-host-egui` become the de facto host contract by
  accreting runtime meaning or host-neutral APIs that belong in
  `worth-ui-host-contract`.
- Do not split by implementation taste. The split must reflect authority,
  lifecycle, and truth source boundaries.
- Do not leave old internal modules in `worth-ui` as shadow implementation
  entry points after the new crates exist.

**Test requirements**
- Compiler-enforced lifecycle propagation test: adding a new runtime subsystem
  field must fail every constructor or facade propagation site that did not
  update.
- DSL-boundary test: facade or runtime crates must fail structural audit if
  they grow public DSL parsing/lowering entry points that belong in
  `worth-ui-dsl`.
- Adapter-replaceability test: a second host adapter crate must be able to
  implement `worth-ui-host-contract` without importing `worth-ui-host-egui`
  internals or changing runtime public types.
- Dependency-boundary test: `worth-ui-host-egui` must fail if it imports
  runtime-internal modules instead of the admitted host contract surface.
- Topology audit test: no crate may deep-import another crate’s internal module
  tree across the new boundaries.

**Engineering decisions**
- `worth-ui` stays as the single public product facade, with other crates
  exposed only through curated public capability surfaces.
- `worth-ui-dsl` begins now because the DSL is an authority boundary, not an
  implementation detail hidden inside facade or runtime crates.
- `worth-ui-inspection` begins now because inspection authority is a runtime
  concern, not later tooling sugar.
- `worth-ui-certification` remains separate so anti-cheating proof does not
  drift into ad hoc test helpers.

**Open questions**
- None. The crate split is a product decision lock for the rest of 3.x.

### Phase 2: Legacy Surface Residue Closure

This phase maps the current workspace crates onto the target 3.1 ownership
split so the new topology does not coexist with parallel public authorities.

**Relevant subsystems**
- existing `worth-ui-*` workspace crates
- existing `worth-ui` public facade
- migration shims
- structural residue audits

**Relevant APIs**
- workspace `Cargo.toml` members for current and target crates
- crate-level public exports
- migration-only shim exports where explicitly justified
- structural audit harnesses

**Warnings**
- Do not leave existing `worth-ui-*` crates as parallel public homes for
  runtime, inspection, DSL, support, or host-neutral types after the new owners
  exist.
- Do not preserve migration convenience by keeping multiple public authority
  paths alive.
- Do not let shim crates become permanent unlabeled product surfaces.
- Do not move names around without also closing residue in docs, exports, and
  certification.

**Test requirements**
- Residue audit test: every pre-3.1 UI crate must have an explicit disposition:
  owner, shim, deprecated, or removed.
- Parallel-authority test: no legacy crate may continue exporting host-neutral
  runtime, inspection, DSL, or support-truth types once a target 3.1 owner
  exists.
- Shim-honesty test: any temporary migration shim must forward only to the new
  owner and must not reintroduce local semantics or hidden construction paths.
- Public-surface diff test: moving responsibility into the new split must narrow
  or retire the old surface rather than duplicating it under a new path.

**Engineering decisions**
- The current workspace shape is treated as migration residue, not as proof
  that multiple UI crate families should survive into the target architecture.
- Migration is allowed to use short-lived shims, but only when certification
  can prove they are non-authoritative forwarding surfaces.
- A crate is not considered retired until its public authority has been removed
  or mechanically downgraded to a shim/deprecated surface.

**Open questions**
- None. Residue closure is required for the crate split to become real.

### Phase 3: Facade and Lifecycle Propagation Closure

This phase closes the public construction and propagation contract so later
runtime subsystems cannot be added without compiler-visible updates.

**Relevant subsystems**
- public facade
- app/builder construction flow
- runtime subsystem aggregation
- inspection subsystem aggregation

**Relevant APIs**
- `WorthUiApp`
- `WorthUiBuilder`
- runtime bundle/bootstrap structs
- inspection bundle/bootstrap structs
- facade-only public constructors

**Warnings**
- Do not export raw subsystem constructors just because the facade is not rich
  yet.
- Do not allow lifecycle propagation by convention. New subsystem fields must
  break construction at compile time until every site is updated.
- Do not leak a partially initialized runtime/inspection pair across the facade
  boundary.

**Test requirements**
- Exhaustive construction test: adding `inspection` or `runtime` fields to the
  aggregate must fail until every builder and app-construction site initializes
  them.
- Compile-fail test: external code cannot instantiate lifecycle-bearing runtime
  aggregates without using the public facade/builder path.
- Snapshot-isolation test: read-path facade calls for inspection setup must not
  require mutable access to future runtime write-path authority.
- Inventory-propagation test: adding subsystem `N+1` must fail not only facade
  construction but also support-row inventories, inspection-scope inventories,
  and certification closure inventories until every owned registry updates.

**Engineering decisions**
- Public construction remains facade-driven even while internal runtime crates
  are still thin.
- Lifecycle-bearing aggregates must use exhaustive struct construction or
  equivalent typestate/builder enforcement, not optional late wiring.
- The facade is allowed to be broad only as orchestration; it must not become a
  hidden implementation bucket.

**Open questions**
- None. This milestone chooses compile-time lifecycle propagation over
  documentation or TODOs.

### Phase 4: Inspection Authority Contract

This phase establishes the formal AI/human inspection entry point and contract
shape without pretending later evidence families are already implemented.

**Relevant subsystems**
- inspection substrate
- public inspection facade
- runtime/inspection boundary
- consumer-neutral inspection API

**Relevant APIs**
- `UiInspectionTarget`
- `UiInspectionQuery`
- `UiInspectionScope`
- `UiInspectionSupportReport`
- `UiInspectionScopeSupportRow`
- `UiInspectionClosureReport`
- `UiEvidenceBudget`
- `UiEvidenceRichness`
- `UiInspectionRelevance`
- `UiInspectionSupportStatus`
- `UiInspectionMilestoneExpectation`
- `UiInspectionReceipt`
- inspection facade entry method(s)

**Warnings**
- Do not let the first inspection surface be screenshot-first or panel-first.
- Do not make separate “AI query” and “human inspector query” contracts.
- Do not smuggle undeclared evidence meaning through opaque JSON blobs or string
  maps just because the contract is early.
- Do not make support/admission posture live only in prose; if a scope is
  unsupported, the runtime must be able to say so structurally.

**Test requirements**
- Contract-identity test: AI-facing and human-facing callers must both compile
  against the same `UiInspectionQuery` and `UiInspectionReceipt` types.
- Compile-fail test: app or host code cannot construct proof-bearing inspection
  receipts directly.
- Scope-shape test: the initial contract must support target/scope/budget
  separation without requiring later phases to break the public API.
- Relevance-shape test: the public query contract must carry first-class
  relevance filtering so later evidence families do not require a breaking API
  expansion or broad-scan fallback.
- Support-report test: the inspection subsystem can report that a scope belongs
  architecturally but is not yet admitted, with machine-readable milestone
  expectation instead of a string-only explanation.
- Query-boundary test: inspection contracts for Query-backed targets must name
  whether the answer comes from Query inspection, Query projection consumption,
  or Worth-local evidence, and must fail structural audit if they introduce a
  UI-local pseudo-Query lane.

**Engineering decisions**
- The first milestone establishes contract shape, not rich evidence breadth.
- Inspection contracts are proof-bearing runtime surfaces, not generic debug
  DTOs.
- The inspection facade is part of the runtime boundary, not a sidecar helper
  API.
- 3.1 support truth must be reportable independently of any successful
  inspection query so later phases can remain honest about partial support.

**Open questions**
- None. The contract must exist now even if some scopes remain unsupported
  initially.

### Phase 5: Unsupported Inspection Posture

This phase makes “not implemented yet” an admitted, typed runtime posture
instead of missing APIs, panics, or string-only fallback behavior.

**Relevant subsystems**
- inspection substrate
- support/admission posture
- future evidence expansion compatibility

**Relevant APIs**
- unsupported/not-yet-admitted inspection result variants
- `UiInspectionSupportReport`
- `UiInspectionScopeSupportRow`
- `UiInspectionClosureReport`
- inspection error/denial topology
- future scope-family placeholders

**Warnings**
- Do not represent unsupported posture as `Option`, `None`, or silent empty
  results.
- Do not use string messages as the public unsupported contract.
- Do not let callers infer support by probing methods or feature flags.
- Do not collapse “belongs here but not yet admitted” into the same category as
  “architecturally invalid target.”

**Test requirements**
- Typed-posture test: unsupported inspection scopes must return structured
  posture through `UiInspectionReceipt`, not missing APIs or panics.
- Boundary-localization test: unsupported `measurement`, `mounting`, or
  `rebind` scopes must fail locally without implying the whole inspection
  substrate is unavailable.
- Future-compatibility test: adding a new scope family later must extend the
  existing posture system rather than fork a second unsupported mechanism.
- Support-report test: support rows must be able to state `unsupported`,
  `reason=belongs-architecturally-not-yet-admitted`, and a milestone
  expectation for the scope.

**Engineering decisions**
- Unsupported posture is a first-class runtime outcome because the harness
  begins before all evidence families exist.
- Typed posture is stronger than feature flags because it preserves one public
  entry point while keeping later evolution honest.
- Unsupported posture belongs to the runtime boundary, not to client-side
  wrappers.
- Support reporting is a separate artifact from query receipts so the system can
  truthfully answer “what should exist here?” before rich evidence exists.

**Open questions**
- None. This is the mechanism that keeps 3.1 truthful while 3.2+ broaden
  support.

### Phase 6: Dependency and Import Guards

This phase turns the crate split and inspection boundary into mechanical rules
instead of architecture prose.

**Relevant subsystems**
- crate dependency graph
- visibility boundaries
- forbidden import enforcement
- adapter/runtime separation

**Relevant APIs**
- crate public exports
- `pub(crate)` and narrower internal topology
- dependency audit scripts or compile checks
- forbidden import regression tests

**Warnings**
- Do not accept a “we just won’t import that” rule without enforcement.
- Do not let the host adapter or app layer gain convenience imports into
  runtime or inspection internals.
- Do not let facade or runtime grow convenience DSL ownership that bypasses
  `worth-ui-dsl`.
- Do not let host-neutral types, contracts, or lifecycle flow leak into
  `worth-ui-host-egui` just because it is the first adapter.
- Do not leave legacy modules publicly visible just to reduce migration pain.
- Do not use `Default`, `Option`, dynamic maps, or trait-object expansion to
  weaken required lifecycle propagation for owned subsystems.

**Test requirements**
- Forbidden-import test: `worth-ui-host-egui` must fail if it reaches
  `worth-ui-runtime` or `worth-ui-inspection` internals directly.
- DSL-import test: non-DSL crates must fail structural audit if they expose or
  import private DSL-lowering internals instead of consuming admitted DSL
  boundary types.
- Host-contract test: adapter-facing runtime types used by `worth-ui-host-egui`
  must live in `worth-ui-host-contract`, not in the egui adapter crate.
- Public-surface test: app code must fail when attempting to import internal
  runtime/inspection modules instead of facade-level exports.
- Decorative-architecture test: moving a type into a new crate without
  narrowing exports must fail the structural audit.
- Query-lane test: `worth-ui-runtime` and `worth-ui-inspection` must fail
  structural audit if they recreate Query-owned support, async/result,
  inspection, causal-explanation, or projection-fact lanes behind local public
  types.
- Foundational-lowering test: runtime-local receipts, support rows, and closure
  posture must fail structural audit if they lower into Foundational vocabulary
  before crossing a real export, report, or support boundary.
- Lifecycle-cheating test: required subsystem aggregates must fail structural
  audit if they implement `Default`, store required subsystems as `Option`, or
  use builders/maps that silently omit owned subsystems.

**Engineering decisions**
- Visibility is the first enforcement tool; tests are the second.
- Import guards should live in certification/structural audit surfaces, not
  hidden in a local shell script no one runs.
- The milestone should prefer smaller public surfaces even if it increases
  initial internal wiring work.
- Required subsystem propagation must be represented concretely enough that
  aggregate expansion forces compiler breakage when a subsystem is added.

**Open questions**
- None. Enforcement is mandatory for this boundary to mean anything.

### Phase 7: Inspection Crate Seed Topology

This phase gives `worth-ui-inspection` an honest internal structure so later
evidence, replay, visual snapshot, and inspector work has a real home instead
of landing in one giant “inspection” file.

**Relevant subsystems**
- inspection facade
- inspection query contracts
- posture/denial contracts
- future evidence families

**Relevant APIs**
- `worth-ui-inspection/src/query/`
- `.../target/`
- `.../scope/`
- `.../receipt/`
- `.../posture/`
- `.../facade/`

**Warnings**
- Do not collapse query types, receipts, unsupported posture, and future
  evidence placeholders into one catch-all module.
- Do not create `helpers`, `utils`, or `common` folders for inspection.
- Do not let the first file order or folder order follow authoring chronology
  instead of semantic hierarchy.
- Do not export mixed query + receipt + posture + facade responsibilities from
  one public module.

**Test requirements**
- Public-topology test: `worth-ui-inspection` must expose no public module named
  `internal`, `common`, `helpers`, `utils`, `data`, `manager`, or `debug`.
- Growth-path test: adding the first real evidence family in 3.5 must have an
  obvious home without reopening the public contract or renaming the crate.
- Module-contract test: each public module must export either contract types,
  sealed receipt/posture types, facade methods, support/admission types, or
  certification hooks, but not mixed responsibilities.

**Engineering decisions**
- The inspection crate is seeded now with empty-but-real topology because later
  milestones will otherwise dump evidence, replay, and visual inspection into
  one blob.
- Query, receipt, posture, and facade responsibilities remain separate from day
  one.
- Future human inspector UI is intentionally excluded; this crate owns runtime
  inspection substrate only.

**Open questions**
- None. The structure must predict later work cleanly.

### Phase 8: Certification and Anti-Bypass Proof

This phase closes Milestone 3.1 with proof that the boundary is real, not
ceremonial.

**Relevant subsystems**
- certification crate
- compile-fail proof surfaces
- structural boundary audits
- hostile regression suite

**Relevant APIs**
- certification crate entry points
- compile-fail harness
- dependency/import audit harness
- unsupported inspection behavior tests

**Warnings**
- Do not defer proof until later milestones. This milestone’s entire value is
  that later work cannot quietly bypass it.
- Do not rely only on unit tests. This boundary needs compile-fail and
  structural-audit proof.
- Do not couple certification to renderer behavior or later evidence families.

**Test requirements**
- Compile-fail WORTHry test: external code cannot mint proof-bearing inspection
  receipts or unsupported posture witnesses directly.
- Replay-honesty precursor test: repeated inspection queries against the same
  unsupported scope must return equivalent typed posture rather than drifting
  through side effects or logs.
- Anti-bypass regression test: adding a new adapter or consumer crate without
  going through the inspection facade must fail certification.

**Engineering decisions**
- `worth-ui-certification` is where milestone-level anti-cheating proof lives.
- Compile-fail tests are mandatory because documentation cannot enforce this
  boundary.
- The first certification surface stays narrow and architecture-focused; it
  should not wait for rich feature scenarios from later 3.x slices.

**Open questions**
- None. The milestone is not complete until anti-bypass proof exists.

## Must Ship

- crate topology for `worth-ui`, `worth-ui-runtime`, `worth-ui-inspection`,
  `worth-ui-dsl`, `worth-ui-query-binding`, `worth-ui-host-contract`,
  `worth-ui-host-egui`, and `worth-ui-certification`
- facade-only runtime construction and lifecycle propagation with compiler
  failure on omitted subsystem propagation
- sealed inspection contract types and one formal inspection facade entry point
- typed unsupported/not-yet-admitted inspection posture
- import/visibility enforcement that prevents adapter or app code from reaching
  runtime and inspection internals directly
- seed topology inside `worth-ui-inspection` that cleanly separates query,
  scope, target, posture, receipt, and facade responsibilities
- support/admission reporting artifacts for inspection scope truth
- certification and compile-fail proof that the boundary cannot be bypassed
  casually

## Must Preserve

- Worth UI remains one public product facade even while implementation splits
  into multiple crates
- DSL parsing/lowering ownership remains explicit and separate from runtime
  truth ownership
- host adapters remain mechanics-only and do not gain semantic or diagnostic
  authority
- inspection remains one runtime-owned substrate serving AI and human consumers
  instead of branching into separate systems
- later milestones remain able to enrich the inspection harness without
  breaking the public contract shape
- unsupported inspection remains typed and explicit rather than degrading into
  missing API surface or string-only fallback
- inspection receipts remain sealed-but-projectable: externally readable,
  externally unWORTHable
- this milestone does not drag in later evidence families, snapshots, replay
  richness, or inspector UI work out of order

## Acceptance Evidence

- adding a new runtime subsystem after this milestone forces explicit lifecycle
  propagation updates at every constructor/fork boundary
- an app or adapter can access runtime and inspection only through admitted
  public contract surfaces
- AI-facing and human-facing inspection consumers compile against the same
  contract types and facade entry point
- public consumers can read/project inspection receipt identity, target, scope,
  posture, budget, and evidence refs without being able to mint those claims
- unsupported inspection scopes return structured posture rather than panics,
  missing APIs, or strings
- support reports can state that a scope belongs architecturally, is not yet
  admitted, and is expected in a later milestone
- inspection contract types and receipts cannot be WORTHd from outside the
  owning runtime boundary
- the physical tree now teaches where runtime truth, inspection authority, host
  mechanics, DSL ownership, Query binding, and certification live

## Sequencing Notes

- Milestone 3.1 is the contract-and-boundary slice, not the rich-evidence
  slice.
- Milestone 3.2 through 3.4 make declarations, graph topology, and obligations
  real enough that inspection can begin answering real semantic questions.
- Milestone 3.5 operationalizes this milestone’s harness with the first
  substantial evidence families and relevance indexes.
- Milestone 3.10, 3.11, 3.15, 3.16, 3.17, and 3.18 enrich the same inspection
  substrate with visual snapshots, rebind explanation, diagnostics closure,
  visual evaluation, AI replay/tooling, and the human inspector.
- If later work needs a second inspection entry point, a renderer-local
  diagnostic model, or a panel-owned truth store, this milestone was violated.
