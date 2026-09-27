# Milestone 9.4 Engineering Spec: Runtime-Backed Temporal And Async Query Surface

> **Status:** Draft rewrite
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Primary predecessors:** [milestone-9.3.7.md](./milestone-9.3.7.md), [milestone-9.3.8.md](./milestone-9.3.8.md), [runtime-api-public-stabilization-plan.md](./runtime-api-public-stabilization-plan.md), [runtime-authoritative-mutation-evidence-plan.md](./runtime-authoritative-mutation-evidence-plan.md), and [milestone-17.md](../worth-runtime-bridge/milestone-17.md)
>
> **Purpose:** replace the old `9.4` through `9.7` split with one Query milestone whose internal phases project Bridge Milestone 17's temporal/async basis, causality, mixed-cause ordering, restart posture, and certification law into one public Query product surface before store-backed and durable work continue.

## Goal

Make temporal wakes, async/resource lifecycle, and mixed truth/time/async
delivery part of one canonical Query surface so application code can declare,
admit, inspect, execute, and consume those families through Query's existing
platform-entry/runtime contracts without introducing sidecar APIs, ambient
timer folklore, host-local async state conventions, or a second causality law
above the bridge.

## Why This Milestone Exists

Bridge Milestone 17 is nearly the full lower-authority closure for
temporal/async semantics. It freezes:

- temporal bridge basis binding
- time-aware subscription admission
- time-only cause routing
- historical temporal replay posture
- async source declaration binding
- async completion causality
- mixed-cause ordering
- restart/resume basis
- preview residue and discard/promotion boundaries
- offline diagnostics and certification bundles

Query now has to do the product-surface work honestly.

Without this milestone:

- downstream domains would have to reach around Query to author temporal or
  async behavior
- app surfaces would invent local pending/fulfilled/stale/cancelled meaning
  that drifts from Bridge completion law
- time-only changes would be treated as diagnostics noise or fake truth patches
- `worth-server` would later inherit half-Query, half-host delivery semantics
  instead of one typed contract
- certification would fragment into separate temporal, async, and mixed-cause
  stories that can each pass locally while drifting together

This milestone therefore belongs immediately after runtime API stabilization
and the authoritative mutation evidence gate. Those gates freeze the ordinary
Query runtime facade and evidence story first; this milestone extends that same
public model into temporal and async execution instead of creating a parallel
surface.

## Governing Summaries

- `MENTALITY.md`: protect the hostile cross-runtime truth first. The hard part
  is not adding temporal builders; it is preventing Query from reopening
  bridge-closed basis and causality questions.
- `arch_laws.md`: Query must expose one facade, consume lowered plans, keep
  boundary crossings explicit, and preserve lower-crate authority. Query owns
  product projection, not clock truth, async lifecycle truth, or mixed-cause
  authority.
- `composition_laws.md`: this milestone cannot be a bucket called "temporal
  async support." Temporal declaration, admission, delivery, result-state,
  ordering, remask, inspection, and certification are separate
  responsibilities and need separate phase homes.
- `domain_structure_laws.md`: temporal basis, async result-state, mixed-cause
  delivery, support posture, and inspection artifacts need explicit Query homes
  rather than disappearing into generic runtime helpers.
- `perf_laws.md`: lowering, admission, and delivery must stay bounded by the
  declared cause set, basis width, subscription width, and materialized result
  shape. Query must not introduce broad rescans or host-local recomputation
  just because time or async is involved.
- `AI_README.md`: Query owns named public categories such as public runtime
  facade, support/admission, configured domain handles, signal compatibility,
  continuation, cross-runtime causal inspection, projection consumption, and
  writes/intents. This milestone should extend those categories, not invent a
  parallel temporal/async taxonomy beside them.
- `worth_query_roadmap.md`: `9.4` belongs after the public runtime and
  mutation-evidence gates because Query must extend one stable ordinary facade.
  It must prove runtime-backed temporal query basis semantics and time-aware
  subscription lowering first, then async families, mixed-cause delivery, and
  hostile certification inside one merged milestone.
- `milestone-17.md`: the most important thing to protect is that Bridge already
  owns temporal/async lower-authority law. Query must project temporal basis,
  async identity, completion causality, mixed-cause ordering, restart posture,
  and diagnostics; it must not redefine them.

## Adversarial Constraint

For the same canonical Query declaration, truth-view basis, temporal posture,
tenant/policy context, preview posture, and async source family, Query must
produce the same admitted Query basis, the same result-state meaning, the same
mixed-cause delivery ordering, and the same explanation artifacts regardless of
whether the observed change came from a relational truth patch, a time-only
wake, an async completion, a retry/revalidation path, replay, restart/resume,
or preview promotion/discard.

This milestone fails if any supported path:

- lets Query invent temporal or async basis identity that is not derived from
  Bridge-native artifacts
- treats time-only changes as fake truth changes or async completions as
  transport-local events
- makes result-state meaning depend on UI convention, adapter callback order,
  or host runtime memory
- lets policy or tenant remask happen after temporal or async materialization
  rather than before it
- changes mixed-cause delivery meaning under host event reordering
- or requires downstream domains or `worth-server` to reopen lower-authority
  causality questions that Bridge already closed

## Product Decision Lock

- This milestone absorbs the old roadmap Milestones `9.4`, `9.5`, `9.6`, and
  `9.7` into one Query milestone with internal phases.
- Query remains the public product facade and projection layer. It does not
  become the owner of clocks, wake scheduling, retry policy, async lifecycle,
  branch truth, restart truth, or replay truth.
- `worth-runtime-bridge` remains authoritative for temporal bridge basis,
  async request identity, completion causality, mixed-cause ordering,
  restart/resume basis, preview residue law, and offline certification bundle
  shape.
- `worth-signal` remains authoritative for temporal eligibility, previous-value
  semantics, wake readiness, async lifecycle, retry, timeout, cancellation,
  supersession, and revalidation policy.
- `worth-relational` remains authoritative for truth-view basis, branches,
  snapshots, historical identity, retained history, and preview truth.
- Query must expose temporal and async behavior through the same stabilized
  `Workspace` / `Handle` / `State` / `Inspection` world rather than through a
  sidecar API family.
- Runtime-backed first ship is allowed. Durable temporal replay, persisted
  async continuation, and restart-stable saved artifacts remain deferred to
  Milestones `10` and `11`.

## Phase Plan

### Phase 1: Public Runtime Facade Extension Boundary

Freeze how temporal and async families appear in the stabilized runtime-backed
facade so `workspace`, retained handles, state snapshots, and inspection all
extend one existing public runtime world rather than growing a sibling
"temporal runtime" or "async runtime" API family.

**Relevant subsystems**
- `worth-query` public runtime facade and handle contract
- `worth-query` runtime-backed support posture
- `worth-runtime-bridge` temporal/async basis and lifecycle artifacts

**Relevant Query docs**
- [Workspace Overview](../../crates/worth-query/docs/foundations/workspace-overview.md)
- [Support Matrix And Admission](../../crates/worth-query/docs/foundations/support-matrix-and-admission.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [runtime/workspace.rs](../../crates/worth-query/src/runtime/workspace.rs)
- [runtime/public_api.rs](../../crates/worth-query/src/runtime/public_api.rs)
- [runtime/handle_contract.rs](../../crates/worth-query/src/runtime/handle_contract.rs)
- [runtime/state.rs](../../crates/worth-query/src/runtime/state.rs)
- [runtime/inspection.rs](../../crates/worth-query/src/runtime/inspection.rs)

**Relevant APIs**
- `runtime.workspace(...)`
- `workspace.state(...)`
- `workspace.inspect(...)`
- `workspace.public_api_contract()`
- `workspace.public_handle_contract()`

**Warnings**
- Do not add separate temporal or async facade roots.
- Do not let method presence imply support before support rows exist.

**Test requirements**
- Add a public-facade parity test proving temporal/async-capable handles use the same retained-handle and inspection contracts as ordinary runtime-backed handles.
- Add a compile-fail/export test proving external code cannot construct temporal/async facade artifacts without the public workspace path.

**Engineering decisions**
- Temporal/async work extends the stabilized ordinary facade.
- The public handle contract must carry temporal/async posture explicitly where meaning changes.

**Open questions**
- None.

### Phase 2: Support Matrix And Family Admission Boundary

Freeze the support contract for temporal and async neighbors so every visible
family is row-addressable, typed, and fail-closed before downstream code can
teach it as a production surface.

**Relevant subsystems**
- `worth-query` support matrix and family admission
- `worth-query` runtime support profiles
- `worth-runtime-bridge` capability availability and deferred-debt posture

**Relevant Query docs**
- [Support Matrix And Admission](../../crates/worth-query/docs/foundations/support-matrix-and-admission.md)
- [Downstream Runtime Integration](../../crates/worth-query/docs/foundations/downstream-runtime-integration.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [runtime/support_matrix.rs](../../crates/worth-query/src/runtime/support_matrix.rs)
- [runtime/support/mod.rs](../../crates/worth-query/src/runtime/support/mod.rs)
- [runtime/support/profile.rs](../../crates/worth-query/src/runtime/support/profile.rs)

**Relevant APIs**
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`
- `WORTHQueryRuntimeFacadeFamily`
- `WORTHQueryRuntimeFamilySupportStatus`

**Warnings**
- Do not rely on autocomplete as support posture.
- Do not let temporal/async rows remain "visible but undescribed."

**Test requirements**
- Add matrix-row certification for `Temporal` and `AsyncResource` families with exact support status, fail-closed posture, and sibling-API-forbidden posture.
- Add admission-denial tests proving deferred or unsupported temporal/async families deny through the same typed family-admission path as other public rows.

**Engineering decisions**
- Temporal/async support posture is a first-class runtime-facade contract, not a narrative note in docs.
- The matrix must differentiate visible vocabulary from admitted runtime-backed support.

**Open questions**
- None.

### Phase 3: Configured Domain Handle And Operating Context Boundary

Extend configured domain handles so a downstream domain's operating context can
carry temporal/async capability requirements, basis posture, and continuation
readmission posture without falling back to host-local context objects or raw
runtime ids.

**Relevant subsystems**
- `worth-query` platform entry and configured-domain-handle lifecycle
- `worth-query` continuation execution readmission observation
- `worth-query` domain support snapshots

**Relevant Query docs**
- [Platform Entry](../../crates/worth-query/docs/domain-capabilities/platform-entry.md)
- [Configured Domain Handles](../../crates/worth-query/docs/domain-capabilities/configured-domain-handles.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [application/domain_handle/mod.rs](../../crates/worth-query/src/application/domain_handle/mod.rs)
- [application/domain_handle/operating_context.rs](../../crates/worth-query/src/application/domain_handle/operating_context.rs)
- [application/domain_handle/admitted_world_basis.rs](../../crates/worth-query/src/application/domain_handle/admitted_world_basis.rs)
- [application/domain_entry/support_snapshot.rs](../../crates/worth-query/src/application/domain_entry/support_snapshot.rs)

**Relevant APIs**
- `WORTHQueryDomainOperatingContext`
- `WORTHQueryDomainEntryRoot::with_operating_context(...)`
- `WORTHQueryAdmittedConfiguredDomainHandle`
- `continuation_execution_readmission_observation(...)`

**Warnings**
- Do not smuggle temporal/async capability needs through ambient domain context.
- Do not let configured handles hide temporal/async drift that affects continuation execution.

**Test requirements**
- Add configured-domain-handle admission tests proving temporal/async capability requirements deny early when support snapshots do not admit them.
- Add readmission observation tests proving operating contexts can report basis drift for temporal/async continuations through the standard handle lifecycle.

**Engineering decisions**
- Temporal/async operating posture belongs in configured handles, not in ad hoc runtime helper state.
- Downstream domains keep identity; Query keeps lifecycle and admission.

**Open questions**
- None.

### Phase 4: Temporal Declaration Vocabulary Boundary

Define the canonical Query declaration families for stale-after, interval,
deadline, rolling/sliding window, and equivalent time-aware read meaning so
time-aware intent lives inside Query declaration identity rather than in UI
timers or host observer setup.

**Relevant subsystems**
- `worth-query` declaration grammar and family taxonomy
- `worth-query` live-read and subscription-oriented declaration entry
- `worth-runtime-bridge` temporal basis and temporal subscription families

**Relevant Query docs**
- [Query Expressions And Result Shapes](../../crates/worth-query/docs/authoring/query-expressions-and-result-shapes.md)
- [Declaration Family Taxonomy](../../crates/worth-query/docs/domain-capabilities/declaration-family-taxonomy.md)
- [Canonical Domain Declarations](../../crates/worth-query/docs/domain-capabilities/canonical-domain-declarations.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [runtime/workspace_declaration.rs](../../crates/worth-query/src/runtime/workspace_declaration.rs)
- [application/declaration_entry_orchestration/grammar.rs](../../crates/worth-query/src/application/declaration_entry_orchestration/grammar.rs)
- [application/declaration_entry_seam/mod.rs](../../crates/worth-query/src/application/declaration_entry_seam/mod.rs)

**Relevant APIs**
- declaration-entry inputs and canonical declaration artifacts
- family taxonomy and canonical declaration digest surfaces

**Warnings**
- Do not model temporal intent as observer options layered after declaration.
- Do not allow two syntactic forms to mean different canonical time families accidentally.

**Test requirements**
- Add declaration normalization tests proving equivalent temporal authoring forms lower to one canonical declaration identity.
- Add hostile declaration-drift tests proving time posture changes mutate declaration meaning explicitly rather than patching metadata post hoc.

**Engineering decisions**
- Temporal intent is declaration-native.
- Rolling, deadline, and freshness families are query meaning, not transport hints.

**Open questions**
- None.

### Phase 5: Async Declaration Vocabulary Boundary

Define the canonical Query declaration families for async/resource-backed
neighbors so request identity, loading posture, failure semantics, and optional
temporal participation begin inside Query's declaration model rather than in
component-local status enums or transport adapters.

**Relevant subsystems**
- `worth-query` declaration grammar and family taxonomy
- `worth-query` configured domain handles and declaration evidence
- `worth-runtime-bridge` async source declaration and request identity

**Relevant Query docs**
- [Query Expressions And Result Shapes](../../crates/worth-query/docs/authoring/query-expressions-and-result-shapes.md)
- [Configured Domain Handles](../../crates/worth-query/docs/domain-capabilities/configured-domain-handles.md)
- [Declaration Family Taxonomy](../../crates/worth-query/docs/domain-capabilities/declaration-family-taxonomy.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [runtime/workspace_declaration.rs](../../crates/worth-query/src/runtime/workspace_declaration.rs)
- [application/domain_handle/admitted_handle/declaration.rs](../../crates/worth-query/src/application/domain_handle/admitted_handle/declaration.rs)
- [application/declaration_entry_orchestration/grammar.rs](../../crates/worth-query/src/application/declaration_entry_orchestration/grammar.rs)

**Relevant APIs**
- configured-handle declaration entrypoints
- canonical declaration artifacts and family taxonomy surfaces

**Warnings**
- Do not collapse every async shape into a single generic "resource query."
- Do not let adapter identity become declaration identity.

**Test requirements**
- Add declaration parity tests proving equivalent async authoring lowers to one canonical declaration family and one request-identity basis.
- Add hostile source-family mismatch tests proving source changes force new declaration identity rather than mutating existing declarations.

**Engineering decisions**
- Async/resource declaration families are canonical Query families.
- Query declaration identity must carry optional temporal participation explicitly when admitted.

**Open questions**
- None.

### Phase 6: Temporal Legality And Readiness Projection Boundary

Project Bridge temporal admission law into Query legality and declaration-entry
readiness so unsupported truth-basis, temporal-family, and preview-family
combinations deny before route planning, activation, or materialization.

**Relevant subsystems**
- `worth-query` declaration legality and readiness projection
- `worth-query` support reports and declaration-entry inspection
- `worth-runtime-bridge` temporal admission and wake-evidence requirements

**Relevant Query docs**
- [Declaration Legality](../../crates/worth-query/docs/domain-capabilities/declaration-legality.md)
- [Declaration Entry Readiness](../../crates/worth-query/docs/domain-capabilities/declaration-entry-readiness.md)
- [Support Matrix And Admission](../../crates/worth-query/docs/foundations/support-matrix-and-admission.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [application/declaration_entry_orchestration/lower/legality.rs](../../crates/worth-query/src/application/declaration_entry_orchestration/lower/legality.rs)
- [application/declaration_entry_seam/readiness_projection.rs](../../crates/worth-query/src/application/declaration_entry_seam/readiness_projection.rs)
- [application/declaration_entry_seam/inspection/mod.rs](../../crates/worth-query/src/application/declaration_entry_seam/inspection/mod.rs)

**Relevant APIs**
- readiness projection and declaration-entry inspection artifacts
- legality and denial projection surfaces

**Warnings**
- Do not let temporal illegality degrade into non-temporal execution.
- Do not postpone temporal support posture until activation.

**Test requirements**
- Add readiness-projection tests proving every unsupported temporal combination denies before route planning.
- Add hostile preview/historical temporal readiness tests proving unsupported retained-basis combinations remain typed and localized.

**Engineering decisions**
- Temporal readiness is a declaration-entry artifact, not an activation-only surprise.
- Query owns user-facing legality/readiness explanation; Bridge owns the lower law being projected.

**Open questions**
- None.

### Phase 7: Async Legality And Readiness Projection Boundary

Project Bridge async source-admission and lifecycle-admission law into Query
legality and declaration-entry readiness so invalid source families, preview
posture mismatches, and unsupported lifecycle regimes deny before active
subscription or materialized result-state exists.

**Relevant subsystems**
- `worth-query` declaration legality, readiness, and support matrix
- `worth-runtime-bridge` async source admission and request identity binding
- `worth-signal` async capability family substrate

**Relevant Query docs**
- [Declaration Legality](../../crates/worth-query/docs/domain-capabilities/declaration-legality.md)
- [Declaration Entry Readiness](../../crates/worth-query/docs/domain-capabilities/declaration-entry-readiness.md)
- [Support Matrix And Admission](../../crates/worth-query/docs/foundations/support-matrix-and-admission.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [application/declaration_entry_orchestration/lower/legality.rs](../../crates/worth-query/src/application/declaration_entry_orchestration/lower/legality.rs)
- [application/declaration_entry_seam/readiness_projection.rs](../../crates/worth-query/src/application/declaration_entry_seam/readiness_projection.rs)
- [runtime/support_matrix.rs](../../crates/worth-query/src/runtime/support_matrix.rs)

**Relevant APIs**
- readiness projection
- support snapshot and declaration-entry denial artifacts

**Warnings**
- Do not allow async source families to drift into host-local fallback.
- Do not let preview mismatch surface only after completion arrives.

**Test requirements**
- Add async readiness denial tests proving unsupported source families and preview posture mismatches deny before route planning.
- Add hostile request-basis incompatibility tests proving basis mismatch is a readiness failure, not a late completion artifact.

**Engineering decisions**
- Async legality and readiness project before execution, not during callback handling.
- Async declaration readiness remains part of one Query declaration-entry seam.

**Open questions**
- None.

### Phase 8: Query Basis Lifecycle Projection Boundary

Project Bridge temporal basis, async request basis, replay basis, and preview
basis into Query's basis capability lifecycle so observation, execution,
inspection, materialization, and continuation all consume typed basis posture
instead of raw branch/snapshot/time/source ids.

**Relevant subsystems**
- `worth-query` query-basis lifecycle and binding
- `worth-query` runtime state and inspection basis exposure
- `worth-runtime-bridge` temporal basis, async request identity, and resume basis

**Relevant Query docs**
- [Historical Diff And Basis](../../crates/worth-query/docs/capabilities/historical-diff-and-basis.md)
- [Inspection](../../crates/worth-query/docs/capabilities/inspection.md)
- [Configured Domain Handles](../../crates/worth-query/docs/domain-capabilities/configured-domain-handles.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [query_basis_lifecycle/mod.rs](../../crates/worth-query/src/query_basis_lifecycle/mod.rs)
- [query_basis_lifecycle/binding.rs](../../crates/worth-query/src/query_basis_lifecycle/binding.rs)
- [query_basis_lifecycle/eligibility.rs](../../crates/worth-query/src/query_basis_lifecycle/eligibility.rs)
- [query_basis_lifecycle/compatibility.rs](../../crates/worth-query/src/query_basis_lifecycle/compatibility.rs)
- [runtime/state.rs](../../crates/worth-query/src/runtime/state.rs)

**Relevant APIs**
- basis binding, eligibility, compatibility, and capability surfaces

**Warnings**
- Do not collapse truth-view basis and execution-time basis.
- Do not let raw ids bypass typed basis artifacts for temporal/async lanes.

**Test requirements**
- Add basis-projection parity tests across authoritative, branch-head, historical, preview, replay, and restart contexts.
- Add hostile stale/cross-branch/cross-source basis denial tests through the Query basis lifecycle.

**Engineering decisions**
- Temporal/async basis posture becomes a Query basis capability, not a sidecar field bag.
- Query basis lifecycle is the only public route for basis-sensitive temporal/async execution.

**Open questions**
- None.

### Phase 9: Route Planning And Bridge Continuation Projection Boundary

Extend route planning so temporal and async declarations produce one canonical
Query route plan and bridge-continuation lowering path rather than a local
"temporal planner" or "async planner" beside the existing declaration-routing
story.

**Relevant subsystems**
- `worth-query` declaration route plans and bridge routing
- `worth-query` configured-domain-handle route entrypoints
- `worth-runtime-bridge` temporal/async continuation and boundary-envelope artifacts

**Relevant Query docs**
- [Declaration Route Plan](../../crates/worth-query/docs/domain-capabilities/declaration-route-plan.md)
- [Declaration Bridge Continuation Routing](../../crates/worth-query/docs/domain-capabilities/declaration-bridge-continuation-routing.md)
- [Configured Domain Handles](../../crates/worth-query/docs/domain-capabilities/configured-domain-handles.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [application/declaration_bridge_routing/mod.rs](../../crates/worth-query/src/application/declaration_bridge_routing/mod.rs)
- [application/declaration_bridge_routing/lower.rs](../../crates/worth-query/src/application/declaration_bridge_routing/lower.rs)
- [application/declaration_bridge_routing/request.rs](../../crates/worth-query/src/application/declaration_bridge_routing/request.rs)
- [application/domain_handle/admitted_handle/bridge_routing.rs](../../crates/worth-query/src/application/domain_handle/admitted_handle/bridge_routing.rs)

**Relevant APIs**
- `route_bridge_continuation(...)`
- `route_bridge_continuation_checked(...)`
- declaration bridge-routing artifacts and support reports

**Warnings**
- Do not create a special planner path that bypasses declaration route planning.
- Do not let bridge lowering rediscover temporal/async declaration meaning.

**Test requirements**
- Add route-plan parity tests proving ordinary, temporal, and async declarations lower through the same route-planning surface with temporal/async differences preserved as typed route artifacts.
- Add hostile route-denial tests proving unsupported bridge temporal/async combinations deny before active lifecycle or result materialization.

**Engineering decisions**
- Temporal/async routing extends the declaration-route-plan family.
- Bridge lowering consumes typed declaration posture and basis posture only once.

**Open questions**
- None.

### Phase 10: Signal Compatibility And Prepared Continuation Boundary

Extend Query's signal-compatibility and prepared-continuation surfaces so
temporal and async declarations become explicit compatibility subjects before
active subscription or delivery work begins, rather than relying on implicit
lower-runtime behavior.

**Relevant subsystems**
- `worth-query` declaration signal compatibility and orchestration
- `worth-query` continuation pipeline preparation artifacts
- `worth-runtime-bridge` temporal/async continuation posture and signal-facing readiness

**Relevant Query docs**
- [Declaration Signal Compatibility](../../crates/worth-query/docs/domain-capabilities/declaration-signal-compatibility.md)
- [Signal Compatibility Orchestration](../../crates/worth-query/docs/domain-capabilities/signal-compatibility-orchestration.md)
- [Continuation Pipeline](../../crates/worth-query/docs/domain-capabilities/continuation-pipeline.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [application/domain_handle/admitted_handle/signal_compatibility.rs](../../crates/worth-query/src/application/domain_handle/admitted_handle/signal_compatibility.rs)
- [application/domain_handle/admitted_handle/signal_compatibility_orchestration.rs](../../crates/worth-query/src/application/domain_handle/admitted_handle/signal_compatibility_orchestration.rs)
- [application/declaration_signal_compatibility/handle_gate.rs](../../crates/worth-query/src/application/declaration_signal_compatibility/handle_gate.rs)
- [continuation_pipeline/mod.rs](../../crates/worth-query/src/continuation_pipeline/mod.rs)
- [continuation_pipeline/artifacts.rs](../../crates/worth-query/src/continuation_pipeline/artifacts.rs)

**Relevant APIs**
- signal-compatibility support and orchestration artifacts
- prepared continuation artifacts and next-step surfaces

**Warnings**
- Do not infer temporal/async signal compatibility from later activation success.
- Do not let prepared continuation skip temporal/async compatibility posture.

**Test requirements**
- Add signal-compatibility tests proving temporal/async declarations expose prepared, denied, or deferred posture before active lifecycle work.
- Add hostile continuation-preparation tests proving unsupported temporal/async compatibility cannot advance into continuation artifacts.

**Engineering decisions**
- Signal compatibility remains a first-class Query lane for temporal/async declarations.
- Prepared continuation artifacts must preserve temporal/async posture rather than re-deriving it at execution time.

**Open questions**
- None.

### Phase 11: Automatic Subscription Family Selection Boundary

Extend automatic family selection so time-aware and async-aware live queries
either map into one admitted subscription family honestly or deny typed before
declaration and activation, without product code hand-picking a second
subscription API.

**Relevant subsystems**
- `worth-query` subscription family selection and diagnostics
- `worth-query` view-shape-aware live meaning
- `worth-runtime-bridge` time-aware and async-capable live families

**Relevant Query docs**
- [Subscription Selection And Diagnostics](../../crates/worth-query/docs/capabilities/subscription-selection-and-diagnostics.md)
- [Scopes, Templates, Saved Queries, And View Shapes](../../crates/worth-query/docs/authoring/scopes-templates-saved-queries-and-view-shapes.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [subscription/selection.rs](../../crates/worth-query/src/subscription/selection.rs)
- [subscription/family.rs](../../crates/worth-query/src/subscription/family.rs)
- [subscription/signal_strategy.rs](../../crates/worth-query/src/subscription/signal_strategy.rs)
- [subscription/support/report.rs](../../crates/worth-query/src/subscription/support/report.rs)

**Relevant APIs**
- `select_query_subscription_family(...)`
- `report_query_subscription_support(...)`
- `QuerySubscriptionFamilySelection`
- `QuerySubscriptionSupportReport`

**Warnings**
- Do not manually pick temporal or async subscription families in product code.
- Do not collapse grouped, detail, inspector, bounded-materialization, temporal, and async live meaning into one generic family.

**Test requirements**
- Add family-selection parity tests proving equivalent temporal/async live meaning selects the same family regardless of authoring path.
- Add hostile budget/basis/view-shape selection denials proving impossible temporal/async live shapes deny during family selection rather than later.

**Engineering decisions**
- Family selection remains automatic where meaning is already canonical.
- Automatic does not mean heuristic; the family decision is a typed semantic artifact.

**Open questions**
- None.

### Phase 12: Subscription Declaration And Basis-Binding Boundary

Extend query-owned subscription declarations so time-aware and async-capable
live meanings bind canonical query meaning, view shape, basis posture, policy,
tenant, and temporal/async identity into one declaration digest before active
lifecycle begins.

**Relevant subsystems**
- `worth-query` subscription declaration, declaration digests, and basis requests
- `worth-query` query-basis lifecycle integration with live meaning
- `worth-runtime-bridge` temporal basis requests and async live declaration binding

**Relevant Query docs**
- [Subscription Selection And Diagnostics](../../crates/worth-query/docs/capabilities/subscription-selection-and-diagnostics.md)
- [Historical Diff And Basis](../../crates/worth-query/docs/capabilities/historical-diff-and-basis.md)
- [Support Matrix And Admission](../../crates/worth-query/docs/foundations/support-matrix-and-admission.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [subscription/declaration.rs](../../crates/worth-query/src/subscription/declaration.rs)
- [subscription/declaration_digest.rs](../../crates/worth-query/src/subscription/declaration_digest.rs)
- [subscription/basis_request.rs](../../crates/worth-query/src/subscription/basis_request.rs)
- [subscription/bridge_lowering.rs](../../crates/worth-query/src/subscription/bridge_lowering.rs)

**Relevant APIs**
- subscription declaration artifacts and declaration digests
- bridge lowering and basis-request surfaces

**Warnings**
- Do not bind temporal basis separately from the canonical subscription declaration.
- Do not allow async request identity to drift after declaration without minting a new declaration.

**Test requirements**
- Add subscription-declaration digest tests proving temporal basis and async request identity change declaration meaning explicitly.
- Add hostile post-declaration drift tests proving policy, tenant, basis, temporal, or async identity mutation cannot patch existing declarations.

**Engineering decisions**
- Time-aware and async-aware live meaning belongs in the subscription declaration digest.
- The live declaration remains the canonical boundary before activation.

**Open questions**
- None.

### Phase 13: Active Subscription Lifecycle Boundary

Extend active lifecycle, sharing, fanout, checkpoints, continuation, and
preview isolation so temporal/async subscriptions become real runtime-backed
active objects instead of declaration-only paperwork.

**Relevant subsystems**
- `worth-query` active subscription lifecycle
- `worth-query` active sharing, fanout, continuation, and preview isolation
- `worth-runtime-bridge` mixed-cause live maintenance and restart posture

**Relevant Query docs**
- [Subscription Selection And Diagnostics](../../crates/worth-query/docs/capabilities/subscription-selection-and-diagnostics.md)
- [Continuation Pipeline](../../crates/worth-query/docs/domain-capabilities/continuation-pipeline.md)
- [Branches And Previews](../../crates/worth-query/docs/foundations/branches-and-previews.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [subscription/activation.rs](../../crates/worth-query/src/subscription/activation.rs)
- [subscription/active.rs](../../crates/worth-query/src/subscription/active.rs)
- [subscription/active_handle.rs](../../crates/worth-query/src/subscription/active_handle.rs)
- [subscription/fanout.rs](../../crates/worth-query/src/subscription/fanout.rs)
- [subscription/continuation.rs](../../crates/worth-query/src/subscription/continuation.rs)
- [subscription/preview_isolation.rs](../../crates/worth-query/src/subscription/preview_isolation.rs)

**Relevant APIs**
- active subscription handles and continuation artifacts
- preview isolation and closeout surfaces

**Warnings**
- Do not stop at declaration support while leaving lifecycle semantics implicit.
- Do not allow authoritative and preview temporal/async subscriptions to share active state when basis identity differs.

**Test requirements**
- Add active-lifecycle tests for temporal-only, async-only, and mixed-cause subscriptions covering activation, sharing, fanout, and continuation.
- Add hostile preview-isolation and checkpoint tests proving temporal/async active state respects preview ownership and checkpoint identity.

**Engineering decisions**
- Temporal/async subscriptions are active runtime objects, not observer wrappers.
- Equivalent-subscription sharing remains semantic, not callback-local.

**Open questions**
- None.

### Phase 14: Time-Only Delivery Boundary

Freeze the Query delivery contract for time-only changes so freshness,
window-entry/window-exit, deadline, and previous-value-driven transitions can
produce canonical deliveries even when no relational patch arrived.

**Relevant subsystems**
- `worth-query` delivery and retained runtime state
- `worth-query` ordinary outcomes and live inspection
- `worth-runtime-bridge` time-only cause routing

**Relevant Query docs**
- [Ordinary Outcomes](../../crates/worth-query/docs/domain-capabilities/ordinary-outcomes.md)
- [Inspection](../../crates/worth-query/docs/capabilities/inspection.md)
- [Workspace Overview](../../crates/worth-query/docs/foundations/workspace-overview.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [runtime/delivery.rs](../../crates/worth-query/src/runtime/delivery.rs)
- [subscription/delivery.rs](../../crates/worth-query/src/subscription/delivery.rs)
- [runtime/state.rs](../../crates/worth-query/src/runtime/state.rs)
- [runtime/inspection/live.rs](../../crates/worth-query/src/runtime/inspection/live.rs)

**Relevant APIs**
- delivery artifacts
- retained state snapshots
- ordinary outcome posture

**Warnings**
- Do not fake a truth patch just to reuse ordinary delivery infrastructure.
- Do not hide time-only changes inside support diagnostics instead of delivery semantics.

**Test requirements**
- Add time-only delivery parity tests across runtime-backed and replay lanes.
- Add hostile missing-previous-value and stale-temporal-basis tests proving time-only delivery either produces truthful delivery or typed denial, never fabricated values.

**Engineering decisions**
- Time-only changes are first-class Query deliveries.
- Time-only cause identity must survive into state and inspection.

**Open questions**
- None.

### Phase 15: Async Result-State Boundary

Freeze the public Query result-state model for pending, fulfilled, failed,
stale, cancelled, retried, revalidating, superseded, and denied async/resource
outcomes so downstream code never has to mint its own status taxonomy above
Query.

**Relevant subsystems**
- `worth-query` retained state snapshots and delivery surfaces
- `worth-query` ordinary outcomes and support posture
- `worth-runtime-bridge` completion causality and denial taxonomy

**Relevant Query docs**
- [Workspace Overview](../../crates/worth-query/docs/foundations/workspace-overview.md)
- [Ordinary Outcomes](../../crates/worth-query/docs/domain-capabilities/ordinary-outcomes.md)
- [Support Matrix And Admission](../../crates/worth-query/docs/foundations/support-matrix-and-admission.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [runtime/state.rs](../../crates/worth-query/src/runtime/state.rs)
- [runtime/delivery.rs](../../crates/worth-query/src/runtime/delivery.rs)
- [ordinary_outcome/mod.rs](../../crates/worth-query/src/ordinary_outcome/mod.rs)
- [runtime/inspection.rs](../../crates/worth-query/src/runtime/inspection.rs)

**Relevant APIs**
- state snapshots
- delivery artifacts
- ordinary outcomes

**Warnings**
- Do not let UI-local loading/error state outcompete Query result-state.
- Do not reconstruct lifecycle state from callbacks after Query already materialized it.

**Test requirements**
- Add result-state parity tests across current, stale, cancelled, retried, superseded, denied, and replayed completions.
- Add hostile generation-drift and preview-mismatch tests proving stale/superseded/denied states remain typed and basis-bound.

**Engineering decisions**
- Query owns one public async result-state vocabulary.
- Completion-causality projection is the source of truth for async result-state meaning.

**Open questions**
- None.

### Phase 16: Mixed-Cause Ordering And Coalescing Boundary

Freeze how truth patches, time-only wakes, async completions, retries,
cancellations, remasks, and preview transitions order and coalesce into one
canonical Query delivery stream.

**Relevant subsystems**
- `worth-query` delivery shaping, suppression, and view-shape delivery
- `worth-query` live inspection and ordinary outcomes
- `worth-runtime-bridge` mixed-cause ordering and suppression posture

**Relevant Query docs**
- [Subscription Selection And Diagnostics](../../crates/worth-query/docs/capabilities/subscription-selection-and-diagnostics.md)
- [Scopes, Templates, Saved Queries, And View Shapes](../../crates/worth-query/docs/authoring/scopes-templates-saved-queries-and-view-shapes.md)
- [Inspection](../../crates/worth-query/docs/capabilities/inspection.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [runtime/delivery.rs](../../crates/worth-query/src/runtime/delivery.rs)
- [subscription/delivery.rs](../../crates/worth-query/src/subscription/delivery.rs)
- [view_shape/delivery.rs](../../crates/worth-query/src/view_shape/delivery.rs)
- [ordinary_outcome/topology.rs](../../crates/worth-query/src/ordinary_outcome/topology.rs)

**Relevant APIs**
- delivery and suppression artifacts
- view-shape delivery surfaces

**Warnings**
- Do not let host event order define semantic delivery order.
- Do not let coalescing erase semantically distinct time/async/truth causes.

**Test requirements**
- Add mixed-cause replay-equivalence tests under hostile host event reordering.
- Add hostile coalescing-boundary tests proving semantically distinct cause sets cannot collapse into misleading deliveries.

**Engineering decisions**
- Ordering comes from Bridge law; Query owns the public projection and coalesced delivery contract.
- Coalescing follows basis identity and cause-order identity, not UI convenience.

**Open questions**
- None.

### Phase 17: Ordinary Outcome, State, And Scalar Inspection Boundary

Extend ordinary outcomes, retained runtime state, and ordinary `workspace.inspect(...)`
so temporal/async posture appears in the same compact runtime-backed product
language already used for ordinary reads, live views, writes, and previews.

**Relevant subsystems**
- `worth-query` ordinary outcomes
- `worth-query` retained runtime state snapshots
- `worth-query` ordinary inspection

**Relevant Query docs**
- [Ordinary Outcomes](../../crates/worth-query/docs/domain-capabilities/ordinary-outcomes.md)
- [Inspection](../../crates/worth-query/docs/capabilities/inspection.md)
- [Workspace Overview](../../crates/worth-query/docs/foundations/workspace-overview.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [ordinary_outcome/mod.rs](../../crates/worth-query/src/ordinary_outcome/mod.rs)
- [ordinary_outcome/posture.rs](../../crates/worth-query/src/ordinary_outcome/posture.rs)
- [runtime/state.rs](../../crates/worth-query/src/runtime/state.rs)
- [runtime/inspection/unified/mod.rs](../../crates/worth-query/src/runtime/inspection/unified/mod.rs)

**Relevant APIs**
- ordinary outcome variants
- `workspace.state(...)`
- `workspace.inspect(...)`

**Warnings**
- Do not force temporal/async consumers to leave ordinary state/inspection just to understand basic runtime posture.
- Do not flatten temporal/async posture into untyped notes.

**Test requirements**
- Add state/inspection contract tests proving temporal/async handles expose retained posture through the same scalar inspection and state paths as other retained runtime objects.
- Add hostile omission tests proving basis, support posture, and cause posture cannot disappear from scalar inspection where they affect meaning.

**Engineering decisions**
- Ordinary outcomes stay the default compact language for runtime-backed temporal/async posture.
- Scalar inspection remains useful and truthful without becoming cross-runtime causal inspection.

**Open questions**
- None.

### Phase 18: Cross-Runtime Causal Inspection Boundary

Extend the dedicated causal inspection lane so temporal wakes, async
completions, mixed-cause suppressions, remasks, replay drift, and
restart/resume posture can all be explained through one Query-owned
cross-runtime explanation path without direct bridge, signal, or relational
imports in downstream code.

**Relevant subsystems**
- `worth-query` causal inspection request, admission, materialization, and certification
- `worth-runtime-bridge` offline diagnostics, mixed-cause evidence, and failure taxonomy

**Relevant Query docs**
- [Inspection](../../crates/worth-query/docs/capabilities/inspection.md)
- [Cross-Runtime Causal Inspection](../../crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [runtime/inspection/causal/mod.rs](../../crates/worth-query/src/runtime/inspection/causal/mod.rs)
- [runtime/inspection/causal/materialization/mod.rs](../../crates/worth-query/src/runtime/inspection/causal/materialization/mod.rs)
- [runtime/inspection/causal/certification/mod.rs](../../crates/worth-query/src/runtime/inspection/causal/certification/mod.rs)

**Relevant APIs**
- causal inspection request/admission/materialization artifacts

**Warnings**
- Do not turn ordinary inspection into a partial causal inspection clone.
- Do not require lower-runtime spelunking for temporal/async â€œwhyâ€ questions.

**Test requirements**
- Add causal-inspection materialization tests for changed, suppressed, denied, replayed, and remasked temporal/async artifacts.
- Add hostile offline-diagnosis tests proving stale completion, replay drift, preview residue, and resume mismatch can be explained from retained artifacts alone.

**Engineering decisions**
- Cross-runtime causal inspection remains a dedicated lane distinct from scalar inspection.
- Temporal/async explanation richness is projected from bridge artifacts, not recreated locally.

**Open questions**
- None.

### Phase 19: Preview, Branch, Promotion, Discard, And Rebinding Boundary

Close preview-local and branch-local temporal/async lifecycle semantics so
preview-owned wakes, pending async work, completion residue, and promotion or
discard transitions cannot leak into authoritative flows or be reconstructed as
if they were already authoritative.

**Relevant subsystems**
- `worth-query` preview workflow, branch sessions, basis rebinding, and inspection
- `worth-query` active preview isolation and continuation
- `worth-runtime-bridge` preview residue and re-admission law

**Relevant Query docs**
- [Branches And Previews](../../crates/worth-query/docs/foundations/branches-and-previews.md)
- [Continuation Pipeline](../../crates/worth-query/docs/domain-capabilities/continuation-pipeline.md)
- [Inspection](../../crates/worth-query/docs/capabilities/inspection.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [preview/mod.rs](../../crates/worth-query/src/preview/mod.rs)
- [preview/scoped.rs](../../crates/worth-query/src/preview/scoped.rs)
- [subscription/preview_isolation.rs](../../crates/worth-query/src/subscription/preview_isolation.rs)
- [runtime/inspection/preview.rs](../../crates/worth-query/src/runtime/inspection/preview.rs)
- [query_basis_lifecycle/binding.rs](../../crates/worth-query/src/query_basis_lifecycle/binding.rs)

**Relevant APIs**
- `workspace.preview(...)`
- `workspace.branch(...)`
- preview isolation and continuation artifacts

**Warnings**
- Do not let preview-owned temporal or async state silently survive discard.
- Do not treat promotion as structural reuse of preview-owned basis.

**Test requirements**
- Add preview discard and promotion parity tests for temporal-only, async-only, and mixed-cause cases.
- Add hostile preview-crossed-completion and promotion-mismatch tests with typed inspection and recovery posture.

**Engineering decisions**
- Promotion is a rebinding boundary.
- Preview residue stays preview-owned until authoritative re-admission proves otherwise.

**Open questions**
- None.

### Phase 20: Policy, Tenant, Relationship-Proof, And Schema-Context Remask Boundary

Freeze how temporal/async families interact with policy changes, tenant truth
and schema drift, and relationship-proof posture so authorization and context
changes resolve before delivery or result-state materialization.

**Relevant subsystems**
- `worth-query` policy basis, tenant basis, support matrix, and policy delivery
- `worth-query` runtime inspection and denial projection
- `worth-runtime-bridge` remask and drift denial artifacts

**Relevant Query docs**
- [Support Matrix And Admission](../../crates/worth-query/docs/foundations/support-matrix-and-admission.md)
- [Historical Diff And Basis](../../crates/worth-query/docs/capabilities/historical-diff-and-basis.md)
- [Inspection](../../crates/worth-query/docs/capabilities/inspection.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [policy_basis/mod.rs](../../crates/worth-query/src/policy_basis/mod.rs)
- [tenant_basis/mod.rs](../../crates/worth-query/src/tenant_basis/mod.rs)
- [policy_delivery/mod.rs](../../crates/worth-query/src/policy_delivery/mod.rs)
- [relationship_proof/support.rs](../../crates/worth-query/src/relationship_proof/support.rs)
- [runtime/support_matrix.rs](../../crates/worth-query/src/runtime/support_matrix.rs)

**Relevant APIs**
- policy basis and tenant basis artifacts
- support matrix and inspection postures

**Warnings**
- Do not materialize then remask.
- Do not classify policy, tenant, or schema drift as generic async failure.

**Test requirements**
- Add remask parity tests proving temporal/async result meaning stays canonical under policy/tenant/schema transitions.
- Add hostile drift-localization tests proving policy, tenant, proof, and schema drift deny before delivery and surface through typed support/inspection artifacts.

**Engineering decisions**
- Remask posture is part of canonical support and denial posture.
- Temporal/async families reuse existing Query policy/tenant/proof narrowing law instead of bypassing it.

**Open questions**
- None.

### Phase 21: View Shape, Scopes, Templates, And Saved Query Boundary

Decide which view-shape, scope, template, and saved-query surfaces genuinely
support temporal/async meaning in `9.4`, which must fail closed, and which stay
visible but deferred, so no reuse surface silently drops temporal/async
semantics.

**Relevant subsystems**
- `worth-query` view-shape support and delivery
- `worth-query` saved-query support and canonicalization
- `worth-query` declaration composition over scopes and templates

**Relevant Query docs**
- [Scopes, Templates, Saved Queries, And View Shapes](../../crates/worth-query/docs/authoring/scopes-templates-saved-queries-and-view-shapes.md)
- [Read Composition](../../crates/worth-query/docs/authoring/read-composition.md)
- [Support Matrix And Admission](../../crates/worth-query/docs/foundations/support-matrix-and-admission.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [view_shape/support.rs](../../crates/worth-query/src/view_shape/support.rs)
- [view_shape/delivery.rs](../../crates/worth-query/src/view_shape/delivery.rs)
- [saved_query/support.rs](../../crates/worth-query/src/saved_query/support.rs)
- [canonicalization/projection.rs](../../crates/worth-query/src/canonicalization/projection.rs)

**Relevant APIs**
- view-shape support reports
- saved-query support posture
- canonical declaration/reuse artifacts

**Warnings**
- Do not let saved-query reload or template expansion silently erase temporal/async declaration posture.
- Do not claim durable saved temporal/async artifacts before Milestone `11`.

**Test requirements**
- Add scope/template expansion parity tests proving temporal/async declaration identity survives composition.
- Add fail-closed tests for unsupported view-shape and saved-query combinations where temporal/async semantics are not honestly admitted in `9.4`.

**Engineering decisions**
- Reuse surfaces must either preserve temporal/async meaning or deny/defer explicitly.
- Durable saved-artifact completion remains later debt even if canonical declaration reuse works now.

**Open questions**
- None.

### Phase 22: Read Composition And Projection Consumption Boundary

Extend graph-shaped read composition and projection consumption so temporal/async
materialized facts can be composed, consumed, and receipt-bound without
reopening lower authority or degrading to row-bag folklore.

**Relevant subsystems**
- `worth-query` read composition and materialization
- `worth-query` projection consumption declarations, receipts, and facts
- `worth-runtime-bridge` temporal/async materialization and delivery artifacts

**Relevant Query docs**
- [Read Composition](../../crates/worth-query/docs/authoring/read-composition.md)
- [Projection Consumption](../../crates/worth-query/docs/capabilities/projection-consumption.md)
- [Inspection](../../crates/worth-query/docs/capabilities/inspection.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [runtime/read_composition.rs](../../crates/worth-query/src/runtime/read_composition.rs)
- [runtime/read_composition_materialization.rs](../../crates/worth-query/src/runtime/read_composition_materialization.rs)
- [projection_consumption/mod.rs](../../crates/worth-query/src/projection_consumption/mod.rs)
- [projection_consumption/receipt.rs](../../crates/worth-query/src/projection_consumption/receipt.rs)
- [projection_consumption/facts/mod.rs](../../workspaces/worth-query/crates/worth-query/src/projection_consumption/facts/mod.rs)

**Relevant APIs**
- `workspace.compose_read(...)`
- `workspace.materialize(...)`
- projection-consumption declarations, receipts, and typed fact artifacts

**Warnings**
- Do not require consumers to reopen bridge/runtime artifacts directly just to consume temporal/async facts.
- Do not let materialization hide temporal/async basis or cause posture.

**Test requirements**
- Add projection-consumption receipt tests for time-only and async-backed materialized facts.
- Add hostile fact-consumption tests proving temporal/async consumable facts remain basis-bound, policy-bound, and support-bound.

**Engineering decisions**
- Projection consumption remains the receipt-backed fact lane for temporal/async materialized meaning.
- Read composition and retained live surfaces must share the same lower declaration identity where admitted.

**Open questions**
- None.

### Phase 23: Continuation And Recovery Boundary

Extend continuation execution and recovery boundaries so temporal basis drift,
async request drift, replay drift, remask drift, stale completion, and
preview-crossed residue become typed continuation/recovery posture rather than
ambient exceptions or log-only failures.

**Relevant subsystems**
- `worth-query` continuation pipeline and execution readmission
- `worth-query` recovery boundary and checked stops
- `worth-runtime-bridge` resume basis and denial taxonomy

**Relevant Query docs**
- [Continuation Pipeline](../../crates/worth-query/docs/domain-capabilities/continuation-pipeline.md)
- [Ordinary Outcomes](../../crates/worth-query/docs/domain-capabilities/ordinary-outcomes.md)
- [Inspection](../../crates/worth-query/docs/capabilities/inspection.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [continuation_pipeline/mod.rs](../../crates/worth-query/src/continuation_pipeline/mod.rs)
- [continuation_pipeline/execution/readmission.rs](../../crates/worth-query/src/continuation_pipeline/execution/readmission.rs)
- [recovery_boundary/checked/continuation.rs](../../crates/worth-query/src/recovery_boundary/checked/continuation.rs)
- [application/domain_handle/admitted_handle/recovery.rs](../../crates/worth-query/src/application/domain_handle/admitted_handle/recovery.rs)

**Relevant APIs**
- continuation execution and readmission artifacts
- recovery boundary and checked-stop artifacts

**Warnings**
- Do not throw temporal/async drift into generic runtime errors.
- Do not make recovery depend on host-local caches or callbacks.

**Test requirements**
- Add continuation readmission tests for temporal-basis drift, async source drift, replay drift, and remask drift.
- Add hostile checked-stop and recovery-brief tests proving each failure localizes to a typed recovery surface.

**Engineering decisions**
- Continuation and recovery remain first-class Query product lanes for temporal/async posture.
- Readmission observations from configured domain handles remain the extensibility point for domain-specific drift checks.

**Open questions**
- None.

### Phase 24: Intent Admission, Effects, Workflow, And Write-Adjacent Boundary

Tie temporal/async-triggered follow-on work into Query's existing intent,
effect, workflow, and evidence story so completion-driven or time-driven
actions do not invent a second admission pipeline or a second causality model
for write-adjacent work.

**Relevant subsystems**
- `worth-query` intent admission lattice and typed handoffs
- `worth-query` runtime effect lifecycle and delivery
- `worth-query` preview/workflow/write-adjacent orchestration
- `worth-runtime-bridge` completion causality and write-adjacent continuation posture

**Relevant Query docs**
- [Writes And Intents](../../crates/worth-query/docs/execution/writes-and-intents.md)
- [Intent Admission](../../crates/worth-query/docs/execution/intent-admission.md)
- [Continuation Pipeline](../../crates/worth-query/docs/domain-capabilities/continuation-pipeline.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [intent_admission/mod.rs](../../crates/worth-query/src/intent_admission/mod.rs)
- [intent_admission/handoffs/mod.rs](../../crates/worth-query/src/intent_admission/handoffs/mod.rs)
- [intent_admission/trace/mod.rs](../../crates/worth-query/src/intent_admission/trace/mod.rs)
- [runtime/effect/mod.rs](../../crates/worth-query/src/runtime/effect/mod.rs)
- [runtime/effect/delivery.rs](../../crates/worth-query/src/runtime/effect/delivery.rs)
- [workflow/inspection.rs](../../crates/worth-query/src/workflow/inspection.rs)

**Relevant APIs**
- intent admission and handoff artifacts
- effect delivery and effect inspection surfaces

**Warnings**
- Do not let async completions trigger local callback workflows outside Query intent/effect lanes.
- Do not let time-driven effects bypass typed intent admission where admitted write-adjacent families already exist.

**Test requirements**
- Add intent-admission and effect-lifecycle tests for temporal/async-triggered follow-on work.
- Add hostile duplicate-admission and local-callback-bypass tests proving time/async write-adjacent work cannot create a second authority path.

**Engineering decisions**
- Temporal/async follow-on work composes through existing intent and effect families.
- Write-adjacent temporal/async causality must stay aligned with the public evidence story.

**Open questions**
- None.

### Phase 25: Downstream Delivery Contract Boundary

Project the merged runtime-backed temporal/async surface into one stable
downstream delivery contract so `worth-server` and later transport consumers
inherit typed basis negotiation, delivery classes, resume posture, and support
posture instead of rediscovering temporal/async semantics at the network edge.

**Relevant subsystems**
- `worth-query` public delivery/result contracts
- `worth-query` lower-runtime routing support and integration posture
- `worth-runtime-bridge` delivery and restart/resume bundle artifacts

**Relevant Query docs**
- [Downstream Runtime Integration](../../crates/worth-query/docs/foundations/downstream-runtime-integration.md)
- [Workspace Overview](../../crates/worth-query/docs/foundations/workspace-overview.md)
- [Support Matrix And Admission](../../crates/worth-query/docs/foundations/support-matrix-and-admission.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [runtime/delivery.rs](../../crates/worth-query/src/runtime/delivery.rs)
- [runtime/public_api.rs](../../crates/worth-query/src/runtime/public_api.rs)
- [runtime/handle_contract.rs](../../crates/worth-query/src/runtime/handle_contract.rs)
- [lower_runtime_routing/support.rs](../../crates/worth-query/src/lower_runtime_routing/support.rs)

**Relevant APIs**
- public delivery and handle contracts
- support reports and lower-runtime routing support surfaces

**Warnings**
- Do not make transport integration the place where basis or cause semantics are re-decided.
- Do not hide deferred durable resume debt behind a runtime-backed delivery contract.

**Test requirements**
- Add downstream-contract tests proving runtime-backed temporal/async deliveries project to one stable typed downstream contract.
- Add hostile resume-negotiation and unsupported-posture tests proving missing or stale basis denies explicitly rather than degrading silently.

**Engineering decisions**
- Runtime-backed delivery contract shape ships now; durable resume completion remains later debt.
- Downstream contract is a projection of canonical Query meaning, not a new authority layer.

**Open questions**
- None.

### Phase 26: Runtime-Backed Certification And Reference Workload Closure Boundary

Close the milestone with one runtime-backed certification and workload story
that proves the full Query-facing temporal/async surface as one integrated
system across authoritative, historical, preview, replay, restart, time-only,
async-only, and mixed-cause lanes.

**Relevant subsystems**
- `worth-query` subscription certification, preview certification, causal inspection certification, and public-doc/support closure
- `worth-runtime-bridge` temporal/async certification bundles and pricing-shock workload extensions

**Relevant Query docs**
- [Support Matrix And Admission](../../crates/worth-query/docs/foundations/support-matrix-and-admission.md)
- [Downstream Runtime Integration](../../crates/worth-query/docs/foundations/downstream-runtime-integration.md)
- [Runtime-Installed Domains And Operations](../../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/runtime-installed-domains.md)
- [Domain Capability Documentation Certification](../../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/public-doc-coverage.md)

**Documentation follow-through**
- Add or revise the phase's feature-facing docs with the `feature-doc-writer` skill before this phase can close.
- Treat doc updates as part of the same authority boundary as the code/API change, not as post-hoc cleanup.

**Relevant Query source surfaces**
- [subscription/certification.rs](../../crates/worth-query/src/subscription/certification.rs)
- [subscription/tests/runtime_certification.rs](../../crates/worth-query/src/subscription/tests/runtime_certification.rs)
- [runtime/inspection/causal/certification/mod.rs](../../crates/worth-query/src/runtime/inspection/causal/certification/mod.rs)
- [harness/preview_certification/mod.rs](../../crates/worth-query/src/harness/preview_certification/mod.rs)
- [public_doc_coverage/tests/support.rs](../../crates/worth-query/src/public_doc_coverage/tests/support.rs)

**Relevant APIs**
- runtime certification harness surfaces
- causal-inspection certification artifacts
- bridge certification bundle projections

**Warnings**
- Do not close on API coverage or a single happy-path demo.
- Do not split temporal, async, and mixed-cause certification into separate closure stories.

**Test requirements**
- Add one runtime-backed reference workload that exercises authoritative, historical, preview, replay, restart, time-only, async-only, mixed-cause, remask, and write-adjacent follow-on lanes.
- Add one hostile certification matrix proving equivalent lanes compare equal, intentionally different lanes compare unequal, and unsupported-neighbor behavior fails closed through support, delivery, continuation, and inspection artifacts.

**Engineering decisions**
- This milestone closes only as one merged proof band that is stricter than the old `9.7` split.
- Query certification consumes Bridge bundles but must prove the Query projection remains honest at every public surface named above.

**Open questions**
- None.

## Must Ship

- Query-native temporal declaration families that lower only into Bridge-admitted temporal basis and readiness artifacts
- Query-native async/resource declaration families with projected bridge-owned request identity
- one public temporal/async result-state model projected from Bridge completion causality
- one public mixed-cause delivery model projected from Bridge ordering and suppression law
- preview rebinding, remask, support matrix, and inspection coverage for temporal/async families
- one stable runtime-backed delivery contract shape for downstream consumers
- one merged runtime-backed certification and reference-workload closure story

## Must Preserve

- Bridge authority over temporal basis, async identity, completion causality, mixed-cause ordering, restart posture, and certification bundle law
- Signal authority over temporal readiness, previous-value semantics, and async lifecycle policy
- Relational authority over truth-view basis, branch/history identity, and preview truth
- the stabilized ordinary Query facade rather than a second temporal/async API family
- explicit typed support, denial, and inspection posture before activation, delivery, or materialization
- runtime-backed/store-backed separation honesty

## Acceptance Evidence

- canonical lowering parity for temporal and async declaration families
- adversarial basis-projection parity across authoritative, branch-head, historical, preview, and replay lanes
- time-only delivery parity with no relational patch
- async completion causality parity across current, stale, cancelled, retried, revalidated, superseded, and denied lanes
- mixed-cause ordering and suppression equivalence under hostile host event reordering
- preview discard/promotion and remask localization proof
- support-matrix and offline-inspection proof for every admitted, denied, deferred, and unsupported temporal/async family
- one merged runtime-backed reference workload and hostile certification matrix proving the full Query-facing temporal/async surface

## Sequencing Notes

This milestone belongs here because the Query public runtime surface and
authoritative mutation-evidence story must stabilize before temporal/async work
extends them. It also must land after Bridge Milestone 17 because Query should
project a closed lower-authority temporal/async law rather than inventing one.

It belongs before Milestones `10` and `11` because store-backed execution,
durable restart, and persisted artifacts need one stable runtime-backed product
surface to preserve. If those later milestones arrived first, they would have
to guess temporal/async Query meaning and would almost certainly freeze the
wrong contract.


