# Milestone 4: Concurrent Operation Admission And Product Surface Runtime

> **Status:** Draft
>
> **Purpose:** update `worth-server` to consume the real `worth-query`
> concurrency and consumer-kit closures, then admit product-application
> operations through one server-owned operation runtime so product-editor web
> servers do not grow route-local semantics, global locks, copied snapshots, or
> client-owned stale-basis folklore.

## Goal

Milestone `4` closes the gap between the shipped WORTH-native/compatibility
surfaces and the server-owned runtime needed by real product applications.

By the end of this milestone:

- `worth-server` consumes `worth-query` Milestones `9.7` and `9.8` through
  their public support, shared-read, deterministic-submission, and consumer-kit
  surfaces rather than preserving pre-concurrency assumptions
- every server operation enters through a typed operation request contract with
  canonical identity before any route, facade, or product adapter can execute
  product work
- every operation declares authority footprint, authorization, support,
  precondition, and concurrency posture before execution
- the server lowers operation requests into proof-bearing execution plans and
  schedules them through one operation runtime
- shared-read-safe operations can run concurrently while mutation/submission
  operations remain deterministic
- product-application facades can bind operations such as editor render,
  select, action inspection, apply, and stricter finalization without teaching
  `worth-server` product semantics
- optimistic product sessions carry explicit base-digest, idempotency, stale
  basis, conflict, and rebase posture
- external routes are assembled from operation declarations rather than
  handwritten semantic handlers
- product-editor-shaped operations are certified as a pressure case while
  remaining outside `worth-server` semantic authority

Milestone `4` does **not** ship durable lease persistence, WebSocket sync,
restart-stable resume, shared subscription bases, product-editor domain
semantics, or Postgres-backed product storage. It freezes the server operation
runtime those later capabilities and product servers must consume.

## Why This Milestone Exists

Milestones `1` through `3` give `worth-server` a real front door, WORTH-native
facade, and external HTTP/binary surface. But they were built before Query's
real concurrency and consumer-kit closures were available.

The tempting next mistake is to move directly into leases, sync, or product
editor endpoints:

- keep global or route-local locking because server code still assumes the old
  single-borrow Query topology
- let product applications wrap their own stale-basis, idempotency, support,
  and response folklore around the server
- treat product editor operations as "just routes" and accidentally make the
  first real product web app a second server runtime
- execute route handlers from decoded DTOs rather than lowered operation plans
- infer parallel safety from handler names instead of declared authority
  footprints

This milestone prevents that failure by introducing the missing server-owned
operation admission, planning, scheduling, product-adapter, and route assembly
boundary before later lease/sync work and before a downstream product server
has a chance to harden a one-off architecture.

## Governing Summaries

- `MENTALITY.md`: protects adversarial-constraint-first design and scope
  expansion. Server concurrency, operation planning, and product adapter
  authority are foundations, not features to patch around with endpoint glue.
- `arch_laws.md`: protects autonomous subsystems, facade-only access,
  proof-bearing phase transitions, lowered execution plans, structured
  envelopes, authority/derivation separation, and concurrency from structural
  disjointness rather than speculative locks.
- `composition_laws.md`: protects named responsibility flow. Request decoding,
  operation identity, authority-footprint classification, planning, scheduling,
  product adaptation, response shaping, diagnostics, and certification must not
  collapse into handlers.
- `domain_structure_laws.md`: protects visible authority boundaries. Query
  runtime authority, server operation authority, product application semantics,
  product sessions, diagnostics, transport, and persistence must occupy distinct
  structural homes.
- `perf_laws.md`: protects cost honesty. Shared reads must use Query's
  concurrency primitives; mutation serialization must be explicit; APIs must
  expose coordination breadth, stale-basis posture, and scheduling cost through
  counters and receipts.
- `worth_server_roadmap.md`: protects the server's role as network delivery and
  operation runtime, not a second Query runtime or route handler bag.
- `worth-query` Milestone `9.7`: protects the real upstream concurrency
  contract: shared read contexts, deterministic submission, published derived
  artifacts, typed journal/replay, and hostile concurrent certification.
- `worth-query` Milestone `9.8`: protects correct downstream consumption:
  evidence-report kit, prohibition registry/audit, support snapshots, support
  pinning, and in-memory test backend. Server and product consumers must use
  these rather than hand-built support or adapter folklore.

## Adversarial Constraint

For the same authenticated principal, tenant/workspace target, branch/basis
posture, Query support posture, diagnostics posture, product-operation
identity, and declared operation authority footprint, `worth-server` must lower
Query-direct and product-application operations into one typed operation plan,
schedule all shared-read-safe work concurrently, serialize or deny
mutation/submission conflicts deterministically, preserve equivalent envelopes
across WORTH-native and HTTP surfaces, and prevent product-editor or any
product crate from smuggling route-local semantics, copied snapshot state,
global locks, or client-owned stale-basis recovery into the server boundary.

This milestone fails if any admitted path:

- executes product work from a raw route handler or facade DTO without an
  operation request contract and lowered plan
- keeps pre-`9.7` Query concurrency assumptions alive behind server locks or
  copied snapshot state
- derives journal, support, or stale-basis posture from strings, route-local
  status codes, or consumer-owned rows
- lets product code bypass request context, middleware, operation planning,
  scheduler, response shaping, or diagnostics
- hides conflicting mutation or stale snapshot posture behind retry folklore
- makes a downstream product server the first owner of server concurrency or
  operation-admission semantics

## Product Decision Lock

- `worth-server` owns operation admission, planning, scheduling, response
  envelopes, diagnostics, route assembly, and transport projection.
- `worth-query` owns Query runtime meaning, shared read authority,
  deterministic submission, support posture, projection consumption, and
  consumer-kit artifacts.
- Product crates own product semantics. A product editor facade may render,
  select, apply, or finalize its own product operations, but it does so through
  server operation declarations and adapters.
- Surface families remain transport and entry families. Operation families are
  a separate admission/category axis. Operation family never determines
  scheduling by itself.
- Concurrency is admitted only from authorization proof, support posture,
  precondition posture, declared authority footprints, concurrency class, and
  lowered execution plans together. Executors do not infer parallel safety.
- Optimistic product sessions are server-admitted coordination artifacts, not
  browser-owned truth and not product-domain authority.
- HTTP route assembly is derived from operation declarations. Route handlers
  decode and enter the operation runtime; they do not execute semantics.
- Product-editor readiness is certification pressure, not a reason to put
  product-specific rules inside `worth-server`.

## Core Operation Model

Milestone `4` must keep these concepts distinct. They are related, but no type
may collapse them into one magic operation bag:

- `OperationFamily`: admission/category axis. It says what kind of operation is
  being requested, such as Query direct read, product application mutation, or
  binary transfer. It does not schedule work.
- `OperationDeclaration`: the named operation contract inside a family, such as
  `product_editor.render` or `query.direct_read`. It binds payload schema
  identity, adapter support, route projection, and expected authority shape.
- `OperationIdentity`: what was asked, independent of diagnostics richness. It
  includes caller/tenant/workspace/branch targeting, operation family,
  declaration identity, product/session target where present, basis inputs, and
  payload identity.
- `AuthorityFootprint`: what authority or state the operation touches. It
  names Query basis, product basis, product draft/session scope, binary
  transfer scope, and diagnostics-only scope where relevant.
- `AuthorizationProof`: whether the authenticated principal may perform the
  operation over the declared footprint. Footprint classification is not
  authorization.
- `AdmissionPosture`: whether the requested operation family/declaration is
  registered, enabled, supported, and available under current server, Query,
  product, and diagnostics policy.
- `PreconditionPosture`: whether caller-supplied basis, session, idempotency,
  branch, and base digest inputs are current and usable.
- `ConcurrencyClass`: how a planned operation may be scheduled relative to
  other plans, derived from the footprint, authorization proof, admission
  posture, and precondition posture.
- `PlanIdentity`: what the server is allowed to run. It changes when support
  snapshots, adapter schema versions, authority footprints, authorization
  proof, preconditions, or execution strategy change.
- `EvidenceIdentity`: what proof was materialized. Diagnostics richness may
  change evidence identity and response evidence, but not operation identity,
  plan identity, scheduling result, or product outcome.

### Product Basis Contract

Product read and mutation declarations must name the basis kind they consume:

- `QueryDerived`: product operation reads from Query-owned basis or projection
  posture.
- `ProductSessionDerived`: product operation reads from a server-admitted
  product session snapshot.
- `DurableProductDerived`: product operation reads from durable product truth
  admitted by a later persistence boundary.
- `FixtureOnly`: certification/test-only product basis that may not satisfy
  production readiness.

Concurrent product reads are admitted only when basis kind, digest semantics,
and comparability are declared. A product adapter declaration that claims
shared-read safety without comparable basis identity is unsupported.

### Denial Families

Typed denial is not a single enum bucket. Milestone `4` admits these denial
families:

- `TransportDenial`: malformed JSON, oversized body, unsupported encoding, or
  operational route decoding failures before operation request construction.
- `RequestDenial`: unknown operation, malformed idempotency key, conflicting
  branch/preview target, malformed basis input, or invalid operation envelope.
- `AdmissionDenial`: absent/disabled surface or operation family, unsupported
  declaration, or unavailable route/operation registration.
- `SupportDenial`: missing Query support, missing product support, incompatible
  product basis, fixture-only support requested in production, or unknown
  product support posture.
- `AuthorizationDenial`: authenticated principal may not perform the operation
  over the declared footprint.
- `AuthorityDenial`: missing, ambiguous, broad, or undeclared authority
  footprint.
- `PreconditionDenial`: stale basis, branch mismatch, foreign session, missing
  base digest, idempotency conflict, or invalid product session posture.
- `SchedulerDenial`: conflict, unsupported queue posture, stale scheduled
  basis, or closed execution lane.
- `ProductDenial`: product-owned validation, product stale/rebase reason, or
  product semantic refusal.

Every denial family must shape through the server response facade. Route-local
status strings and product-local error strings are not denial posture.

### Support Composition

The planner composes Query support and product support without merging their
meaning:

- If a product operation depends on Query and Query support is unavailable,
  planning returns `SupportDenial`.
- If a product operation is Query-independent, Query support absence does not
  deny it.
- If product support is unavailable, unknown, or fixture-only outside an
  admitted certification/test profile, planning returns `SupportDenial`.
- If Query and product support are individually present but their basis posture
  is incompatible, planning returns `SupportDenial` or `PreconditionDenial`
  according to whether the failure is capability or caller-basis freshness.
- The composed support receipt names the Query rows consulted, product rows
  consulted, dependency relation, and final planner posture.

### Locking Rule

Locks are forbidden as authority substitutes, not as implementation mechanisms.
Any lock that affects operation ordering, admission, shared-read concurrency,
or mutation/submission execution must be classified and counted. Immutable
registry access, metrics, local queue mechanics, and adapter-map reads may use
implementation locks only when they do not serialize independent authority
footprints. Ordinary admitted shared-read and deterministic
mutation/submission paths may not rely on a global execution lock; any nonzero
forbidden-global-lock counter is support failure except on explicitly
classified legacy or static-test-only paths.

### Route Scope

Operation-declared route assembly covers semantic operation routes. Server
operational routes such as health checks, metrics scrape endpoints, static
asset serving, CORS/preflight mechanics, and documentation/spec export may
exist outside operation declarations when they do not execute Query or product
semantics. Their failures still shape through transport or response policy,
not route-local folklore.

## Phase Plan

### Phase 1: Query Concurrency And Consumer-Kit Debt Audit

This phase adds the audit boundary that determines where `worth-server` still
depends on pre-`9.7` Query topology or pre-`9.8` consumer folklore before any
new operation runtime is admitted.

By the end of this phase, the server has an inventoried, support-visible
classification of every path that reads Query state, submits Query work,
constructs support posture, fabricates evidence, or assembles test backends.
The audit records separate dimensions rather than one overlapping label:
`PathKind`, `RuntimeReadiness`, `ConsumerKitPosture`, `ScopePosture`, and
`ClosurePosture`.

**Relevant subsystems**
- Query dependency audit
- WORTH-native direct facade
- compatibility HTTP read/mutation execution
- query handoff
- server certification support

**Relevant APIs**
- `WORTHServerQueryDependencyAudit`
- `WORTHServerQueryDependencyAuditRow`
- `WORTHServerQueryDependencyAuditPathKind`
- `WORTHServerQueryDependencyRuntimeReadiness`
- `WORTHServerQueryDependencyConsumerKitPosture`
- `WORTHServerQueryDependencyScopePosture`
- `WORTHServerQueryDependencyClosurePosture`
- `WORTHServerQueryDependencyAuditReceipt`
- `WORTHServerQueryDependencySupportPosture`

**Required boundaries now**
- Query `9.7` closure is consumed through shared-read, deterministic
  submission, journal/replay, and support/profile artifacts where server paths
  need those capabilities.
- Query `9.8` closure is consumed through support snapshots, support pinning,
  boundary audit, and in-memory test backend where server tests or downstream
  posture need those capabilities.
- A server path may not be silently treated as concurrent merely because it is
  read-shaped; it must be inventoried and classified.

**Warnings**
- Do not begin operation scheduling work while covered Query paths remain
  `LegacyAssumption` without an owner phase.
- Do not preserve hand-built support rows in `worth-server` when Query `9.8`
  supplies a pinned support snapshot surface.
- Do not classify copied snapshots, display-string journal ordering, or broad
  mutex protection as acceptable concurrency posture.

**Test requirements**
- `server_query_concurrency_dependency_audit_classifies_every_covered_path`:
  adversarial inventory completeness proving every WORTH-native direct read,
  state, inspection, mutation, projection, compatibility read, compatibility
  mutation, query handoff, and covered test-backend path has one classification.
- `server_paths_consume_query_9_7_shared_read_and_submission_closures`:
  equivalence proof that representative read and submission paths preserve
  canonical envelopes when lowered through Query's real shared-read and
  deterministic-submission surfaces.
- `server_support_posture_uses_query_9_8_pinning_instead_of_local_rows`:
  residue proof that covered support posture no longer comes from server-local
  gap rows or hand-built support matrices.
- `legacy_concurrency_assumption_blocks_operation_runtime_closure`: hostile
  guard proving the milestone cannot report operation-runtime readiness while
  any covered ordinary path has `RuntimeReadiness::LegacyAssumption`,
  `ConsumerKitPosture::LocalFolklore`, `ScopePosture::Unclassified`, or
  `ClosurePosture::Blocked` without a later phase owner.

**Engineering decisions**
- This audit is a production support artifact, not a test-only grep.
- Static/test-only paths must be explicitly classified and excluded from
  operation-runtime closure by classification, not by omission.
- Query dependency posture is derived from public Query support artifacts and
  server path inventory together.

**Open questions**
- None.

### Phase 2: Server Operation Family Registry

This phase introduces operation families as a distinct server authority axis
from transport surface families. A surface family says how a caller enters. An
operation family says what kind of server-owned operation authority is being
requested.

By the end of this phase, route families, WORTH-native entrypoints, future sync
leases, binary transfers, and product application operations all register
through one operation-family registry before execution is possible.

**Relevant subsystems**
- operation family registry
- surface registry integration
- WORTH-native facade
- compatibility facade
- diagnostics counters

**Relevant APIs**
- `WORTHServerOperationFamily`
- `WORTHServerOperationRegistration`
- `WORTHServerOperationRegistry`
- `WORTHServerOperationInventory`
- `WORTHServerOperationCapabilities`
- `WORTHServerOperationRegistryError`

**Required boundaries now**
- Operation families include at minimum:
  - `QueryDirectRead`
  - `QueryDirectSubmission`
  - `QueryDirectProjection`
  - `ProductApplicationRead`
  - `ProductApplicationMutation`
  - `ProductSessionCoordination`
  - `BinaryTransfer`
  - `SyncLease`
- Surface registration may expose operation families, but may not execute them.
- Duplicate, disabled, absent, or unsupported operation families deny typed
  before request-context or semantic lowering performs product work.

**Warnings**
- Do not add product-specific operation families for individual downstream
  editors. Product editor operations are product-application operations with
  product-owned semantics.
- Do not collapse operation family into compatibility route family; HTTP route
  taxonomy is a transport concern.
- Do not let operation registration accept arbitrary closures that bypass
  planning or scheduling.

**Test requirements**
- `operation_family_inventory_is_distinct_from_surface_inventory`: adversarial
  parity proving a WORTH-native request and a compatibility HTTP request can
  share operation family while retaining distinct surface families.
- `unregistered_operation_family_denies_before_execution`: rejection proof that
  a route or facade entry cannot execute operation work when its operation
  family is absent, disabled, duplicated, or unsupported.
- `operation_registry_rejects_transport_shaped_semantic_shortcuts`: sabotage
  proof that registering a compatibility route family without an operation
  family cannot smuggle executable server behavior.

**Engineering decisions**
- Operation families are server-owned vocabulary because they decide admission,
  planning, scheduling, and response shaping.
- Product operation specialization happens below the product-application
  family through operation declarations, not by widening the top-level enum for
  each product.

**Open questions**
- None.

### Phase 3: Canonical Operation Request Contract

This phase builds the canonical request contract that binds caller input to one
operation identity before operation planning. The contract is the first
server-owned artifact that can be compared across WORTH-native, compatibility
HTTP, and product-adapter entry.

By the end of this phase, no operation can reach planning from raw path
parameters, raw JSON, product DTOs, or direct facade arguments.

**Relevant subsystems**
- operation request decoding
- request context integration
- operation identity
- idempotency and basis input normalization
- response denial localization

**Relevant APIs**
- `WORTHServerOperationRequestInput`
- `WORTHServerOperationRequest`
- `WORTHServerOperationIdentity`
- `WORTHServerOperationInputEnvelope`
- `WORTHServerOperationRequestDenial`
- `WORTHServerOperationRequestReceipt`

**Required boundaries now**
- Operation identity includes surface family, operation family, tenant,
  workspace, branch or preview target, declared operation name, optional basis,
  optional product session, and optional idempotency key where those inputs are
  admitted. Diagnostics profile is request/evidence policy, not semantic
  operation identity.
- Request contract construction consumes a resolved request context. It cannot
  construct or replace request-context authority.
- Product payloads remain opaque at this phase except for envelope-level
  identity, size, schema-version, idempotency, and basis fields needed for
  admission.
- Product payload structural validation before scheduling may occur only
  through a product-supplied schema validator declared on the product operation;
  route code may not own product payload schema semantics.

**Warnings**
- Do not parse product semantics in request-contract construction.
- Do not let HTTP path strings become operation identity without canonical
  normalization and typed operation-family binding.
- Do not accept caller-provided digest strings as current authority; they are
  precondition inputs that later phases admit or deny.

**Test requirements**
- `equivalent_surface_inputs_lower_to_identical_operation_identity`:
  equivalence proof that WORTH-native and compatibility HTTP forms of the same
  operation produce identical canonical operation identity where their semantic
  inputs match.
- `malformed_operation_request_denies_before_planning`: hostile denial proof
  for missing operation family, conflicting branch/preview target, malformed
  idempotency key, unsupported diagnostics profile, unknown operation name,
  invalid payload envelope, and invalid declared schema identity.
- `diagnostics_policy_does_not_change_semantic_operation_identity`: equivalence
  proof that changing only diagnostics richness changes evidence/response
  materialization posture but not operation identity.
- `operation_request_identity_rejects_display_string_ordering`: residue proof
  that operation identity and idempotency comparison do not depend on display
  formatting, path order, or raw JSON object order.

**Engineering decisions**
- Operation request identity is a boundary artifact and must be digest-bearing.
- Product payload digest participates as payload identity, not as proof that
  product semantics were admitted.
- Request contract receipts are diagnostic-policy aware but diagnostics
  richness cannot change operation identity. Diagnostics richness may change
  evidence identity.

**Open questions**
- None.

### Phase 4: Operation Authority Footprint And Authorization Proof

This phase adds the first half of the proof-bearing posture set that makes
concurrency a planned property instead of route-local optimism.

By the end of this phase, every operation request must produce a distinct
authority footprint and authorization proof before any support composition,
precondition evaluation, or execution planning occurs. The server can
distinguish shared read, deterministic submission, product draft mutation,
product session coordination, binary streaming, diagnostics-only work, and
future lease work without executing the operation body.

**Relevant subsystems**
- operation authority footprint
- operation authorization
- operation admission posture
- product application declaration metadata
- diagnostics counters

**Relevant APIs**
- `WORTHServerOperationAuthorityFootprint`
- `WORTHServerOperationAuthorizationProof`
- `WORTHServerOperationAdmissionPosture`
- `WORTHServerOperationScope`
- `WORTHServerOperationFootprintReceipt`

**Required boundaries now**
- `SharedReadOnly` operations must declare the Query shared-read basis or the
  product snapshot basis they consume.
- Product read declarations must classify basis as `QueryDerived`,
  `ProductSessionDerived`, `DurableProductDerived`, or `FixtureOnly`. Only
  comparable, digest-bearing basis kinds can later admit concurrent product
  reads.
- `DeterministicSubmission` operations must declare the Query submission lane
  and journal posture they require.
- `ProductDraftMutation` operations must declare product session identity,
  draft scope, base digest posture, and idempotency posture.
- `BinaryStreaming` operations must remain distinct from structured truth
  operations even when they share request context. In this milestone,
  `BinaryTransfer` is operation-family reserved/minimal unless a declaration
  supplies preflight, size, cancellation, and partial-failure posture.
- Footprint classification is not authorization. Authorization consumes request
  context, operation identity, and footprint and emits an authorization proof
  before later posture composition or planning succeeds.
- Unknown, broad, or ambiguous footprint becomes `AuthorityDenial`, not
  "serialize everything just in case."

**Warnings**
- Do not infer concurrency from method names such as `read`, `render`, or
  `apply`; only declared footprints classify concurrency.
- Do not represent broad unknown authority as a successful footprint. Unknown
  authority denies or remains unsupported.
- Do not allow product adapters to declare footprints after execution has
  started.
- Do not treat adapter declarations as self-proving. Runtime scheduling trusts
  adapter declarations only after registration-time validation and
  certification prove required footprint fields cannot be omitted.

**Test requirements**
- `declared_shared_read_footprints_admit_concurrent_planning`: equivalence
  proof that independent shared-read footprints over the same admitted basis
  can be planned together and produce serialized-equivalent envelopes later.
- `product_read_basis_must_be_declared_and_comparable`: rejection proof that
  product reads claiming shared-read safety without declared basis kind,
  comparable digest semantics, or production-admitted support fail before
  later concurrency admission.
- `authorization_proof_is_required_after_footprint_classification`: hostile
  proof that a valid footprint with an unauthorized principal fails as
  `AuthorizationDenial`, not as support, scheduler, or product denial.
- `conflicting_product_draft_footprints_fail_before_execution`: hostile denial
  proving two mutation operations over the same product draft/session cannot
  both enter mutable execution without deterministic ordering or typed
  conflict posture.
- `ambiguous_operation_authority_cannot_fall_back_to_global_lock`: sabotage
  proof that an operation with missing or broad unknown footprint fails
  `AuthorityDenial` instead of passing by taking a global lock.

**Engineering decisions**
- Footprints and authorization proofs are separate proof-bearing inputs to
  later posture composition and planning; neither is an advisory annotation.
- Concurrency is not closed in this phase. This phase freezes the authority
  facts that later support, precondition, and planning phases consume.
- Product operations may be conservative, but conservatism must be explicit
  and typed.

**Open questions**
- None.

### Phase 5: Support Composition, Preconditions, And Concurrency Classification

This phase adds the second half of the posture set: Query/product support,
caller-supplied freshness and session preconditions, and the concurrency class
that planning may rely on.

By the end of this phase, every authorized operation request must produce
support composition, precondition posture, and concurrency classification
before lowering to an execution plan. The server can distinguish capability
denial from caller-freshness denial and can classify when concurrent scheduling
is admitted versus when deterministic serialization or typed denial is
required.

**Relevant subsystems**
- operation admission posture
- operation precondition posture
- concurrency classification
- Query support/admission posture
- Query/product support composition
- product application declaration metadata
- diagnostics counters

**Relevant APIs**
- `WORTHServerOperationAdmissionPosture`
- `WORTHServerOperationPreconditionPosture`
- `WORTHServerOperationConcurrencyClass`
- `WORTHServerOperationSupportPosture`
- `WORTHServerOperationSupportCompositionReceipt`

**Required boundaries now**
- Missing Query/product support becomes `SupportDenial`; stale or invalid
  caller basis/session/idempotency becomes `PreconditionDenial`.
- Query and product support compose without merging their meanings or
  flattening product-independent operations into Query-coupled denials.
- Concurrency classification consumes operation identity, authority footprint,
  authorization proof, support composition, and precondition posture together.
  It may not infer parallel safety from route shape or operation names.
- Product read declarations that lack production-admitted support or
  comparable digest semantics may not claim shared-read-safe concurrency.

**Warnings**
- Do not let precondition posture silently widen into capability posture or
  vice versa.
- Do not let support composition become a second planner. It classifies
  readiness and compatibility; it does not choose execution strategy.
- Do not let product adapters recover unsupported or stale requests by
  inventing local fallback posture after this phase denies them.

**Test requirements**
- `query_and_product_support_compose_without_meaning_merge`: matrix proof for
  Query unsupported/product dependent, Query unsupported/product independent,
  product unsupported, product unknown, fixture-only outside test profile, and
  incompatible basis posture.
- `stale_or_invalid_preconditions_deny_before_planning`: hostile denial proof
  for stale basis, foreign session, malformed base digest, missing idempotency
  binding inputs, and conflicting branch/base posture.
- `concurrency_class_requires_support_and_precondition_closure`: classification
  proof that shared-read, deterministic submission, product draft mutation, and
  session coordination lanes are admitted only after support and preconditions
  are both resolved.
- `support_and_precondition_denials_do_not_fall_back_to_serialization`: sabotage
  proof that unsupported or stale operations deny typed instead of being forced
  through a conservative global queue.

**Engineering decisions**
- Support posture, precondition posture, and concurrency class are separate
  proof-bearing inputs to planning; none is an advisory annotation.
- The concurrency class is server-owned because it controls scheduling, but it
  consumes Query and product declarations, authorization proof, support
  posture, and precondition posture for authority facts.

**Open questions**
- None.

### Phase 6: Lowered Operation Planning Boundary

This phase adds the operation planner. The planner consumes a canonical
operation request plus authority footprint and emits the only artifact an
executor or scheduler may run.

By the end of this phase, execution cannot branch on raw request shape,
transport family, product DTO, or route-local policy. The lowered plan carries
the request proof, authority footprint, authorization proof, admission posture,
support composition, precondition posture, evidence policy, and execution
strategy.

**Relevant subsystems**
- operation planning
- Query handoff
- response planning
- evidence policy
- execution strategy selection

**Relevant APIs**
- `WORTHServerOperationPlanner`
- `WORTHServerLoweredOperationPlan`
- `WORTHServerOperationPlanReceipt`
- `WORTHServerOperationPlanDenial`
- `WORTHServerOperationExecutionStrategy`
- `WORTHServerOperationPlanCounters`

**Required boundaries now**
- The planner is the only place that may choose between shared read,
  deterministic submission, product adapter execution, session coordination,
  and binary transfer strategies.
- Executors consume `WORTHServerLoweredOperationPlan`, never raw operation
  request input.
- Plan receipts carry support composition, footprint digest, strategy
  selection, authorization proof, precondition posture, expected scheduler
  lane, plan identity, and evidence policy.
- Query support and product adapter support must both be checked before
  execution planning succeeds.
- Plan identity changes when support snapshots, adapter schema versions,
  authority footprints, authorization proof, preconditions, or execution
  strategy change. Diagnostics richness changes evidence identity, not plan
  identity.

**Warnings**
- Do not let the scheduler re-decide execution strategy. The scheduler
  schedules the lowered plan it is given.
- Do not let product adapters perform admission that belongs to request
  context, middleware, operation family, authorization, support composition,
  precondition posture, or concurrency planning.
- Do not flatten plan denial into HTTP status or product error strings.

**Test requirements**
- `equivalent_operation_requests_lower_to_identical_plans`: replay/equivalence
  proof that repeated and cross-surface equivalent operation requests produce
  identical lowered plan receipts.
- `executor_cannot_accept_raw_operation_input`: boundary proof, preferably
  compile-fail where possible, that execution entry points require lowered
  plans and cannot be called with raw request contracts or product payloads.
- `unsupported_query_or_product_support_denies_at_planning`: hostile denial
  proving unsupported Query support rows, missing product adapter support,
  product fixture-only posture outside test profile, or disabled operation
  family fails before scheduling.
- `plan_identity_excludes_diagnostics_richness_but_includes_support_and_strategy`:
  matrix proof that diagnostics-only changes affect evidence identity, while
  Query support snapshot, product adapter schema, footprint, authorization,
  precondition, or scheduler strategy changes affect plan identity.
- `plan_counters_explain_strategy_selection`: exact-counter proof that plan
  receipts expose support rows consulted, footprint breadth, strategy choice,
  and evidence policy without host logs.

**Engineering decisions**
- Planning is a proof-widening phase: request plus footprint becomes lowered
  plan.
- Plan identity is not operation identity. Operation identity says what was
  asked; plan identity says what the server is allowed to run.
- Diagnostic richness can affect evidence materialization, but not operation
  identity, plan identity, scheduling result, or operational result.

**Open questions**
- None.

### Phase 7: Shared-Read Scheduler

This phase introduces the shared-read half of the server-owned scheduler for
lowered operation plans. It is the first boundary that may run multiple
read-safe operation plans together.

By the end of this phase, shared-read-safe plans execute concurrently through
Query `9.7` shared read contexts or product read snapshots where admitted.
Independent read failures stay isolated, and the scheduler can prove
serialized-replay-equivalent envelopes for legal shared-read interleavings.

**Relevant subsystems**
- operation scheduler
- execution slots
- Query shared-read execution
- scheduler diagnostics and counters

**Relevant APIs**
- `WORTHServerOperationScheduler`
- `WORTHServerScheduledOperationBatch`
- `WORTHServerOperationExecutionSlot`
- `WORTHServerScheduledOperationOutcome`
- `WORTHServerSchedulerConflictDenial`
- `WORTHServerSchedulerFailurePosture`
- `WORTHServerSchedulerCancellationPosture`
- `WORTHServerOperationSchedulerCounters`

**Required boundaries now**
- Shared-read plans may run concurrently only when their footprints prove
  compatible basis and no mutable product/session scope.
- Scheduler outcomes include runtime-sourced counters for read slots,
  queue decisions, cancellation stops, isolated failures, and
  forbidden-global-lock acquisitions.
- A failed shared read does not poison independent shared-read plans in the
  same scheduled batch unless they share the failed Query/product basis.
- Client cancellation posture for shared reads is explicit and counted.

**Warnings**
- Do not use a global server mutex as the concurrency model. If a lock remains
  as implementation detail, it must be classified, counted, and absent from
  ordinary admitted shared-read paths.
- Do not let product adapters spawn their own unsupervised concurrent work for
  shared-read authority-bearing operations.

**Test requirements**
- `concurrent_shared_read_scheduler_matches_serialized_replay`: hostile matrix
  with multiple Query-direct and product-read plans proving byte-identical
  response envelopes to serialized replay across repeated interleavings.
- `shared_read_hot_path_reports_exact_zero_global_lock_acquisitions`: exact
  counter proof for the admitted shared-read path.
- `scheduler_failures_are_isolated_by_declared_dependency`: hostile proof that
  one shared-read/product-read failure does not fail independent scheduled
  reads, while dependent reads sharing the failed basis receive typed failure
  posture.
- `scheduler_cancellation_records_only_after_admission`: cancellation proof for
  before-admission, after-admission-before-execution, and during-execution
  cases for shared-read scheduling.

**Engineering decisions**
- The scheduler is server-owned mechanism; it does not own Query meaning or
  product semantics.
- Concurrency proof comes from lowered plans and authority footprints, not
  runtime object inspection.
- Scheduler certification must include sabotage-sensitive counters, not
  hard-coded zeros.

**Open questions**
- None.

### Phase 8: Deterministic Submission And Product Mutation Scheduler

This phase extends the scheduler from shared-read concurrency into the ordered
lanes that must never rely on ambient thread timing: Query deterministic
submission, product draft mutation, and session-coordination mutation work.

By the end of this phase, deterministic submissions and product mutations are
ordered by the authority lane declared in their plans. Conflicts, stale basis,
queue closure, and post-admission cancellation become typed scheduler outcomes
with exact counters.

**Relevant subsystems**
- operation scheduler
- execution slots
- Query deterministic submission
- product mutation execution
- session-coordination execution
- scheduler diagnostics and counters

**Relevant APIs**
- `WORTHServerOperationScheduler`
- `WORTHServerScheduledOperationBatch`
- `WORTHServerOperationExecutionSlot`
- `WORTHServerScheduledOperationOutcome`
- `WORTHServerSchedulerConflictDenial`
- `WORTHServerSchedulerFailurePosture`
- `WORTHServerSchedulerCancellationPosture`
- `WORTHServerOperationSchedulerCounters`

**Required boundaries now**
- Deterministic submission plans execute through one ordered submission lane
  and carry journal/submission posture from Query where applicable.
- Product mutation plans over the same product draft/session either serialize
  deterministically or deny conflict before product execution.
- A denied or failed mutation records idempotency according to the later
  idempotency matrix once scheduler admission has accepted the operation.
- Client cancellation posture is explicit: cancellation before scheduler
  admission records no idempotent result; cancellation after scheduler
  admission records the scheduler outcome according to operation family.

**Warnings**
- Do not let thread scheduling determine submission order.
- Do not let product adapters spawn their own unsupervised concurrent work for
  authority-bearing mutation operations.
- Do not turn unsupported mutation concurrency into "serialize everything"
  fallback without a typed denial or explicit scheduler lane.

**Test requirements**
- `submission_and_product_mutation_order_is_deterministic`: equivalence proof
  that interleaved mutation/submission requests produce stable ordered receipts
  and replay-equivalent outcomes.
- `conflicting_mutation_plans_localize_scheduler_denial`: rejection proof that
  same-session draft mutations with incompatible base posture fail or serialize
  according to plan, never by product-local race.
- `scheduler_cancellation_for_mutation_paths_records_after_admission_only`:
  cancellation proof for before-admission, after-admission-before-execution,
  and during-execution mutation cases with replay posture localized by phase.
- `mutation_scheduler_reports_exact_lane_and_conflict_counters`: exact-counter
  proof for submission slots, product mutation slots, conflicts denied, stale
  stops, and forbidden global-lock residue.

**Engineering decisions**
- Ordered mutation scheduling is a distinct closure from shared-read
  scheduling because the failure modes are different and easier to hide.
- Deterministic submission and product mutation share one scheduler runtime but
  not one semantic authority model.

**Open questions**
- None.

### Phase 9: Product Application Adapter Boundary

This phase admits product-application operations as a first-class server
operation lane without making `worth-server` own product semantics.

By the end of this phase, a product crate can declare operation families such
as editor render, visible-node selection, action availability, edit apply, and
stricter finalization. The product adapter executes the
product facade only after server request context, middleware, operation
identity, authority footprint, planning, and scheduling have all admitted the
operation.

**Relevant subsystems**
- product operation adapter
- product operation declaration
- product support posture
- product outcome mapping
- response envelope integration

**Relevant APIs**
- `WORTHServerProductApplicationAdapter`
- `WORTHServerProductOperationDeclaration`
- `WORTHServerProductOperationPayload`
- `WORTHServerProductOperationOutcome`
- `WORTHServerProductOperationDenial`
- `WORTHServerProductOperationErrorMap`
- `WORTHServerProductOperationSupportSnapshot`
- `WORTHServerProductOperationBasisKind`
- `WORTHServerProductPayloadSchemaValidator`
- `WORTHServerProductAdapterRegistrationReceipt`

**Required boundaries now**
- Product adapters declare operation names, payload schema identity, support
  posture, product basis kind, authority footprint requirements, optional
  payload schema validator, and result/denial mapping.
- Product adapters receive lowered product execution plans, not raw HTTP
  requests or unresolved request context.
- Product errors map into typed server product denials or product failures
  while preserving product-owned reason keys and server-owned envelope
  structure.
- Product adapter declarations are trusted inputs only after registration-time
  validation and certification prove required footprint, support, basis, and
  schema fields cannot be omitted.
- If product payload structural validation is required before scheduling, it is
  performed through the declared `WORTHServerProductPayloadSchemaValidator`;
  product semantic validation remains product-owned execution/admission logic.
- Product-editor-shaped operations are expressed as declarations such as
  `product_editor.render`, `product_editor.select_visible_node`,
  `product_editor.available_actions`, `product_editor.apply_edit`, and
  `product_editor.finalize_change`, but those names are product adapter
  examples, not hardcoded server variants.

**Warnings**
- Do not add downstream product editor modules to `worth-server`.
- Do not let product adapters construct server response envelopes directly.
  They return product outcomes and mapped denials; the server shapes the
  envelope.
- Do not hide product support posture behind "method exists" ergonomics.
- Do not trust a product adapter's "shared-read-safe" declaration unless the
  declaration supplies comparable product basis identity and certification
  proves omission fails.

**Test requirements**
- `product_adapter_operation_parity_across_direct_and_http_surfaces`:
  equivalence proof using a fixture product facade that WORTH-native and
  HTTP-shaped entry produce identical operation plans and response envelopes.
- `product_adapter_cannot_bypass_server_operation_runtime`: boundary proof
  that product execution cannot occur without lowered plans and scheduler
  admission.
- `product_denials_preserve_product_reason_keys_inside_server_envelopes`:
  denial-localization proof for product validation denial, unsupported product
  operation, malformed product payload, and product stale-basis stop.
- `product_adapter_registration_rejects_incomplete_authority_or_basis_contract`:
  rejection proof that product operation declarations missing footprint,
  support posture, basis kind, schema identity, or denial mapping fail
  registration before runtime.
- `product_payload_schema_validation_is_adapter_declared_not_route_owned`:
  boundary proof that malformed product payload structure fails through the
  product-declared schema validator or product denial mapping, never route-local
  semantic validation.
- `product_editor_shaped_operations_register_without_server_semantics`:
  anti-theatre proof that editor-like operation declarations can register and
  execute through a fixture adapter while `worth-server` contains zero
  product-specific semantic branches.

**Engineering decisions**
- The adapter is the boundary a downstream product server should consume.
- Product support snapshots are product-owned declarations surfaced through
  server support posture; they are not Query support rows unless the product
  operation depends on Query support.
- Product payload schemas should be versioned and digest-bearing so web clients
  can pin them later.
- Product basis kind is part of the product operation declaration and flows
  into Phase 4 authority classification and Phase 5 support/concurrency
  classification.

**Open questions**
- None.

### Phase 10: Product Session Identity And Lifecycle

This phase adds the server-owned coordination identity required by responsive
web applications: product session identity, lifecycle, preview-versus-mutation
posture, expiry, close, and branch/base rebind denial.

By the end of this phase, a product web UI can hold a server-admitted session
across requests without becoming authoritative, and the server can classify
read-only preview, mutable draft coordination, expiry, close, and foreign or
moved-session denial before mutation-specific replay logic runs.

**Relevant subsystems**
- product session coordination
- product session lifecycle
- session diagnostics

**Relevant APIs**
- `WORTHServerProductSession`
- `WORTHServerProductSessionIdentity`
- `WORTHServerProductSessionLifecycle`
- `WORTHServerProductSessionCreationRequest`
- `WORTHServerProductSessionExpiryPosture`

**Required boundaries now**
- Product sessions are server-admitted coordination artifacts, not product
  domain truth and not browser-owned truth.
- Product sessions are created by a session coordination operation or by a
  product operation whose declaration explicitly admits session creation. Route
  code and product adapters may not fabricate session identity.
- A product session may exist without a product draft only if its lifecycle
  posture is `ReadOnlyPreview` or another declared non-mutating posture.
- Product session identity is stable across HTTP requests until explicit close,
  expiry, or rebind denial. Expiry posture is counted and typed.
- Session lookup happens before product mutation scheduling; branch/base
  movement requires explicit rebase or rebind posture.

**Warnings**
- Do not let product sessions become durable-storage claims. Durable product
  persistence remains product/server storage milestone work.
- Do not allow expired sessions to silently rehydrate or migrate branch/base
  posture.
- Do not let transport connection identity masquerade as product session
  identity.

**Test requirements**
- `product_session_lifecycle_denies_expired_foreign_or_moved_sessions`:
  lifecycle proof for create, read-only preview without draft, mutation session
  with draft, expiry, explicit close, foreign session, and branch/base rebind.
- `product_session_identity_is_server_admitted_not_adapter_fabricated`:
  boundary proof that route code and product adapters cannot mint sessions
  outside the operation runtime.
- `preview_and_mutation_session_postures_remain_distinct`: parity proof that
  non-mutating preview sessions and mutation-capable sessions retain different
  lifecycle and denial posture.

**Engineering decisions**
- Product session identity is separate from transport connection identity,
  Query branch identity, and durable product draft identity.
- Session coordination is admitted as an operation family so it shares the
  same front door and diagnostics path.
- In-memory session certification proves operation-runtime semantics only. It
  does not claim durable resume, process restart, cross-node coordination, or
  storage-level conflict behavior.

**Open questions**
- None.

### Phase 11: Product Idempotency, Replay, And Stale-Basis Contract

This phase adds the server-owned optimistic coordination contract required by
responsive web applications: base snapshot digests, idempotency, conflict
posture, and typed stale-basis denials.

By the end of this phase, a product web UI can submit fast operations against
the snapshot it last observed, and the server can admit, apply, reject,
replay, or require rebase without making the client authoritative.

**Relevant subsystems**
- optimistic preconditions
- idempotency
- stale-basis denial
- product operation replay
- session diagnostics

**Relevant APIs**
- `WORTHServerProductSnapshotPrecondition`
- `WORTHServerProductOperationBaseDigest`
- `WORTHServerProductIdempotencyKey`
- `WORTHServerProductIdempotencyRecord`
- `WORTHServerProductIdempotencyConflict`
- `WORTHServerProductStaleBasisDenial`
- `WORTHServerProductRebaseRequired`
- `WORTHServerProductOperationReplayReceipt`

**Required boundaries now**
- Session lookup and base-precondition admission happen before product mutation
  scheduling; branch/base movement requires explicit rebase or rebind posture.
- Base snapshot digest, bundle digest, or product revision digest are
  preconditions. They cannot promote themselves into authority.
- Idempotency binding includes tenant, workspace, branch/preview target,
  session identity, operation identity, base digest, and payload digest.
- Same idempotency key, same binding, and same payload digest returns the
  original envelope.
- Same idempotency key with different payload digest, operation name, session,
  branch, or base digest returns `PreconditionDenial::IdempotencyConflict`.
- A failure before scheduler admission records no idempotency result unless the
  denial is itself an admitted request denial. A scheduler-accepted operation
  records its final success, denial, product failure, or scheduler failure for
  replay.
- Stale, foreign, missing, or conflicting preconditions deny before product
  mutation.

**Warnings**
- Do not auto-rebase mutation payloads in the server. The server may report
  rebase-required posture; product semantics decide whether an explicit rebase
  operation exists.
- Do not let idempotency keys compare only by caller-provided strings without
  tenant/workspace/session/operation identity binding.
- Do not let replay closure depend on transport retry folklore or product-local
  cache state.

**Test requirements**
- `optimistic_product_apply_matches_serialized_current_basis`: equivalence
  proof that apply-on-current preconditions produce the same product outcome
  and next digest as serialized server execution.
- `stale_product_snapshot_denies_before_mutation`: hostile denial proof for
  old base digest, foreign session, branch mismatch, missing base digest, and
  product revision mismatch with exact zero product mutation count.
- `idempotent_product_operation_replay_returns_original_envelope`: replay
  proof that duplicate idempotency key plus identical operation identity and
  payload identity returns the original receipt/envelope rather than
  re-executing product mutation.
- `idempotency_key_collision_across_sessions_does_not_replay_foreign_result`:
  leakage proof that tenant/workspace/session/branch binding prevents
  cross-session idempotency collision.
- `idempotency_matrix_is_explicit_and_replay_stable`: matrix proof for same
  key/same operation/same payload, same key/different payload, same key/
  different operation name, same key/different base digest, failure before
  scheduler admission, and failure after scheduler admission.

**Engineering decisions**
- Stale-basis posture is a typed denial family because web clients need a
  first-class rebase prompt, not a generic conflict string.
- Product session identity is consumed here as an already-admitted
  coordination artifact; this phase does not own lifecycle closure.
- Idempotency and stale-basis are split from lifecycle because replay/conflict
  pressure and session-lifetime pressure fail differently and need separate
  certification.

**Open questions**
- None.

### Phase 12: Operation-Declared Route Assembly

This phase replaces empty or framework-shaped route bootstrap with real route
assembly derived from registered operation declarations.

By the end of this phase, Axum routes decode transport input into operation
request inputs and then immediately enter the operation runtime. Route handlers
do not own product meaning, Query meaning, stale-basis recovery, response
envelope construction, or execution strategy.

**Relevant subsystems**
- transport route assembly
- operation declaration registry
- compatibility HTTP facade
- product operation route declarations
- route diagnostics and inventory
- transport denial shaping
- operational route classification

**Relevant APIs**
- `WORTHServerOperationRouter`
- `WORTHServerRouteAssembly`
- `WORTHServerDeclaredRoute`
- `WORTHServerRouteInventory`
- `WORTHServerRouteExecutionBridge`
- `WORTHServerRouteAssemblyError`
- `WORTHServerTransportDenial`
- `WORTHServerOperationalRoute`

**Required boundaries now**
- Route assembly consumes operation registrations, surface registrations, and
  product operation declarations.
- Route handlers may decode body/headers/path/query into
  `WORTHServerOperationRequestInput`; they may not execute semantic work.
- Before semantic Query or product work executes, transport/facade input must
  be decoded into a canonical operation request. Route matching, method
  classification, envelope decoding, body limits, and content negotiation may
  happen before operation request construction.
- Route inventory maps every route to surface family, operation family,
  operation declaration, payload schema, diagnostics policy, and response
  transform.
- Missing, duplicate, or ambiguous route declarations fail build or server
  assembly before serving.
- Malformed JSON, unsupported encoding, oversized payload, unknown route,
  CORS/preflight, health, metrics, static asset, and docs/spec export failures
  are transport or operational-route outcomes. They shape through server
  response policy but do not pretend to be product operation denials.

**Warnings**
- Do not expose raw `axum::Router` as the server architecture.
- Do not let product crates install arbitrary Axum handlers inside the server.
- Do not create product-editor convenience routes outside the declaration
  mechanism.
- Do not force health checks, metrics, static assets, CORS/preflight, or docs
  export through semantic operation declarations unless they execute Query or
  product semantics.

**Test requirements**
- `declared_routes_and_direct_facade_share_operation_plan`: parity proof that
  a route and WORTH-native direct call for the same operation lower to the
  same request contract, plan, scheduler outcome, and response envelope.
- `route_handler_cannot_execute_without_operation_runtime`: boundary proof
  that no registered route can call product adapter or Query execution
  directly.
- `route_inventory_explains_every_served_path`: exact inventory proof that
  every served route has operation family, payload schema, support posture,
  evidence policy, and response transform rows.
- `duplicate_or_ambiguous_operation_route_fails_server_assembly`: hostile
  assembly denial for duplicate method/path, conflicting operation identity,
  and unsupported payload schema version.
- `transport_errors_shape_without_product_semantics`: denial ownership proof
  for malformed JSON, oversized payload, unknown route, unsupported encoding,
  and invalid content negotiation.
- `operational_routes_do_not_enter_product_operation_runtime`: boundary proof
  that health, metrics, static assets, CORS/preflight, and docs/spec export
  routes either stay operational or explicitly declare a semantic operation.

**Engineering decisions**
- Axum remains transport mechanism. Operation declarations are the server
  contract.
- Route assembly happens after operation registry and product adapter
  registration so route topology derives from authority topology.
- Route diagnostics must be inspectable without consulting host logs.
- Route errors before operation request construction are transport denials or
  operational-route outcomes, shaped by server response policy.

**Open questions**
- None.

### Phase 13: Product-Editor Readiness Certification

This phase certifies the milestone against the product pressure that motivated
it: product-editor-shaped operations running through the product adapter and
operation runtime without any product-specific semantics entering
`worth-server`.

By the end of this phase, `worth-server` publishes a readiness artifact proving
that a downstream product server can be built as product adapter plus operation
declarations over a product application facade, rather than as route-local
server logic.

**Relevant subsystems**
- product operation certification
- product-editor fixture adapter
- scheduler hostile matrix
- route/facade parity
- support/profile publication

**Relevant APIs**
- `WORTHServerProductEditorReadinessCertification`
- `WORTHServerProductOperationRuntimeCertification`
- `WORTHServerEditorLikeOperationFixture`
- `WORTHServerProductOperationRuntimeSupportRow`
- `WORTHServerOperationRuntimeCloseoutDigest`

**Required boundaries now**
- The certification fixture may model editor-like product operations, but may
  not import downstream product domain code unless the test is explicitly an
  integration fixture outside `worth-server` core.
- Certification must prove render/select/action operations can run as
  concurrent product reads and apply/finalize operations run through
  deterministic product mutation posture.
- The editor-like fixture must model structural behavior, not just operation
  names:
  - render reads from a declared basis digest
  - select depends on visible surface state
  - available actions depend on product-owned state
  - apply changes draft state and emits a new digest
  - finalize requires stricter preconditions than apply
  - stale apply denies before product mutation
  - idempotency replay returns the original envelope
- Certification must prove stale base digests and idempotency are handled by
  server session contracts, not by product-specific route code.
- Certification publishes support posture for product-operation runtime
  readiness.

**Warnings**
- Do not close this milestone with a toy read-only product adapter. The
  pressure case must include product mutation, stale-basis denial, idempotent
  replay, and route/facade parity.
- Do not import product-specific semantics into server production modules.
- Do not use product-specific names as server authority types.

**Test requirements**
- `product_editor_like_render_select_and_actions_run_concurrently`: hostile matrix
  proving multiple render/select/action operations over compatible product
  basis produce serialized-equivalent envelopes and exact shared-read counters.
- `product_editor_like_apply_and_finalize_are_deterministic_and_stale_safe`:
  mutation proof that apply/finalize operations over the same draft/session
  serialize or deny conflict deterministically with typed stale/rebase posture.
- `product_editor_like_http_and_WORTH_native_paths_are_plan_equivalent`:
  route/facade parity proof for render, select, apply, and finalize operation
  declarations.
- `worth_server_contains_no_product_semantic_branches`: anti-theatre proof
  through dependency graph checks, operation registry inventory, compile
  boundaries, and reason-key pass-through proof that product-specific
  authoring, rendering, or edit semantics do not exist inside `worth-server`.
- `product_editor_fixture_has_real_pressure_shape`: fixture proof for
  basis-bound render, visible-state select, state-dependent actions,
  digest-emitting apply, stricter finalize preconditions, stale apply denial,
  and idempotent replay.
- `product_operation_runtime_support_row_closes_only_with_all_phase_artifacts`:
  derived closure proof that support/profile reports readiness only when Query
  audit, operation registry, request contract, footprint/authorization/support/
  precondition posture, planner, scheduler, product adapter, session, route
  assembly, product-editor-readiness, and no-product-dependency artifacts are
  simultaneously green.

**Engineering decisions**
- Product-editor readiness is a certification artifact, not a dependency from
  `worth-server` to a downstream product application.
- The expected downstream product server shape after this milestone is adapter
  plus operation declarations plus product persistence, not custom server
  runtime.
- No-product-dependency proof prefers dependency/type-boundary checks over
  string scans.
  Forbidden-name scans may supplement, but cannot be the sole proof.
- This phase is the closeout aggregator for the milestone; it does not
  re-audit earlier phases except through their published artifacts.

**Open questions**
- None.

## Must Ship

- Query concurrency and consumer-kit dependency audit over all covered
  `worth-server` Query-facing paths
- operation-family registry distinct from surface-family registry
- canonical operation request contract, operation identity, plan identity, and
  evidence identity boundaries
- authority-footprint, authorization-proof, admission-posture,
  precondition-posture, support-composition, and concurrency-classification
  artifacts
- denial taxonomy covering transport, request, admission, support,
  authorization, authority, precondition, scheduler, and product denials
- lowered operation planner with plan receipts, support composition,
  authorization proof, precondition posture, strategy selection, evidence
  policy, and counters
- concurrent operation scheduler using Query `9.7` shared-read and
  deterministic-submission surfaces where applicable
- product-application adapter boundary with product operation declarations,
  product basis kind, product support posture, payload schema validation, and
  typed product denial mapping
- optimistic product session, base-digest, idempotency, stale-basis, conflict,
  lifecycle, replay/conflict matrix, and rebase posture
- operation-declared route assembly over Axum without route-local semantic
  handlers, plus explicit transport/operational route denial ownership
- product-editor-shaped readiness certification that proves product adapter
  fit without importing product-specific semantics into `worth-server`
- support/profile row that reports product-operation runtime readiness only
  from phase-local evidence artifacts

## Must Preserve

- Query remains the ordinary runtime authority for Query reads, state,
  inspection, projection consumption, mutation/submission, support posture, and
  shared-read concurrency.
- `worth-server` remains network, operation-admission, planning, scheduling,
  envelope, diagnostics, and transport authority.
- Product crates remain owners of product semantics.
- Surface families remain transport/entry topology; operation families remain
  admission/category topology. Authority, scheduling, and execution come from
  declarations, footprints, proofs, postures, and lowered plans.
- Product sessions remain coordination artifacts, not durable product truth and
  not client-owned authority.
- Route assembly derives from operation declarations; route handlers never
  become semantic execution owners.
- Diagnostic richness may materialize more evidence but cannot change
  operation identity, plan identity, scheduling result, or product outcome.
- Authorization remains distinct from authority footprint. A valid footprint
  does not imply the principal may execute it.
- Query support and product support compose through server planner receipts
  without becoming one shared support meaning.

## Acceptance Evidence

This milestone is complete only when `worth-server` can prove:

- the Query dependency audit has zero unclassified covered dimensions and zero
  `RuntimeReadiness::LegacyAssumption` or `ConsumerKitPosture::LocalFolklore`
  rows on ordinary operation-runtime paths
- equivalent WORTH-native and compatibility HTTP operation inputs lower to the
  same canonical operation identity and lowered operation plan where their
  semantic inputs match
- diagnostics richness changes evidence identity/materialization but not
  operation identity, plan identity, scheduling result, or product outcome
- every operation execution flows through operation family registration,
  request contract, authority footprint, authorization proof, support
  composition, precondition posture, lowered plan, scheduler, and response
  shaping
- shared-read-safe Query-direct and product-read operations execute
  concurrently with serialized-replay-equivalent envelopes and exact scheduler
  counters
- deterministic submission and product mutation plans serialize or deny
  conflicts through typed scheduler posture
- product adapters cannot bypass server request context, middleware, operation
  planning, scheduler, response shaping, or diagnostics
- product read declarations classify basis kind and comparable digest semantics
  before concurrent scheduling is admitted
- idempotency and session lifecycle behavior passes the explicit replay,
  conflict, expiry, close, foreign-session, and branch/base rebind matrix
- transport errors and operational routes are shaped without becoming product
  operation denials
- stale base digests, foreign product sessions, branch mismatches, malformed
  idempotency keys, unsupported product operations, and product denials localize
  to typed server envelopes
- HTTP routes assembled from operation declarations are parity-equivalent with
  WORTH-native direct calls
- product-editor-shaped render/select/action/apply/finalize operations run
  through the product adapter boundary with real basis/digest/stale/idempotency
  pressure and no product-specific semantic branches inside `worth-server`
- support/profile readiness is derived from phase-local artifacts, not hard
  coded closeout posture

## Sequencing Notes

- This milestone belongs after Milestone `3` because the server already has a
  front door, WORTH-native facade, and external compatibility surface; the next
  architectural gap is not another route family but the operation runtime those
  surfaces must share.
- It belongs before Milestone `5` because lease and subscription work must not
  inherit pre-`9.7` concurrency assumptions, route-local operation semantics,
  or product-owned support folklore.
- It also belongs before downstream product servers become serious product
  surfaces: those servers should bind product application facades through
  product operation declarations and adapters rather than inventing server
  concurrency, stale-basis, idempotency, or response-envelope rules locally.
- Durable product sessions, Postgres-backed product persistence, restart-stable
  operation replay, WebSocket sync, shared subscription bases, and durable
  lease persistence remain later milestone work.

## Required Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes: it creates the missing server-owned operation runtime
  between transport surfaces and product applications.
- Is the adversarial constraint precise and load-bearing? Yes: it requires
  cross-surface operation identity, declared authority footprints, concurrent
  scheduling, deterministic mutation posture, and no product route-local
  semantics.
- Does the roadmap justify this milestone now? Yes: Milestone `3` leaves a
  real gap before leases/sync and before product-server work.
- Does the spec preserve crate authority boundaries? Yes: Query owns runtime
  meaning, server owns operation admission and transport, products own product
  semantics.
- Are the phases carrying most of the real design information? Yes.
- Is each phase centered on one conceptual detail or boundary? Yes.
- Does each phase contain at least 2 adversarial tests by default? Yes.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes.
- Does each phase say how to build the boundary, not just what the boundary is?
  Yes.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  It belongs between external HTTP/binary surface closure and runtime-backed
  lease/subscription work.
