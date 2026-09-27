# WORTH Server Test Requirements

## Scope

This document defines the certification-grade server test requirements for:

- Milestone 1
- Milestone 2
- Milestone 3
- Milestone 4
- Milestone 5
- Milestone 6
- Milestone 7
- Milestone 8
- Milestone 9
- Milestone 10
- Milestone 11
- Milestone 12
- Milestone 13
- Milestone 14

## Purpose

`worth-server` cannot be considered shipped merely because:

- an HTTP route returns a body
- a WebSocket sends frames
- a lease survives one reconnect
- a WORTH-native client can fetch a view
- a background webhook fires once
- two broad result objects happen to compare equal in one happy-path run

The server is making claims about:

- Query-first semantic projection rather than route-local meaning
- direct WORTH-native product ergonomics without endpoint-glue reimplementation
- runtime-backed lease identity and runtime-backed resume honesty
- typed delivery classes, basis negotiation, and mixed-cause delivery
- tenant, branch, policy, and remask safety on active delivery
- audit-grade provenance and regulated deployment evidence
- shared subscription reuse, view-shaped patching, and materialized fanout
- branch-aware optimistic mutation and provenance-bearing result closure
- background delivery, CDC-facing integration lanes, and outbox/saga honesty
- durable restart-stable resume and anti-entropy recovery
- blind-server and transport-upgrade parity
- cluster routing, invalidation coordination, and distributed certification

Those are all adversarial surfaces. They need miserable-path certification, not
feature checks.

## Global Adversarial Constraint

The server test suite must prove the following:

> Under reconnect churn, resume mismatch, branch drift, tenant drift,
> permission drift, remask pressure, mixed-cause delivery races, pacing
> variation, slow consumers, stale clients, restart boundaries, retention
> truncation, transport variation, view-shape overlap, integration callback
> hostility, diagnostics-tier variation, topology variation, and deliberate
> misuse of WORTH-native facade surfaces, the server must preserve canonical
> Query-owned meaning, typed denial, explicit authority boundaries, machine-
> checkable provenance, and strict capability honesty without allowing route-
> local glue, broad equality comparisons, scheduler timing, socket order,
> transport history, or operator folklore to redefine semantics.

For Milestone 2 specifically, this also means the direct facade must preserve
canonical declaration identity, admitted-versus-visible support posture,
retained async/time result-state posture, and projection-fact receipts rather
than allowing a WORTH-native caller to recreate those contracts through local
builders, status enums, or cache folklore.

If a server surface works only:

- on one transport
- in one tenant
- on one branch
- with one consumer pace
- without restart
- without permission drift
- without retention hostility
- or with only broad "looks equivalent" assertions

then it is not certified.

## Meta-Rules

These tests are all certification tests. They must:

- emit canonical machine-checkable artifacts, not "the response looked right"
- compare independently produced artifacts under declared semantic
  relationships
- prove typed failure localization for rejected, stale, truncated, or drifted
  lanes
- prove diagnostics richness changes retained detail only, not semantic truth
- prove runtime-backed and durable-later surfaces remain distinct until a
  milestone explicitly closes the durable contract
- verify exact counter contracts whenever the server claims boundedness,
  pacing discipline, shared reuse, materialization efficiency, or backpressure
  honesty
- prove transport variation changes transport shape only when semantic parity
  is claimed
- prove that regulated-evidence surfaces are reconstructable from retained
  artifacts rather than ad hoc host logs
- prove that WORTH-native facade ergonomics do not hide capability posture,
  replay debt, or route-local fallback behavior

These requirements are mandatory, not advisory.

## Global Certification Shape

Every named certification suite must define at least these lanes unless the
suite explicitly states a narrower reason:

- `control_lane` - canonical admitted baseline
- `hostile_lane` - adversarial variation being certified
- `equivalent_lane` or `replay_lane` - an independently produced equivalent,
  resumed, restarted, or alternate-surface execution

If the suite is about explicit rejection, the hostile lane may terminate in a
typed failure, but it still requires a successful or equivalent comparison
basis.

## Mandatory Assertion Classes

Every named certification suite must include all applicable assertion classes:

- equality assertions for semantically equivalent lanes
- inequality assertions for intentionally different semantic lanes
- typed-failure assertions for rejected lanes
- zero-or-absence assertions for forbidden residue, forbidden fallback,
  forbidden lane bleed, forbidden authority widening, and forbidden evidence
  gaps

## Anti-Fake-Test Rule

The following do not count as certification:

- asserting only that a route returned `200`
- asserting only that a socket received frames
- asserting only that a digest is present or non-empty
- comparing a value only to itself from the same run
- asserting only top-level object equality when the server claims structured
  lane, basis, provenance, or policy meaning
- validating only a happy path without an adversarial lane
- validating only one transport or one facade when the milestone claims parity
- relying on host logs, tracing text, or debugger inspection as the primary
  proof artifact

## Anti-Broad-Equality Rule

Broad high-level equality is specifically insufficient for `worth-server`.

When a milestone claims parity, the suite must compare the narrow canonical
artifacts that actually encode the claimed meaning. At minimum, suites must
compare the smallest applicable set from:

- `surface_contract_digest`
- `declaration_digest`
- `request_context_digest`
- `lease_digest`
- `lease_registry_digest`
- `basis_digest`
- `resume_digest`
- `delivery_digest`
- `delivery_class_digest`
- `lane_digest`
- `presence_digest`
- `view_patch_digest`
- `materialization_digest`
- `support_posture_digest`
- `retained_state_digest`
- `fact_receipt_digest`
- `policy_digest`
- `remask_digest`
- `tenant_digest`
- `branch_digest`
- `mutation_result_digest`
- `provenance_digest`
- `audit_evidence_digest`
- `integration_digest`
- `cdc_digest`
- `topology_digest`
- `failure_digest`
- `counter_snapshot`

If a suite claims equivalence but only compares one top-level response object
or one fully flattened payload digest, the suite is insufficient unless it
also proves why no narrower artifact could have diverged silently.

## Strict Assertion Rule

Every suite must declare what must be asserted exactly.

Acceptable examples:

- exact frontier advancement
- exact lane membership
- exact ordered delivery-class sequence
- exact basis or resume incompatibility code
- exact zero count for forbidden shared-base widening
- exact counter value for branch-scope lookups, tenant mismatches, or replay
  fallback invocations

Unacceptable examples:

- "some failure occurred"
- "the client eventually converged"
- "the logs showed the right behavior"
- "the objects were deeply equal" without narrower artifact checks

## Counter Assertion Rule

Whenever a milestone claims boundedness, reuse, pacing discipline, replay
honesty, materialization efficiency, or degraded-freshness correctness, the
suite must assert exact counter values for the representative scenario,
including counters that must remain zero.

Range checks are acceptable only when the suite proves why the variability is
real contract surface rather than measurement slop.

## Canonical Server Certification Bundle

At minimum, certification bundles should emit the canonical fields applicable
to the suite scope:

- `surface_contract_digest`
- `declaration_digest`
- `request_context_digest`
- `response_digest`
- `lease_digest`
- `lease_registry_digest`
- `basis_digest`
- `resume_digest`
- `delivery_digest`
- `delivery_class_digest`
- `lane_digest`
- `presence_digest`
- `view_patch_digest`
- `materialization_digest`
- `support_posture_digest`
- `retained_state_digest`
- `fact_receipt_digest`
- `policy_digest`
- `remask_digest`
- `tenant_digest`
- `branch_digest`
- `mutation_result_digest`
- `provenance_digest`
- `audit_evidence_digest`
- `integration_digest`
- `cdc_digest`
- `topology_digest`
- `failure_digest`
- `counter_snapshot`

Not every suite uses every field, but every suite should emit a stable,
scope-appropriate canonical bundle rather than free-form debug text.

## Section Index

- [Milestones 1-3: Facade, Direct Consumption, And Compatibility Surface](#milestones-1-3-facade-direct-consumption-and-compatibility-surface)
- [Milestone 3: Binary And Asset Boundary](#milestone-3-binary-and-asset-boundary)
- [Milestones 4-4.1: Product Operation Results And Durable Mutation](#milestones-4-41-product-operation-results-and-durable-mutation)
- [Milestones 5-6: Leases, Sync, Resume, And Delivery Classes](#milestones-5-6-leases-sync-resume-and-delivery-classes)
- [Milestones 7-8: Policy, Remask, Regulated Evidence, And Recovery Honesty](#milestones-7-8-policy-remask-regulated-evidence-and-recovery-honesty)
- [Milestones 9-10: Shared Bases, View Patches, And Mutation Closure](#milestones-9-10-shared-bases-view-patches-and-mutation-closure)
- [Milestones 11-14: Integrations, Durability, Zero-Trust, And Distributed Certification](#milestones-11-14-integrations-durability-zero-trust-and-distributed-certification)
- [Cross-Milestone Hostility Suites](#cross-milestone-hostility-suites)

## Milestones 1-3: Facade, Direct Consumption, And Compatibility Surface

### 1. Shared Pipeline Non-Bypass Torture Test

Purpose

Prove that every facade, WORTH-native, and compatibility surface crosses the
same typed middleware and execution pipeline, and that route-local bypasses are
mechanically visible rather than silently possible.

Scenario

- run equivalent operations through:
  - the WORTH-native facade
  - the compatibility HTTP surface
  - alternate route shapes where admitted
- inject tenant mismatch, branch mismatch, auth failure, and diagnostics-policy
  variation
- include at least one direct-surface declaration-intake lane that carries
  canonical declaration identity and admitted support posture before ordinary
  read or mutation execution
- attempt deliberately malformed handlers or test-only route stubs that skip
  one pipeline phase

Must verify

- equivalent surfaces produce identical `request_context_digest` after
  middleware lowering
- denied auth, tenant, branch, and authorization paths fail at the expected
  phase boundary
- skipped middleware phases are mechanically discoverable and fail
  certification
- direct declaration intake cannot bypass canonical declaration or support
  posture lowering before the shared pipeline
- diagnostics richness changes retained detail only

Required verification output

- `surface_contract_digest`
- `declaration_digest`
- `request_context_digest`
- `support_posture_digest`
- `policy_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

No admitted surface can silently become its own semantic pipeline.

### 2. WORTH-Native No-Glue Equivalence Test

Purpose

Prove that a WORTH-native application can consume server-managed Query meaning
directly without requiring a parallel handwritten endpoint family to preserve
semantics.

Scenario

- express the same ordinary product flow through:
  - direct WORTH-native facade consumption
  - a compatibility API surface
- vary branch targeting, basis posture, remask posture, and diagnostics
  richness
- vary declaration/view-shape intake, admitted-versus-visible support posture,
  and at least one admitted retained async/time or projection-fact lane that
  belongs to the same product flow
- inject a tempting endpoint-only convenience shortcut that would flatten or
  omit capability posture

Must verify

- direct-consumption and compatibility lanes compare equal on canonical Query
  meaning where overlap exists
- direct-consumption lanes preserve canonical declaration identity and
  admitted-versus-visible support posture rather than teaching support from
  visible method names
- direct-consumption lanes retain explicit capability posture rather than
  hiding unsupported runtime-backed versus durable-later distinctions
- admitted retained async/time posture and projection-fact receipts remain
  parity-safe and do not degrade into caller-owned status enums, anonymous
  payloads, or cache folklore
- shortcut surfaces that erase basis, remask, or support posture fail
  certification

Milestone 2 split note

- this suite must certify declaration intake and retained-state/fact-consumption
  parity for admitted direct-consumption lanes
- it does not need to certify later view-patch family transport behavior; that
  belongs to Milestone 9's `View-Patch Family Precision Test`

Required verification output

- `surface_contract_digest`
- `declaration_digest`
- `basis_digest`
- `support_posture_digest`
- `retained_state_digest`
- `fact_receipt_digest`
- `policy_digest`
- `remask_digest`
- `provenance_digest`
- `failure_digest`

Pass condition

WORTH-native ergonomics reduce glue without reducing semantic honesty.

### 3. Compatibility Surface Path-Honesty Test

Purpose

Prove that the compatibility request/response surface does not define a second
meaning model and does not quietly diverge from the WORTH-native facade.

Scenario

- run equivalent reads and mutations across:
  - compatibility HTTP
  - direct facade
- vary streaming versus buffered response shape
- vary declaration/view-shape intake for overlap surfaces where admitted
- vary request ordering and client retry timing
- inject malformed basis, malformed branch targeting, and unsupported request
  combinations

Must verify

- streaming and buffered lanes compare equal on canonical response meaning
- overlap requests preserve the same canonical declaration identity and support
  posture across compatibility and direct surfaces where the same public lane is
  claimed
- compatibility routes cannot widen basis or branch semantics
- retries do not alter canonical result meaning when the operation is
  semantically equivalent
- rejected request combinations fail typed before semantic drift occurs

Required verification output

- `surface_contract_digest`
- `declaration_digest`
- `response_digest`
- `basis_digest`
- `branch_digest`
- `support_posture_digest`
- `mutation_result_digest`
- `failure_digest`

Pass condition

Compatibility HTTP remains an interop surface, not a second server brain.

## Milestone 3: Binary And Asset Boundary

### 4. Blob/Truth Separation Hostility Test

Purpose

Prove that large binary transfer never pollutes the structured sync contract
and that metadata linkage remains explicit and honest.

Scenario

- upload and download large assets while structured truth changes occur in
  parallel
- vary multipart chunk boundaries and range requests
- inject slow consumers, interrupted transfers, and unauthorized range access

Must verify

- blob transfer does not appear inside structured delivery lanes
- file metadata changes remain truth-linked and separately inspectable
- interrupted or unauthorized binary paths fail without corrupting structured
  delivery state
- binary and structured counters remain independently explainable

Required verification output

- `surface_contract_digest`
- `response_digest`
- `delivery_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Blob transport and truth sync remain distinct under hostile transfer behavior.

## Milestones 4-4.1: Product Operation Results And Durable Mutation

### 4A. Canonical Product Result Artifact Test

Purpose

Prove that every successful product operation publishes one bounded,
schema-versioned result artifact and that direct, compatibility HTTP, retry, and
diagnostic surfaces consume that artifact without side-channel result storage.

Scenario

- execute equivalent product reads and mutations through WORTH-native and HTTP
- vary JSON object ordering, diagnostics richness, and request timing
- return wrong-schema and oversized result bodies from hostile adapters
- retry completed operations after rebuilding the server runtime

Must verify

- equivalent bodies canonicalize once to identical body and artifact digests
- output schema identity and version are mandatory registration truth
- wrong-schema and oversized bodies cannot publish a success envelope
- direct and HTTP surfaces expose exact schema, body, and digest parity
- retries return the original artifact without result-key lookup middleware

Required verification output

- `declaration_digest`
- `result_schema_digest`
- `result_body_digest`
- `result_artifact_digest`
- `response_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

No successful product operation requires producer-local storage to interpret or
retrieve its ordinary structured result.

### 4B. Durable Product Mutation Crash Boundary Test

Purpose

Prove that product-owned durable mutation joins basis comparison, idempotency,
state change, next basis, result artifact, and completion into one recoverable
conclusion.

Scenario

- exercise host-connection, manifest-admission, and deployment-shaped mutations
- kill or fault the executor before intent, after intent, after mutation, after
  commit, and before server acknowledgment
- retry with identical and conflicting idempotency bindings after fresh runtime
  construction
- return an indeterminate conclusion and resolve it through the typed handle

Must verify

- acknowledged work survives once and identical retries return its completion
- unacknowledged work resolves to committed, not committed, rejected, or typed
  indeterminate without blind re-execution
- stale basis and conflicting key bindings perform exactly zero mutation
- product mutation and completion never use the process-local retry map
- ordinary retry evidence does not import or claim cert replay authority

Required verification output

- `durable_attempt_digest`
- `basis_digest`
- `idempotency_binding_digest`
- `mutation_result_digest`
- `result_artifact_digest`
- `recovery_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Every crash edge resolves to one permitted durable product conclusion with no
duplicate product effect.

### 4C. Durable Product Scope And Isolation Test

Purpose

Prove that durable mutation locality scales by tenant, workspace, and declared
product authority scope rather than a session key or global lock.

Scenario

- submit same-scope and disjoint-scope durable mutations concurrently
- reuse identical key text across tenants and workspaces
- vary operation version, expected basis, and product authority scope
- pressure local scheduling and product transaction conflict independently

Must verify

- same-scope work shares a deterministic lane while disjoint scopes remain
  independently admissible
- cross-tenant and cross-workspace keys never disclose another completion
- scheduler ordering is not accepted as proof of current durable basis
- exact counters explain attempts, basis comparisons, commits, previous
  completions, conflicts, stale bases, indeterminate outcomes, and result bytes

Required verification output

- `authority_digest`
- `plan_digest`
- `scheduler_digest`
- `idempotency_binding_digest`
- `result_artifact_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Durable product mutation remains tenant-isolated, scope-local, and transaction-
correct without server-wide serialization.

## Milestones 5-6: Leases, Sync, Resume, And Delivery Classes

### 5. Lease Identity And Reconnect Fracture Test

Purpose

Prove that lease identity survives reconnect churn while remaining distinct
from socket identity, route identity, and raw CDC cursor identity.

Scenario

- create equivalent and intentionally different lease declarations
- churn connections repeatedly while the server process stays live
- vary consumer pacing and ack timing
- inject partial reconnects, abandoned sessions, and stale reconnect attempts

Must verify

- equivalent lease declarations yield identical `lease_digest`
- distinct declarations compare unequal on canonical lease identity
- reconnect preserves lease identity without conflating it with transport
  session identity
- abandoned or stale reconnect attempts fail typed with no false-positive lease
  resurrection

Required verification output

- `lease_digest`
- `lease_registry_digest`
- `resume_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Lease identity is runtime-owned, stable under reconnect, and not transport
folklore.

### 6. Runtime-Backed Resume Honesty Test

Purpose

Prove that runtime-backed resume is admitted and useful now, but never
misrepresented as durable restart-stable closure.

Scenario

- resume within one live server process from valid runtime-backed bases
- attempt restart-shaped resumption with no durable contract
- inject stale, incompatible, and truncated runtime-backed bases

Must verify

- valid runtime-backed resume lanes compare equal to uninterrupted control
  lanes on canonical visible truth
- stale or incompatible bases fail typed before delivery
- restart-shaped resumptions without durable support cannot silently pass
- capability posture exposes runtime-backed versus durable-later distinction

Required verification output

- `basis_digest`
- `resume_digest`
- `delivery_digest`
- `failure_digest`
- `surface_contract_digest`

Pass condition

The server is honest about what resume it has and what resume it does not have.

### 7. Delivery-Class And Lane Differentiation Torture Test

Purpose

Prove that delivery classes and lanes are real semantic distinctions, not
labels pasted onto one generic payload stream.

Scenario

- produce authoritative, replaceable, coalescible, presence, and advisory
  deliveries over one shared workload
- vary pacing, coalescing, and consumer lag
- inject cases where only one lane should fire and cases where multiple lanes
  must remain distinct

Must verify

- delivery classes remain mechanically distinguishable in canonical artifacts
- invalidation, patch, and presence lanes cannot alias into one another
- coalescing or pacing variation does not collapse class identity
- forbidden lane bleed remains zero

Required verification output

- `delivery_digest`
- `delivery_class_digest`
- `lane_digest`
- `presence_digest`
- `counter_snapshot`

Pass condition

The server can prove not just that something was delivered, but exactly what
kind of delivery happened and what did not happen.

### 8. Multiplexing And Backpressure Miserable-Path Test

Purpose

Prove that large payloads, slow consumers, and degraded freshness posture do
not redefine canonical delivery meaning or starve high-priority small frames.

Scenario

- mix large patch payloads with small presence and invalidation frames
- introduce slow consumers, artificial socket backpressure, and pacing churn
- trigger admitted degradation from strict to coalesced freshness postures

Must verify

- small high-priority frames are not starved behind large payload transfer
- degradation changes pacing policy only, not canonical semantic parity
- backpressure counters match the exact declared degradation path
- forbidden reorder or cross-lane collapse remains zero

Required verification output

- `delivery_digest`
- `lane_digest`
- `presence_digest`
- `counter_snapshot`
- `failure_digest`

Pass condition

Backpressure changes cost posture and timing only; it does not change meaning.

## Milestones 7-8: Policy, Remask, Regulated Evidence, And Recovery Honesty

### 9. Remask Drift And Permission-Churn Test

Purpose

Prove that remask, denial, and permission drift affect active delivery before
network emission and never appear as after-the-fact filtering folklore.

Scenario

- begin with admitted active leases
- change permission, policy, tenant, and relationship-proof posture while
  leases remain active
- vary the timing relative to queued outbox work and reconnect attempts

Must verify

- remask posture changes `remask_digest` while preserving canonical denial
  semantics
- forbidden delivery after remask or denial remains zero
- outbox items that became unauthorized do not escape after drift
- equivalent policy changes produce equivalent remask artifacts across direct
  and compatibility surfaces where overlap exists

Required verification output

- `policy_digest`
- `remask_digest`
- `delivery_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Permission drift changes what may be emitted, not what the test hopes was
filtered out later.

### 10. Cross-Tenant And Cross-Branch Leak Resistance Test

Purpose

Prove that branch-local and tenant-local truth cannot bleed across active
delivery, even under shared infrastructure pressure.

Scenario

- run overlapping workloads across multiple tenants and branches
- reuse shared infrastructure where admitted
- vary reconnect timing, shared-base candidates, and simultaneous updates

Must verify

- cross-tenant and cross-branch bleed remains zero
- shared infrastructure reuse does not change `tenant_digest` or
  `branch_digest` correctness
- intentionally different tenant or branch lanes compare unequal in the right
  artifacts and do not require payload inspection to prove the difference

Required verification output

- `tenant_digest`
- `branch_digest`
- `lease_digest`
- `delivery_digest`
- `counter_snapshot`

Pass condition

Isolation is structural, not best-effort filtering.

### 11. Regulated Evidence Sufficiency Test

Purpose

Prove that regulated operators can reconstruct exposure, denial, and policy
authorization from retained server artifacts alone.

Scenario

- execute admitted and denied operations across reads, delivery, and mutations
- vary diagnostics richness
- attempt offline reconstruction from retained artifacts only

Must verify

- audit-grade evidence remains sufficient without host log spelunking
- diagnostics richness changes retained detail only
- evidence surfaces distinguish:
  - exposed truth
  - denied truth
  - remasked truth
  - basis or policy reason for each outcome

Required verification output

- `audit_evidence_digest`
- `provenance_digest`
- `policy_digest`
- `failure_digest`

Pass condition

Regulated evidence is reconstructable, typed, and not dependent on operator
memory or logs.

### 12. Recovery-Honesty Boundary Test

Purpose

Prove that clients and operators can tell exactly when the server is still in
runtime-backed territory versus when durable closure would be required.

Scenario

- compare live reconnect, process restart, retention truncation, and resume
  mismatch paths
- vary capability advertisement and diagnostics tiers
- attempt operations that would only be honest after durable closure

Must verify

- runtime-backed success paths and durable-later rejection paths are distinct
  in canonical artifacts
- capability advertisement matches actual recovery behavior
- forbidden "soft success" on durable-only operations remains zero

Required verification output

- `surface_contract_digest`
- `resume_digest`
- `audit_evidence_digest`
- `failure_digest`

Pass condition

The server never markets runtime-backed behavior as durable closure.

## Milestones 9-10: Shared Bases, View Patches, And Mutation Closure

### 13. Shared-Base Reuse Without Semantic Bleed Test

Purpose

Prove that shared subscription bases can reduce work without changing
per-client visible truth or weakening per-client policy posture.

Scenario

- run equivalent and overlapping leases with and without shared-base reuse
- vary remask posture, branch targeting, and consumer pace
- inject lookalike leases that should not share a base

Must verify

- shared and non-shared lanes compare equal on client-visible truth when they
  are semantically equivalent
- forbidden base sharing remains zero for lookalike-but-non-equivalent leases
- shared reuse does not weaken policy, tenant, or branch separation
- exact reuse counters match the declared scenario

Required verification output

- `lease_digest`
- `lease_registry_digest`
- `delivery_digest`
- `policy_digest`
- `counter_snapshot`

Pass condition

Reuse is explicit and safe, not a heuristic that silently merges neighbors.

### 14. View-Patch Family Precision Test

Purpose

Prove that view-shaped patches are canonical projections of Query meaning
rather than server-local patch inventions.

Scenario

- project the same truth changes into multiple admitted view shapes such as:
  - table
  - detail
  - grouped
  - timeline
  - chart
- vary shared-base reuse and materialization posture
- inject patch-shape perturbations that should alter packaging but not meaning

Must verify

- different admitted view shapes produce intentionally different
  `view_patch_digest` values while preserving canonical truth parity
- materialized and non-materialized paths compare equal on visible meaning
- server-local patch invention or undocumented patch widening fails
  certification

Required verification output

- `view_patch_digest`
- `materialization_digest`
- `delivery_digest`
- `basis_digest`
- `counter_snapshot`

Pass condition

View patches differ where they should and only where they should.

### 15. Materialization Drift And Fanout Pressure Test

Purpose

Prove that materialized hot views and non-materialized recomputation converge
to the same truth under fanout pressure and update churn.

Scenario

- create high-fanout collaborative surfaces
- run one lane with maintained materialization and one lane without
- inject rapid update storms, reconnect churn, and coalescing variation

Must verify

- materialized and recomputed lanes compare equal on canonical visible truth
- fanout, reuse, and materialization counters match the declared workload
- materialization residue after teardown remains zero

Required verification output

- `materialization_digest`
- `delivery_digest`
- `view_patch_digest`
- `counter_snapshot`

Pass condition

Materialization is an optimization, not a second source of visible truth.

### 16. Optimistic Mutation Rollback Truth Test

Purpose

Prove that optimistic server-facing mutation flows remain branch-aware and
provenance-safe under rejection, drift, and confirmation races.

Scenario

- run optimistic mutation attempts across admitted and rejected branches
- vary confirmation timing, basis drift, and concurrent conflicting writes
- compare direct facade and compatibility mutation surfaces where overlap
  exists

Must verify

- confirmed and rejected mutation flows produce distinct, typed
  `mutation_result_digest` values
- rollback preserves canonical explanation truth rather than transport-local
  folklore
- optimistic mismatch or drift never lands as silent success
- provenance surfaces localize the exact reason for rejection or confirmation

Required verification output

- `mutation_result_digest`
- `branch_digest`
- `provenance_digest`
- `failure_digest`

Pass condition

Optimistic UX remains fast without becoming a second conflict model.

## Milestones 11-14: Integrations, Durability, Zero-Trust, And Distributed Certification

### 17. CDC/App-Surface Non-Aliasing Test

Purpose

Prove that integration-facing CDC remains an explicit lane and does not silently
become the ordinary app-facing semantic surface.

Scenario

- consume one workload through:
  - ordinary Query-shaped app delivery
  - CDC-shaped integration delivery
- vary restart, pacing, and replay posture
- inject attempts to reconstruct app meaning from raw CDC alone where not
  admitted

Must verify

- app-facing and CDC-facing lanes remain intentionally distinct in canonical
  artifacts
- CDC consumers can resume honestly on their own contract without redefining
  app-facing delivery meaning
- attempted semantic aliasing fails certification

Required verification output

- `delivery_digest`
- `cdc_digest`
- `resume_digest`
- `failure_digest`

Pass condition

CDC is honest and useful, but it is not the ordinary app semantic surface.

### 18. Integration Callback And Outbox Failure Containment Test

Purpose

Prove that external callback failure, webhook hostility, and outbox retries do
not create partial authoritative success or split-brain mutation folklore.

Scenario

- run outbox-backed cross-system flows with success, retry, timeout, and
  rejection paths
- inject malformed callbacks, duplicate callbacks, and delayed callbacks
- compare authoritative truth before and after failure injection

Must verify

- outbox and callback failures do not silently partially commit authoritative
  truth
- duplicate callbacks do not become duplicate authority effects
- retry and rejection posture remains typed and provenance-bearing

Required verification output

- `integration_digest`
- `mutation_result_digest`
- `provenance_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Cross-system hostility remains contained and typed rather than becoming split-
brain ambiguity.

### 19. Durable Resume, Truncation, And Anti-Entropy Recovery Test

Purpose

Prove that once durable closure is admitted, restart-stable resume and
anti-entropy recovery are real contracts rather than optimistic heuristics.

Scenario

- compare uninterrupted, runtime-backed, durable-restart, truncation, and
  anti-entropy recovery lanes
- inject retention truncation, stale checkpoints, and incompatible retained
  bases
- require independent artifact production before comparison

Must verify

- uninterrupted and valid durable-restart lanes compare equal on canonical
  visible truth
- truncation and incompatibility produce typed failure or typed anti-entropy
  recovery posture exactly where declared
- anti-entropy recovery remains explicit and mechanically distinguishable from
  cursor resume
- replay of every missed transport event is not required for canonical parity

Required verification output

- `resume_digest`
- `basis_digest`
- `delivery_digest`
- `failure_digest`
- `counter_snapshot`

Pass condition

Durable restart and anti-entropy are explicit, typed, and parity-safe.

### 20. Blind-Server And Transport-Upgrade Parity Test

Purpose

Prove that optional blind-server modes and transport upgrades preserve
canonical meaning where admitted and fail typed where not admitted.

Scenario

- run equivalent workloads across:
  - ordinary WebSocket transport
  - WebTransport where admitted
  - blind-server compatible deployment posture where admitted
- inject unsupported combinations, partial metadata visibility, and transport
  downgrade pressure

Must verify

- admitted transport variations compare equal on canonical visible semantics
- unsupported combinations fail typed rather than silently degrading
- blind-server compatible lanes preserve the same declared meaning without
  inventing a second sync contract

Required verification output

- `surface_contract_digest`
- `delivery_digest`
- `provenance_digest`
- `failure_digest`

Pass condition

Transport and cryptographic mode changes alter mechanics only where parity is
claimed.

### 21. Cluster Parity, Affinity, And Distributed Invalidation Test

Purpose

Prove that single-node and multi-node deployments preserve canonical delivery
truth under affinity routing, invalidation propagation, reconnect churn, and
topology change.

Scenario

- run equivalent workloads on:
  - single-node control
  - multi-node affinity-routed cluster
- inject node-local lag, invalidation propagation delay, reconnect churn, and
  topology movement
- compare direct facade, compatibility, and sync surfaces where overlap exists

Must verify

- equivalent single-node and cluster lanes compare equal on canonical visible
  truth
- invalidation propagation delay does not become semantic drift
- topology changes do not widen tenant or branch visibility
- exact counters prove the declared routing and invalidation path

Required verification output

- `topology_digest`
- `delivery_digest`
- `tenant_digest`
- `branch_digest`
- `counter_snapshot`

Pass condition

Distributed topology changes routing mechanics, not delivery truth.

## Cross-Milestone Hostility Suites

### 22. Miserable-Path Matrix Requirement

Every certification milestone must include at least one suite that combines
multiple hostility classes at once rather than testing each in isolation.

Minimum combined hostility classes:

- reconnect or restart pressure
- policy or permission drift
- pacing or backpressure variation
- branch or tenant divergence
- diagnostics-tier variation

The point of this rule is to prevent false confidence from single-axis tests.

### 23. No-Fallback-Without-Evidence Rule

Any fallback, degradation, anti-entropy path, replay recovery, or transport
downgrade must prove all of the following:

- exactly why the primary path could not continue
- exactly which fallback path was selected
- exactly which canonical artifacts changed because of the fallback
- exact zero counts for forbidden hidden fallback paths

Without those assertions, the suite is insufficient.

### 24. Strict Narrow-Artifact Comparison Rule

For any parity claim involving:

- remask
- delivery classes
- lanes
- lease identity
- resume
- view patches
- mutation provenance
- regulated evidence
- CDC versus app delivery
- cluster coordination

the suite must compare at least three narrow canonical digests from the
relevant surface plus any exact counters the milestone claims.

One broad response equality check is never enough.

### 25. What These Tests Collectively Prove

Together, these tests prove that `worth-server` is:

- Query-first rather than route-local
- ergonomically direct for WORTH-native apps rather than endpoint-glue-driven
- explicit about runtime-backed versus durable-later capability boundaries
- strict about lease identity, basis honesty, and resume honesty
- incapable of silently collapsing delivery classes, lanes, or presence
  semantics into one generic stream
- strict about tenant, branch, policy, and remask isolation on active delivery
- fit for regulated environments because evidence, denial, and recovery posture
  are machine-checkable
- explicit about shared-base reuse, materialization, and view-patch semantics
- provenance-bearing and branch-aware for optimistic mutation flows
- honest about CDC as an integration lane rather than ordinary app meaning
- parity-safe across transport, topology, and cluster coordination where
  parity is claimed
- certifiable through narrow canonical artifacts and exact assertions rather
  than broad high-level equality theater

## Milestone Certification Rule

No `worth-server` milestone should be considered closed until its named
certification suites pass across:

- original execution
- one or more hostile miserable-path lanes
- one independently produced equivalent, replay, resumed, restarted, or
  alternate-surface lane where applicable

Without that, the server surface may look promising, but it is not yet
trust-grade.
