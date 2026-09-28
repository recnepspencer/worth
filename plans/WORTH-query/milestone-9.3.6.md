# Milestone 9.3.6 Engineering Spec: Lower-Runtime Capability Routing And Boundary Envelopes

> **Status:** Completed. Closeout: [milestone-9.3.6-closeout.md](./milestone-9.3.6-closeout.md).
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-9.3.5.md](./milestone-9.3.5.md)
>
> **Next milestone:** [Runtime API Public Stabilization Gate](./runtime-api-public-stabilization-plan.md)
> freezes the ordinary public facade only after 9.3.6 closes the remaining
> lower-runtime contact model. If this milestone leaves convenience seams in
> place, the stabilization gate would freeze an escape-hatch architecture.
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make every Query-to-lower-runtime crossing
> either disappear behind a better lower-runtime contract or become one typed,
> capability-routed, receipt-backed boundary lane, with only typed deferral for
> neighbors that belong to later milestones.
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [composition_laws.md](../../docs/coding-guidelines/composition_laws.md)
> - [domain_structure_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [test-requirements-milestone-9_3-and-runtime-gates.md](./test-requirements-milestone-9_3-and-runtime-gates.md)
> - [milestone-9.3.1.md](./milestone-9.3.1.md)
> - [milestone-9.3.2.md](./milestone-9.3.2.md)
> - [milestone-9.3.3.md](./milestone-9.3.3.md)
> - [milestone-9.3.4.md](./milestone-9.3.4.md)
> - [milestone-9.3.5.md](./milestone-9.3.5.md)
> - [runtime-api-public-stabilization-plan.md](./runtime-api-public-stabilization-plan.md)

## Goal

Make lower-runtime contact one explicit Query-owned lifecycle:

```text
LowerRuntimeCapabilityRequest
  -> CapabilityEligibility
  -> LowerRuntimeRoutePlan
  -> BoundaryExecutionReceipt
  -> LowerRuntimeBoundaryEnvelope
  -> LowerRuntimeBoundaryCertificationBundle
```

This milestone is not allowed to settle for "we have a backend trait now."
It must close the architectural seam between Query and its lower runtimes so
that ordinary callers, internal Query modules, future maintainers, and
certification programs all face the same rule:

- Query is the only ordinary public runtime facade
- lower-runtime authorities stay in their owning crates
- every crossing is named, typed, route-aware, receipt-backed, and inspectable
- any remaining direct path is either on an explicit elimination path inside
  this milestone or explicitly deferred to a later milestone, never a hidden
  convenience lane

## Why This Milestone Exists

Milestones 9.3.1 through 9.3.5 already established real Query-owned lifecycles:

- 9.3.1 made causal inspection a public Query artifact rather than downstream
  lower-runtime stitching
- 9.3.2 made basis use a typed capability lifecycle
- 9.3.3 made effect execution one authority-scoped lowered pipeline
- 9.3.4 made projection consumption declared and receipt-backed
- 9.3.5 made Query-crossing admission one public decision lattice and handoff
  model

What remains open is the lower-runtime contact story after admission.

The baseline for this milestone is the post-Phase-5.5 form of 9.3.5, not the
earlier "runtime floor" interpretation of that milestone.

That baseline matters because 9.3.5 no longer stops at representative family
closure. It is expected to close concrete adoption for:

- named read-execution entrypoints rather than deferred read neighbors
- named inspection and diagnostic-materialization entrypoints rather than
  representative advisory-only posture
- a checked mutation-entrypoint audit proving public write/update/delete
  surfaces delegate through authoritative intent
- basis-use and projection-consumption as first-class adopted families in DX,
  inventory, and certification
- the pre-9.3.6 lower-runtime capability-routing family on concrete supported
  authoring surfaces where real bridge-backed execution semantics already
  exist

9.3.6 therefore does not get to rediscover whether those families are adopted.
It inherits them as concrete covered surfaces and is responsible only for the
lower-runtime crossing story that remains after their admission and handoff
model is already canonical.

Today Query still contains several specialist seams where code reaches directly
into runtime bridge, relational, or signal facades for real work. Some of those
seams are honest adapters. Some are migration residue. Some exist because a
lower-runtime facade does not yet expose the exact contract Query needs.

That is the seam this milestone closes.

Without 9.3.6:

- the runtime stabilization gate would freeze a public facade that still relies
  on scattered internal convenience crossings
- later temporal, async, store-backed, and durable work would inherit several
  different "close enough" lower-runtime contact shapes instead of one
  capability-routing model
- downstream domains and future engineers would have a standing temptation to
  route around Query basis, admission, projection, effect, and inspection
  contracts by convenience
- support metadata and certification could claim that Query owns a surface
  whose real execution path is still chosen ad hoc inside specialist modules

This milestone exists to make that impossible.

## Governing Summaries

- `MENTALITY.md`: the default stance is deletion, not justification. A
  specialist seam is suspect by default. The first question is not "is this
  seam acceptable?" It is "why does this seam still exist, and which lower
  runtime failed to expose the contract Query needs?"
- `arch_laws.md`: facade integrity and authority boundaries are load-bearing.
  Query must own the public capability-routing lifecycle without absorbing
  bridge, relational, signal, or store authority.
- `composition_laws.md`: capability inventory, eligibility, route planning,
  receipt shaping, seam-elimination tracking, DX helpers, and certification must stay
  separate responsibilities. This milestone must not collapse into one generic
  adapter bag.
- `domain_structure_laws.md`: the tree must make crossing families, route-plan
  forms, elimination/deferred registries, authority ownership, and certification artifacts
  physically locatable. The structure must reveal which seams remain and why.
- `perf_laws.md`: lower-runtime routing must be cost-honest. Cheap-looking
  APIs must not hide broad scans, broad bridge reconstruction, route-family
  rediscovery, or evidence reassembly. Exact counters and slope proofs must
  bind cost to route width, evidence width, and deferred-neighbor width.
- `worth_query_vision.md`: ordinary consumers ask Query for truth. They do not
  spelunk bridge, relational, or signal internals for convenience.
- `worth_query_roadmap.md`: 9.3.6 must make lower-runtime contact
  capability-routed and boundary-enveloped before the runtime API freezes.
- `test-requirements.md`: the milestone needs named certification, hostile
  lanes, compile-fail boundaries, and machine-checkable route/evidence outputs.
- `milestone-9.3.2.md`: lower-runtime capability routing must consume admitted
  basis proof rather than raw branch, preview, historical, tenant, or policy
  identifiers.
- `milestone-9.3.3.md`: routed write/effect contact must consume effect-owned
  lowering and receipt artifacts instead of reconstructing bridge authority
  locally.
- `milestone-9.3.4.md`: routed source/materialization contact must bind to
  declared consumed-fact and materialization source contracts rather than raw
  row bags.
- `milestone-9.3.5.md`: lower-runtime execution must consume admitted plans and
  typed handoffs, not rediscover admission from raw requests.
- `milestone-9.3.5.md` Phase 5.5: 9.3.6 inherits a concrete covered-entrypoint
  inventory, mutation delegation audit, and adopted read/inspection/routing
  families rather than an earlier runtime-floor-only baseline.
- `runtime-api-public-stabilization-plan.md`: the public runtime API may freeze
  only after lower-runtime contact stops depending on implementation-colored
  convenience seams.

## Adversarial Constraint

Under read execution, historical materialization, subscription activation,
writeback, effect execution, intent execution, causal inspection
materialization, projection-consumption sourcing, frontier intake, preview
basis reuse, policy masking, tenant/schema variation, relationship-proof
denial, and support-gated future neighbors, the same canonical Query
capability must always cross into lower-runtime authority through one typed
capability-routing model.

If a crossing still exists outside that model, one of only three things may be
true:

1. the lower-runtime facade already exposes the needed authority and Query must
   delete the seam in favor of that contract
2. the lower-runtime facade is missing a specific capability, receipt, or
   envelope that should be added so Query can delete the seam
3. deletion is not yet possible within this milestone, so the seam is pushed
   into explicit later-milestone deferral with owner, exit criteria, and
   certification coverage

Anything else is failure.

In particular, 9.3.6 fails if it:

- treats direct lower-runtime imports as acceptable by default
- hides several seam shapes behind one generic backend convenience bucket
- allows operational `()` or `String` returns where a receipt or envelope is
  required
- lets Query remint lower-runtime authority as Query-owned truth
- allows one capability family to choose among multiple lower-runtime paths by
  convenience rather than by an inspectable route plan
- claims routed support while omitting remaining direct paths from explicit
  elimination or deferred-neighbor records

## Product Decision Lock

- `worth-query` owns the public lower-runtime capability-routing lifecycle:
  request, eligibility, route plan, boundary receipt, boundary envelope,
  elimination/deferred registry, support metadata, DX helpers, and
  certification.
- `worth-query` does not own truth authority, signal scheduling, bridge
  protocol authority, or durable storage authority.
- `worth-runtime-bridge` remains authoritative for bridge route/evaluation,
  truth-view materialization, subscription admission, preview/writeback
  protocol, and cross-runtime evidence.
- `worth-relational` remains authoritative for truth, snapshots, lineage,
  branch/head semantics, grouped/materialized truth, and relational decision
  evidence.
- `worth-signal` remains authoritative for observation, invalidation,
  evaluation, replay posture, lineage, and signal forensic evidence.
- `worth-store` remains deferred for store-backed route parity, durable route
  replay, and persisted boundary artifacts.

- 9.3.6 inherits these predecessor facts from 9.3.5 and must treat them as
  settled input rather than open design questions:
  - covered read execution is already adopted on named public entrypoints
  - covered inspection and diagnostic-materialization is already adopted on
    named public entrypoints
  - covered mutation-shaped public entrypoints have already been audited for
    authoritative-intent delegation
  - basis-use and projection-consumption are already first-class adopted
    families in the public admission lattice
  - the pre-9.3.6 lower-runtime capability-routing family already exists as an
    admitted/deferred family vocabulary on concrete public surfaces
- because those predecessor facts are settled, 9.3.6 may not reopen whether a
  covered family belongs in the shared lattice. It may only decide how the
  already-adopted family crosses into lower-runtime authority, whether an
  existing lower-runtime facade is sufficient, and which seams are true missing
  contracts versus forbidden duplicates.

9.3.6 introduces one default rule for specialist seams:

- if Query reaches through to a lower-runtime specialist surface, the burden is
  on the implementation to prove why that seam still exists and whether the
  correct fix is a better lower-runtime contract
- "this direct seam is convenient" is not a valid reason
- "this direct seam is historically present" is not a valid reason

Every crossing must be classified as exactly one of:

- `CanonicalLowerRuntimeReuse`
  - Query routes into an already-authoritative lower-runtime contract
- `QueryBoundaryAdapter`
  - Query adds only public capability-routing and envelope shape around an
    authoritative lower-runtime contract
- `CompatibilityDebtLane`
  - the direct seam still exists temporarily and is explicitly recorded with
    owner, blocking gap, exit criteria, and certification coverage
- `DeferredNeighbor`
  - the capability belongs to a later milestone and must fail deferred or
    unsupported
- `ForbiddenDuplicate`
  - a second route that must not exist because it duplicates or bypasses an
    admitted lower-runtime authority path

No seam may remain unclassified at closeout.
No in-scope seam may remain `CompatibilityDebtLane` at closeout.

## Locked Scope Decisions

9.3.6 is not "inventory every API in four crates." Its implementation scope is
the concrete Query-to-lower-runtime crossings that already exist today.

Its inventory source is also locked now:

- the primary source of covered ordinary surfaces is the concrete
  covered-entrypoint inventory finalized by 9.3.5, especially the rows added
  or strengthened by Phase 5.5
- 9.3.6 may extend that inventory only to classify the remaining
  lower-runtime-specialist seams required to execute, materialize, activate,
  or route those already-covered families
- 9.3.6 must not create a second drifting notion of "covered family" that
  disagrees with the 9.3.5 inventory, support matrix, DX examples, or crate
  documentation
- if 9.3.6 finds a mismatch between its seam inventory and the inherited 9.3.5
  covered-entrypoint inventory, the mismatch must be resolved explicitly in the
  spec or by updating the predecessor inventory first; silent reinterpretation
  is forbidden

In scope for this milestone:

- runtime-backed read evaluation and live declaration
- runtime-backed write authority and signal invalidation routing
- subscription declaration/activation/continuity contact
- preview basis and lower-runtime basis readmission
- historical bridge lowering and materialization-path interpretation
- effect-backed relational mutation/merge execution
- effect-backed bridge writeback execution
- intent-backed runtime execution for the currently admitted runtime-backed
  intent family
- projection-consumption source intake from bridge/relational/runtime-owned
  artifacts
- causal inspection bridge materialization
- frontier and parallel-admission signal intake

Explicitly not completed in this milestone:

- store-backed route parity
- durable route replay
- temporal routing families
- async/resource routing families
- mixed truth/time/async routing

Those neighbors must remain deferred rather than weakly supported.

## Concrete Crossing Decisions Locked Now

This section answers the main routing questions now, so implementation is not
left to rediscover the architecture.

### 1. Runtime backend remains the canonical runtime-backed crossing hub

The seam centered on `runtime/backend/contracts.rs` and
`runtime/backend/bridge_backed.rs` remains the canonical Query-owned boundary
adapter for ordinary runtime-backed live/view/write/subscription/preview/
inspection/intent operations.

That is a locked decision.

9.3.6 does **not** scatter those operations across more specialist modules.
Instead it strengthens this seam so it stops returning weak operational values
and starts returning typed boundary receipts and envelopes.

Concrete consequence:

- concrete `RuntimeBridge` and `RelationalRuntime` handle types may remain
  inside this internal boundary layer
- outside this boundary layer, covered runtime-backed operations should not
  introduce fresh direct lower-runtime imports without an explicit inventory row

### 2. Weak backend returns are not acceptable and must be replaced

The following current shapes are under-specified and must be replaced in
implementation:

- `WORTHQueryRuntimeSchemaAdapter::admit_live_view(...) -> Result<(), ...>`
- `WORTHQueryRuntimeSignalSinkAdapter::route_write_receipt(...) -> Result<(), ...>`
- `WORTHQueryRuntimeSubscriptionActivationAdapter::admit_activation(...) -> Result<String, ...>`

Those are no longer acceptable endpoint contracts for covered seams.

9.3.6 must replace them with typed receipts at minimum:

- `LiveViewDeclarationAdmissionReceipt`
- `SignalInvalidationRoutingReceipt`
- `SubscriptionActivationReceipt`

The exact final names may shift to match crate style, but the semantic decision
is locked: these seams must return proof-bearing boundary receipts, not `()`
or string identities.

### 3. Historical bridge lowering stays as a Query boundary adapter

`historical/bridge_lowering.rs` is not a seam we are trying to delete in
9.3.6.

It is a real Query adapter that translates bridge historical policy and
materialization-path meaning into Query-owned historical capability and
materialization descriptors. The bridge already owns the underlying historical
authority and path classes, and Query needs this translation layer to preserve
Query historical semantics.

Classification:

- `QueryBoundaryAdapter`

Concrete consequence:

- keep this lowering surface
- bind it into the 9.3.6 route-plan/envelope lifecycle
- do not record it as compatibility debt unless implementation finds a second
  direct historical seam outside this adapter

### 4. Projection-consumption source intake stays as a Query boundary adapter

`projection_consumption/source.rs` and the associated extraction modules are
also not seams we are trying to delete in 9.3.6.

They consume already-authoritative source artifacts:

- `WORTHQueryReadReceipt`
- `WORTHQueryWriteReceipt`
- `QueryContextExecutionArtifact`
- `RelationalAuthoritativeRowSetArtifact`
- `RelationalGroupedProjectionArtifact`
- `BridgeMaterializedRowSetArtifact`
- `BridgeGroupedTruthViewArtifact`

That is already the right shape. Query is not re-opening lower-runtime
authority there; it is adapting retained authoritative artifacts into
Query-owned projection-consumption contracts.

Classification:

- `QueryBoundaryAdapter`

Concrete consequence:

- keep these source adapters
- require them to participate in the 9.3.6 envelope story
- do not spend 9.3.6 trying to hide these source artifact types behind a fake
  generic layer

### 5. Causal inspection bridge materialization stays as a Query boundary adapter

`runtime/inspection/causal/builder_bridge.rs` is also an acceptable retained
adapter in 9.3.6.

It is already doing the right architectural job:

- Query owns admission/redaction/materialization policy
- bridge owns causal envelope assembly
- Query materializes bridge output into Query inspection artifacts

Classification:

- `QueryBoundaryAdapter`

Concrete consequence:

- keep this seam
- route it through the 9.3.6 boundary-envelope model
- do not move causal envelope ownership out of bridge

### 6. Frontier signal specialist imports are a real gap and must be removed

`frontier_signal_adapter.rs` currently imports signal specialist types:

- `worth_signal::facade::specialist::ParallelAdmissionReason`
- `worth_signal::facade::specialist::StageExecutionRecord`

This is a real boundary problem, not an acceptable permanent adapter.

Locked fix:

- 9.3.6 requires a facade-level signal receipt/evidence contract that
  lets Query recover serial-fallback and parallel-admission evidence without
  importing specialist-only signal types

Concrete consequence:

- implementation must add or consume a signal facade contract for frontier
  route evidence
- `frontier_signal_adapter.rs` must stop importing
  `specialist::*`
- this seam is allowed to exist only during the implementation transition and
  must not survive milestone closeout

### 7. Bridge writeback choreography is a real gap and must collapse into one bridge contract

`effect_lifecycle/execution_bridge.rs` currently performs a full bridge
choreography itself:

- admit policy declaration
- lower admitted policy
- admit writeback declaration
- build causality basis
- lower writeback effect
- classify idempotence
- execute writeback authority

That is too much bridge protocol choreography living in Query.

Locked fix:

- 9.3.6 requires one bridge facade contract for executing an admitted
  Query writeback route, returning a proof-bearing bridge receipt that includes
  policy, writeback, causality, effect, idempotence, authority outcome, and
  truth-writeback receipt linkage

Concrete consequence:

- implementation must add that narrow bridge contract rather than wrapping the
  current choreography more elaborately in Query
- `effect_lifecycle/execution_bridge.rs` must collapse into a thin adapter over
  that contract before milestone closeout

### 8. Relational mutation and merge execution remain canonical lower-runtime reuse

`effect_lifecycle/execution.rs` directly executes relational mutation and merge
plans through `RelationalRuntime`.

That is acceptable in 9.3.6.

Query already owns the lowered effect declaration and relational remains the
truth authority. This is not a hidden bridge-protocol reconstruction seam; it
is direct use of the authoritative relational execution surface.

Classification:

- `CanonicalLowerRuntimeReuse`

Concrete consequence:

- keep this execution shape
- add route-plan/receipt/envelope coverage around it
- do not force a new bridge-like abstraction over relational mutation/merge
  just for symmetry

### 9. Intent support is conditional, not universally ordinary

The current code and docs disagree slightly on `Intent`.

This spec resolves that ambiguity now:

- `Intent` is a runtime-backed family
- it is admitted only when a backend installs `intent_authority(...)`
- it is therefore not an unconditional always-on ordinary runtime family

Concrete consequence:

- 9.3.6 support metadata and boundary envelopes must advertise intent as
  conditionally supported based on installed intent authority
- the runtime stabilization gate may not overclaim universal intent support

### 10. Existing basis compatibility debt must either close or move out of scope explicitly

The following rows are already known at milestone entry:

- `query_context::{bind_query_basis_context,admit_query_basis_context,execute_query_basis_context}`
- `preview::{assess_preview_live_drift,PreviewLiveExecutionEnvelope::preview_live}`
- causal inspection's remaining observation-receipt-centered compatibility seam
- `subscription::{declaration,activation,support,diagnostic}::*`

Concrete consequence:

- 9.3.6 must import these rows into its broader elimination registry
- any row that is in scope for 9.3.6 must close before milestone completion
- any row that is not in scope for 9.3.6 must be reclassified as a typed
  `DeferredNeighbor` with the owning later milestone named explicitly
- implementation does not get to silently inherit them as tolerated debt

## Concrete Crossing Inventory

9.3.6 must not hide behind broad nouns like "runtime contact." The
implementation and certification suite must maintain an executable inventory of
the concrete Query-to-lower-runtime crossing families it owns.

Initial inventory floor:

- read evaluation and current-read execution
- historical bridge lowering and historical materialization
- preview basis admission and lower-runtime basis readmission
- subscription declaration and activation lowering
- subscription continuity delivery and support lookups
- write authority execution and signal invalidation routing
- effect-backed bridge execution
- intent-backed bridge/runtime execution
- projection-consumption source intake from bridge/relational/runtime-owned
  artifacts
- causal inspection bridge materialization
- frontier and parallel-admission signal intake
- support/discovery lookups that depend on lower-runtime capability posture

Each inventory row must name at least:

- Query capability family
- concrete public entrypoint or internal covered seam
- owning lower-runtime crate
- owning lower-runtime facade type/function/trait
- current classification
- whether the seam is route-planning or readmission/handoff only
- current route authority and evidence owner
- current returned artifact shape
- missing contract fields, if any
- whether deletion is possible now
- if not, which lower-runtime capability/receipt/envelope is missing
- debt owner, exit criteria, and certification row if the seam remains debt

This inventory must be executable metadata or a compile-visible fixture, not
only milestone prose.

## Locked Covered Crossing Table

The following rows are the minimum covered inventory for 9.3.6. Implementation
may add rows, but it may not silently remove or merge these rows.

| Query capability | Concrete seam | Classification | Route kind | Lower-runtime owner | Required 9.3.6 action |
| --- | --- | --- | --- | --- | --- |
| Composed current read | `WORTHQueryWorkspace::compose_read(...)` | `CanonicalLowerRuntimeReuse` | route-planning | bridge/runtime-backed read execution surface | delegate through the current-read route row; no side-door execution |
| Composed current read with invariant pack | `WORTHQueryWorkspace::compose_read_with_invariant_pack(...)` | `CanonicalLowerRuntimeReuse` | route-planning | bridge/runtime-backed read execution surface | delegate through the current-read route row after invariant admission |
| Defined read-family execution | `WORTHQueryWorkspace::execute_read_family(...)` | `CanonicalLowerRuntimeReuse` | route-planning | bridge/runtime-backed read execution surface | delegate through the current-read route row |
| Basis-context read-family execution | `WORTHQueryWorkspace::execute_read_family_in_basis_context(...)` | `CanonicalLowerRuntimeReuse` | route-planning | bridge/runtime-backed read execution surface | delegate through the basis-context route row |
| Current read execution | `runtime/workspace_queries.rs` -> `execute_runtime_current_read_graph(...)` | `CanonicalLowerRuntimeReuse` | route-planning | bridge/runtime-backed read execution surface | add route plan, receipt, and boundary envelope |
| Basis-context read execution | `runtime/workspace_queries.rs` -> `execute_runtime_basis_context_read_graph(...)` | `CanonicalLowerRuntimeReuse` | route-planning | bridge/runtime-backed read execution surface | add route plan, receipt, and boundary envelope |
| Public live view declaration | `WORTHQueryWorkspace::live_view(...)` and `live_view_request(...)` | `QueryBoundaryAdapter` | route-planning | Query runtime backend over source/runtime | delegate through live declaration and live installation route rows |
| Live view schema admission | `WORTHQueryRuntimeSchemaAdapter::admit_live_view(...)` | `QueryBoundaryAdapter` | readmission/handoff | Query runtime backend | replace `Result<(), ...>` with typed admission receipt |
| Live view source declaration | `WORTHQueryRuntimeSourceAdapter::declare_live_view(...)` | `QueryBoundaryAdapter` | route-planning | Query runtime backend over source/runtime | bind route plan + declaration receipt + envelope |
| Runtime live installation orchestration | `WORTHQueryRuntime::install_live_subscription_for_request(...)` | `QueryBoundaryAdapter` | route-planning | Query runtime backend over subscription/runtime/bridge surfaces | emit installation boundary envelope composed from declaration, activation, and continuity rows |
| Runtime intent authority seam | `WORTHQueryIntentAuthorityAdapter::execute_intent(...)` | `QueryBoundaryAdapter` | route-planning | Query runtime backend over bridge/relational | treat this as an allowed backend-boundary seam, not silent leakage |
| Subscription activation | `WORTHQueryRuntimeSubscriptionActivationAdapter::admit_activation(...)` | `QueryBoundaryAdapter` | route-planning | Query runtime backend over bridge/runtime | replace `Result<String, ...>` with `SubscriptionActivationReceipt` |
| Subscription continuity | bridge continuity and Query continuation surfaces | `CanonicalLowerRuntimeReuse` | route-planning | runtime bridge | route through bridge continuity artifacts and include continuity route envelope |
| Preview basis admission | `WORTHQueryRuntimePreviewBasisAdapter::admit_preview_basis(...)` | `QueryBoundaryAdapter` | readmission/handoff | Query runtime backend | include boundary envelope but no new lower-runtime contract |
| Basis readmission from truth-view evidence | `query_basis_lifecycle/binding.rs` truth-view readmission | `CanonicalLowerRuntimeReuse` | readmission/handoff | runtime bridge / relational | keep current readmission pattern; add 9.3.6 envelope coverage |
| Basis readmission from subscription evidence | `query_basis_lifecycle/binding.rs` subscription declaration/activation readmission | `CanonicalLowerRuntimeReuse` | readmission/handoff | runtime bridge | keep current readmission pattern; add 9.3.6 envelope coverage |
| Historical policy lowering | `historical/bridge_lowering.rs` | `QueryBoundaryAdapter` | route-planning | runtime bridge | keep adapter; bind to route plan and envelope |
| Effect-backed relational mutation | `effect_lifecycle/execution.rs` mutation path | `CanonicalLowerRuntimeReuse` | route-planning | relational | keep path; add route receipt/envelope |
| Effect-backed relational merge | `effect_lifecycle/execution.rs` merge path | `CanonicalLowerRuntimeReuse` | route-planning | relational | keep path; add route receipt/envelope |
| Effect-backed bridge writeback | `effect_lifecycle/execution_bridge.rs` | `QueryBoundaryAdapter` after bridge contract lands | route-planning | runtime bridge | add the required bridge writeback contract and reduce this seam to a thin adapter before closeout |
| Write authority backend execution | `WORTHQueryRuntimeWriteAuthorityAdapter::write(...)` | `QueryBoundaryAdapter` | route-planning | Query runtime backend over bridge/relational | return typed `WriteAuthorityExecutionReceipt` |
| Signal invalidation routing | `WORTHQueryRuntimeSignalSinkAdapter::route_write_receipt(...)` | `QueryBoundaryAdapter` | route-planning | Query runtime backend over signal | replace `Result<(), ...>` with `SignalInvalidationRoutingReceipt` |
| Intent runtime execution | `WORTHQueryRuntimeBackend::execute_intent(...)` via installed `intent_authority` | `QueryBoundaryAdapter` | route-planning | Query runtime backend over bridge/relational | mark conditionally supported; return boundary receipt/envelope for execution |
| Projection source intake from Query receipts | `projection_consumption/source.rs` from read/write/query-context receipts | `QueryBoundaryAdapter` | readmission/handoff | Query-owned receipts | keep adapter; include source boundary envelope |
| Projection source intake from relational artifacts | `projection_consumption/source.rs` relational row/grouped artifacts | `QueryBoundaryAdapter` | readmission/handoff | relational | keep adapter; no new lower-runtime contract required |
| Projection source intake from bridge artifacts | `projection_consumption/source.rs` bridge row/grouped artifacts | `QueryBoundaryAdapter` | readmission/handoff | runtime bridge | keep adapter; no new lower-runtime contract required |
| Causal bridge materialization | `runtime/inspection/causal/builder_bridge.rs` | `QueryBoundaryAdapter` | route-planning | runtime bridge | keep adapter; include route/receipt/envelope |
| Frontier evidence intake | `frontier_signal_adapter.rs` | `QueryBoundaryAdapter` after signal contract lands | route-planning | signal | add the required signal frontier receipt contract and delete specialist imports before closeout |

The rows above answer the main "what exactly is covered?" question for this
milestone. Any extra seam discovered during implementation must be added to the
inventory in the same patch that touches it.

## Exact Backend Receipt Replacements

9.3.6 locks the following runtime backend signature outcomes. Implementers do
not get to invent weaker receipt shapes.

### Live view schema admission

Current seam:

- `WORTHQueryRuntimeSchemaAdapter::admit_live_view(...) -> Result<(), WORTHQueryWorkspaceError>`

Required replacement:

- `WORTHQueryRuntimeSchemaAdapter::admit_live_view(...) -> Result<LiveViewDeclarationAdmissionReceipt, WORTHQueryWorkspaceError>`

Signature decision:

- direct signature replacement, not a sibling helper
- old `Result<(), ...>` shape should not survive as a parallel canonical seam

Mandatory receipt fields:

- `view_name`
- `query_digest`
- `basis_digest`
- `schema_view_digest`
- `support_row_digest`
- `admission_stage_digest`
- `lower_runtime_boundary_digest`

### Signal invalidation routing

Current seam:

- `WORTHQueryRuntimeSignalSinkAdapter::route_write_receipt(...) -> Result<(), WORTHQueryWorkspaceError>`

Required replacement:

- `WORTHQueryRuntimeSignalSinkAdapter::route_write_receipt(...) -> Result<SignalInvalidationRoutingReceipt, WORTHQueryWorkspaceError>`

Signature decision:

- direct signature replacement, not a sibling helper
- batch routing may still derive from repeated single-receipt routing, but the
  single-route receipt is canonical

Mandatory receipt fields:

- `write_receipt_digest`
- `signal_route_digest`
- `invalidation_scope_digest`
- `delivery_mode_digest`
- `realized_signal_receipt_digest`
- `counter_snapshot_digest`

### Subscription activation

Current seam:

- `WORTHQueryRuntimeSubscriptionActivationAdapter::admit_activation(...) -> Result<String, WORTHQueryWorkspaceError>`

Required replacement:

- `WORTHQueryRuntimeSubscriptionActivationAdapter::admit_activation(...) -> Result<SubscriptionActivationReceipt, WORTHQueryWorkspaceError>`

Signature decision:

- direct signature replacement, not a sibling helper
- the bare activation identity string must disappear from the covered seam

Mandatory receipt fields:

- `view_name`
- `subscription_family_digest`
- `admitted_subscription_identity`
- `activation_digest`
- `bridge_support_evidence_digest`
- `continuation_posture_digest`
- `boundary_execution_digest`

### Write authority execution

Current seam:

- `WORTHQueryRuntimeWriteAuthorityAdapter::write(...) -> Result<WORTHQueryMutationReceipt, ...>`

Required 9.3.6 refinement:

- keep the returned `WORTHQueryMutationReceipt` as the public mutation artifact
- require `WORTHQueryMutationReceipt` to carry an inspectable embedded
  `WriteAuthorityExecutionReceipt`

Signature decision:

- do **not** add a sibling or tuple-returning write method
- keep the current write method signature
- strengthen `WORTHQueryMutationReceipt` so the boundary receipt is retrievable
  from the returned public artifact

Mandatory receipt fields:

- `authority_lane`
- `lower_runtime_owner`
- `execution_route_digest`
- `write_effect_digest`
- `affected_live_view_digest`
- `signal_handoff_digest`

## Exact Bridge Contract Required For Writeback

9.3.6 locks the bridge-side fix for `effect_lifecycle/execution_bridge.rs`.

Query must stop manually orchestrating:

- policy admission
- policy lowering
- writeback admission
- causality basis construction
- effect lowering
- idempotence classification
- authority execution

Required bridge facade contract:

- one bridge entrypoint that executes an already-lowered Query writeback route
- the entrypoint must accept a Query-originated lowered writeback declaration
  or a one-field bridge wrapper over that declaration
- the choreography after that point belongs entirely to bridge

Required semantic shape:

```text
QueryLoweredWritebackRouteRequest
  -> BridgeAdmittedWritebackRoute
  -> BridgeWritebackRouteExecutionReceipt
```

Input decision:

- Query supplies the already-lowered `QueryWritebackDeclaration` meaning
- bridge owns admission, policy lowering, causality/effect/idempotence
  choreography, and final authority execution
- Query does not first construct a second bridge-native admitted contract on
  its own

The exact public bridge names may differ, but the returned bridge receipt must
contain or reference:

- policy declaration/admission digest
- lowered policy digest
- writeback declaration/admission digest
- causality basis digest
- lowered effect digest
- idempotence classification digest
- authority outcome digest
- truth writeback receipt digest

Concrete Query-side consequence:

- `effect_lifecycle/execution_bridge.rs` should become a thin adapter over this
  bridge contract
- if implementation still contains the full choreography in Query after 9.3.6,
  the milestone is not closed

## Exact Signal Contract Required For Frontier Evidence

9.3.6 locks the signal-side fix for `frontier_signal_adapter.rs`.

Query must stop importing:

- `worth_signal::facade::specialist::ParallelAdmissionReason`
- `worth_signal::facade::specialist::StageExecutionRecord`

Required signal facade contract:

- one facade-level frontier execution evidence artifact for plan/execution
  parity
- one facade-level route evidence artifact for serial-fallback versus
  parallel-admitted posture

Ownership decision:

- the default owner of frontier/planning receipts is `worth-signal`, not
  `worth-runtime-bridge`
- frontier posture, parallel-admission reasoning, stage execution evidence,
  and invalidation/planning topology are signal authority and must therefore
  be exposed from a signal facade contract first
- Query owns the public lower-runtime routing/request/receipt/envelope
  lifecycle that consumes those signal-owned receipts
- `worth-runtime-bridge` may aggregate signal frontier evidence only when a
  later artifact is genuinely cross-runtime and combines signal-owned posture
  with bridge-owned routing or evaluation authority
- bridge is not the default home for orphan frontier semantics just because
  Query needs a public boundary envelope

Required semantic shape:

```text
SignalFrontierPlanReceipt
SignalFrontierExecutionReceipt
SignalFrontierRouteEvidenceReceipt
```

Evidence decision:

- signal must expose both plan-time and execution-time receipts
- signal must expose the canonical route evidence receipt itself
- Query may summarize or re-label that route evidence for Query planning
  surfaces, but Query must not derive the canonical serial-fallback or
  parallel-admission classification from specialist-only records on its own

The exact names may differ, but Query must be able to obtain from facade-level
types:

- frontier surface digest
- predicted breadth
- realized breadth
- disjointness class
- parallel-admission posture
- serial-fallback reason where applicable
- route/evidence digest for certification

Concrete Query-side consequence:

- Query may continue translating signal evidence into Query planning/support
  vocabulary
- Query must consume signal-owned canonical frontier posture rather than
  re-authoring it locally or relocating it into bridge by default
- Query may not depend on specialist-only signal types once the 9.3.6 contract
  lands

## Subscription Crossing Decision

Subscription contact is split into three distinct crossing families in 9.3.6.

They must not be flattened into one generic "subscription support" seam.

### Subscription declaration

- classification: `CanonicalLowerRuntimeReuse`
- route kind: readmission/handoff
- owner: runtime bridge
- Query role: consume bridge admission and bind it to Query basis/admission
  lifecycle
- required Query artifact family:
  - `SubscriptionDeclarationBoundaryReceipt`
  - `SubscriptionDeclarationBoundaryEnvelope`

### Subscription activation

- classification: `QueryBoundaryAdapter`
- route kind: route-planning
- owner: Query runtime backend over bridge/runtime activation
- Query role: emit `SubscriptionActivationReceipt` and boundary envelope
- required Query artifact family:
  - `SubscriptionActivationReceipt`
  - `SubscriptionActivationBoundaryEnvelope`

### Continuity delivery

- classification: `CanonicalLowerRuntimeReuse`
- route kind: route-planning
- owner: runtime bridge
- Query role: consume continuity planning/lowering artifacts and expose them in
  boundary envelopes
- required Query artifact family:
  - `ContinuityDeliveryBoundaryReceipt`
  - `ContinuityDeliveryBoundaryEnvelope`

Compatibility consequence:

- the existing `subscription::{declaration,activation,support,diagnostic}::*`
  debt row remains until those surfaces fully adopt the scoped lifecycle and
  these three crossing classes explicitly

## Read Execution Coverage Decision

The ordinary read paths covered by 9.3.6 are locked now.

Covered public entrypoints:

- `WORTHQueryWorkspace::compose_read(...)`
- `WORTHQueryWorkspace::compose_read_with_invariant_pack(...)`
- `WORTHQueryWorkspaceQueries::execute_read_family(...)`
- `WORTHQueryWorkspaceQueries::execute_read_family_in_basis_context(...)`
- `WORTHQueryWorkspace::live_view(...)`
- `WORTHQueryWorkspace::live_view_request(...)`
- live declaration flows that call:
  - `declare_live_view(...)`
  - `install_live_subscription(...)`

Read execution classification:

- current read execution: `CanonicalLowerRuntimeReuse`, route-planning
- basis-context read execution: `CanonicalLowerRuntimeReuse`, route-planning
- live declaration admission: `QueryBoundaryAdapter`, readmission/handoff
- live declaration installation: `QueryBoundaryAdapter`, route-planning

Delegation rule:

- `compose_read(...)` and `compose_read_with_invariant_pack(...)` are not
  independent route rows; they are public authoring wrappers over the current
  read execution row
- `live_view(...)` and `live_view_request(...)` are not independent activation
  protocols; they must delegate through the live declaration and runtime live
  installation rows

Not covered as ordinary read-routing rows in 9.3.6:

- preview workflow helpers that still sit under existing basis compatibility
  debt
- deferred temporal/async/store-backed read-routing neighbors

## 9.3.5 Handoff To 9.3.6 Route Plan Boundary

This relationship is now explicit.

- 9.3.5 owns:
  - raw intent
  - eligibility
  - admission decision
  - admitted intent plan
  - admitted execution handoff
- 9.3.6 owns:
  - lower-runtime capability request derived from admitted handoff
  - capability eligibility for lower-runtime crossing
  - lower-runtime route plan
  - boundary execution receipt
  - boundary envelope

Hard rule:

- 9.3.6 may not re-decide or re-classify 9.3.5 admission
- if a lower-runtime route is unavailable, 9.3.6 may deny the crossing as
  unsupported/deferred at the lower-runtime boundary, but it may not
  reinterpret the original Query admission semantics

## Locked 9.3.6 Seam Elimination Rows

9.3.6 adds the following explicit seam-elimination rows on top of carried
forward basis rows. These rows exist to force contract delivery and seam
removal before closeout, not to normalize lingering debt:

- `frontier_signal_adapter.rs`
  - current shape: Query imports signal specialist types
  - missing contract: signal facade-level frontier route evidence receipts
  - required closeout: Query consumes only signal facade route evidence and the
    specialist imports are deleted
  - ownership decision: the canonical frontier planning/route evidence receipt
    is signal-owned unless and until a later artifact is genuinely
    cross-runtime and therefore bridge-owned
- `effect_lifecycle/execution_bridge.rs`
  - current shape: Query performs bridge writeback choreography
  - missing contract: one bridge admitted-writeback execution contract with
    proof-bearing receipt
  - required closeout: Query becomes a thin adapter over that bridge contract
- `runtime/backend/contracts.rs` weak-return seams
  - current shape: `()`, `String`, or equally weak operational results
  - missing contract: typed boundary receipts
  - required closeout: every covered seam returns the locked receipt shape
- `runtime/intent/mod.rs`
  - current shape: concrete `RuntimeBridge` and `RelationalRuntime` handle
    imports outside `runtime/backend/*`
  - required closeout: either move under the backend boundary subtree or treat
    it as an explicitly named backend-boundary module in the executable import
    allowlist
  - forbidden closeout: leaving it as an unclassified specialist seam

No additional seam-elimination row may be added for a covered seam without
naming the missing lower-runtime contract and required closeout in the same
patch. Covered rows must close in this milestone unless they are explicitly
reclassified to a later `DeferredNeighbor`.

## Intent Support Matrix Decision

Intent support must be represented as:

- `supported_when_runtime_installs_intent_authority`

It must not be represented as either:

- universally supported
- generically unsupported

Support-matrix consequence:

- if `intent_authority(...)` is installed and backend validation succeeds,
  the support row is `Supported` with evidence including
  `intent-authority-adapter`
- if no intent authority is installed, the support row is `Unsupported` with
  denial reason `intent support requires an executable intent authority adapter`
- the ordinary public runtime support report must therefore show intent as
  supported-with-installed-backend-capability rather than universally stable
  or generically unsupported
- the certification suite must include both:
  - a control lane with installed intent authority
  - a hostile lane where the profile would claim intent support without
    installed authority and backend validation rejects it

## Downstream Domain Boundary Rule

9.3.6 does not stop at `worth-query` internals. Downstream crates that integrate
Query with domain runtimes must obey the same boundary model.

Locked rule:

- `worth-query` remains the only ordinary public query/runtime facade
- downstream crates may implement Query runtime extension traits and runtime
  assembly only inside one declared domain runtime boundary subtree
- outside that subtree, downstream code may not import bridge, relational, or
  signal facades in order to satisfy ordinary Query-facing read/write/live/
  inspect/intent behavior

Allowed downstream boundary examples include modules like a domain-owned
`projection/runtime_boundary/query_runtime/*` subtree whose sole job is:

- building a Query runtime from domain-owned runtime pieces
- implementing Query runtime adapter traits
- translating authoritative lower-runtime artifacts into Query boundary receipts

Required downstream rule:

- lower-runtime imports inside these subtrees must still be classified row by
  row as `CanonicalLowerRuntimeReuse`, `QueryBoundaryAdapter`,
  `DeferredNeighbor`, or `ForbiddenDuplicate`
- if a downstream subtree needs a missing lower-runtime contract, 9.3.6 should
  prefer adding that contract over normalizing a new convenience seam
- downstream crates may not weaken the locked Query receipt contracts into
  local `String`, `()`, or ad hoc token shapes

Forbidden downstream topology:

- scattering bridge/relational/signal imports across ordinary domain authoring,
  read staging, fixture helpers, or convenience utilities while still claiming
  that Query is the boundary
- introducing a second ordinary public query/runtime facade in the domain crate
- re-deriving canonical signal route evidence from signal specialist internals
  outside the declared runtime-boundary subtree

Certification consequence:

- the 9.3.6 hostile suite must include at least one downstream domain fixture
  proving that allowed runtime-boundary modules can implement the Query
  extension seam while ordinary downstream modules fail the direct-import audit

## Relational And Signal Boundary Asymmetry Lock

9.3.6 intentionally does not force false symmetry between relational and
signal.

Locked policy:

- Query may consume authoritative relational artifacts directly when the
  capability is truth-owned and the relational artifact is already the
  authoritative output
- Query may consume bridge artifacts directly when the capability is genuinely
  cross-runtime and the bridge artifact is already the authoritative routed
  output
- Query may not consume signal specialist/runtime artifacts directly when doing
  so would require Query to derive canonical route or evidence meaning itself

Concrete consequence:

- `projection_consumption/source.rs` may continue accepting
  `RelationalAuthoritativeRowSetArtifact` and
  `RelationalGroupedProjectionArtifact` directly because those are
  truth-authoritative artifacts, not hidden bridge work
- `projection_consumption/source.rs` may also continue accepting
  `BridgeMaterializedRowSetArtifact` and `BridgeGroupedTruthViewArtifact`
  directly because those are already bridge-routed cross-runtime artifacts
- 9.3.6 must not introduce a pretend bridge passthrough over relational row-set
  or grouped-truth artifacts just to make the boundary diagram look uniform
- if Query needs canonical signal-owned frontier or execution evidence, that
  evidence must come from a signal facade contract or a bridge-mediated
  cross-runtime contract, not from Query reading signal specialist internals

## Boundary Enforcement Mechanism

9.3.6 does not leave enforcement vague.

Required enforcement:

- compile-fail boundaries for ordinary public callers
- executable inventory tests for covered Query seams
- direct-import audit for covered implementation modules using a checked
  allowlist fixture or equivalent executable audit, not a manual code review

The direct-import audit must fail if a covered module imports lower-runtime
facade types outside its declared allowed class:

- allowed adapter modules:
  - `historical/bridge_lowering.rs`
  - `projection_consumption/source.rs`
  - `runtime/inspection/causal/builder_bridge.rs`
  - runtime backend boundary modules
- transition-only modules that must be eliminated or reduced before closeout:
  - `frontier_signal_adapter.rs`
  - `effect_lifecycle/execution_bridge.rs`

The same audit model must apply to downstream Query-integrated domain crates:

- one declared runtime-boundary subtree may be allowlisted for direct
  lower-runtime imports used to implement Query runtime extension seams
- ordinary downstream modules outside that subtree must fail the audit if they
  import bridge, relational, or signal facades for Query-facing behavior

Any new covered module using a direct lower-runtime import must be added to the
inventory and either marked as an allowed adapter, a transition-only
elimination row, or an explicit `DeferredNeighbor` in the same change.

## Existing Surfaces To Consolidate

This milestone starts from existing Query and lower-runtime surfaces. It must
consolidate them instead of inventing a second routing universe.

Canonical lower-runtime authorities Query should reuse rather than replace
already include:

- `RuntimeBridge::evaluate`
- `RuntimeBridge::evaluate_current`
- `RuntimeBridge::plan_truth_view_packet`
- `RuntimeBridge::materialize_source_packet`
- `RuntimeBridge::admit_subscription`
- `RuntimeBridge::admit_subscription_preview_basis`
- `RuntimeBridge::deliver_continuity`
- relational truth/history/snapshot/lineage facade APIs
- signal live observation/invalidation facade APIs

Existing Query-owned lifecycles that 9.3.6 must consume rather than bypass:

- basis lifecycle proofs and lower-runtime binding artifacts from 9.3.2
- effect lowering and execution receipts from 9.3.3
- projection-consumption source/materialization contracts from 9.3.4
- admitted plans and execution handoffs from 9.3.5
- causal inspection bridge-envelope materialization from 9.3.1

Existing explicit seam-elimination or deferral rows already visible before
9.3.6 include:

- legacy `query_context` basis entrypoints
- preview-live follow-on seams
- causal inspection paths still centered on earlier observation artifacts
- subscription surfaces still centered on pre-lifecycle artifacts

Normative consequences:

- if a lower-runtime authority already exists, 9.3.6 must route into it
- if Query still crosses directly because the authority is missing a usable
  contract, 9.3.6 must specify that missing contract precisely
- if a seam cannot be deleted yet, 9.3.6 must either add the missing contract
  in this milestone or reclassify the seam as an explicit deferred neighbor
- no crossing may remain a silent convenience lane

## Required Crate Changes

### `worth-query`: required changes

`worth-query` is the owning crate for 9.3.6. It must add:

- a new `lower_runtime_routing` subdomain under `crates/worth-query/src`
- typed routing artifacts:
  - `LowerRuntimeCapabilityRequest`
  - `CapabilityEligibility`
  - `LowerRuntimeRoutePlan`
  - `BoundaryExecutionReceipt`
  - `LowerRuntimeBoundaryEnvelope`
  - `LowerRuntimeBoundaryCertificationBundle`
- an executable crossing inventory and an elimination/deferred registry
- family-aware route-plan and boundary-envelope adapters that consume existing
  basis, effect, projection, inspection, and admission artifacts
- support/discovery metadata for admitted, deferred, and forbidden crossing
  classes
- public facade exports and DX helpers for ordinary callers
- compile-fail boundaries proving ordinary callers cannot reach covered
  lower-runtime seams without Query capability routing
- named 9.3.6 certification coverage and all required digests

`worth-query` must also tighten existing internal and public boundary surfaces:

- covered lower-runtime seams may not return bare `()`, raw `String`, or ad
  hoc weak handles where a receipt or envelope is required
- specialist modules may not keep direct lower-runtime imports unless they are
  classified in the executable crossing inventory
- the internal backend boundary must stop behaving like a generic adapter bag;
  each seam must be mechanically classifiable by capability family, authority,
  and route type
- common-path APIs must expose routed capability usage rather than encouraging
  callers or maintainers to use lower-runtime vocabulary directly

### `worth-runtime-bridge`: required changes

`worth-runtime-bridge` remains authoritative for route/evaluate/materialize/
preview/writeback/subscription bridge behavior.

Required bridge boundary for this milestone:

- Query must route through bridge-owned route/evaluate/materialization
  contracts where they already exist
- Query must consume bridge-owned route receipts, evidence digests, preview
  basis artifacts, subscription-admission artifacts, and writeback provenance
  rather than reconstructing them
- if a remaining Query specialist seam exists because bridge does not expose a
  capability-shaped receipt or envelope yet, 9.3.6 should prefer adding that
  bridge contract over preserving the Query seam

Allowed bridge changes:

- narrow facade exports for already-authoritative route/evidence artifacts
- new capability-shaped bridge receipts or envelopes when Query currently has
  to reach through due to missing bridge boundary shape

Forbidden bridge changes:

- moving Query capability requests, public support metadata, or Query debt
  ownership into bridge
- teaching bridge Query-owned policy, fact-taxonomy, or public certification
  rows

### `worth-relational`: required changes

`worth-relational` remains authoritative for truth, history, snapshots,
grouped/materialized truth, branch meaning, and relational decision evidence.

Required relational boundary for this milestone:

- Query must consume relational authority artifacts rather than recreating
  row-set, grouped-truth, branch, or historical meaning locally
- if a specialist seam remains because relational lacks a usable authority
  receipt or grouped/history envelope, 9.3.6 should prefer adding that
  relational contract over preserving the Query seam

Allowed relational changes:

- narrow facade exports for already-authoritative truth/history/grouping
  artifacts
- one explicit authority receipt or envelope when Query cannot otherwise delete
  a direct seam honestly

Forbidden relational changes:

- adding Query-owned route plans, capability requests, or Query public receipts
- moving public boundary-envelope ownership out of Query

### `worth-signal`: required changes

`worth-signal` remains authoritative for observation, invalidation,
evaluation, lineage, replay posture, and signal diagnostics/forensics.

Required signal boundary for this milestone:

- Query must consume signal observation/invalidation/frontier evidence through
  signal-owned contracts
- if Query still has a specialist signal seam because the signal facade does
  not expose the needed frontier/evidence/receipt contract, 9.3.6 should
  prefer adding that signal contract over normalizing the Query seam
- frontier planning receipts, parallel-admission posture, serial-fallback
  posture, and stage-execution evidence are signal-owned by default; Query
  wraps them into the public lower-runtime routing model, and bridge may own
  only later cross-runtime aggregation artifacts that combine signal evidence
  with bridge-owned routing or evaluation meaning

Allowed signal changes:

- narrow facade exports for already-authoritative observation/frontier/
  invalidation evidence
- one capability-shaped signal receipt or envelope where needed to delete a
  Query seam

Forbidden signal changes:

- moving Query routing lifecycle or support/debt ownership into signal
- teaching signal about Query public capability families

### `worth-store`: required changes

- none for the runtime-backed 9.3.6 slice
- store-backed route parity and durable route replay remain later-milestone
  work
- any current store-adjacent mention in 9.3.6 must remain typed deferred or
  explicit later-milestone deferral until Milestones 10 and 11 close

## Typed Phase Progression Lock

Milestone 9.3.6 must introduce or certify this progression:

```text
LowerRuntimeCapabilityRequest
  -> CapabilityEligibility
  -> LowerRuntimeRoutePlan
  -> BoundaryExecutionReceipt
  -> LowerRuntimeBoundaryEnvelope
  -> LowerRuntimeBoundaryCertificationBundle
```

The public phase names above are normative.

Minimum semantic meaning of each phase:

- `LowerRuntimeCapabilityRequest`
  - one admitted Query capability asking to cross into lower-runtime authority
  - binds canonical query, basis, admission, projection, effect, inspection,
    or support context as applicable
  - never starts from raw lower-runtime handles
- `CapabilityEligibility`
  - proves whether the crossing is admitted, deferred, denied, or forbidden
  - localizes missing lower-runtime contract versus unsupported later-milestone
    neighbor versus true denial
- `LowerRuntimeRoutePlan`
  - freezes which lower-runtime authority, contract, and execution posture will
    be used
  - names route family, cost posture, failure topology, and retained evidence
    posture
  - distinguishes route-planning crossings from simple readmission/handoff
    crossings
- `BoundaryExecutionReceipt`
  - the first operational artifact proving the crossing actually happened
  - binds the route plan to the concrete lower-runtime receipt/evidence
  - never degrades to `()` or a weak string token for covered seams
- `LowerRuntimeBoundaryEnvelope`
  - the offline-readable and inspection-ready Query-owned summary of the
    crossing
  - names authority owner, route, capability, cost posture, failure topology,
    retained evidence, and deferred-neighbor posture where relevant
- `LowerRuntimeBoundaryCertificationBundle`
  - proves executable inventory closure, seam-elimination closure,
    deferred-neighbor honesty, compile-boundary
    enforcement, route parity, and slope/counter honesty

## Phase Implementation Order

### Phase 1: Freeze The Crossing Inventory

Purpose:
identify every current Query-to-lower-runtime crossing before any cleanup work
can hide it.

This phase starts from inherited 9.3.5 truth. It does not re-prove which
families are adopted or re-audit whether mutation-shaped public entrypoints
delegate through authoritative intent. It imports those predecessor facts and
classifies only the lower-runtime crossing seams that remain.

Phase owns these rows:

- all rows in the locked covered crossing table
- any newly discovered downstream Query-integrated runtime-boundary subtree
  rows
- carried-forward basis seam-elimination or deferred-neighbor rows that remain
  visible at milestone entry

Work to complete in order:

1. import the inherited 9.3.5 covered-entrypoint inventory and mutation
   delegation audit results as predecessor inputs
2. enumerate all covered crossing families and concrete covered seams that
   remain relevant at the lower-runtime boundary
3. classify each row as reuse, adapter, debt, deferred neighbor, or forbidden
   duplicate
4. mark each row as route-planning or readmission/handoff
5. record current returned artifact strength and missing contract fields
6. add executable coverage so no row can disappear socially

Required code moves:

- create the executable crossing inventory substrate
- encode row classification, route kind, authority owner, and current artifact
  strength in executable form rather than prose only
- encode transition-only seam-elimination rows and deferred-neighbor rows in
  executable form
- add direct-import audit inputs for Query internal modules and downstream
  Query-integrated runtime-boundary subtrees

This phase does not close if:

- any known seam remains represented only narratively
- the worth-topo-style downstream runtime-boundary pattern is not classified
- a row is inventoried only by family label without its concrete seam/module
- a new direct-import allowlist exists without a matching executable inventory
  row

Completion gate:

- every known direct lower-runtime contact is visible in the inventory
- no seam remains uncategorized
- no later phase may introduce a new direct seam without an inventory row

### Phase 2: Delete What Can Already Be Deleted

Purpose:
remove seams that exist only because Query historically reached through for
convenience.

This phase also has a hard non-goal: it must not reopen the 9.3.5 question of
whether write/update/delete public entrypoints delegate through authoritative
intent. That delegation audit is inherited. Phase 2 only verifies that the
already-adopted routed lanes do not fan back out into stray lower-runtime
execution paths after admission.

Phase owns these rows:

- every locked covered row currently classified as `CanonicalLowerRuntimeReuse`
- every locked covered row currently classified as `QueryBoundaryAdapter` that
  already has a sufficient lower-runtime contract
- any downstream Query-integrated seam that is only a convenience bypass over
  an already-authoritative lower-runtime facade

Work to complete in order:

1. route covered seams through already-authoritative lower-runtime facades
2. replace convenience direct imports where no lower-runtime gap exists
3. prove parity against the old path
4. remove or lock down the superseded seam
5. certify non-bypass behavior

Required code moves:

- route read execution, basis readmission, relational mutation/merge reuse,
  projection source intake, and causal bridge materialization through their
  already-authoritative lower-runtime contracts without side-door duplicates
- remove any redundant direct-import helper path that exists only for
  convenience
- reduce wrappers so that one routed lane becomes the only ordinary execution
  path for the owned rows

This phase does not close if:

- a seam that could use an existing lower-runtime contract is merely wrapped in
  another Query helper
- old and new paths coexist without one being mechanically blocked or removed
- parity is asserted narratively instead of proven with executable coverage

Completion gate:

- every seam deletable through already-existing lower-runtime contracts is gone
- the inventory shows which rows were eliminated rather than merely wrapped
- no certification row depends on a convenience seam that could have been
  deleted

### Phase 3: Specify Missing Lower-Runtime Contracts

Purpose:
turn the remaining specialist seams into explicit lower-runtime contract gaps
rather than normalizing them as permanent Query behavior.

Phase owns these rows:

- `frontier_signal_adapter.rs`
- `effect_lifecycle/execution_bridge.rs`
- any other locked covered row whose inventory entry says the lower-runtime
  contract is still missing
- any downstream Query-integrated runtime-boundary row that still requires a
  missing bridge/relational/signal contract

Work to complete in order:

1. identify the exact missing capability, receipt, or envelope for each
   remaining seam
2. decide whether the missing contract belongs in bridge, relational, or
   signal
3. add or require the narrow lower-runtime contract where feasible
4. delete the Query seam once the contract exists
5. reclassify only truly out-of-scope seams as explicit `DeferredNeighbor`
   rows tied to a named later milestone

Required code moves:

- add the signal frontier facade receipts required to delete specialist signal
  imports from `frontier_signal_adapter.rs`
- add the bridge admitted-writeback execution contract required to collapse
  `effect_lifecycle/execution_bridge.rs` into a thin adapter
- name the exact owning crate for each missing contract and land the contract
  there rather than compensating in Query
- reduce former specialist seams to either a thin allowed adapter or complete
  deletion

Ownership lock for this phase:

- the frontier/planning receipt family is not an open ownership question during
  implementation; for `frontier_signal_adapter.rs`, the missing contract
  belongs in `worth-signal`
- Phase 3 may still decide bridge versus relational versus signal ownership for
  other in-scope seams whose authority is not already locked by this spec

This phase does not close if:

- the phase merely documents the missing contract without landing it
- Query grows a richer local wrapper around a missing lower-runtime contract
  instead of adding the contract
- an in-scope row remains on a seam-elimination path at phase exit
- out-of-scope deferral is used to dodge a gap that belongs in this milestone

Completion gate:

- every surviving specialist seam points to one named missing lower-runtime
  contract or one explicit deferred-neighbor row
- "Query just does this itself" is not an accepted outcome

### Phase 4: Install Route Plans And Boundary Receipts

Purpose:
make all admitted crossings mechanically visible as route-bearing execution
paths instead of loose calls into lower runtimes.

Phase owns these rows:

- `WORTHQueryRuntimeSchemaAdapter::admit_live_view(...)`
- `WORTHQueryRuntimeSignalSinkAdapter::route_write_receipt(...)`
- `WORTHQueryRuntimeSubscriptionActivationAdapter::admit_activation(...)`
- `WORTHQueryRuntimeWriteAuthorityAdapter::write(...)`
- all route-planning and readmission/handoff rows in the locked covered
  crossing table

Work to complete in order:

1. define family-aware `LowerRuntimeCapabilityRequest` authoring
2. define `CapabilityEligibility` across admitted, deferred,
   unsupported, and forbidden states
3. shape `LowerRuntimeRoutePlan` for route-planning seams
4. shape readmission/handoff receipts for non-planning seams
5. ensure every covered operational crossing returns a
   `BoundaryExecutionReceipt`

Required code moves:

- replace weak backend returns with the locked receipt shapes
- embed `WriteAuthorityExecutionReceipt` in the write authority result path
- add `LowerRuntimeCapabilityRequest`, `CapabilityEligibility`,
  `LowerRuntimeRoutePlan`, and `BoundaryExecutionReceipt` artifacts
- thread those artifacts through every covered route-planning seam and every
  covered handoff seam

This phase does not close if:

- a covered operational seam still returns bare `()`, `String`, or an equally
  weak token
- route-plan types exist but are not actually threaded through covered seams
- handoff seams are silently treated like planning seams or vice versa
- downstream adapter crates are left free to re-wrap the new receipts into
  weaker local tokens

Completion gate:

- every covered seam yields a typed route plan or typed handoff posture
- every covered operational seam returns a receipt stronger than `()` or
  `String`
- downstream inspection/certification can recover how the crossing happened

### Phase 5: Shape Boundary Envelopes And Elimination Records

Purpose:
turn receipts into the stable public and internal explanation surface.

Phase owns these rows:

- every covered row that now emits a `BoundaryExecutionReceipt`
- every seam-elimination row
- every explicit `DeferredNeighbor` row

Work to complete in order:

1. define `LowerRuntimeBoundaryEnvelope`
2. bind envelope fields to authority-owned evidence rather than Query-owned
   pseudo-authority
3. define seam-elimination or deferred-neighbor records with owner, missing
   contract or later milestone, required closeout, and certification row
4. expose common-path and advanced-path inspection helpers
5. ensure support metadata and envelopes agree

Required code moves:

- define `LowerRuntimeBoundaryEnvelope`
- bind envelope fields to authority-owned evidence digests and route metadata
- bind support metadata directly to envelope/elimination/deferred posture
- encode seam-elimination records and deferred-neighbor rows in the same
  executable registry consumed by certification

This phase does not close if:

- envelopes summarize crossings without naming authority, route, and retained
  evidence concretely
- elimination or deferred rows live only in prose
- support metadata can drift from envelope or registry posture
- any in-scope seam still relies on “everyone knows this is temporary”

Completion gate:

- every covered crossing can explain authority, route, capability, cost
  posture, failure topology, and retained evidence
- no in-scope direct seam survives as tolerated debt
- no direct seam survives as undocumented convenience

### Phase 6: Close Public And Internal Boundaries

Purpose:
make it mechanically difficult to bypass routed capability lanes.

Phase owns these rows:

- every public facade row in the locked covered crossing table
- every allowed Query internal adapter row
- every downstream domain runtime-boundary subtree row allowed to implement
  Query extension seams

Work to complete in order:

1. add compile-fail boundaries for ordinary callers
2. add internal non-bypass audits for covered Query modules
3. ensure covered public entrypoints delegate through the routed lane
4. ensure support matrices and executable behavior agree
5. close direct-import shortcuts where practical

Required code moves:

- add compile-fail boundaries for ordinary public callers
- add executable direct-import audits for covered Query modules
- add executable direct-import audits for downstream Query-integrated domain
  crates so only one declared runtime-boundary subtree may import lower-runtime
  facades for Query-facing behavior
- ensure covered public entrypoints and downstream adapter seams delegate only
  through the routed lane

This phase does not close if:

- the boundary check is manual rather than executable
- downstream crates like worth-topo are not covered by the audit model
- ordinary domain modules outside the declared runtime-boundary subtree can
  still import bridge/relational/signal for Query-facing behavior
- covered public entrypoints still have more than one ordinary execution path

Completion gate:

- ordinary callers cannot reach lower-runtime internals through the certified
  public surface
- covered Query seams cannot silently bypass the routed lane
- support claims match executable routing behavior

### Phase 7: Certify Closure And Stabilization Readiness

Purpose:
prove the milestone is strong enough for the runtime API freeze to depend on.

Phase owns these rows:

- the final executable inventory
- the final seam-elimination/deferred registry
- every hostile certification lane required to prove route parity,
  non-bypass, and downstream boundary honesty

Work to complete in order:

1. add the named 9.3.6 certification suite
   `9.3.6. Lower-Runtime Capability Routing And Boundary Envelope Closure Test`
2. prove route parity and seam-elimination honesty across hostile lanes
3. prove compile-boundary and non-bypass guarantees
4. prove exact counter and slope honesty
5. bind the final inventory, support matrix, and elimination/deferred registry
   into one
   certification bundle
6. expose a proof-bearing phase artifact manifest that the runtime
   stabilization gate can consume directly instead of re-deriving the 9.3.6
   lifecycle from internal modules
7. expose one public closeout report that binds the final bundle, phase
   manifest, acceptance suite, boundary reconciliation report, and synthetic
   tail report into the exact stabilization-facing artifact

Required code moves:

- add the named 9.3.6 hostile certification suite
  `9.3.6. Lower-Runtime Capability Routing And Boundary Envelope Closure Test`
- expose that named suite as one public certification artifact rather than
  leaving it implied by generic acceptance and bundle surfaces
- add hostile lanes for former specialist seams, deferred neighbors, and
  downstream runtime-boundary enforcement
- bind inventory, support metadata, envelope behavior, and boundary audits into
  one final certification bundle
- add a public phase artifact manifest naming the 9.3.6 lifecycle artifacts,
  their producers, required inputs, next consumers, and enforcement proof
- add a public stabilization closeout report so callers do not reassemble
  Phase 7 from separate certification pieces

This phase does not close if:

- the hostile suite proves only Query internals and not downstream boundary
  behavior
- certification allows a former in-scope seam-elimination row to survive as a
  tolerated posture
- the final bundle can pass while inventory, support metadata, and executable
  behavior disagree

Completion gate:

- the runtime stabilization gate can consume 9.3.6 as one closed routing model
- every in-scope remaining seam is deleted and every out-of-scope remaining
  seam is explicitly deferred by later milestone

## Required Topology

Milestone 9.3.6 should map into responsibility-specific subdomains.

Required subdomains:

- `lower_runtime_routing/inventory`
  - owns crossing inventories, classifications, and elimination/deferred
    registries
- `lower_runtime_routing/eligibility`
  - owns admitted/deferred/forbidden posture resolution
- `lower_runtime_routing/plans`
  - owns route-plan artifacts and planning vocabulary
- `lower_runtime_routing/receipts`
  - owns execution receipt shaping and binding to lower-runtime evidence
- `lower_runtime_routing/envelopes`
  - owns offline-readable boundary envelopes
- `lower_runtime_routing/adapters`
  - owns family-aware Query adapters over lower-runtime contracts
- `lower_runtime_routing/dx`
  - owns public/common-path helpers and inspection affordances
- `lower_runtime_routing/support`
  - owns support metadata, deferred-neighbor reporting, and future-neighbor
    rows
- `lower_runtime_routing/certification`
  - owns route-parity suites, non-bypass audits, slope reports, and
    compile-fail fixtures

Forbidden topology:

- one generic `backend.rs` or `routing.rs` bag that mixes inventory, route
  planning, debt recording, receipt shaping, DX, and certification
- one generic adapter trait so loose that capability family, authority owner,
  and route posture become comment-only
- host/test-only helper modules becoming the de facto routing authority story
- debt registries that live only in prose rather than executable fixtures or
  compile-visible metadata

## Must Ship

- one typed lower-runtime capability-routing lifecycle
- an executable crossing inventory for all covered Query-to-lower-runtime
  seams
- explicit seam classifications: reuse, adapter, debt, deferred neighbor, or
  forbidden duplicate
- route plans and boundary execution receipts for admitted route-planning seams
- typed handoff/receipt closure for admitted readmission-only seams
- lower-runtime boundary envelopes naming authority, route, capability, cost
  posture, failure topology, and retained evidence
- seam-elimination or deferred-neighbor records for every remaining direct
  path, including owner, missing lower-runtime contract or later milestone,
  required closeout, and certification coverage
- support metadata synchronized with executable routed behavior
- compile-fail and internal non-bypass tests proving ordinary callers and
  covered Query modules use the routed lane
- runtime-backed closure for admitted current bridge/relational/signal seams
  while store-backed route parity remains deferred

## Must Preserve

- Query remains the ordinary facade, not the authority
- lower runtimes remain autonomous authorities with contractual facades
- capability routing never hides cost or failure posture behind one generic
  adapter bag
- route planning and readmission/handoff-only lanes remain mechanically
  distinct where downstream behavior differs
- inventory and elimination/deferred records are executable truth, not
  hand-maintained prose
- deferred temporal, async, mixed-cause, store-backed, and durable neighbors
  remain explicit deferred or unsupported posture until their owning
  milestones close

## Acceptance Evidence

This milestone is complete only when a hostile certification program can:

- enumerate every covered Query-to-lower-runtime crossing from the executable
  inventory
- prove every covered crossing is classified exactly once
- prove deletable seams were deleted rather than merely normalized
- prove surviving specialist seams correspond to named missing lower-runtime
  contracts or explicit deferred-neighbor rows
- prove every admitted crossing yields one route plan or handoff posture, one
  boundary receipt, and one boundary envelope
- prove equivalent admitted paths normalize to the same lower-runtime route
  meaning and evidence family
- prove intentionally different route families, authorities, or basis/support
  postures change the declared digests
- prove ordinary callers and covered Query modules cannot bypass the routed
  lane through lower-runtime convenience paths
- prove support metadata, elimination/deferred registry, and executable
  behavior agree

## Required Verification Output

The 9.3.6 certification bundle must emit:

- `query_digest`
- `capability_request_digest`
- `capability_family_digest`
- `capability_eligibility_digest`
- `lower_runtime_route_plan_digest`
- `boundary_execution_receipt_digest`
- `lower_runtime_boundary_envelope_digest`
- `crossing_inventory_digest`
- `crossing_classification_digest`
- `compatibility_debt_registry_digest`
- `debt_exit_criteria_digest`
- `route_authority_digest`
- `route_evidence_digest`
- `route_cost_posture_digest`
- `route_failure_topology_digest`
- `route_support_matrix_digest`
- `route_public_surface_digest`
- `route_target_dx_digest`
- `route_golden_transcript_digest`
- `route_proof_shape_digest`
- `route_phase_progression_digest`
- `route_parity_digest`
- `route_non_bypass_digest`
- `lower_runtime_gap_registry_digest`
- `compile_fail_boundary_digest`
- `failure_digest`
- `counter_snapshot`
- `crossing_inventory_width`
- `compatibility_debt_width`
- `route_plan_width`
- `boundary_evidence_width`
- `capability_eligibility_slope_digest`
- `route_plan_assembly_slope_digest`
- `boundary_receipt_assembly_slope_digest`
- `boundary_envelope_assembly_slope_digest`
- `support_lookup_slope_digest`
- `debt_registry_lookup_slope_digest`

## Architectural Notes

- A direct lower-runtime seam is not neutral. It is either something to delete
  or something to push into explicit later-milestone deferral.
- Query owns routing and envelope shape, not lower-runtime authority.
- "Backend abstraction exists" is not enough. 9.3.6 closes only when each seam
  is individually classifiable and certified.
- A weak operational return like `()` or `String` is evidence that the
  boundary story is still under-specified for a covered seam.
- The correct fix for a surviving specialist seam is often a better bridge,
  relational, or signal contract, not a more elaborate Query wrapper.
- Store-backed route parity and durable route replay are real future work and
  must stay explicit deferred posture until their milestones close.
- Intent family support ambiguity must be resolved honestly in routing/support
  metadata. If a family is publicly described but backend-gated in practice,
  the route/support story must say so explicitly.

## Deferred Scope

The following remain explicitly deferred:

- store-backed route parity
- durable route replay
- persisted boundary execution receipts
- restart-stable boundary envelope reload
- temporal query basis routing
- async/resource routing
- mixed truth/time/async routing
- final temporal/async/store/durable certification closure

Any 9.3.6 surface that encounters those neighbors must report typed deferred,
or unsupported posture rather than partial support.

## Sequencing Notes

Milestone 9.3.6 belongs after 9.3.5 because routing should consume admitted
plans and typed handoffs rather than inventing another admission story.

It belongs after 9.3.4 because lower-runtime source contact must bind to
declared projection/materialization contracts rather than raw source payloads.

It belongs after 9.3.3 because routed write/effect contact needs
authority-scoped lowered execution receipts already in place.

It belongs after 9.3.2 because every crossing must consume admitted basis
capability rather than raw branch/history/preview/policy inputs.

It belongs before the runtime stabilization gate because the public runtime API
must freeze only after lower-runtime contact is one closed routing model rather
than a collection of convenience seams.

## Closeout Standard

This milestone may close only when:

- every covered Query-to-lower-runtime crossing is present in the executable
  inventory
- every covered crossing is classified as reuse, adapter, debt, deferred, or
  forbidden duplicate
- every deletable seam has been deleted
- every surviving specialist seam points to one named missing lower-runtime
  contract or one explicit deferred-neighbor row
- every admitted crossing yields one route plan or handoff posture, one
  boundary execution receipt, and one boundary envelope
- compile-fail and internal non-bypass audits prove covered paths cannot route
  around Query capability routing by convenience
- support metadata, executable routing behavior, and elimination/deferred
  records agree
- exact counters and slope digests prove routing work scales with route width,
  evidence width, and deferred-neighbor width rather than unrelated runtime
  breadth
- roadmap and test-requirement references point at this spec and its named
  certification suite accurately

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes: it closes the remaining lower-runtime escape-hatch seam
  before the runtime API freeze.
- Is the adversarial constraint precise and load-bearing? Yes: it forbids
  convenience-based lower-runtime path choice and forces deletion, better
  lower-runtime contracts, or explicit deferral.
- Does the milestone preserve crate authority boundaries? Yes: Query owns
  routing and envelopes while bridge, relational, signal, and store keep their
  own authorities.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes: executable inventories, seam classifications, route receipts,
  elimination/deferred rows, compile-fail boundaries, non-bypass audits, and
  exact slope evidence are all explicit requirements.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes: the phase progression, inventory model, crate changes, topology,
  and verification outputs name the required artifacts directly.
- Does the milestone belong in this roadmap sequence? Yes: it turns the
  9.3.1 through 9.3.5 lifecycles into one closed lower-runtime crossing model
  before the public runtime facade freezes.
