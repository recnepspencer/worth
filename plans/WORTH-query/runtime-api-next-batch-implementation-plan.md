# WORTH Query Runtime API Next Batch Implementation Plan

> **Parent direction:** WORTH Query Runtime API implementation plan
>
> **Closeout:** [runtime-api-next-batch-closeout.md](./runtime-api-next-batch-closeout.md)
>
> **Purpose:** finish the next runtime facade batch by designing from the
> non-negotiable public developer experience first, then working backward into
> the query, runtime-bridge, signal, relational, preview, derived-computation,
> effect, and intent-commit machinery required to make that surface honest.
>
> **Bias:** build the proper generic infrastructure, not demo-only shortcuts.
> Keep domain semantics out of `worth-query`; domain crates provide schema,
> lowering, derived-view maintenance, and operation meaning through adapters.
> If the desired facade cannot be implemented cleanly, fix the lower runtime
> seam that blocks it. Do not weaken the facade, add host-local hacks, smuggle
> domain semantics into Query, or bypass proof-bearing runtime artifacts.

## Governing Summaries

- `MENTALITY.md`: protects adversarial constraint first, hard foundations
  before convenience, and enforcement over convention. This plan must make the
  ideal DX the constraint and force the runtimes to satisfy it, not relax the
  API until it fits current shortcuts.
- `arch_laws.md`: protects facade-only access, proof-bearing phase chains,
  declaration-owned resource lifecycle, authority/derivation separation, and
  explicit boundary crossings. The runtime facade must consume and produce
  typed artifacts rather than raw callbacks, raw CDC, broad options bags, or
  mutable handles.
- `perf_laws.md`: protects bounded work, visible cost, locality, explicit
  equivalence, and plan-before-execute behavior. The API must not hide broad
  scans, per-view maintenance, host-side delivery reshaping, or diagnostic
  rediscovery behind pleasant method names.
- `domain_laws.md`: protects subsystem responsibility boundaries and facade
  access. Query runtime API work must be decomposed by capability and proof
  phase, not gathered into a single convenience module.
- `worth_query_vision.md`: protects the promise that developers declare query
  intent once and can use the same shape for reads, live promotion, branch
  contexts, derived computation, and query-shaped delivery.
- `worth_query_roadmap.md`: protects the rule `declare query intent once,
  lower it once, execute it against canonical truth`; 9.1 through 9.3 are now
  shipped runtime-backed subscription declaration, lifecycle, diagnostics, and
  bridge-parity foundations this plan must consume.
- `test-requirements.md`: protects certification-grade proof. This plan must
  add tests and artifacts that prove canonical meaning, bridge honesty,
  query-shaped delivery, preview isolation, no silent widening, and support
  metadata honesty.
- `milestone-5.6.md`: protects the daily-driver facade and sectioned runtime
  configuration without erasing lower-crate authority.
- `milestone-9.1.md`: protects query-owned subscription family selection,
  declaration, bridge lowering, and admission as a proof chain before
  activation.
- `milestone-9.2.md`: protects active subscription lifecycle, sharing,
  continuation, preview isolation, and query-shaped delivery from admitted
  activation input only.
- `milestone-9.3.md`: protects automatic subscription diagnostics, bridge
  parity, support reports, and runtime-backed certification for admitted
  subscription families.

## Adversarial Constraint

An ordinary consumer must be able to declare a complicated runtime surface
through one Query facade: schema/invariant-backed truth, branch or preview
basis, live query handles, nested derived computations, conditional expression
nodes, effects, intent commits, query-shaped patch delivery, and inspection.
The facade must stay simple enough that the consumer composes handles rather
than wiring runtimes, while every internal step remains proof-bearing,
authority-preserving, query-shaped, bridge-honest, and counter-visible.

If the implementation cannot make that shape work, the correct response is to
repair the missing runtime capability or adapter seam. The wrong response is
to:

- expose raw RuntimeBridge, Signal, relational, declarative-live, or
  subscription internals to ordinary consumers
- ask consumers to register signal observations, bridge subscriptions, grouped
  baselines, active lanes, or CDC filters manually
- let domain DSLs reshape raw patches into query-shaped delivery
- weaken the public API into current implementation-shaped helpers
- introduce temporary broad refresh, raw callback, or host-local cache behavior
  without explicit debt, denial, and proof artifacts
- put domain-specific semantics in `worth-query`

## Current Baseline

Already landed:

- `WORTHQueryRuntime` exists as the high-level facade.
- `WORTHQueryRuntimeBackend` exists as the first backend seam.
- `WORTHQueryRuntimeBackendParts`, bridge-backed runtime backend assembly, and
  runtime builder seams for schema/source/write/signal adapters exist in code
  and should be treated as the starting point, not as the final DX.
- a memory-backed scaffold backend exists for local/runtime certification, but
  it is not the ordinary architectural path.
- Compiled program effects can bind typed operation inputs into
  `WORTHQueryWriteCommandTemplate`.
- Write receipts expose changed deltas, affected live view ids, affected
  derived view ids, and refresh fallback status.
- Preview direct writes are staged until `promote()`.
- `WORTHQueryArtifactInspector` returns structured inspected artifact handles
  instead of plain labels.
- Query subscription 9.1 through 9.3 artifacts exist: family selection,
  declaration, bridge lowering, admission, active lifecycle, query delivery,
  continuation, preview isolation, support reporting, diagnostics, bridge
  parity, and runtime certification surfaces.

Still open:

- The runtime facade does not yet consume the 9.1-9.3 subscription pipeline
  automatically when declaring live views.
- Live handles do not yet retain the full query/subscription/bridge/signal lane
  identity needed for automatic patch delivery, sharing, and inspection.
- Runtime delivery still has memory-workspace affected-view inference and
  patch-note routing paths that must become query-shaped subscription delivery.
- Derived views are still Query-local routing and maintainer callbacks rather
  than a first-class nested computed surface backed by signal/bridge where
  admitted.
- Effects exist as program/runtime concepts, but the facade does not yet expose
  effect declarations as a normal composable handle surface with typed
  delivery and optional write-intent lowering.
- Preview branches isolate some direct writes, but live declarations,
  subscriptions, derived views, effects, and `run_operation` still need full
  preview-scoped basis and residue handling.
- Intent commits and extensible relational commit strategies are not yet a
  normal Query facade operation shape.
- Artifact inspection is structured but still shallow relative to the desired
  "explain this handle" DX.
- Tests cover pieces of the seam, but not the hardest composed facade shape:
  live + branch/preview + nested computeds + effects + expression conditions +
  intent commits + subscription diagnostics + no silent widening.

## Non-Negotiable Public DX

WORTH Query should feel like handle composition. Ordinary consumers should
declare what they want, receive typed handles, and inspect what the runtime
installed. They should not wire lower runtimes.

```rust
let workspace = query.workspace("workspace-id")?;

let graph_view = workspace.live_view("runtime.graph_view", |q| {
    q.from("RootEntity")
        .where_eq("id", root_id)
        .traverse("child")
        .include_relations(["dependency"])
        .select([
            "display.label",
            "layout.frame",
            "layout.position",
            "validation.state",
            "expression.source",
            "runtime_value.preview",
        ])
        .order_by("layout.position")
        .as_graph_view()
})?;

let field_values = workspace.computed("runtime.field_values", |c| {
    c.from(&graph_view)
        .for_each("Node")
        .using_expression("expression.source")
        .reads(["expression.source", "runtime_value.input"])
        .produces(["runtime_value.preview"])
})?;

let aggregate_validity = workspace.computed("runtime.aggregate_validity", |c| {
    c.from(&graph_view)
        .depends_on(&field_values)
        .for_each("Group")
        .reads(["runtime_value.preview", "validation.rule"])
        .produces(["validation.state"])
})?;

let effect = workspace.effect("ui.validation_badges", |e| {
    e.when(aggregate_validity.changed("validation.state"))
        .deliver("ui.validation_badges")
        .authority_lanes(["authoritative"])
        .meaningful_change()
})?;

let branch = workspace.branch("try-new-shape", |b| {
    b.effects().derive_only()
})?;

let intent_receipt = branch.intent("reconcile_shape", |intent| {
    intent.ensure("Node", node_id)
        .set("display.label", "updated label");
    intent.ensure_relation("child", root_id, node_id);
    intent.remove_dangling_relations("dependency");
})?;

let preview_graph_view = branch.use_view(&graph_view)?;
let merge_plan = branch.plan_merge_to_current()?;
let promotion = branch.promote(merge_plan)?;

let explanation = workspace.inspect(&preview_graph_view)?;
```

This is illustrative API shape, not a domain requirement. The plan must keep
the final naming idiomatic to Rust and existing WORTH Query conventions, but
the DX properties are not negotiable:

- one facade is the ordinary consumer path
- consumers compose typed handles
- schema and invariant declarations are foundational safety, not repeated
  caller ceremony after runtime construction
- named live views are stable application surfaces, not just one-off queries
- live declaration installs subscriptions automatically
- derived computation and effects are declaration-owned resources
- branches and previews reuse the same declared handles under a different
  explicit basis
- writes and intents return receipts that trigger query-shaped updates
- inspection explains what was installed and why it changed
- unsupported combinations fail typed and early, before fallback or broadening

## Lowering Contract

The ideal DX maps backward into one internal proof chain:

1. facade declaration captures query, computed, effect, branch, write, or
   intent meaning as typed input
2. Query canonicalizes, validates, applies policy/schema/aspect narrowing, and
   plans result or operation shape
3. live declarations lower through view-shape/live promotion and then through
   the 9.1 subscription declaration/admission chain
4. active live handles open or join 9.2 active subscription lanes and emit
   query-shaped delivery batches
5. 9.3 support, diagnostic, bridge-parity, and certification artifacts become
   the inspector's source of truth
6. derived computation declarations lower into admitted signal/bridge-backed
   computed resources where supported, or fail/debt explicitly
7. effects lower into phase-typed delivery or write-intent declarations; they
   do not become hidden host callbacks
8. writes and intent commits lower into relational or bridge authority,
   preserve relational invariants, and publish one canonical receipt
9. relational aspect-tagged patches route through RuntimeBridge aspect mapping
   into signal invalidation/observation and then back into query-shaped
   subscription maintenance deltas
10. preview and branch contexts bind every handle to explicit basis,
    subscription, derived, effect, and write-intent isolation artifacts

The executor may not rediscover subscription family, policy masking, basis,
view shape, effect strategy, intent strategy, or delivery shape after these
phases have lowered them.

## Authority Lanes, Effect Policy, And Phase Boundaries

The public API can stay simple only if authority lanes are explicit underneath
it. The same aspect path can appear in multiple lanes with different meaning;
the runtime must not collapse those meanings because their string labels match.
Authority lanes are semantic ownership/basis lanes; they are distinct from
active subscription lanes, which are delivery lifecycle resources.

Required authority lanes:

- `AuthoritativeTruth`
- `BranchLocalTruth`
- `PreviewTruth`
- `DerivedRuntimeState`
- `EffectDeliveryState`
- `PendingWriteIntent`
- `BridgeExternalState` where a bridge/resource integration owns lifecycle

Rules:

- every facade handle, receipt, patch, computed output, effect, and inspection
  bundle must identify its authority lane or basis lane
- Query preserves and exposes lane identity, but it does not steal the lower
  runtime's authority for that lane
- derived output written into truth must cross an explicit write-intent or
  commit-strategy boundary; it cannot silently mutate from
  `DerivedRuntimeState` into `AuthoritativeTruth`
- branch and preview reuse must bind declarations to new lane/basis evidence
  rather than cloning authoritative handles in place
- fields such as `validation.state` or `runtime_value.preview` are ambiguous
  unless their lane is known; inspection must show whether they are
  derived-only, branch-local, preview-only, committed truth, or pending intent
  output

Branch and preview effect policy is mandatory. Reusing a view or computed in a
branch must not imply that all effects fire normally.

Required policy classes:

- `DeriveOnly`: activate subscriptions and computeds; mute delivery and
  write-intent effects
- `Muted`: bind declarations but emit no effect delivery or write-intent work
- `Redirected`: deliver effects to an admitted branch/preview-local target
- `SandboxedWriteIntent`: allow write intents only against preview or
  branch-local authority
- `AuthoritativeAllowed`: allow authoritative delivery or write intent only
  through explicit policy admission

Default branch/preview posture must be conservative: `DeriveOnly` or a stricter
policy. Any effect that can deliver externally or write authoritative truth
must opt into the correct lane and be denied when the active branch/preview
policy does not admit it.

Reactive feedback is allowed only through phase-typed boundaries. This cycle is
valid only when each edge is explicit and inspectable:

```text
truth read -> derive -> effect delivery -> write intent -> commit -> bridge route -> resubscribe
```

Rules:

- computed resources derive state; effects deliver or request writes
- effect-triggered writes must produce loop-prevention, idempotence, and phase
  evidence
- the inspector must show phase graph, authority lanes, effect policy,
  feedback edges, and whether feedback terminated, coalesced, suppressed, or
  denied
- hidden callback loops, host-local equality checks, or effect writes that
  bypass intent/commit authority are out of spec

## Batch 1: Runtime Facade Root And Backend Assembly Hardening

Goal: make the runtime facade the daily-driver entrypoint and make backend
assembly honest enough that later batches can install live, computed, effect,
branch, and intent resources without reaching around Query.

Current code already has part of this seam. This batch hardens it against the
new DX contract rather than treating builder assembly as done merely because a
backend can be built.

Must ship:

- a normative facade root for runtime/workspace operations
- a sectioned runtime configuration path aligned with Milestone 5.6
- authority-lane and effect-policy support metadata for facade families that
  can read, derive, deliver, preview, branch, write, or commit
- backend parts for schema, source, write authority, RuntimeBridge, Signal
  sink/adapter, subscription activation, preview basis, and inspector evidence
- typed backend support metadata for each facade family: read, live, computed,
  effect, branch/preview, write, intent, inspect
- typed denial when a backend cannot support a facade family

Implementation notes:

- Keep `WORTHQueryRuntimeBackend` as the runtime-facing trait, but do not let it
  become an untyped bag of optional hooks.
- Split support into narrow adapter traits or proof-bearing capability structs
  where responsibilities differ.
- Existing `WORTHQueryRuntimeBackendParts` and `WORTHQueryBridgeBackedRuntimeBackend`
  should be extended only when they preserve subsystem ownership.
- Memory-backed paths are scaffold/convenience backends, not the
  architectural source of truth for the API.
- Do not expose domain operation names, domain entity kinds, or DSL-specific
  concepts in these types.

Tests:

- Builder rejects each missing required part independently.
- Support metadata and executable admission behavior agree.
- Memory and fake external backends expose the same facade families where both
  claim support.
- Runtime facade never stores or branches on concrete memory-scaffold backend
  types.
- Unsupported facade families fail before lower-runtime probing or fallback.

## Batch 2: Live Declaration Automatically Installs Query Subscriptions

Goal: `live_view(...)` and existing `declare_live_view(...)` must run the 9.1
through 9.3 subscription proof chain automatically. Consumers declare a live
query; Query installs the subscription, opens or joins the active lane, and
returns a handle.

This is the first non-negotiable DX batch. If live declaration cannot install
subscriptions automatically, fix the runtime seam. Do not ask consumers to call
subscription family selection, bridge lowering, signal observation, grouped
baseline, active lane, or support-report APIs manually.

Required internal lowering:

1. canonical query request
2. schema/policy/aspect validation and narrowing
3. view-shape plan and live promotion artifact
4. `LiveQueryAdmissionArtifact`
5. `QuerySubscriptionFamilySelection`
6. `QuerySubscriptionDeclarationArtifact`
7. `BridgeSubscriptionLoweringPlan`
8. `QuerySubscriptionAdmissionArtifact`
9. `SubscriptionActivationInput`
10. active lane open/join and consumer attachment
11. 9.3 support/diagnostic/bridge-parity evidence retained for inspection

Handle requirements:

- live handles carry query digest, view-shape digest, subscription family
  digest, declaration digest, bridge declaration digest, signal strategy digest,
  active lane digest, consumer attachment digest, basis digest, policy digest,
  authority lane digest, support report digest, and diagnostic bundle digest
  where emitted
- consumers can read snapshots and drain query-shaped patches from the handle
- consumers cannot mutate lane meaning, patch policy, bridge strategy, or
  signal observation through the handle

Tests:

- Equivalent direct/facade-authored live declarations lower to the same
  subscription declaration and support evidence.
- Detail, collection, grouped, and inspector-like admitted shapes install
  distinct query-side subscription families.
- Missing bridge family, unsupported slice, unsupported grouped metadata,
  policy masking drift, or tenant/basis mismatch fails typed before activation.
- No public API path can activate from raw live descriptors, raw CDC filters,
  raw bridge declarations, or host callbacks.
- Inspector can explain the installed subscription without re-running lowering.

## Batch 3: Query-Shaped Delivery, Grouped Baselines, And Patch Draining

Goal: `drain_patches(handle)` must drain query-shaped subscription delivery
batches, not memory-workspace affected-view guesses or raw lower-runtime
events. Grouped baselines and grouped movement must be automatic parts of the
admitted grouped live family.

Implementation notes:

- Replace or bypass affected-view inference with active subscription delivery
  records from the 9.2 runtime.
- Introduce runtime-facing delivery adapters that accept
  `QuerySubscriptionMaintenanceDelta` or bridge/signal-maintenance evidence and
  produce `QueryDeliveryBatch`.
- Grouped live declarations must materialize or admit grouped baseline evidence
  before activation. If the backend cannot do this honestly, deny the grouped
  family.
- Delivery patch families must be typed: detail field patch, inspector focused
  patch, collection membership/order patch, grouped membership patch, bounded
  materialization patch, refresh/gap patch with explicit reason.
- Dense refresh remains typed debt or typed denial unless the family has
  explicit certified refresh semantics.

Tests:

- Relevant aspect update emits a query-shaped patch for affected live handles.
- Irrelevant aspect update emits no delivery.
- Grouped membership movement emits grouped membership/order patches, not raw
  CDC and not host-side regrouping.
- Delivery counters prove patch width scales with changed semantic surface, not
  total rows or total registered views.
- Refresh fallback carries reason, affected handle, basis, and counter evidence.

## Batch 4: Nested Computed And Derived View Handles

Goal: make derived computation a normal facade resource. Consumers declare
computed handles that depend on live handles, other computed handles, query
results, or admitted expression nodes. Query and the lower runtimes own
dependency registration, invalidation, materialization, patching, and fallback.

Target API properties:

- `computed(name, declaration)` returns a typed handle
- computed declarations can depend on live views and other computed handles
- dependency aspects are declared structurally
- outputs declare produced aspects/result shape
- produced outputs declare whether they remain `DerivedRuntimeState` or must
  cross a later write-intent boundary before becoming truth
- patch delivery is query-shaped or computed-result-shaped
- unsupported dependency combinations fail typed and early

Implementation notes:

- Keep `WORTHQueryDerivedView` domain-neutral.
- Replace patch-note routing with an admitted computed resource lifecycle:
  declaration, dependency admission, signal/bridge strategy lowering, active
  materialization, patch delivery, closeout.
- Maintainers or expression evaluators may be supplied by domain crates, but
  they produce desired output or declared effects; Query/runtime own diff,
  routing, fallback, and receipts.
- Nested computed dependencies must use handle/digest equivalence, not host
  callback identity.
- Computed handles must carry authority-lane evidence for every produced
  aspect, because a produced path is not enough to know whether the output is
  derived-only, branch-local, preview-only, or committed truth.
- Current aspect scanning in `route_derived_view_patches` is temporary debt and
  must either be replaced or explicitly contained behind a `DebtExplicit`
  support posture.

Tests:

- computed A depends on live view; computed B depends on computed A; a relevant
  write updates both in deterministic order.
- irrelevant aspect updates do not trigger computed patches.
- maintainer-requested refresh is visible as a typed fallback artifact.
- cyclic computed dependencies reject before registration.
- branch/preview use of computed handles binds to the branch/preview basis
  rather than authoritative state.

## Batch 5: Effects And Conditional Expression Nodes

Goal: effects and conditional expression nodes become declaration-owned runtime
resources, not hidden host callbacks embedded inside computed closures or UI
code. They may deliver query-shaped output, trigger admitted write intents, or
record diagnostics, but they do not own truth.

Target API properties:

- `effect(name, declaration)` returns a handle
- effect declarations consume query/computed handle changes, declared aspects,
  and explicit conditions
- conditional nodes use admitted expression evaluators with deterministic input
  contracts
- effects choose an admitted delivery class or write-intent class
- effects declare admitted authority lanes and branch/preview policy behavior
- every effect can be inspected, closed, branch-bound, preview-bound, or denied
  as unsupported

Implementation notes:

- Separate conditional expression evaluation from effect delivery.
- Expression evaluators are adapters with deterministic descriptors, allowed
  reference declarations, input aspect declarations, failure taxonomy, and
  counters.
- Effects lower into signal observation or bridge delivery only through typed
  declarations; no raw callback becomes authority.
- Write-intent effects must lower through the same intent/commit strategy path
  as explicit user intents, with loop-prevention and idempotence evidence.
- Meaningful-change suppression uses signal/query support where admitted; it
  must not be host-local equality checks hidden behind callbacks.
- Effect execution must record phase evidence from trigger to delivery or
  write intent. Feedback cycles must terminate, coalesce, suppress, or deny
  with typed loop-prevention evidence.
- Branch/preview effect policy must be evaluated before effect activation; an
  effect admitted on authoritative truth may not fire in preview unless its
  lane and policy explicitly allow it.

Tests:

- conditional expression node reads declared inputs and produces a typed output
  patch.
- expression failure emits typed failure without corrupting computed state.
- meaningful-change effect suppresses irrelevant churn and exposes suppression
  counters.
- write-intent effect lowers into bridge/relational authority and prevents
  writeback loops.
- preview branch with `DeriveOnly` policy activates computed dependencies but
  mutes or redirects external delivery/write-intent effects.
- host callback, raw function pointer, or raw JSON effect registration cannot
  become a public authority path.

## Batch 6: Branch And Preview Isolation For Every Facade Handle

Goal: branch and preview contexts must be able to reuse live, computed, effect,
write, and intent declarations under an explicit basis without leaking into the
authoritative runtime. Discard and promotion must prove residue and authority
boundaries.

Implementation notes:

- Add a branch/preview handle layer that binds existing declarations to an
  explicit branch or preview basis.
- Add branch/preview effect-policy admission; default policy must be
  conservative and must not allow authoritative side effects implicitly.
- `branch.use_view(handle)`, `branch.use_computed(handle)`,
  `branch.use_effect(handle)`, and branch-scoped writes/intents must consume
  proof-bearing basis artifacts.
- Preview live subscriptions must use bridge/query preview subscription
  typestates and residue proof, not in-place mutation of authoritative lanes.
- Preview computed/effect resources must close with typed residue artifacts.
- Promotion consumes relational/bridge preview promotion evidence and creates
  new authoritative receipts, handles, or continuations as appropriate.
- Discard proves zero authoritative residue for routing, subscription,
  delivery, derived/computed state, effect state, diagnostics, and writeback.
- Effect delivery state and write-intent pending state must remain separate
  residue classes from derived runtime state and subscription state.

Tests:

- authoritative live/computed/effect handles are unchanged by preview writes
  before promotion.
- preview live patches deliver only to preview-bound handles.
- preview computed and effect outputs are discarded without authoritative
  residue.
- branch/preview effect policies distinguish derive-only, muted, redirected,
  sandboxed write intent, and authoritative-allowed behavior.
- promotion rejects stale authoritative basis before relational/bridge
  authority execution.
- promotion does not mutate preview active lanes in place; it mints explicit
  authoritative evidence.

## Batch 7: Intent Commits And Extensible Commit Strategy Facade

Goal: make desired-state operations a first-class Query facade surface that
lowers into relational extensible commit strategies or bridge writeback
authority without Query becoming a mutation engine.

Target API properties:

- `write(...)` remains the direct mutation command path
- `intent(name, declaration)` expresses desired state, reconciliation, or
  strategy input
- intent execution returns an authority receipt with canonical changed
  surfaces and query-delivery effects
- branch and preview contexts can run intents under their own basis
- effect-triggered write intents use the same path as user-authored intents
- intent declarations state which authority lane they target and whether they
  are allowed from branch, preview, derived, or effect-triggered contexts

Implementation notes:

- Query owns intent declaration shape, input binding, facade-level admission,
  and inspection.
- `worth-relational` owns commit strategy registration, execution, invariant
  validation, commit publication, history, replay, and truth patches.
- RuntimeBridge owns writeback admission, idempotence, loop prevention, and
  truth/signal handoff where the intent crosses that boundary.
- Intent commit receipts must expose strategy identity, input digest, produced
  mutation digest, invariant result, commit/patch identity, basis, authority
  lane transition, and affected query handles.
- Unsupported strategies, missing invariant authority, nondeterministic
  descriptors, or preview/branch basis mismatches fail before commit
  execution.
- A derived output cannot become committed truth by being named in an intent;
  the receipt must prove the explicit lane transition from derived/effect or
  preview/branch state into the lower runtime authority that accepted it.

Tests:

- direct write and strategy-backed intent both publish canonical receipts and
  route query-shaped patches.
- intent commit that violates relational invariants fails without partial
  publication.
- idempotent intent produces no mutation and still emits an inspectable
  receipt.
- effect-triggered intent cannot loop indefinitely and exposes loop-prevention
  evidence.
- replay or independent reference lane can verify strategy identity and
  outcome digest, with a produced mutation digest only for mutating executions
  where the lower runtime supports it.

## Batch 8: Artifact Inspector Becomes Handle Explanation

Goal: `inspect(handle_or_receipt)` should explain the whole hidden runtime path
without exposing mutable internals or requiring consumers to know which lower
runtime owns each phase.

Inspected views must include:

- authority lane and basis-lane evidence for every inspected handle, patch,
  computed output, effect, intent, and receipt
- canonical query and result-shape artifacts
- schema/policy/aspect narrowing artifacts
- live view shape and live promotion artifacts
- 9.1 subscription declaration, bridge lowering, admission, and support
  artifacts
- 9.2 active lane, consumer attachment, delivery, continuation, preview, and
  closeout artifacts
- 9.3 diagnostic, bridge-parity, support, and certification artifacts
- computed/derived dependency and materialization artifacts
- effect and expression artifacts
- write, writeback, intent, commit-strategy, invariant, and receipt artifacts
- branch/preview basis, residue, promotion, and comparison artifacts
- effect policy, feedback phase graph, loop-prevention, idempotence, and
  suppression artifacts
- counters, fallback/debt posture, denial class, and support matrix evidence

Implementation notes:

- Inspector views are read-only and cannot construct proof-bearing artifacts.
- Diagnostics richness may vary by policy, but changing diagnostics richness
  must not change runtime meaning.
- Inspection must consume retained artifacts and stable digests, not re-run
  lowering or scan live runtime registries unless explicitly marked as debt.
- Inspection must make ambiguous aspect paths legible by showing whether each
  inspected value is authoritative truth, branch-local truth, preview truth,
  derived runtime state, effect delivery state, pending write intent, or bridge
  external state.
- Inspection must show feedback loops as phase graphs rather than flattening
  them into ordinary dependency edges.
- The inspector should be the primary way to debug generated DSL output and
  runtime facade behavior.

Tests:

- inspecting a live handle reconstructs the subscription proof chain.
- inspecting a computed handle shows dependency handles and aspect contracts.
- inspecting an effect handle shows trigger, condition, delivery/write-intent,
  authority lane, effect policy, suppression, and loop-prevention posture.
- inspecting an intent receipt shows strategy, invariant, commit, bridge, and
  patch delivery evidence, including authority lane transition.
- inspecting a feedback path shows `truth read -> derive -> effect delivery ->
  write intent -> commit -> bridge route -> resubscribe` phase evidence or a
  typed denial/suppression/coalescing reason.
- denied and unsupported cases produce sufficient failure bundles without
  requiring hidden runtime access.

## Batch 9: Surface Hardening, Migration, And Shortcut Rejection

Goal: make the new runtime API the preferred path and prevent app/demo/DSL code
from depending on lower-level facades for ordinary work.

Implementation notes:

- Keep any memory-backed helper path as an explicit scaffold over the hardened
  backend and subscription path where possible.
- Update demos and internal consumers to use only runtime/workspace facade
  families for ordinary read/live/computed/effect/branch/write/intent/inspect
  work.
- Add compile-fail or lint-style tests for public shortcut traps:
  raw CDC subscription, raw bridge declaration activation, raw signal observer
  registration, host callback effect authority, grouped baseline injection,
  raw preview lane mutation, direct mutable active lane access, and untyped
  intent strategy execution, derived-to-truth mutation without write-intent
  authority, and preview effects with implicit authoritative delivery.
- Keep low-level modules available for crate-internal certification and
  advanced lower-runtime work, but docs must identify the runtime facade as the
  ordinary consumer path.
- Any remaining implementation-shaped API must be marked compatibility or
  explicit debt.

Tests:

- `cargo check -p worth-query -p worth-ui`
- runtime facade tests pass
- demos compile without direct low-level Query/Bridge/Relational/Signal calls
  for ordinary runtime use
- existing workflow/writeback/live/view-shape/subscription tests remain passing
- shortcut compile-fail tests prove the non-negotiable DX is enforced rather
  than merely documented

Closeout evidence added during Batch 9 hardening:

- Effect and intent lane override methods are runtime-internal, with
  compile-fail fixtures for public policy/source/target lane overrides.
- Preview and branch options expose only named safe effect policies; public
  callers cannot construct an implicit authoritative preview policy.
- Preview and branch session constructors now return typed support denials
  instead of panicking when the backend does not admit the branch/preview
  facade family.
- Memory-backed runtime assembly is marked as a scaffold backend, and the
  old `in_memory_collections` builder methods are deleted with compile-fail
  coverage proving those constructor names are missing.
- WORTH UI's todo workspace uses the runtime facade over an explicit
  memory-backed scaffold, with a regression covering read/live/write behavior.
- Grouped baseline helpers are not exported through the ordinary facade.
- Intent execution artifacts cannot be spent as commit receipts or inspected as
  admitted runtime receipts without going through runtime admission.
- Derived-runtime-to-authoritative-truth mutation shortcuts are blocked by the
  sealed intent source-lane boundary.
- Existing subscription and active-lane compile-fail fixtures cover raw CDC
  subscription, raw bridge activation, host observer callbacks, direct active
  lane mutation, raw preview lane sharing, and lifecycle closeout shortcuts.

## Suggested Execution Order

1. Runtime facade root and backend assembly hardening.
2. Live declaration automatically installs query subscriptions.
3. Query-shaped delivery, grouped baselines, and patch draining.
4. Nested computed and derived view handles.
5. Effects and conditional expression nodes.
6. Branch and preview isolation for every facade handle.
7. Intent commits and extensible commit strategy facade.
8. Artifact inspector as handle explanation.
9. Surface hardening, migration, and shortcut rejection.

This order keeps the runtime seam honest first, then installs the 9.1-9.3 live
subscription machinery into the facade, then expands the same handle/composition
model across computed values, effects, preview, intents, and inspection.

## Acceptance Criteria

- Consumers can build a runtime from memory collections or explicit backend
  parts, with support metadata matching executable admission behavior.
- Live views declared through the facade automatically lower through the
  9.1-9.3 subscription chain and return handles with inspectable proof.
- Patches drained from live handles are query-shaped delivery batches, not raw
  CDC or host-reshaped events.
- Grouped live views declare through the facade with no caller-owned grouped
  baseline.
- Computed/derived handles support nested dependencies and aspect-scoped
  invalidation without broad runtime scans or host callback authority, and
  carry authority-lane evidence for produced outputs.
- Effects and conditional expression nodes are declaration-owned resources with
  typed delivery, typed write-intent lowering, branch/preview effect policy,
  loop-prevention, and inspection.
- Preview and branch contexts isolate live, computed, effect, write, and intent
  behavior until explicit promotion, and do not allow authoritative effects by
  default.
- Intent commits lower into relational or bridge authorities and preserve
  relational invariants, strategy identity, authority-lane transitions, commit
  receipts, and query delivery.
- Inspector views explain handles and receipts across query, bridge, signal,
  relational, preview, effect, intent, authority-lane, and phase-graph
  artifacts.
- Unsupported or out-of-order combinations fail typed and early with
  diagnostic bundles; no silent widening, raw CDC fallback, host callback
  authority, or facade weakening is allowed.
- No domain-specific concepts are added to `worth-query`.

## Non-Goals For This Batch

- Durable store-backed execution or restart-stable subscription replay.
- Product- or domain-specific semantics inside Query.
- Full worth-schema or worth-topo migration.
- A new policy engine.
- A new UI redesign.
- Public removal of low-level internal modules used for certification or
  lower-runtime development.
- Temporal/async query semantics from Milestones 9.4-9.7.

## Self-Check

- This plan solves a real structural problem: it makes the ideal facade the
  constraint and maps it to existing 5.6 and 9.1-9.3 proof chains.
- The adversarial constraint is load-bearing: it forbids weakening the facade
  when lower-runtime wiring is hard.
- Authority boundaries are preserved: Query owns declaration, lowering,
  result shape, facade handles, and inspection; relational owns truth and
  invariants; signal owns computation execution; RuntimeBridge owns cross-
  runtime routing, writeback, subscription protocol, and aspect mapping.
- Proof obligations are explicit: support metadata, typed denials, compile-fail
  shortcut rejection, query-shaped delivery, preview residue, and inspection
  bundles are required.
- A competent engineer can map each batch to types, modules, adapters, and
  tests without inventing domain semantics.
- The plan belongs here because it is the runtime API hardening pass that must
  consume completed 9.1-9.3 subscription work before 9.4-9.7 temporal/async and
  Milestone 10 store-backed execution build on top.
