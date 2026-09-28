# WORTH Query Runtime API Public Stabilization Plan

> **Status:** Closeout implemented; see
> [runtime-api-public-stabilization-closeout.md](./runtime-api-public-stabilization-closeout.md)
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Predecessor:** [runtime-api-next-batch-closeout.md](./runtime-api-next-batch-closeout.md)
>
> **Next roadmap family:** Runtime Authoritative Mutation Evidence Gate,
> followed by Milestones 9.4 through 9.7 temporal/async query semantics
>
> **Purpose:** freeze the ordinary public runtime API shape before temporal
> and async query semantics land, so domain runtimes can build against a
> beautiful, stable, proof-bearing facade now without requiring a breaking
> rewrite when async/resource-backed properties arrive later.
>
> **Bias:** make the ideal developer experience the executable contract. If
> current runtime seams cannot support that contract, fix the runtime seams.
> Do not weaken the public API, hide orchestration cost behind cheap-looking
> calls, smuggle domain semantics into Query, or route around proof-bearing
> facade handles.

## Goal

Finalize the public runtime API contract that ordinary application, DSL,
geometry, workflow, table, and kernel code should build against now:

- named durable surfaces
- typed handles as the composition unit
- aspect-declared dependency contracts
- branch and preview reuse through handle binding
- computeds that derive only
- effects that deliver or stage work
- intents that cross authority boundaries
- inspection as the trust surface
- support metadata and typed denial for unsupported combinations
- async-safe result/state vocabulary that can later admit temporal and
  resource-backed properties without changing the facade model

This stabilization pass does not implement temporal/async behavior. It freezes
the API posture those later milestones must extend.

## Why This Plan Exists

The runtime API batch closed the proof-bearing foundation: live declarations
install subscriptions automatically, patch delivery is query-shaped, computeds
and effects are declaration-owned handles, branch/preview reuse is isolated,
intents cross sealed authority lanes, and inspection explains retained
artifacts.

That is enough foundation to start building serious runtime features. It is
not yet enough to call the public API final.

Without this stabilization gate:

- geometry, workflow, table, and app runtimes may start depending on
  infrastructure-colored method names that later become embarrassing
- the sync-first shape may accidentally imply that every property is always
  immediately available
- temporal/async milestones may need to add parallel APIs instead of extending
  the same handle and state model
- domain DSLs may generate code against a facade that is technically correct
  but not the beautiful daily-driver shape promised in the spec
- inspection may remain a strong debugging surface without becoming part of
  the official public contract

This plan exists to freeze the public shape before those mistakes become
callers.

## Governing Summaries

- `MENTALITY.md`: protects adversarial foundations and enforcement over
  convention. The stabilization spec must start from the hardest future use:
  domain runtimes using the API heavily now while async semantics arrive later.
- `arch_laws.md`: protects facade-only access, declaration-owned resources,
  explicit boundary crossings, authority/derivation separation, framework-owned
  lifecycle, phase-typed handles, and proof-bearing artifacts. The public API
  must make the right path natural and the wrong path unrepresentable.
- `perf_laws.md`: protects cost-honest API shape. No public method may look
  like a cheap local property access if it can cross a query, subscription,
  branch, async, or diagnostic boundary.
- `domain_laws.md`: protects subsystem responsibility and domain neutrality.
  This API can be validated against geometry/workflow/table examples, but it
  must not place geometry, workflow, or table semantics inside `worth-query`.
- `worth_query_vision.md`: protects the product promise that developers
  declare query intent once and reuse the same shape for reads, live
  subscriptions, branches, history, derived computation, and delivery.
- `worth_query_roadmap.md`: protects the rule `declare query intent once,
  lower it once, execute it against canonical truth`. This stabilization gate
  belongs after 9.1-9.3 runtime-backed subscription closure and before 9.4-9.7
  temporal/async semantics.
- `milestone-9.3.6.md` and
  `test-requirements-milestone-9_3-and-runtime-gates.md`: protect the named
  `9.3.6. Lower-Runtime Capability Routing And Boundary Envelope Closure Test`
  and the exported lower-runtime named closure-test artifact, phase artifact
  manifest, and public closeout report that the stabilization gate must
  consume rather than re-deriving lower-runtime routing closure from internal
  details.
- `test-requirements.md`: protects certification-grade proof and explicitly
  rejects trivial tests. Stabilization must add golden DX transcript tests plus
  adversarial cross-feature proof, not just examples that compile.
- `milestone-5.6.md` and `milestone-5.6-closeout.md`: protect the daily-driver
  facade and support/configuration honesty that this API must use rather than
  bypass.
- `milestone-9.1-closeout.md`: protects subscription declaration and admission
  as a typed proof chain, not caller-owned live plumbing.
- `milestone-9.2-closeout.md`: protects active lifecycle, query-shaped
  delivery, continuation, and preview isolation as runtime-owned resources.
- `milestone-9.3.md`: protects diagnostics, bridge parity, and family support
  reporting as the explanation substrate for runtime handles.
- `runtime-api-next-batch-closeout.md`: protects the closed runtime facade
  foundation that this plan now stabilizes into the final public shape.

## Adversarial Constraint

An ordinary developer must be able to build a serious runtime feature now
against the public Query facade, using only named surfaces and typed handles,
while later temporal and async/resource semantics can be admitted into the same
model without breaking the public API or asking the domain runtime to rewrite
its code.

The public API must survive these hostile conditions:

- a workflow DSL generator creates hundreds of sections, groups, controls,
  expressions, computeds, effects, branch previews, and intent commits
- a geometry kernel declares topology neighborhoods, expensive derived
  surfaces, preview branches, invariant-preserving commits, and inspection
  points
- an Excel-grade table declares cells, formulas, dropdown constraints,
  layout/resizing state, grouped views, batched edits, and live delivery
- all of those start with synchronous runtime-backed data
- Milestones 9.4 through 9.7 later add temporal wakes, async/resource-backed
  properties, cancellation, retry, stale completion denial, mixed-cause
  ordering, and certification

If the final public API makes any of the following likely, this plan has
failed:

- domain code must later replace `read`/`live`/`computed` calls with separate
  async-specific APIs
- a public getter hides query execution, subscription activation, async
  resolution, or broad inspection work
- branch/preview reuse activates effects or external writes by surprise
- a computed can silently become authoritative truth without an intent
- aspect paths are meaningful without authority lane and basis identity
- `inspect()` is treated as a debug add-on rather than the trust contract
- golden examples are beautiful but tests only prove trivial compilation
- unsupported temporal/async neighbors can be represented as best-effort
  host-local loading state

## Non-Negotiable Public DX

The final API should feel like durable surface declaration and handle
composition. The names below are illustrative Rust-shaped API contracts; the
implementation may choose exact names that better fit existing WORTH Query
style, but it may not weaken the DX properties.

```rust
let workspace = query.workspace("company-runtime")?;

let canvas = workspace.live_view("editor.canvas", |q| {
    q.from("Node")
        .scope("visible_in_canvas")
        .select([
            "identity.id",
            "display.label",
            "layout.frame",
            "constraint.expression",
            "validation.state",
            "runtime.value",
        ])
        .order_by("layout.frame")
        .as_surface("canvas")
})?;

let field_values = workspace.computed("editor.field_values", |c| {
    c.from(&canvas)
        .for_each("Control")
        .reads(["constraint.expression", "runtime.input"])
        .produces(["runtime.value"])
        .with_expression("expr.control.value")
})?;

let readiness = workspace.computed("editor.readiness", |c| {
    c.from(&canvas)
        .depends_on(&field_values)
        .for_each("Section")
        .reads(["runtime.value", "validation.rule"])
        .produces(["validation.state"])
        .with_expression("expr.section.readiness")
})?;

let validation_effect = workspace.effect("editor.validation_badges", |e| {
    e.when(readiness.changed("validation.state"))
        .condition("expr.validation.visible")
        .deliver("ui.validation_badges")
})?;

let branch = workspace.branch("try-approval-rule", |b| {
    b.effects().derive_only()
})?;

let preview_canvas = branch.use_view(&canvas)?;
let preview_readiness = branch.use_computed(&readiness)?;

let receipt = branch.intent("apply-approval-rule", |intent| {
    intent.set("Section", section_id, "validation.rule", new_rule);
})?;

let explanation = workspace.inspect(&preview_readiness)?;
```

Required DX properties:

- schema and invariant safety disappear after declaration; they remain
  foundational but not repeated as caller ceremony
- named live views are durable application surfaces, not throwaway
  subscriptions
- computed resources are declarative but auditable through aspects, lanes,
  dependencies, and inspection
- effects are separate from computeds; computeds derive, effects deliver or
  stage work
- branch and preview reuse preserves declarations and rebinds basis/lane
  evidence
- intents are the public mutation/authority-crossing surface, including
  effect-triggered and branch-local variants
- inspection is part of the normal DX and must explain the hidden runtime path
- unsupported combinations deny through typed errors before fallback or
  widening

## Stable Public Concepts

The public API must freeze these concepts as the long-lived vocabulary.

### Workspace

`Workspace` is the ordinary facade context for runtime-backed application
surfaces. It owns declaration entrypoints and support/inspection access, but it
does not own truth semantics, signal scheduling, bridge protocol semantics, or
store durability.

Required properties:

- created through the unified application facade or runtime facade, not by
  directly assembling lower-runtime internals in ordinary code
- carries support metadata for read, live, computed, effect, branch, preview,
  write, intent, inspect, temporal, async, and mixed-cause families
- can expose memory-backed scaffold posture, but that posture must remain
  named and inspectable

### Durable Surface

A durable surface is a named product-facing declaration such as
`editor.canvas`, `workflow.readiness`, `geometry.neighborhood`, or
`table.visible_rows`.

Required properties:

- names are stable debugging and inspection anchors
- names do not encode domain logic inside Query; domain runtimes provide schema,
  adapters, expressions, and commit strategies
- declarations produce handles with retained proof, not raw callbacks
- redeclaration and reuse rules must be explicit: exact reuse, compatible
  update, or typed conflict

### Handle

Handles are the stable unit of composition.

Every handle must carry or inspectably reference:

- durable surface name
- declaration digest
- query/result-shape digest where applicable
- aspect read contract
- aspect produce contract where applicable
- dependency handle digests
- authority lane
- basis lane
- support posture
- lifecycle/activation evidence where applicable
- inspection digest

Handle rules:

- handles are cheap references to framework-owned resources, not resource
  ownership escapes
- handles can be rebound into branch/preview contexts only through explicit
  basis evidence
- handles cannot be mutated to change authority lane, effect policy,
  subscription family, or result shape
- handles must be future-compatible with temporal/async state without changing
  their role as composition anchors

### State Snapshot

Public reading must not imply "this value is synchronously available forever."
The API needs a state vocabulary that is useful for sync data now and async
data later.

Required result-state vocabulary:

- `Ready`
- `Pending`
- `Stale`
- `Failed`
- `Cancelled`
- `Superseded`
- `Denied`
- `Unsupported`

Rules:

- synchronous runtime-backed data can produce `Ready` immediately
- unsupported async/resource neighbors must produce typed denial, not
  host-local loading strings
- stale, cancelled, superseded, and failed states must be digest-bound to the
  query/resource/basis that produced them once 9.5 lands
- public APIs should prefer `read`, `snapshot`, `observe`, `state`, or
  `materialize` vocabulary over cheap-looking `value()` getters when the call
  may cross a runtime boundary

### Aspect Contract

Aspects are the dependency and invalidation contract.

Required rules:

- computeds declare `reads` and `produces`
- effects declare trigger aspects, condition inputs, and delivery/write-intent
  outputs
- live views declare projected aspects and view-shape delivery requirements
- async/resource declarations later declare resource inputs and produced result
  state through the same aspect/basis vocabulary
- inspection must show aspect paths together with authority lane and basis
  identity

### Authority Lane

Authority lanes remain first-class public semantics through inspection and
typed evidence, even when ordinary API calls stay pleasant.

Required lanes:

- `AuthoritativeTruth`
- `BranchLocalTruth`
- `PreviewTruth`
- `DerivedRuntimeState`
- `EffectDeliveryState`
- `PendingWriteIntent`
- `BridgeExternalState`
- future `TemporalExecutionState` where 9.4 admits it
- future `AsyncResourceState` where 9.5 admits it

Rules:

- identical aspect paths in different lanes are not the same state
- derived state may become truth only through explicit intent/commit authority
- branch and preview lanes do not leak into authoritative truth without
  promotion/commit evidence
- effect delivery state is not pending write-intent state
- async resource state later remains external/lifecycle state until admitted
  materialization says otherwise

## Phases

### Phase 1: Golden DX Transcript Contracts

Purpose:

- freeze the final public shape before more implementation deepens around
  implementation-colored names
- prove the ordinary consumer writes handle-composition code rather than lower
  runtime plumbing
- establish non-domain-specific examples that still pressure the API like real
  geometry, workflow, and table runtimes

Must ship:

- a golden workflow-editor transcript covering:
  - live sections/groups/controls
  - relational invariant declarations through adapters
  - nested computeds
  - conditional expression nodes
  - effect delivery
  - branch preview
  - branch-local intent
  - inspection
- a golden geometry-kernel transcript covering:
  - topology neighborhood live view
  - derived geometric outputs
  - expensive recompute/fallback posture
  - branch experiment
  - invariant-preserving intent commit
  - inspection of dependency and authority lanes
- a golden table transcript covering:
  - visible rows live view
  - formula/dropdown/layout computeds
  - grouped or ordered delivery
  - batched edit intent
  - branch/preview reuse
  - async-ready result-state vocabulary without implementing async
- one adversarial composed runtime transcript that combines the nastiest
  admitted surfaces in a single test or doc-test-like fixture

Must preserve:

- examples are usage-shape proof, not domain logic inside `worth-query`
- domain semantics stay in adapters, expressions, schemas, and commit
  strategies supplied by test fixtures or domain crates
- transcripts assert proof-bearing artifacts and typed denials, not just
  compile success

Acceptance evidence:

- each transcript maps to executable tests or compile-checked examples
- tests assert meaningful receipts, support posture, authority lanes,
  dependency aspects, residue counters, delivery shape, and inspection output
- every golden transcript has at least one unsupported-neighbor denial row

Forbidden shortcuts:

- examples that bypass the facade to install subscriptions or signal observers
- tests that assert only "it compiled" or "the handle exists"
- domain-specific APIs added to `worth-query`

### Phase 2: Canonical Public Naming And Facade Shape

Purpose:

- settle the public names before downstream runtimes depend on provisional
  implementation names
- align existing runtime methods with the stable concepts: workspace, durable
  surface, handle, state, effect, intent, branch, preview, and inspection

Must ship:

- final naming decision record for:
  - workspace creation
  - live view declaration
  - computed declaration
  - effect declaration
  - branch and preview creation
  - handle reuse
  - state/snapshot/read access
  - intent declaration and commit
  - inspection
- alternate-name or deletion plan for any existing runtime names that remain
  useful internally but should not be the ordinary public story
- facade exports that make the preferred path obvious
- compile-fail or lint-style proof that new ordinary examples do not reach
  lower-runtime internals

Must preserve:

- current proof-bearing runtime APIs remain available where needed for
  crate-internal certification and lower-runtime development
- memory-backed scaffolds remain explicit scaffold posture
- no rename may hide an orchestration boundary behind a cheap-looking method

Acceptance evidence:

- golden transcripts use only final public names
- old ambiguous names are deleted, internalized, or explicitly marked as
  alternate/debt-only seams
- public docs identify the preferred path without relying on prose alone; tests
  enforce shortcut rejection

Forbidden shortcuts:

- one dynamic `workspace.surface(kind, ...)` entrypoint that erases family
  meaning
- one generic `handle.value()` call for work that may cross runtime, branch,
  temporal, async, or inspection boundaries
- "nice" aliases that bypass typed support admission or authority lanes

### Phase 3: Async-Safe State And Boundary Vocabulary

Purpose:

- make sync runtime data safe to consume now without committing the public API
  to a sync-only mental model
- reserve the state and causality vocabulary that 9.4 through 9.7 will extend

Must ship:

- public state/read vocabulary for handle materialization:
  - ready/snapshot state
  - pending/deferred state
  - stale state
  - failure state
  - denied/unsupported state
- explicit method naming rules for cheap local handle access versus boundary-
  crossing read/materialization/observation work
- support metadata rows that distinguish:
  - sync runtime-backed ready state
  - temporal state deferred until 9.4
  - async/resource state deferred until 9.5
  - mixed-cause delivery deferred until 9.6
- typed unsupported-neighbor tests proving async-like requests fail closed
  before 9.5

Must preserve:

- no temporal/async semantics are claimed in this stabilization plan
- state vocabulary is query/result-shape meaning, not host-local loading UI
  convention
- store-backed/durable async replay remains future work

Acceptance evidence:

- public APIs no longer imply that every handle has an always-synchronous raw
  value
- unsupported temporal/async requests return typed support denials with
  inspectable reason and zero delivery residue
- current sync reads continue to be ergonomic through `Ready` fast paths

Forbidden shortcuts:

- `Option<T>` or stringly status as the public state model for future async
  properties
- ambient host timers or host-local promises masquerading as query state
- accepting stale async/resource completions as ordinary updates before 9.5

### Phase 4: Handle Contract And Inspection Freeze

Purpose:

- make handles and `inspect()` the durable trust contract for generated DSL
  output and domain-runtime debugging
- ensure every ordinary facade handle is inspectable enough to explain what
  the runtime installed and why it changed

Must ship:

- public handle contract document or module-level spec for:
  - live handles
  - computed handles
  - effect handles
  - branch/preview-bound handles
  - intent receipts
  - future temporal/async-capable handles
- `inspect()` acceptance matrix for every handle/receipt family the public API
  advertises
- stable inspection sections for:
  - declaration identity
  - dependency/aspect contract
  - authority lane
  - basis lane
  - support posture
  - lifecycle/subscription evidence
  - delivery/pending/residue evidence
  - feedback phase graph
  - temporal/async deferred posture
- tests proving inspection consumes retained artifacts and stable digests
  rather than re-running lowering or scanning broad registries

Must preserve:

- inspection remains read-only and cannot mint proof-bearing artifacts
- diagnostics richness policy cannot change operational meaning
- denied and unsupported cases remain inspectable through failure bundles

Acceptance evidence:

- every golden transcript ends with an inspection assertion that would let a
  developer debug generated code
- inspection identifies lane/basis ambiguity for reused aspect paths
- feedback cycles show phase boundaries rather than untyped dependency edges

Forbidden shortcuts:

- debug strings as the primary inspection contract
- inspection that requires mutable runtime access to explain a retained handle
- inspection that hides branch/preview/effect policy or pending-intent residue

### Phase 5: Support Matrix, Debt, And Future-Async Gates

Purpose:

- freeze what may be considered stable now
- prevent later temporal/async work from adding parallel APIs or silently
  widening the public contract
- make support metadata the gate between stable sync surfaces and deferred
  async/resource surfaces

Must ship:

- public support matrix rows for stable runtime-backed API families
- explicit deferred rows for:
  - temporal basis and time-aware subscriptions
  - async/resource query families
  - mixed truth/time/async delivery
  - temporal/async certification
  - store-backed parity
  - durable reload/restart
- fail-closed support admission for any temporal/async request before its
  milestone closes
- roadmap linkage requiring 9.4 through 9.7 to extend this stabilized facade,
  not introduce a second facade family

Must preserve:

- support metadata and executable admission behavior remain synchronized
- deferred support is honest debt, not implied future support
- runtime-backed stable surfaces do not overclaim store/durable/async behavior

Acceptance evidence:

- beta/stable support matrix enforcement includes the public API stabilization
  rows
- unsupported temporal/async neighbors fail typed and early
- docs and tests agree on exactly which surfaces are stable, deferred, or
  unsupported

Forbidden shortcuts:

- "experimental async" APIs outside the stabilized handle/state model
- support claims inferred from method presence
- future temporal/async milestones adding sibling APIs that bypass handles,
  aspects, lanes, or inspection

### Phase 6: Closeout Certification And Migration Readiness

Purpose:

- prove the stabilized API is ready for serious downstream runtime work
- produce the closeout artifact that geometry/workflow/table runtimes can use
  as their dependency contract

Must ship:

- public API stabilization closeout
- golden transcript certification results
- compatibility/deprecation list for non-preferred names
- migration guidance for downstream runtimes
- explicit "safe to build now" assumptions and "must not assume yet" list

Must preserve:

- no claim that 9.4 through 9.7 temporal/async behavior is implemented
- no claim that store-backed/durable semantics are admitted
- no domain-specific runtime semantics inside `worth-query`

Acceptance evidence:

- full `worth-query` tests pass
- phase-boundary compile-fail suite passes
- golden DX transcript suite passes
- support metadata enforcement suite passes
- closeout self-check answers the stabilization questions explicitly

Forbidden shortcuts:

- closing because examples look nice without hostile proof
- closing while any golden transcript requires lower-runtime plumbing in
  ordinary code
- closing while async-ready vocabulary remains prose-only

## Must Ship

- final public API vocabulary for workspace, durable surfaces, handles,
  state/snapshot/read access, computeds, effects, intents, branch/preview
  reuse, and inspection
- golden DX transcript suite for workflow, geometry, table, and composed
  adversarial runtime surfaces
- async-safe result-state vocabulary and support metadata gates
- handle contract and inspection contract that 9.4 through 9.7 must extend
- alternate-name or deletion plan for implementation-colored or ambiguous
  existing names
- support matrix rows distinguishing stable runtime-backed surfaces from
  deferred temporal/async/store/durable surfaces
- compile-fail shortcut rejection for lower-runtime plumbing, dynamic family
  erasure, and async/temporal pre-claims
- closeout document that downstream domain runtimes can cite as the stable API
  dependency contract

## Must Preserve

- `worth-query` remains domain-neutral; geometry/workflow/table examples are
  pressure tests, not Query-owned semantics
- lower runtimes remain authorities for truth, signal scheduling, bridge
  protocol, temporal execution, async lifecycle, store durability, and commit
  semantics
- facade handles remain proof-bearing and inspectable
- public method names reveal real boundary crossings
- support metadata and executable admission behavior remain synchronized
- temporal/async/store/durable work remains explicit deferred scope until its
  owning milestones close
- memory-backed scaffolds remain named scaffold posture

## Acceptance Evidence

This stabilization plan is complete only when `worth-query` can prove:

- the `Runtime API Golden DX And Async-Safe Facade Test` in
  [test-requirements.md](./test-requirements.md) passes with canonical
  machine-checkable artifacts
- golden transcripts compile and execute through the final public facade
- no golden transcript installs subscriptions, signal observers, bridge
  declarations, grouped baselines, active lanes, or CDC filters manually
- tests assert meaningful receipts, delivery batches, support posture, residue
  counters, authority lanes, dependency aspects, and inspection evidence
- unsupported temporal/async/store/durable neighbors fail typed and early
- support matrix rows identify stable, deferred, and unsupported public API
  families
- inspection explains every advertised handle/receipt family from retained
  artifacts
- public API names do not hide runtime, branch, temporal, async, or diagnostic
  boundary crossings
- phase-boundary compile-fail coverage proves ordinary users cannot bypass the
  facade or synthesize proof-bearing artifacts externally

Required verification commands at closeout:

- `cargo fmt -p worth-query`
- `cargo check -p worth-query --tests`
- `cargo test --manifest-path crates/worth-query/Cargo.toml --test phase_boundaries_compile_fail`
- `cargo test -p worth-query`
- targeted golden transcript tests
- targeted support matrix enforcement tests
- `git diff --check`

## Roadmap Placement

This is a stabilization gate after the runtime API closeout and before
Milestones 9.4 through 9.7.

It belongs here because:

- 5.6 already closed the daily-driver application facade
- 9.1 through 9.3 already closed runtime-backed subscription declaration,
  lifecycle, diagnostics, bridge parity, and certification foundations
- the runtime API batch already consumed those foundations into live,
  computed, effect, branch/preview, intent, delivery, and inspection handles
- temporal/async milestones must now extend the stabilized public model rather
  than discover a second one

It is not a replacement for 9.4 through 9.7. It is the contract those
milestones must respect.

## Architectural Notes

- `Workspace` is a facade context, not a lower-runtime authority.
- `Handle` is the unit of composition and future compatibility.
- `State` is the result/readiness vocabulary that lets synchronous surfaces
  remain ergonomic while async/resource surfaces later fit without changing the
  model.
- `AspectContract` is the dependency, invalidation, policy, and inspection
  contract.
- `AuthorityLane` is the semantic safety boundary between truth, branch,
  preview, derived state, effect state, pending intent, bridge external state,
  and future temporal/async state.
- `Inspection` is part of the product contract, not a debug string layer.
- `SupportMatrix` is the truth source for stable/deferred/unsupported public
  claims.

## Explicit Non-Goals

- implementing temporal query basis semantics from Milestone 9.4
- implementing async/resource query families from Milestone 9.5
- implementing mixed truth/time/async delivery from Milestone 9.6
- certifying temporal/async behavior from Milestone 9.7
- implementing store-backed parity from Milestone 10
- implementing durable restart/reload from Milestone 11
- adding geometry, workflow, table, CAD, spreadsheet, or product-domain
  semantics to `worth-query`
- removing low-level internal APIs needed for certification and lower-runtime
  development

## Self-Check

- Does this plan solve a real structural problem or just package work
  cosmetically? It solves the stability gap between a correct runtime
  foundation and a final public API that downstream runtimes can safely build
  on before async arrives.
- Is the adversarial constraint precise and load-bearing? Yes. It specifically
  forbids sync-shaped API decisions that would break temporal/async extension
  or force domain runtimes to rewrite.
- Does the plan preserve crate authority boundaries? Yes. Query freezes the
  facade, handle, state, aspect, support, and inspection contracts while lower
  runtimes retain truth, signal, bridge, temporal, async, store, and durable
  authority.
- Does the plan define proof obligations, not just implementation tasks? Yes.
  Golden transcript tests, support matrix enforcement, compile-fail shortcuts,
  typed denial, inspection evidence, and exact residue/counter assertions are
  required.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes. Each phase names the surface, forbidden shortcuts, and
  acceptance evidence needed to implement and verify it.
- Does this belong in the roadmap sequence? Yes. It is the stabilization gate
  between closed runtime-backed facade work and the upcoming temporal/async
  milestones that must extend the same public model.
