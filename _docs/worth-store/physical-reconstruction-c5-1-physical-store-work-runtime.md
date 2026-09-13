# C.5.1: Physical Store Instance And Signal-Orchestrated Work

## Goal

Turn the C.3 through C.5 runtime into one cohesive physical Store instance.
Specialized physical authority remains the only source of operation truth;
one private Worth Signal instance derives physical dependency readiness and
owns generic async-resource lifecycle; the existing I/O scheduler admits scarce
resources; and one Store-owned executor is the only route to real media effects.
The instance exposes independently borrowable, generation-fenced physical
submission authority so a future Store-backed Query runtime can run at most one
active Relational writer per branch generation at a time across many branches
without adding a global physical writer lock or teaching Store what a semantic
branch means.

## Why This Milestone Exists

C.5 can write and reopen real records, but current work orchestration is still
distributed across synchronous record methods, buffer-pool claims, scheduler
plans, backend queue completion, publication stages, and lifecycle-local state.
C.6 through C.11 would multiply those seams into competing background runtimes
unless one work topology exists first. C.5.1 installs that topology, migrates
the current production path into it, and deletes every superseded authority.
Part II later assigns branch truth and active branch-write authority to
Relational. C.5.1 must make that composition possible by preserving concurrent
physical submission, exact locality, generation fencing, and terminal effect
fate while remaining completely branch- and MVCC-agnostic.

## Governing Summaries

- `MENTALITY.md` requires the worst credible interleaving to shape the
  foundation before later features increase its surface.
- `arch_laws.md` requires autonomous owners joined by consuming proof-bearing
  transitions; adding the physical Signal owner must break every incomplete
  construction, shutdown, and fork site.
- `composition_laws.md` requires declaration, Signal binding, scheduling,
  execution, settlement, cancellation, and observation to remain named,
  reviewable, and independently deletable responsibilities.
- `domain_structure_laws.md` requires physical effects to remain spatially
  visible and lower mechanisms to remain ignorant of Signal and Query.
- `perf_laws.md` requires pre-effect resource admission, locality-preserving
  concurrency, batch-honest cardinality, and exact structural accounting.
- The reconstruction roadmap places C.5.1 after real C.5 artifacts and before
  C.6 because buffer, WAL, recovery, maintenance, layout, and blob work must all
  enter one physical work topology.
- The runtime-integration roadmap requires many branches to hold independent
  active Relational writer authority while exactly one mutation at a time owns
  a given branch-head generation. C.5.1 protects the lower half of that future
  join:
  independently borrowable physical submission, proof-carrying physical
  locality, and exact settlement, with no Store-owned branch registry or
semantic writer admission.

## Current Verification Policy

C.5.1 behavior is protected by its current direct Cargo tests and repository
gates. Generated proof ledgers, source or binary bindings, mutation catalogs,
and report pipelines are retired and must not be recreated. Git preserves the
implementation history.
- Worth Signal Milestone A already owns deterministic temporal gates, wake
  ordering, timeout/retry timing, previous-value access, and frontier-bounded
  temporal work. Store must consume that clock and wake substrate rather than
  introduce timers, sleep loops, or scheduler-local temporal truth.
- Worth Signal Milestone B already owns generic request identity, generation,
  attempt, epoch, cancellation, timeout, retry, supersession, completion
  admission, denial, retention, and bounded in-flight lifecycle. Store may
  bind physical authority to that lifecycle, but may not reproduce it.
- Worth Signal Milestone C already owns descriptor-backed async policy
  families and their compatibility, budget, diagnostics, and replay law.
  Physical safety may narrow an admitted policy decision; it may never be
  weakened by Store-local callback policy.
- Worth Signal Milestone D makes async a capability on ordinary nodes,
  including aspect-scoped, partitioned, interior, and hierarchical nodes.
  Physical work must use that capability-first surface instead of creating a
  separate physical resource-node species.

## Adversarial Constraint

The completed milestone must survive this single joined condition:

> One real Store concurrently serves disjoint record reads, two independently
> submitted disjoint append preparations, a dirty-frame writeback, a resource-
> denied request, a pre-dispatch cancellation,
> a post-dispatch cancellation, an intentionally reordered late completion,
> and an injected partial filesystem effect while shutdown begins. Exactly one
> physical operation identity follows each admitted request; no safely denied
> or cancelled request reaches media; every possibly started effect settles or
> becomes inspection-required; disjoint work progresses without a global work
> lock; shutdown leaves no unclassified work; and a fresh process reconstructs
> the last published C.5 truth without receiving Signal, scheduler, callback,
> pending-map, or live-runtime state.

The milestone fails if correctness depends on scanning code for duplicate
routes, trusting counters as effect evidence, serializing all work through one
mutable runtime borrow, or keeping an old work path alive behind a compatibility
wrapper. It also fails if a raw Signal aspect slot is allowed to define
physical meaning, if Foundational mask categories are collapsed, or if JSON is
used as an internal value, command, payload, correlation, observation, or proof
carrier.

## Authority Topology

| Responsibility | Sole owner | Explicit non-authority |
| --- | --- | --- |
| Physical operation identity, effect fate, health, settlement | `worth-store` physical instance | Signal ids, scheduler bindings, counters, diagnostics |
| Dependency readiness and generic async-resource lifecycle | private `worth-signal` runtime | durable truth, effect execution, physical settlement |
| Queue, worker, bandwidth, locality, and pacing admission | `worth-store-io-scheduler` | dependency truth, filesystem truth, final settlement |
| Residency, frame leases, dirty ownership, clean publication | `worth-store-buffer-pool` | scheduling and media effects |
| Filesystem effects and exact effect receipts | `worth-store-physical-backend` | Store operation policy and semantic visibility |
| Byte formats and physical coordinates | `worth-store-physical-format` | media ownership and runtime lifecycle |
| Aspect contracts, values, masks, and admitted semantic state | `worth-foundational` through `worth-store-aspect-native` physical witnesses | Signal slots, JSON documents, raw strings or maps |
| Aspect-local invalidation and dependency routing | private `worth-signal` `Aspect`/`AspectMask` slots derived from an admitted Store binding | aspect meaning, value authority, mutation legality |
| Governed phase progression | `worth-proof` witnesses and outcomes | ids, labels, digests, generic marker traits |
| Shared boundary nouns | `worth-foundational` where meaning truly crosses owners | Store-local hot state and private typestates |
| Future branch truth and active branch-write authority | Relational through the Part II Store-Query composition, outside C.5.1 | physical Store, physical Signal, scheduler, backend, branch labels |

## Product Decision Lock

1. C.5.1 adds no Query, Relational, Runtime Bridge, or semantic Signal
   integration.
2. The physical Signal runtime is a distinct instance with a distinct graph,
   identity, node vocabulary, lifecycle, and recovery posture.
3. Signal state, history, replay, diagnostics, and completion artifacts are
   derived. None is serialized as Store recovery authority.
4. `ServingPhysicalRuntime` is evolved into, or replaced by, the sole physical
   Store instance. There is no second `PhysicalStoreRuntime` beside it.
5. `worth-store` is the only ordinary crate allowed to compose Signal,
   scheduler, buffer pool, and physical backend authority.
6. Lower format, backend, buffer-pool, WAL, recovery, isolation, integrity,
   layout, and blob crates remain Signal-agnostic.
7. Worth Signal owns generic request, cancellation, retry, revalidation,
   supersession, timeout, completion, retention, and diagnostic lifecycle.
   Store does not duplicate that state machine.
8. Store owns physical dispatch, exact effect fate, health revocation, and
   settlement. Signal cannot promote a completion into physical truth.
9. The I/O scheduler owns resource admission and queue policy only. It cannot
   rediscover dependencies, call backend effects, or settle Store work.
10. The executor consumes an admitted Store operation and scheduler grant; it
    does not choose strategy, policy, durability, or retry behavior.
11. Pre-effect denial returns retryable authority where safe. Partial or
    indeterminate effect revokes the declared mutation/serving health boundary.
12. Cancellation after possible dispatch detaches consumer interest but does
    not erase the Store's obligation to settle the operation.
13. No new general-purpose runtime, async, work-registry, or scheduler crate is
    created. A domain-neutral missing Signal capability is added to
    `worth-signal`; physical composition remains in `worth-store`.
14. Every migrated production route deletes its predecessor, obsolete feature,
    and unused dependency in the same phase.
15. C.5.1 does not claim full C.6 residency policy, C.7 WAL ordering, or C.10
    QoS. It supplies the only topology those owners may later enter.
16. Physical dependency facts are ordinary Signal nodes. Async lifecycle is
    attached with `AsyncNodeCapabilityDeclaration`; legacy
    `ResourceNodeDeclaration` vocabulary is not the architectural source.
17. Timeout, retry/backoff, stale-after, cancellation, supersession,
    revalidation, observation, retention, diagnostics, and replay policy are
    selected through Signal's frozen policy descriptors before request
    admission. Store does not add local timer, retry, or policy registries.
18. Signal policy can make a physically safe action stricter. It cannot make an
    unsafe retry, cancellation, publication, durability, or continuation legal.
19. Store may retain a bounded move-owned physical command or dispatched-effect
    obligation keyed by `PhysicalWorkIdentity`. It may not retain a second
    generic pending/lifecycle/history registry beside Signal.
20. If physical settlement succeeds and derived Signal completion later
    rejects or rolls back, the physical operation is never repeated. Store
    reconciles or rebuilds disposable Signal state from settled physical truth.
21. Physical semantic boundaries are aspect-native. Foundational
    `AspectContract`, validated values, authoritative state, patches, and
    mode-specific masks define meaning; `worth-store-aspect-native` binds that
    meaning to real Store physical witnesses.
22. Signal `Aspect` is a bounded invalidation slot, not semantic authority.
    Store creates it only through an admitted `PhysicalSignalAspectBinding`
    from Store aspect identity, contract revision/canonical basis, and declared
    dependency role.
23. Foundational `AspectMask<ProjectionMask>`,
    `AspectMask<MutationMask>`, and `AspectMask<DiagnosticMask>` remain distinct
    laws. Signal `AspectMask` is a derived routing representation and cannot be
    spent as any Foundational mask proof.
24. JSON is forbidden from physical work intent, command storage, Signal
    payload correlation, scheduler demand, executor commands, settlement,
    runtime observation, and evidence construction. No internal
    `serde_json::Value`, object map, JSON bytes, or JSON-shaped string contract
    is accepted.
25. JSON may exist only at an explicitly named external compatibility ingress
    or terminal projection/reporting edge. It must use the existing
    `worth-store-aspect-native` readmission/projection APIs and immediately
    lower into, or derive from, native aspect artifacts; it never crosses an
    internal authority transition.
26. Aspect-native does not mean replacing fixed physical typestates with a bag
    of aspects. Executor commands, effect outcomes, and lifecycle owners remain
    precise Rust domain types; aspect artifacts carry cross-owner semantic
    facts and addressable change surfaces.
27. Opaque record, blob, page, WAL, and index bytes remain legal physical data.
    Their identity, contract, placement, and semantic change surfaces are
    aspect-native; their content is not forced through `AspectValue` or JSON.
    The prohibition is against JSON-shaped internal control/semantic state,
    not against storing arbitrary user bytes.
28. C.5.1 owns no branch identity, branch-head generation, MVCC conflict, or
    semantic writer registry. Those remain future Relational authority; a
    branch label, upstream correlation id, or caller-declared lane cannot grant
    physical parallelism or bypass artifact, security, durability, and
    scheduler coordination.
29. Independently borrowable physical submission handles may coexist and issue
    disjoint work concurrently. They are generation-fenced capabilities over
    one physical Store instance, not logical writer leases, and no handle may
    retain or clone the whole instance merely to obtain concurrency.
30. Physical work identity, exact effect fate, recovery disposition, and
    settlement must be sufficient for the future adapter to correlate the
    physical result with its semantic mutation and for Relational alone to
    decide whether branch-write authority advances, releases, or remains
    fenced. Store does not make that semantic decision and consumer cancellation
    cannot erase the physical evidence on which it depends.

## API Surface Rule

Names under **Existing APIs** below identify current production surfaces and
must be reused unless implementation demonstrates a semantic mismatch in the
same change. Names under **Target APIs** specify the required contract and may
be refined for local vocabulary, but their authority and transition semantics
are fixed. A target API does not justify a compatibility wrapper around the
surface it replaces.

The intended progression is:

```text
PhysicalWorkIntent
  -> AdmittedPhysicalWork
  -> Signal ResourceRequestHandle
  -> ReadyPhysicalWork
  -> ResourceAdmittedPhysicalWork(QueueExecutionReadyPlan)
  -> DispatchedPhysicalWork
  -> exact backend effect outcome
  -> SettledPhysicalWork
  -> committed derived Signal completion
```

Only the physical owner may produce `AdmittedPhysicalWork`,
`DispatchedPhysicalWork`, or `SettledPhysicalWork`. The exact names are target
names; the consuming progression is mandatory.

## Implementation Shape And DX Target

The directory shape is a responsibility map, not a quota. Keep an existing
owner where it already has one coherent responsibility; split only when a
named seam below would otherwise be hidden. Do not create one-file directory
theater or retain an old module merely to avoid moving callers.

```text
worth-store/src/physical_runtime/
  instance/
    construction.rs       # constructs the one Store-owned composition
    lifecycle.rs          # generation, close, abort, terminal posture
    observation.rs        # coherent, acquisition-scoped instance truth
  work/
    declaration.rs        # PhysicalWorkIntent and stable identity
    aspect_binding.rs     # Store-native meaning -> Signal routing slots
    admission.rs          # pre-effect policy and authority admission
    signal_binding.rs     # physical intent <-> async-capable node lineage
    command_storage.rs    # bounded move-owned packets, no lifecycle history
    scheduler_lowering.rs # ReadyPhysicalWork -> QueueExecutionReadyPlan
    execution.rs          # only Store-owned dispatch into backend effects
    settlement.rs         # physical truth -> scheduler -> Signal derivation
    lifecycle_join.rs     # Signal lifecycle report <-> physical obligation
    observation.rs        # bounded counters and per-work terminal posture
  record_serving/
    access/               # existing read/scan domain, migrated in phase 13
    publication/          # existing append/publication domain, phase 14
    residency/            # private adapters over PhysicalResidencyPool
```

Lower crates retain mechanism-shaped modules. `worth-signal` must not gain
filesystem nouns, the scheduler must not gain record or manifest nouns, the
buffer pool must not gain Signal nodes, and the backend must not gain runtime
dependency policy. The dependency direction is:

```text
Worth Query runtime                 Physical Store instance
        |                                     |
        |                           +---------+---------+
        |                           |         |         |
        |                        Signal   scheduler   residency
        |                                     |         |
        +---------- future Store-Query adapter          |
                                              \         /
                                           Store executor
                                                 |
                                      owned physical backend
```

The public record DX remains deliberately small. Existing read APIs remain the
starting point; mutation loses the global mutable borrow:

```rust
let reader = serving.records();
let session = reader.open(locator, limits)?;

// Target C.5.1 surface: independently borrowable submission authority.
let first_submission = serving.record_submission();
let second_submission = serving.record_submission();
let first_outcome = first_submission.append_batch(first_batch, placement)?;
let second_outcome = second_submission.append_batch(second_batch, placement)?;

let shutdown = serving.close();
```

`PhysicalRecordSubmission` is a target name, not permission to publish the
internal work machine. It may offer synchronous and asynchronous waiting, but
both forms must drive exactly the same admitted progression. Callers do not
construct Signal nodes, scheduler plans, executor commands, backend tickets,
or settlement receipts.

The internal DX must read like the architecture and preserve consuming
progression:

```rust
let admitted = work_admission.admit(intent)?;
let pending = signal_binding.request(admitted)?;
let ready = readiness.admit(pending)?;
let plan = scheduler_lowering.lower(ready)?;
let dispatched = executor.dispatch(plan);
let settled = settlement.settle(dispatched)?;
signal_binding.publish_derived_completion(&settled)?;
```

These are responsibility names, not a demand for one facade object per line.
An implementation may collapse a mechanically inseparable step, but it may not
collapse readiness, effect execution, and physical settlement into a generic
callback or boolean completion.

## API Status And Reuse Contract

The following are existing production surfaces and must be reused or narrowed,
not cloned:

- Store lifecycle: `PhysicalStore::admit`,
  `AdmittedPhysicalRuntime::try_admit_filesystem_media`,
  `MediaOwnedPhysicalRuntime::{initialize_record_store, open_record_store}`,
  and `ServingPhysicalRuntime::{close, abort}`.
- Record access: `ServingPhysicalRuntime::records`,
  `PhysicalRecordReader::{open, readmit_locator, open_external}`,
  `RecordReadSession::read_next`, and
  `PhysicalRecordScanSession::{scan, read_next_into}`.
- Record mutation: `PhysicalRecordWriter::{append_batch,
  append_batch_reconstructing_manifest_capacity}`, `RecordAppendBatch`, and
  the current publication outcomes. These are migrated behind the target
  submission authority; their durable truth is preserved.
- Foundational native aspects: `aspects().vocabulary()`,
  `aspects().contract()`, `aspects().validate()`,
  `aspects().authoritative_state()`, `aspects().patch()`, and the distinct
  `aspects().{projection_mask, mutation_mask, diagnostic_mask}()` lanes.
- Store aspect-native boundary: `StoreAspectIdentity`,
  `StorePhysicalBoundaryWitness`, `StoreAspectContractAdmission`,
  `StoreValidatedAspectValueAdmission`, `StoreAspectAuthorityInput`,
  `StoreAspectBoundaryFact`, `StoreAspectPatchBoundaryFact`, and
  `StoreDigestAuthority`.
- Signal resource lifecycle: `AsyncNodeCapabilityDeclaration`, its
  `with_lifecycle_policy`, `with_retry_policy`, `with_timeout_policy`,
  `with_cancellation_policy`, `with_stale_after_policy`,
  `with_supersession_policy`, `with_revalidation_policy`,
  `with_observation_policy`, `with_output_continuity_policy`,
  `with_retention_policy`, `with_diagnostics_policy`, and
  `with_replay_policy` builders,
  `AsyncCapableNode::{request_intent, revalidation_intent}`,
  `SignalRuntime::{admit_async_node_request, revalidate_async_node,
  in_flight_resource_request, cancel_resource_request,
  admit_resource_timeout, extend_resource_timeout_heartbeat,
  schedule_resource_retry, admit_scheduled_resource_retry,
  admit_resource_completion}`, and `SignalTransaction` staged completion.
- Signal temporal and locality substrate: `ClockAdvanceRequest`,
  `SignalRuntime::{validate_clock_advance, advance_clock,
  advance_clock_with_summary}`, `Aspect`, `AspectMask`,
  `PartitionSubscription`, `mark_changed`, `mark_changed_with_regions`, ordinary
  graph dependencies, and capability attachment through
  `SignalRuntime::{declare_async_node_capability, attach_async_capability}`.
- Scheduler: `QueueWorkDeclaration`, producer lowering through
  `lower_buffer_pool_queue_declaration`, `lower_wal_queue_declaration`, or
  `lower_background_queue_lease`, `QueueExecutionAdmissionRequest`,
  `admit_queue_execution_plan`, `QueueExecutionReadyPlan`, and
  `execute_ready_queue_plan`.
- Backend: `QualifiedFilesystemMedia::artifact_tree` and
  `ArtifactTreeMedia::{read_exact_at, read_bounded, create_new_file,
  write_exact_at, write_scheduled_exact_at, synchronize_file, replace,
  synchronize_directory}` together with their exact effect outcomes.
- Residency: `PhysicalResidencyPool`, `PhysicalFrameLease`,
  `PhysicalWritebackClaim`, `OperationAllocationGrant`, `FrameLoadPort`,
  `CandidateFrameSet`, and `CandidateFramePublicationPort`.

The following are new C.5.1 concepts. Their names may change only to improve
domain precision; their separate authority meanings may not be erased:

- `PhysicalStoreInstanceParts`
- `PhysicalWorkIntent`, `AdmittedPhysicalWork`, `ReadyPhysicalWork`,
  `DispatchedPhysicalWork`, and `SettledPhysicalWork`
- `PhysicalWorkIdentity`, `PhysicalEffectIdentity`, and
  `PhysicalWorkGeneration`
- `PhysicalRecordSubmission`
- `PhysicalWorkObservation`
- `C6PhysicalWorkHandoff`
- `PhysicalSignalAspectBinding` and frozen
  `PhysicalSignalAspectBindingSet`
- `PhysicalSignalAspectSubscription`
- `PhysicalWorkAspectDelta`

Worth Proof supplies concrete construction, execution, settlement, and
terminal-posture witnesses where crossing the boundary would otherwise depend
on convention. Worth Foundational supplies policy admission receipts and
stable policy identifiers where the decision is genuinely policy. Neither is
used to wrap ordinary data, duplicate a typestate already carried by ownership,
or create a vocabulary bag that obscures the physical subject.

## Non-Fake Acceptance Setup

The acceptance subject is the ordinary `worth-store` production library with
the production Store facade, physical backend, scheduler, Signal runtime, and
canonical residency pool. Certification authority may install fault schedules
and yieldpoints; it may not replace any production owner, effect executor,
receipt, lifecycle transition, or persisted format.

### Executables And Process Roles

- Use focused Cargo tests to exercise the C.5.1 process scenarios.
- Use narrowly scoped child modes in the direct integration-test executable;
  do not maintain a second certification product binary.
- Retain the separately linked existing
  `worth-store-offline-verifier` binary
  `physical_store_offline_observer`. It must continue to have no dependency on
  `worth-store`, Worth Signal, or the I/O scheduler.
- The writer and reopener are distinct OS process invocations. They exchange
  only the Store root, immutable test configuration, and a predeclared oracle
  file written by the parent before either child starts. No runtime object,
  serialized Signal state, scheduler state, expected result, or completion
  receipt crosses the process boundary.

### Initial World And Oracle

- Each scenario begins in a newly created real directory on the platform
  filesystem, admitted through `PhysicalStore::admit` and the normal physical
  media lifecycle.
- The parent declares record identities, input payload digests, legal serial
  histories, fault schedule, capacity limits, and seed before execution. The
  parent never derives expected persisted bytes by calling Store encoding or
  planning code.
- The parent also declares native Foundational aspect contracts and values.
  Store admits them through `worth-store-aspect-native`, freezes the
  `PhysicalSignalAspectBindingSet`, and applies at least one native patch whose
  admitted mutation surface should invalidate exactly one predeclared Signal
  dependency slice. No JSON representation participates in this setup.
- At least one artifact is larger than the admitted residency budget, and at
  least two operations target disjoint artifacts so that boundedness and
  non-global serialization are observable rather than asserted.
- The Store is initialized and reopened using only public production facades.
  No test may construct a backend owner, mutation ticket, scheduler completion,
  residency-clean receipt, or Signal completion directly.

### Independent Observation

- `physical_store_offline_observer` reads persisted artifacts in a fresh
  process and reports artifact coordinates, exact lengths, digests, framing,
  roots, residue, and independently decoded record identity.
- A second fresh test process performs normal Store admission and record reads
  from root and configuration only.
- OS-level artifact metadata and raw range digests are collected independently
  of the Store runtime. In-process observations are supplemental and cannot be
  the sole persistence oracle.
- For a partial or indeterminate effect, the observer must report the exact
  completed prefix when known and the reopener must report inspection-required
  posture. A generic failure or timeout is insufficient evidence.

### Forbidden Substitutes

- in-memory filesystems, mock backends, fake durability, or same-process
  "reopen" as the only evidence;
- assertions over intended calls when raw persisted bytes can be observed;
- test-only constructors for production proofs or effect receipts;
- JSON fixtures passed directly into Store work admission, Signal payload
  correlation, scheduler demand, executor commands, settlement, or evidence
  assembly;
- semantic branch identities, writer leases, or MVCC conflict objects injected
  into C.5.1 as physical work or concurrency authority;
- sleep-based race claims without named yieldpoints and a bounded deadline;
- giant policy/value Cartesian matrices in place of the three joined
  direct joined scenarios; and
- a script that infers architectural ownership by string search alone.

## Must Ship

- one sealed physical Store instance that owns lifecycle, Signal binding,
  scheduler admission, executor dispatch, residency, and settlement;
- one consuming physical-work progression from declaration through exact
  physical settlement;
- migrated C.5 read and publication paths using that progression;
- independently borrowable, generation-fenced physical read and mutation
  submission capabilities that permit disjoint progress without carrying
  branch, MVCC, or semantic writer authority;
- bounded cancellation, retry, shutdown, and observation semantics;
- a narrow C.6 handoff that makes a second runtime mechanically unnecessary;
- deletion and dependency gates for every superseded production path; and
- direct real-filesystem and fresh-process regression tests at the relevant
  production boundaries.

## Must Preserve

- C.3 path, identity, confinement, and format admission truth;
- C.4 media ownership, exact effect fate, artifact coordination, and shutdown
  posture;
- C.5 record identity, placement, framing, manifest, catalog, publication,
  independent offline walking, and existing public read semantics;
- Worth Signal's role as disposable dependency/readiness state, never durable
  database truth;
- Worth Signal A-D as the only temporal, generic async lifecycle, async-policy,
  and arbitrary-node capability substrate used by the physical instance;
- Foundational and `worth-store-aspect-native` contracts as the semantic aspect
  authority, with Signal aspects remaining derived routing slots and JSON
  confined to explicit external compatibility/terminal projection;
- the scheduler's role as resource admission and ordering, never effect or
  settlement authority;
- the backend's role as exact physical effect owner beneath Store composition;
  and
- future Relational ownership of branch truth and exactly one active writer per
  branch-head generation at a time; C.5.1 supplies physical concurrency and
  settlement evidence only and cannot pre-empt that policy with Store-local
  branch state.

## Acceptance Evidence

- every phase's focused unit and integration tests;
- compile-fail authority tests for raw Signal, scheduler, residency, executor,
  backend, and settlement construction;
- compile-fail and dependency tests proving physical Store work cannot import,
  infer, or mint branch/MVCC writer authority, plus concurrency tests proving
  multiple physical submission handles do not require a whole-instance lock;
- feature-tree, dependency-direction, public-surface, dead-code, and deletion
  gates;
- strict Clippy with all targets/features and zero warnings;
- all C.3-C.5 physical lifecycle and record journey suites;
- the focused phase-16 lifecycle, crash/reopen, and bounded-residency scenarios
  using independent observation where persisted truth is at issue;
- `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`;
- `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`; and
- the workspace Rust line-cap and physical-writer/raw-owner gates.

## Sequencing And Completion Gate

Implement phases in order. Phases 1-4 establish the one owner and domain;
phases 5-8 close readiness through settlement; phases 9-12 close the hostile
lifecycle; phases 13-14 migrate real record journeys; phase 15 deletes
substitutes; phase 16 certifies the C.6 handoff. A later phase may expose a
defect in an earlier one, but it must repair the earlier seam rather than add a
local bypass.

C.5.1 is complete only when all sixteen phase obligations, the focused direct
scenarios, independent offline observation, and boundary checks pass on the
ordinary production graph. A green suite with a
duplicate authority, fake effect, serialized recovery shortcut, uncancelled
work leak, or C.6-only alternate path is a failed milestone.

## Phase Plan

### Phase 1: Collapse Physical Work Ownership

Freeze one production responsibility topology and remove entry points that can
already be proven to compete with it.

**Relevant subsystems**

- `worth-store::physical_runtime`
- record-serving lifecycle, publication, residency, and media ownership
- buffer-pool background work and I/O scheduler queue execution
- production features exposing raw backend ownership

**Existing APIs**

- `PhysicalStore::admit`
- `AdmittedPhysicalRuntime::try_admit_filesystem_media`
- `MediaOwnedPhysicalRuntime::{initialize_record_store, open_record_store}`
- `ServingPhysicalRuntime::{records, records_mut, close, abort}`
- `ServingPhysicalRuntime::execute_scheduled_writeback`

**Target APIs**

- one private `PhysicalStoreInstanceParts` exhaustive construction packet
- one responsibility-named physical-instance facade; retain
  `ServingPhysicalRuntime` if that remains the honest name
- independently borrowable physical read and mutation submission capabilities
  whose construction is lifecycle-generation-bound and does not borrow or
  clone the complete physical instance
- one import/dependency gate permitting the Store composition owner, and no
  lower mechanism crate, to depend on `worth-signal`
- one composition-only aspect bridge: `worth-store` may depend on both
  `worth-store-aspect-native` and `worth-signal`, while neither crate depends on
  the other and lower physical crates cannot invent a competing translation

**Warnings**

- Do not produce a CSV, registry, or checked-in ownership ledger. The proof is
  the surviving import graph, visibility, and deleted code.
- Do not rename duplicate authorities while preserving both.
- Existing certification-only raw owner features must not enter ordinary
  dependency resolution.
- Do not move Signal slot vocabulary into `worth-store-aspect-native`; it owns
  Store/Foundational semantic authority, while the Store composition root owns
  the derived Signal binding.

**Test requirements**

- A dependency test must fail if backend, format, buffer-pool, WAL, recovery,
  isolation, integrity, layout, or blob crates import `worth-signal`.
- A dependency/source test must fail if `worth-store-aspect-native` imports
  Signal, Signal imports Store-owned physical aspect vocabulary, or ordinary
  Store work modules import JSON compatibility/projection lanes.
- Compile-fail fixtures must prove product crates cannot construct raw backend
  owners, scheduler backend completions, buffer-pool claims, or a second
  physical runtime authority.
- Compile-fail and dependency fixtures must prove the physical instance exposes
  no branch-head mutation, branch writer, MVCC conflict, or semantic publication
  authority; those concepts may appear only in future Query/Relational and
  integration-side contracts, never the physical Store facade.
- A production-path reachability test must prove initialize, open, record read,
  append, writeback, close, and abort all enter the same physical instance.
- Two independently acquired mutation submission handles must coexist, submit
  disjoint physical preparation concurrently, and become stale together when
  their lifecycle generation closes; acquiring them must not clone media,
  executor, scheduler, Signal, or root-publication ownership.

**Engineering decisions**

- Most implementation remains in the existing `worth-store` crate.
- The existing scheduled-writeback chain is admitted substrate, not a parallel
  runtime to discard.
- Authority is demonstrated by constructors and visibility, not type names.

**Open questions**

- None.

### Phase 2: Define The Canonical Physical Work Domain

Define the Store-owned meaning that survives across Signal, scheduler, and
backend boundaries without allowing those mechanisms to reconstruct authority.

**Relevant subsystems**

- physical runtime identity and lifecycle generation
- physical record coordinates and artifact families
- security scope, durability posture, and effect classification
- Foundational aspect contracts, Store aspect-native physical witnesses, and
  mode-specific masks
- `worth-proof` consuming transitions

**Existing APIs**

- `RuntimeIdentity`, `LifecycleGeneration`, and `StableStoreIdentity`
- `RecordFrameCoordinate`, `PhysicalRecordId`, and
  `ExternalPhysicalRecordLocator`
- `QueueDurabilityClass`, `QueueWorkClass`, and `QueueGroupingBasis`
- `ArtifactRangeWriteDurabilityRequirement`
- `StoreAspectBoundaryFact`, `StoreAspectPatchBoundaryFact`,
  `StoreAspectContractAdmission`, and `StorePhysicalBoundaryWitness`
- `worth_proof::ProofOutcome`

**Target APIs**

- `PhysicalWorkIdentity`, bound to Store identity, runtime identity,
  lifecycle generation, and a monotonic operation identity
- `PhysicalWorkIntent`, carrying operation family, exact artifact/range scope,
  security scope, resource demand, effect class, durability requirement,
  frozen Signal async-capability profile identity, and physical recovery
  disposition
- `PhysicalWorkSemanticBasis`, carrying admitted Store aspect-native facts or
  patch facts plus their contract identity, revision/canonical basis, and the
  exact projection or mutation posture relevant to the operation
- sealed `AdmittedPhysicalWork`, `ReadyPhysicalWork`,
  `ResourceAdmittedPhysicalWork`, `DispatchedPhysicalWork`, and
  `SettledPhysicalWork` transition types
- `PhysicalWorkEffectClass::{ReadOnly, ReversibleBeforePublication,
  IdempotentExactWrite, PublicationBoundary}` or equivalently precise variants
- `PhysicalWorkRecoveryDisposition::{NoEffect, RetryExact,
  ContinueSettlement, InspectionRequired}`

**Warnings**

- `ResourceRequestHandle`, `QueueExecutionPlanBinding`, digests, and counters
  are correlation values, not `PhysicalWorkIdentity` substitutes.
- Do not force reads, exact writes, publication, and future WAL work through one
  failure enum if their effect fates differ.
- A scalar work intent must not wrap an honest append or writeback batch into a
  caller-side loop.
- Do not copy Signal lifecycle or policy enums into the physical intent.
  Capability-profile identity binds the already-lowered Signal contract;
  physical effect and recovery classifications remain Store-owned.
- Do not encode semantic basis as JSON, an untyped key/value map, raw
  `AspectValue`, or a Signal `AspectMask`. Native Foundational validation and
  Store physical-witness admission precede work admission.
- Do not add branch identity, branch-head generation, MVCC visibility,
  semantic writer generation, or caller lane labels to physical work authority.
  The future adapter retains semantic correlation and consumes Store settlement;
  Store admits concurrency only from exact physical scope and capability.

**Test requirements**

- Compile-fail tests must prove raw ids, digests, scheduler bindings, Signal
  handles, backend receipts, raw `AspectValue`, JSON, and bare Signal
  `Aspect`/`AspectMask` values cannot construct or advance physical work.
- Property tests must permute Store/runtime generation, artifact, range,
  security, durability, capability-profile, and operation identity and reject
  every mismatched transition before effects.
- Batch cardinality tests must prove one batch identity retains its exact member
  set and cannot be partially substituted by scalar child work.
- A branch-shaped string, opaque upstream correlation, or caller-declared lane
  must be unable to change `PhysicalWorkIdentity`, construct
  `PhysicalWorkConcurrencyScope`, widen locality, or admit parallel execution.

**Engineering decisions**

- Store-specific identity and settlement stay outside generic Signal payloads.
- Use `worth-proof` for governed cross-owner progression; do not create a local
  bag of proof vocabulary.
- Intent is immutable after admission. Later phases consume proof-bearing
  wrappers instead of mutating a shared work record.

**Open questions**

- None.

### Phase 3: Seal The Physical Store Instance Lifecycle

Make every physical work owner part of one exhaustive construction and terminal
progression before work can be admitted.

**Relevant subsystems**

- C.3 runtime lifecycle and generation owner
- C.4 media ownership and release
- C.5 record-serving lifecycle and shared health
- private physical Signal runtime, scheduler capability, executor, and work
  accounting

**Existing APIs**

- `ServingPhysicalRuntime::from_admission`
- `ServingHealth::{new, revoke, requires_inspection}`
- `ServingPhysicalRuntime::{close, abort}`
- `ServingShutdownOutcome`
- `worth_signal::facade::{SignalGraph, SignalRuntime}`
- `SignalRuntime::build_for`

**Target APIs**

- `PhysicalStoreInstanceParts`, whose exhaustive fields include record owner,
  media, core lifecycle, format/access policy, root/free-space/allocation state,
  serving health, residency ports, physical Signal owner, scheduler admission
  owner, executor authority, frozen physical async-capability declarations,
  frozen `PhysicalSignalAspectBindingSet`, admitted Store-native aspect
  contracts, Signal clock-input bridge, bounded physical command arena, and
  work accounting
- private `PhysicalWorkSignalOwner` holding the only physical
  `SignalRuntime`
- private `PhysicalWorkExecutor` holding the only ordinary route to
  `QualifiedFilesystemMedia`
- `PhysicalStoreShutdownOutcome` extending current serving shutdown evidence
  with declared, ready, queued, dispatched, settling, terminal, and residual
  work posture
- generation-fenced physical submission capabilities whose retained handles
  can submit only while the owning instance admits work and whose existence
  does not prevent consuming close or keep mutation owners alive

**Warnings**

- Do not place new owners behind `Option` merely to preserve old constructors.
- Do not let `Drop` silently detach dispatched work or release media while
  settlement remains possible.
- Observation handles must not keep mutation, scheduler, executor, or Signal
  authority alive after the physical instance is consumed.

**Test requirements**

- Adding or removing any managed owner must cause compile failures at every
  initialize, open, close, abort, and test-construction site until it is handled.
- Instance construction must reject an unknown, duplicate, incompatible, or
  unlowerable Signal capability/policy declaration before creating command
  storage, scheduler work, or media effects.
- Close and abort tests must begin with work in every currently representable
  lifecycle state and prove that no state disappears from terminal evidence.
- Retained observation and request handles must reject after lifecycle
  generation advances, while a fresh reopened instance receives distinct
  runtime and Signal identities.
- Retained physical submission handles must neither block close nor authorize
  post-close work; close evidence must classify every work identity they
  admitted before the generation fence advanced.

**Engineering decisions**

- Construction order is media and C.5 truth, physical work domain, Signal
  graph, scheduler admission, executor binding, then public serving facade.
- Shutdown order is stop work admission, classify or drain work, dispose Signal
  state, close residency, release media, then close core authority.
- Unexpected drop is terminal and observable; it is not equivalent to clean
  close.

**Open questions**

- None.

### Phase 4: Bind Physical Dependencies Into Worth Signal

Use Worth Signal's real async-node and resource machinery as the only generic
physical dependency and async-resource lifecycle.

**Relevant subsystems**

- Worth Foundational aspect contracts, values, authoritative state, patches,
  canonical basis, and projection/mutation/diagnostic masks
- `worth-store-aspect-native` contract admission and physical witnesses
- Worth Signal graph, aspects, async-node capabilities, and resource policies
- Store physical-work declarations and dependency facts
- physical instance construction and observation

**Existing APIs**

- `worth_foundational::aspects()` native contract, validation, authoritative
  state, patch, and mask lanes
- `StoreAspectIdentity`, `StoreAspectContractAdmission`,
  `StoreAspectBoundaryFact`, `StoreAspectPatchBoundaryFact`, and
  `StorePhysicalBoundaryWitness`
- `SignalGraph::{new, set_dependencies}` and node builders
- `Aspect`, `AspectMask`, `mark_changed`, `mark_changed_with_regions`, and
  `PartitionSubscription`
- `SignalRuntime::build_for`
- `AsyncNodePayloadContract::{new, with_max_payload_bytes}`
- `AsyncNodeCapabilityDeclaration::new` and its `with_*_policy` methods
- `SignalRuntime::{declare_async_node_capability, attach_async_capability}`
- `AsyncCapableNode::{request_intent, revalidation_intent}`
- `SignalRuntime::{admit_async_node_request, revalidate_async_node}`
- `ResourceRequestHandle` and `ResourceRuntimeSummary`

**Target APIs**

- `PhysicalSignalAspectBinding`, privately constructed from one admitted
  `StoreAspectContractAdmission`, one bounded Signal `Aspect` slot, declared
  dependency/output role, optional partition lowering, and a canonical binding
  digest. The Store aspect identity and contract revision define meaning; the
  Signal slot only routes change.
- replace the current infallible Store mask setters with fallible
  `StoreAspectContractAdmission::{admit_projection_mask,
  admit_mutation_mask, admit_diagnostic_mask}` or equivalently precise APIs.
  Each must call the matching Foundational contract-admission law before
  returning a mask-bearing Store admission; projection support is required and
  the old unchecked setters are removed.
- frozen `PhysicalSignalAspectBindingSet` with deterministic slot assignment,
  duplicate identity/slot denial, explicit `MAX_ASPECTS` capacity denial, and
  lookup in both directions without string parsing
- `PhysicalWorkAspectDelta`, constructible only from a
  `StoreAspectBoundaryFact` or `StoreAspectPatchBoundaryFact` matching the
  binding, and lowerable to `mark_changed` or `mark_changed_with_regions`
- `PhysicalSignalAspectSubscription`, constructible from the binding plus an
  admitted Foundational projection mask and lowerable to a Signal
  `AspectMask`/`PartitionSubscription`; diagnostic masks have no operational
  lowering and mutation masks produce deltas rather than subscriptions
- `PhysicalWorkSignalDeclaration`, a Store-owned, data-only mapping that builds
  an ordinary physical dependency node from the frozen
  `PhysicalSignalAspectBindingSet`, constructs
  `AsyncNodeCapabilityDeclaration::new`, applies the selected
  `with_*_policy` declarations, then calls
  `SignalRuntime::{declare_async_node_capability, attach_async_capability}`
- stable async-capable node families for read fault, exact writeback,
  publication, and lifecycle work; operation identity remains per request and
  does not require one permanently registered node per operation
- responsibility-named physical aspects for dependency truth only, such as
  source availability, generation freshness, health, publication basis, and
  resource eligibility, plus `PartitionSubscription` where artifact or frame
  locality is real; names must reflect actual dependencies discovered in
  implementation
- `PhysicalWorkReadiness`, produced only from an admitted Signal request and
  the matching Store work identity, and carrying the frozen Signal capability
  and policy descriptor digests used for that admission
- `PhysicalSignalObservation`, exposing bounded derived lifecycle and cost
  without exposing `SignalRuntime`, `NodeId`, or mutation methods publicly

**Warnings**

- Signal evaluators may read immutable physical observations but may not call
  media, buffer-pool mutation, scheduler execution, or Store settlement APIs.
- Do not mirror the complete Store operation packet as a Signal payload. Signal
  needs correlation, dependency, policy, and bounded payload facts only.
- A bare Signal `Aspect::new`, raw slot number, or `AspectMask::from_bits` does
  not establish physical meaning. Production Store construction consumes the
  frozen binding set; raw constructors remain confined to Signal mechanism and
  focused binding certification.
- Foundational projection, mutation, and diagnostic masks are not the same type
  or authority as Signal's bit mask. The binding derives routing only after the
  correct Foundational mask and Store physical witness have been admitted.
- The existing unchecked `with_mutation_mask`/`with_diagnostic_mask` posture is
  insufficient for this seam and may not be used as proof merely because the
  Rust marker type names the mask category.
- No JSON participates in binding, aspect value admission, Signal payload
  correlation, invalidation, completion, or observation. Terminal JSON
  projection/readmission remains an explicit external compatibility edge.
- Do not route physical work through legacy `ResourceNodeDeclaration` as a
  parallel Store-facing model. C.5.1 consumes Milestone D's capability-first
  node surface; legacy resource vocabulary may appear only inside Signal's own
  lowering.
- Do not implement physical retry, timeout, cancellation, revalidation,
  observation, retention, diagnostics, or replay behavior as Store callbacks.
  Select the existing Milestone C policy declarations when constructing the
  capability.
- Signal branch, history, restore, and replay vocabulary must not be used as
  physical crash recovery.
- If an existing generic resource API is insufficient, extend `worth-signal`
  once with domain-neutral semantics; do not fork it in Store.

**Test requirements**

- A hostile evaluator must be unable to reach backend, scheduler execution,
  settlement, or mutable Store authority through its evaluation context.
- Destroying and rebuilding the entire physical Signal owner from one live C.5
  physical world must produce equivalent readiness and no media effects.
- Dependency locality tests must show changing an unrelated artifact or
  generation does not revalidate disjoint work, while changing a declared
  dependency does.
- Binding tests must permute Foundational aspect identity, contract revision,
  Signal slot, role, mask mode, and partition lowering; every mismatch or
  duplicate must deny before graph construction, while equivalent declarations
  produce one canonical binding digest and routing result.
- Contract-mask tests must submit structurally typed but contract-illegal
  projection, mutation, and diagnostic masks and prove all deny before binding.
  A diagnostic mask must never produce an operational Signal subscription or
  change delta.
- A native Store aspect patch must invalidate exactly the bound Signal aspects
  and regions. A diagnostic-only mask, raw Signal bit, JSON object, or same-key
  wrong-revision fact must be unable to trigger mutation/readiness authority.
- Capability-first and legacy-resource alias lowering inside Signal must yield
  equivalent descriptor truth, but the Store dependency graph and public
  vocabulary must expose only the capability-first path.
- Aspect-, partition-, and hierarchy-pressure tests must prove physical
  invalidation touches only declared work families and locality, without a
  graph-wide scan or a second hidden dependency model.
- A compile/dependency test must prove Signal contains no Store-specific node,
  artifact, page, WAL, or backend types.

**Engineering decisions**

- Physical Signal is private to the Store composition owner.
- Physical work uses ordinary Signal nodes with attached async capability; it
  is not a special Signal node species.
- Foundational/Store-native aspects own semantic identity and authority;
  Signal aspects own compact dependency routing. The one admitted binding is
  the only translation between them.
- Generic Signal request lineage is correlated with, but never substitutes for,
  `PhysicalWorkIdentity`.
- Signal's temporal clock, wake, lifecycle, policy, retention, and diagnostics
  machinery are consumed as closed substrate. C.5.1 adds only physical domain
  binding and physically stricter safety law.
- Diagnostic richness is policy-controlled and cannot change operational
  results or hot-path work.

**Open questions**

- None.

### Phase 5: Admit Readiness Without Admitting Effects

Join one exact Store work intent to one async-capable Signal request lineage
while preserving the distinction between dependency readiness and permission to
consume physical resources.

**Relevant subsystems**

- physical work admission and generation fencing
- Signal async-node request admission and revalidation
- Store health, artifact freshness, and operation-local cancellation posture

**Existing APIs**

- `AsyncCapableNode::request_intent`
- `SignalRuntime::admit_async_node_request`
- `AsyncNodeRequestAdmissionReport::{classification, resource_admission}`
- `SignalRuntime::{in_flight_resource_request, resource_runtime_summary}`
- `SignalRuntime::revalidate_async_node`
- `ServingHealth::requires_inspection`

**Target APIs**

- `PhysicalWorkAdmission::admit(intent, physical_authority)` returning a sealed
  `AdmittedPhysicalWork` or typed pre-effect denial
- `PhysicalWorkSignalOwner::request(admitted)` returning
  `PhysicalWorkReadiness::{Blocked, Ready}` as a joined Store view over the
  real `AsyncNodeRequestAdmissionReport`, not a second lifecycle state machine
- `ReadyPhysicalWork` carrying both the exact Store identity and the matching
  `ResourceRequestHandle`, request generation, attempt, epoch, and capability/
  policy descriptor digests without exposing them as construction authority
- bounded move-owned physical command storage keyed by
  `PhysicalWorkIdentity`; it owns immutable domain packets until dispatch or
  safe cancellation and stores no generic lifecycle history
- `PhysicalWorkPreEffectDenial` distinguishing stale generation, unhealthy
  serving state, unsupported effect class, dependency blocked, cancellation,
  and capability absence

**Warnings**

- Signal request admission is not scheduler admission and must perform no I/O.
- A blocked Signal node is not a failed physical operation and must not poison
  Store health.
- Do not rediscover Store generation, security, or artifact scope after
  `AdmittedPhysicalWork`; carry the proven values.
- Signal generation, attempt, timeout, supersession, and cancellation posture
  are read from Signal reports. Store may join physical facts to them but may
  not mirror their transition table.

**Test requirements**

- A blocked dependency, stale runtime generation, and unhealthy Store must each
  deny before scheduler calls, allocation, or media effects and localize to
  distinct outcomes.
- Revalidating an unchanged request must preserve one lineage and exact
  readiness parity; a changed declared dependency must create only the
  policy-admitted refresh behavior.
- A forged or foreign `ResourceRequestHandle` must not correlate to admitted
  physical work even when request ids collide.

**Engineering decisions**

- Store admission precedes Signal request creation so rejected physical work
  never enters the derived graph.
- The binding from physical work identity to Signal request handle has one
  owner. Signal owns lifecycle; Store owns only the bounded physical command
  packet and, after dispatch, the non-discardable effect obligation.
- Request payload byte limits are declared through
  `AsyncNodePayloadContract::with_max_payload_bytes` before admission.

**Open questions**

- None.

### Phase 6: Lower Ready Work Into The Existing I/O Scheduler

Convert ready physical work into one scheduler declaration and one admitted
resource plan without moving dependency or effect authority into the scheduler.

**Relevant subsystems**

- foreground reservations and background capacity leases
- scheduler backend-capability admission
- queue work declarations, grouping, security, budgets, and policy receipts
- Store physical-work resource demand

**Existing APIs**

- `ServingPhysicalRuntime::admit_physical_scheduler_capability`
- `QueueWorkDeclaration::{foreground, foreground_wal_commit, background}`
- `lower_buffer_pool_queue_declaration`, `lower_wal_queue_declaration`, and
  `lower_background_queue_lease`
- `QueueExecutionAdmissionRequest::new`
- `admit_queue_execution_plan`
- `QueueExecutionReadyPlan::{work, grouping_basis, admitted_budget,
  backend_completion_binding}`
- `FoundationalPolicyAdmissionReceipt`

**Target APIs**

- `PhysicalSchedulerDemand`, lowered once from `ReadyPhysicalWork` and carrying
  exact queue slots, worker permits, bandwidth, residency hints, read-ahead or
  writeback windows, flush/sync debt, locality, and durability
- `PhysicalWorkScheduler::admit(demand)` returning
  `ResourceAdmittedPhysicalWork` containing the consuming
  `QueueExecutionReadyPlan`
- `PhysicalSchedulerDenial` preserving queue, policy, backend capability,
  security, and budget denial causes without collapsing them to unavailable

**Warnings**

- `QueueWorkDeclaration` remains scheduler policy input; it does not become the
  canonical Store operation declaration.
- Do not construct `QueueExecutionReadyPlan` outside
  `admit_queue_execution_plan`.
- C.5.1 must not introduce its own queue, semaphore, worker pool, fairness
  registry, or backpressure state beside `worth-store-io-scheduler`.
- Denial must occur before executor command construction or filesystem effects.

**Test requirements**

- Exact budget, security scope, durability, backend requirement, grouping, and
  locality mismatches must deny through `admit_queue_execution_plan` before any
  backend operation identity is issued.
- Two disjoint ready works must obtain independent scheduler progress under
  capacity for two, while one exhausted budget denies without blocking or
  mutating the admitted work.
- A scheduler plan from another Store, work identity, or security scope must be
  rejected before dispatch even when its requested budget is identical.

**Engineering decisions**

- Signal controls when work is eligible; scheduler policy controls whether
  resources are presently available.
- Foreground and background work share one scheduler contract but retain their
  distinct work classes and service posture.
- Policy receipts describe admission decisions and never prove backend effects.

**Open questions**

- None.

### Phase 7: Make The Store Executor The Sole Effect Boundary

Consume admitted physical work into one exact backend operation without
allowing the executor to re-plan, retry, widen scope, or synthesize completion.

**Relevant subsystems**

- Store physical executor and C.4 media ownership
- artifact tree coordination and range I/O
- record artifact routing
- scheduler/backend execution binding

**Existing APIs**

- `QualifiedFilesystemMedia::artifact_tree`
- `ArtifactTreeMedia::{read_exact_at, read_bounded, create_new_file,
  write_exact_at, write_scheduled_exact_at, synchronize_file, replace,
  synchronize_directory}`
- `PhysicalRecordArtifactTree::write_scheduled_exact_at`
- `ArtifactTreeFile`, `RecordFrameCoordinate`, and
  `BackendQueueExecutionPlanBinding`
- `ArtifactRangeWriteOutcome` and `ScheduledArtifactRangeWriteOutcome`

**Target APIs**

- private `PhysicalWorkExecutor::dispatch(ResourceAdmittedPhysicalWork)`
- operation-family-specific `PhysicalExecutorCommand` variants with exact
  artifact coordinates, immutable payload or bounded destination, admitted
  durability, and scheduler binding
- `DispatchedPhysicalWork`, created immediately before the first effect may
  occur and retained until exact settlement
- `PhysicalExecutorOutcome::{DeniedBeforeEffect, ReadCompleted,
  WriteCompleted, PublicationCompleted, Indeterminate}` with family-specific
  payloads rather than one counter-shaped receipt

**Warnings**

- Only the executor may hold the Store-owned route to
  `QualifiedFilesystemMedia`; Signal evaluators and scheduler code must not.
- The executor must not call `admit_queue_execution_plan`, choose durability,
  calculate retry policy, or inspect Signal dependencies.
- Backend operations that already coordinate artifact path and stable file
  identity must remain the sole coordination implementation.
- `ArtifactTreeMedia::write_scheduled_exact_at` is the current honest writeback
  join and should be generalized or called, not bypassed.

**Test requirements**

- Compile-time visibility tests must prove Signal, scheduler consumers, product
  code, and lower mechanism crates cannot invoke Store executor construction or
  raw owned media effects.
- Real-file tests must prove each dispatched identity produces at most one
  observed backend effect and the effect's artifact/range exactly matches the
  command.
- Opposing operations on one artifact must obey backend coordination while a
  disjoint artifact operation completes, proving the executor is not a global
  Store lock.

**Engineering decisions**

- Execution functions consume their command and admitted resource authority.
- Effect-class branching occurs before dispatch; the executor receives a
  monomorphic command variant.
- Backend-created operation identity and effect fate are preserved verbatim for
  settlement.

**Open questions**

- None.

### Phase 8: Join Exact Physical Settlement And Derived Signal Completion

Make backend effect evidence the only route to physical settlement, then update
the derived Signal lifecycle after physical truth is classified.

**Relevant subsystems**

- backend effect receipts and scheduler execution completion
- Store physical settlement and serving health
- buffer-pool dirty claims where present
- Signal completion admission and transactional commit

**Existing APIs**

- `CompletedArtifactRangeWrite` and `IndeterminateArtifactRangeWrite`
- `CompletedScheduledArtifactRangeWrite::{physical, queue}`
- `execute_ready_queue_plan`
- `QueueExecutionOutcome`
- `PhysicalWritebackClaim::publish_clean`
- `RawCompletionEnvelope::new`
- `SignalRuntime::admit_resource_completion`
- `SignalTransaction::{stage_admitted_resource_completion,
  commit_staged_resource_completion}`
- `PhysicalScheduledWritebackOutcome`

**Target APIs**

- `PhysicalWorkSettlement::settle(dispatched, executor_outcome)` returning
  `SettledPhysicalWork`
- exact settlement variants for no effect, completed effect with declared
  durability, written-but-scheduler-rejected, residency-terminal,
  partial/indeterminate effect, and stale/foreign outcome
- `PhysicalWorkSignalOwner::record_settlement(&SettledPhysicalWork)`, the only
  Store path allowed to construct `RawCompletionEnvelope` for physical work;
  the envelope binds Signal request, generation, attempt, epoch, payload
  contract, capability/policy descriptor identity, and admitted physical
  aspect-binding digest to the settled Store operation
- `PhysicalSignalSettlementOutcome::{Committed, ReconciledFromPhysicalTruth,
  DerivedStateUnavailable}` or equivalently precise joined outcomes that retain
  the already-final physical settlement when Signal completion cannot commit
- one causal `PhysicalWorkObservation` joining physical identity, Signal
  lineage, scheduler binding, backend operation, effect fate, and counters
  without treating any observation as authority

**Warnings**

- Signal completion is recorded after physical settlement; it cannot make a
  write durable, clean a frame, publish a root, or restore Store health.
- `execute_ready_queue_plan` accepts only the completion minted by the actual
  backend execution. Caller-supplied counts are forbidden.
- A completed write followed by scheduler or residency rejection must retain
  the physical effect in its terminal outcome.
- Signal completion denial or transaction rollback after physical settlement
  is a derived-state failure. It may trigger exact completion re-admission when
  Signal says that is legal, or disposal and reconstruction of the physical
  Signal owner; it may never retry the filesystem effect.
- Diagnostics must not require cloning payload bytes or materializing rich
  history on the ordinary path.
- `RawCompletionEnvelope` carries lifecycle correlation and byte-count contract
  evidence only. It must not carry JSON, serialized Store aspect state, or a
  second copy of the physical result; the `SettledPhysicalWork` remains the
  authoritative native subject.

**Test requirements**

- A generic or forged `RawCompletionEnvelope`, scheduler counter snapshot, or
  copied backend receipt must be unable to settle physical work or clean a
  frame.
- Reordered, duplicate, foreign-generation, wrong-attempt, wrong-payload, and
  wrong-contract Signal completions must be denied without changing physical
  settlement.
- A completed real write must settle physical truth, satisfy scheduler
  execution, publish the exact dirty claim clean, and only then commit the
  derived Signal completion; mutation tests must kill the proof when any edge
  is skipped or reordered.
- Inject failure after physical settlement but before and during Signal
  completion commit. The filesystem and physical settlement must remain final,
  no second backend effect may occur, and rebuilding Signal from current
  physical truth must converge to the control lane.

**Engineering decisions**

- Physical settlement is canonical; Signal completion is its derived lifecycle
  reflection.
- Signal's transactional completion and denial machinery is reused verbatim.
  Store adds only the physical-settlement-to-completion binding and the
  reconciliation rule for disposable derived state.
- Existing `PhysicalScheduledWritebackOutcome` semantics are preserved and
  generalized into the canonical work settlement rather than duplicated.
- One outcome retains all terminal evidence; callers do not query three owners
  afterward to understand what happened.

**Open questions**

- None.

### Phase 9: Close Cancellation, Timeout, Retry, And Supersession

Make consumer interest and physical effect obligation separate lifecycles so
abandoning a request cannot abandon possible filesystem truth.

**Relevant subsystems**

- Worth Signal resource cancellation, timeout, retry, revalidation, and
  supersession policy
- Store dispatch boundary and operation recovery disposition
- scheduler resource release and client-facing outcomes

**Existing APIs**

- `SignalRuntime::cancel_resource_request`
- `SignalRuntime::{admit_resource_timeout,
  extend_resource_timeout_heartbeat}`
- `SignalRuntime::{schedule_resource_retry,
  admit_scheduled_resource_retry}`
- `SignalRuntime::revalidate_async_node`
- `ResourceCancellationReport`, `ResourceTimeoutReport`,
  `ResourceRetryScheduleReport`, and `ResourceSupersessionRecord`
- `ResourceRequestHandle` generation and attempt identity
- `ClockAdvanceRequest`, `SignalRuntime::advance_clock`, and the
  `ResourceRetryPolicyDeclaration`, `ResourceTimeoutPolicyDeclaration`,
  `ResourceCancellationPolicyDeclaration`,
  `ResourceSupersessionPolicyDeclaration`, and
  `ResourceRevalidationPolicyDeclaration` families

**Target APIs**

- `PhysicalWorkCancellationJoin`, which joins the real
  `ResourceCancellationReport` to
  `PhysicalEffectObligation::{NotDispatched, SettlementContinues}` without
  reproducing Signal's cancellation lifecycle variants
- `PhysicalWorkRetryAdmission`, constructible only from a retryable pre-effect
  settlement, the matching `SignalRuntime::admit_scheduled_resource_retry`
  result, and Store proof that the physical effect class permits retry
- `PhysicalWorkConsumerHandle`, which may be cancelled or dropped without
  owning the dispatched Store operation
- explicit timeout and supersession mapping from Signal reports into Store
  consumer posture without changing physical effect fate
- distinct scheduler/backend deadline or interruption posture for an already
  dispatched I/O operation; it is mechanism evidence and cannot masquerade as
  Signal lifecycle timeout or proof that no physical effect occurred

**Warnings**

- Signal's cancellation result describes resource lifecycle, not proof that a
  backend effect did not begin.
- Post-dispatch cancellation must not release payload buffers, artifact leases,
  scheduler grants, or settlement state required by the in-flight operation.
- Post-dispatch cancellation must not emit a no-effect posture or otherwise
  suggest to a future semantic owner that its branch-write authority may be
  released before physical settlement classifies the effect.
- Retry is legal only for `NoEffect` or an explicitly idempotent exact effect
  with known recovery disposition.
- Timeout and retry timing must use Signal's Milestone A clock and temporal
  wakes. OS I/O deadlines and scheduler wait budgets remain separate mechanism
  limits and do not create lifecycle truth.
- Supersession must not allow a newer operation to consume the older
  operation's completion.

**Test requirements**

- Cancellation before dispatch must produce no backend operation identity and
  release every admitted resource; cancellation at and after dispatch must
  preserve exactly one eventual physical settlement.
- Timeout, retry, supersession, and revalidation schedules must be permuted with
  completion delivery and converge to the same physical truth and one legal
  consumer outcome.
- A stale attempt or superseded generation must be rejected even when its
  payload length, scheduler binding, and artifact coordinate match current
  work.
- For every cancellation/timeout race, the terminal physical outcome must
  distinguish proven no effect, settled effect, and indeterminate effect so a
  future adapter can carry exact evidence into Relational's release, advance,
  or fence transition without consulting Signal lifecycle state.
- Deterministic clock advancement must reproduce timeout and retry outcomes
  without sleeps. Replacing Signal time with wall-clock or scheduler elapsed
  time must fail a focused direct regression while producing no physical
  effect.

**Engineering decisions**

- Consumer cancellation and physical cancellation are distinct named facts.
- Cancellation, timeout, retry, supersession, and revalidation lifecycle plus
  policy selection come from Worth Signal. Physical retry and continuation
  safety come from Store effect classification and may only narrow that result.
- Cancellation and timeout paths use the same settlement owner as ordinary
  completion; they do not add callback cleanup routes.

**Open questions**

- None.

### Phase 10: Preserve Disjoint Progress Under Bounded Pressure

Prove that the canonical topology supports concurrency through actual
independence rather than a runtime-wide mutable borrow or registry lock.

**Relevant subsystems**

- physical work correlation and active-operation storage
- Signal locality/aspect invalidation
- scheduler grouping, foreground reservations, background leases, and
  backpressure
- artifact mutation coordination and C.5 read/write owners

**Existing APIs**

- Signal `AspectMask`, `PartitionSubscription`, and dependency declarations
- `group_ready_queue_pair` and `execute_grouped_ready_queue_plans`
- `QueueGroupingBasis` and `QueueExecutionReplayIdentity`
- `ForegroundReservationReceipt`, `BackgroundIdleCapacityLease`, and
  `BackgroundResourceBudget`
- backend artifact mutation coordination used by `ArtifactTreeMedia`

**Target APIs**

- arena-bounded `PhysicalCommandArena` keyed by direct
  `PhysicalWorkIdentity`, with one direct binding to the corresponding
  `ResourceRequestHandle`; entries contain immutable command packets or
  dispatched-effect obligations, never generic lifecycle/history state
- `PhysicalWorkConcurrencyScope`, lowered from artifact/range and security
  identity rather than caller labels
- independently borrowable physical submission capabilities that can feed the
  bounded command arena concurrently without owning a global queue or
  serializing through one mutable physical-instance borrow
- exact active, blocked, ready, queued, dispatched, settling, and terminal work
  counters split by work family and pressure class
- one bounded completion batch path using
  `SignalRuntime::admit_resource_completion_batch` when batch delivery is the
  honest cardinality

**Warnings**

- A global `Mutex<ServingPhysicalRuntime>`, Store-owned generic
  `Mutex<HashMap<request, lifecycle>>`, or one mutable scheduler borrow is not
  a concurrency architecture. Signal's indexed in-flight registry remains the
  sole generic lifecycle lookup.
- Do not clone `Arc`-shared operation packets on the hot path merely to satisfy
  ownership; move consuming packets and share only true multi-observer state.
- Queue grouping must preserve operation identity and may not group different
  security, durability, recovery-ordering, or writeback-policy scopes.
- A branch label, semantic writer token, upstream queue name, or opaque
  correlation value is not a physical disjointness proof. C.5.1 must neither
  accept one as `PhysicalWorkConcurrencyScope` nor build a Store-local branch
  writer registry; the future adapter lowers already-authorized semantic work
  into exact physical scope while retaining branch authority above Store.
- C.10 owns full starvation and QoS closure; C.5.1 must still provide bounded
  progress and honest backpressure now.

**Test requirements**

- At least two disjoint reads and two disjoint writes must progress while one
  unrelated request is blocked and one queue budget is exhausted; exact
  counters must show no global serialization.
- The disjoint writes must be submitted through independently acquired
  physical mutation capabilities. Replacing those capabilities with a global
  physical submission lock, or accepting a caller branch/lane label as their
  independence proof, must fail a focused concurrency regression.
- Opposing same-artifact mutations must serialize at the artifact coordination
  boundary without blocking a disjoint artifact operation.
- Scale tests must demonstrate lookup and completion work proportional to the
  touched operations, not total active or historical work.
- Churn tests must reconcile Signal's in-flight/terminal reports with the
  command arena: safely cancelled commands leave both owners, dispatched
  commands remain until physical settlement, and retained Signal history does
  not retain command bytes or effect authority.
- Focused grouping regressions must reject erased security, durability,
  locality, or recovery distinctions at the grouping boundary.

**Engineering decisions**

- Parallelism derives from declared physical locality and scheduler capacity.
- Future one-writer-per-branch-generation-at-a-time policy composes above this
  boundary:
  Relational decides which semantic mutation is active, while Store remains
  capable of concurrent physical work from different branches and honest
  coordination wherever their physical or global scopes overlap.
- Physical command storage is bounded by admitted capacity. Generic in-flight
  lifecycle, retained history, and diagnostics remain in Signal's already
  separated hot/cold/projection stores and never retain physical authority.
- Exact structural counters, not elapsed time alone, prove concurrency and
  pressure behavior.

**Open questions**

- None.

### Phase 11: Make Partial And Indeterminate Effects Terminally Honest

Ensure every effect that may have changed bytes revokes the necessary authority
and retains enough evidence for later inspection or recovery.

**Relevant subsystems**

- backend exact effect outcomes and completed-prefix evidence
- Store serving health and mutation authority
- scheduler and residency terminal outcomes
- publication residue and recovery locators

**Existing APIs**

- `ArtifactRangeWriteOutcome::{Completed, DeniedBeforeEffect, Indeterminate}`
- `IndeterminateArtifactRangeWrite::{failure, coordinate, completed_bytes,
  operation}`
- `PhysicalScheduledWritebackOutcome::{RetryableBeforeEffect,
  InspectionRequired, WrittenButNotApplied, Applied, ResidencyTerminal}`
- `UnpublishedRecordEffectFate`, `UnpublishedRecordWorldFate`,
  `RecordPublicationRecoveryLocator`, and `IndeterminateRecordPublication`
- `ServingHealth::revoke`

**Target APIs**

- `PhysicalWorkTerminalFailure` retaining work identity, exact effect fate,
  completed prefix/range, backend operation, scheduler posture, residency
  posture, publication residue, and required recovery disposition
- `PhysicalWorkHealthRevocation`, created only by the physical settlement owner
  and consumed into Store health
- terminal observer output that distinguishes request failure from possible
  persistent-world mutation

**Warnings**

- Never translate an indeterminate effect into retryable merely because the
  scheduler or Signal considers the request incomplete.
- A failed diagnostic write or completion publication cannot erase an already
  completed physical effect.
- Health revocation must be shared by all subsequent read, readmission, scan,
  mutation, and shutdown surfaces that depend on the damaged truth.
- Exact completed-prefix evidence is not permission for ordinary continuation;
  it is recovery and inspection evidence.

**Test requirements**

- Inject zero-byte pre-effect denial, short write, write error after a known
  prefix, completed write followed by scheduler rejection, and completed write
  followed by residency rejection; each must produce its distinct typed fate.
- After partial or indeterminate mutation, every relevant mutation and
  authoritative read/readmission path must observe revoked health before
  further effects.
- A fresh external observer must confirm the reported changed prefix and prove
  the terminal outcome cannot claim more or fewer bytes than the filesystem.

**Engineering decisions**

- Backend evidence defines effect fate; Store defines the resulting health and
  recovery posture.
- Partial and indeterminate mutation are inspection-required in C.5.1.
- Terminal evidence is move-owned by the outcome and remains available after
  Store authority is consumed.

**Open questions**

- None.

### Phase 12: Close, Drain, Abort, And Reconstruct Work Safely

Make physical instance termination a complete work protocol rather than a drop
order that assumes no operation is in flight.

**Relevant subsystems**

- physical Store lifecycle and termination guard
- physical work admission, Signal resources, scheduler plans, executor effects,
  residency, media release, and terminal evidence
- fresh-process C.5 reopen

**Existing APIs**

- `ServingPhysicalRuntime::{close, abort}`
- `ServingShutdownOutcome`, `MediaShutdownOutcome`, and
  `PhysicalResidencyShutdown`
- `SignalRuntime::{resource_runtime_summary, cancel_resource_request}`
- `RecordServingTerminalObservation`
- `PhysicalRecordObserver` and `PhysicalMediaObserver`

**Target APIs**

- consuming `PhysicalStoreClosePlan` with phases equivalent to
  `AdmissionStopped -> SafeCancellationComplete -> DispatchSettlementComplete
  -> SignalDisposed -> ResidencyClosed -> MediaReleased`
- `PhysicalStoreCloseOutcome::{Closed, InspectionRequired}` and a distinct
  `PhysicalStoreAbortOutcome`
- `PhysicalWorkDrainObservation` listing exact counts and identities of work
  settled, cancelled before dispatch, continued after consumer cancellation,
  inspection-required, and residual
- no public or persisted physical Signal snapshot accepted by initialize/open
- close reconciliation that consumes `ResourceRuntimeSummary` plus the bounded
  physical command/effect-obligation arena and refuses clean close if either
  owner has an unmatched live identity
- terminal work evidence preserving proven-no-effect, completed, and
  indeterminate recovery disposition for every admitted mutation so downstream
  semantic recovery never has to infer writer-fence fate from shutdown timing

**Warnings**

- Close may wait only on admitted bounded work and must expose its bound and
  progress. It cannot wait forever on consumer-owned callbacks.
- Abort does not mean forget. Dispatched work still requires terminal effect
  classification before media authority can be reported safely released.
- Signal disposal precedes media release but follows physical settlement and
  derived completion recording for work that completed normally.
- Pending temporal wakes, retry schedules, retained lifecycle history, and
  diagnostics are disposed through Signal's own lifecycle. They do not become
  Store shutdown residue unless a matching physical command or effect
  obligation remains.
- Reopen reconstructs from C.5 manifests, records, residue, and media truth—not
  from Signal history or scheduler replay objects.

**Test requirements**

- Begin close with work blocked, ready, resource-admitted, dispatched,
  consumer-cancelled after dispatch, settled, and inspection-required; prove
  every identity appears exactly once in terminal evidence.
- Kill a process during each close phase, start a fresh executable, and prove
  C.5 reopen reaches the same physically allowed world without receiving work
  or Signal state.
- Crash with a mutation in each pre-effect, dispatched, settled, and
  indeterminate posture; fresh physical recovery must classify enough exact
  fate for a future semantic layer to reacquire, advance, or retain its own
  branch-generation fence without importing physical Signal state.
- Retained request, scheduler, executor, Signal, and observer handles must be
  unusable after their owning generation is consumed.
- Run close with maximal Signal lifecycle-history retention and diagnostics
  richness, then prove those derived records neither retain physical command
  bytes nor prevent clean media release after all physical obligations settle.

**Engineering decisions**

- New work admission stops first.
- Media authority releases last.
- Clean close requires zero unclassified managed work; inspection-required is
  an explicit terminal posture, not a close error that discards ownership.
- Signal checkpoint, branch restore, and replay artifacts are intentionally not
  a physical reopen input. Live derived-state reconciliation may use Signal's
  own contracts, but process recovery reconstructs from C.5 physical truth.

**Open questions**

- None.

### Phase 13: Migrate The C.5 Read Path

Move real locate, readmission, bounded read, scan, and streaming I/O into the
canonical work topology while leaving pure decode and already-resident access
direct.

**Relevant subsystems**

- record locate and external-locator readmission
- record scan cursor and bounded scan sessions
- frame loading, bounded artifact reads, health revocation, and read counters
- physical read facade DX

**Existing APIs**

- `ServingPhysicalRuntime::records`
- `PhysicalRecordReader::{open, readmit_locator, open_external}`
- `RecordReadSession::read_next`
- `PhysicalRecordScanSession::{scan, read_next_into}`
- `RecordScanRequest`, `ExternalRecordScanCursor`, and `RecordReadLimits`
- current `FrameLoadPort`/`RecordFramePorts` loading path

**Target APIs**

- current read methods preserved as the ordinary facade where their semantics
  remain honest, but internally submitting physical read work through the
  canonical readiness/scheduler/executor/settlement progression
- independently borrowable read authority that does not borrow mutation,
  Signal mutation, scheduler mutation, or the complete physical instance
- operation-local read sessions that retain their physical work identity and
  lease until completion or typed cancellation
- read-fault async capability attached to ordinary physical dependency nodes,
  with `PhysicalSignalAspectBindingSet` and admitted
  `PhysicalWorkAspectDelta` lowering to Signal `AspectMask` and
  `PartitionSubscription` for the actual root, artifact, frame, or scan
  partition touched by the request
- direct fast path for a valid resident frame only when the same physical work
  admission, basis, health, and observation contracts remain satisfied

**Warnings**

- Do not wrap checksum verification, decoding already-held bytes, or cheap
  locator parsing in ceremonial Signal nodes.
- A cache hit may skip media execution but may not skip Store admission,
  generation, security, health, stable-lease, or aspect-native semantic-basis
  checks.
- `readmit_locator` and scan continuation damage must revoke the same shared
  health observed by mutation authority.
- Read methods must not conceal full manifest or whole-store materialization.

**Test requirements**

- Hot-resident and cold-file paths must return identical record bytes and
  canonical read observations while exact media and scheduler counters differ
  in the declared way.
- Concurrent locate, external readmission, bounded scan, and append must retain
  stable C.5 basis semantics without borrowing or locking the whole runtime.
- Stale locator, stale scan cursor, wrong Store, corrupted manifest, cancelled
  read, and short/partial backend read must deny or revoke health at their exact
  boundary.
- A bare Signal aspect/region change and a JSON-shaped record description must
  be unable to admit or redirect a read. The equivalent admitted Store-native
  aspect fact must select exactly the declared Signal aspects and partitions.
- Scale proof must bound manifest touches, bytes, allocations, frame faults,
  Signal aspect/partition invalidations, and work lookups by declared access
  breadth rather than Store or graph size.

**Engineering decisions**

- Read work enters Signal only where dependency readiness or asynchronous
  resource lifecycle exists.
- Existing read-facing names remain unless their borrow or authority semantics
  become dishonest.
- Read observations join work and record counters without becoming an authority
  source.

**Open questions**

- None.

### Phase 14: Migrate The C.5 Mutation And Publication Path

Move append, artifact writes, manifest publication, synchronization, and root
replacement through the same work progression and eliminate the global
`&mut ServingPhysicalRuntime` orchestration bottleneck.

**Relevant subsystems**

- append batch planning and placement
- candidate-frame residency and exact writeback
- data, manifest, catalog-candidate, replacement, and directory synchronization
- allocation frontier, free-space truth, root publication, and residue

**Existing APIs**

- `ServingPhysicalRuntime::records_mut`
- `PhysicalRecordWriter::{append_batch,
  append_batch_reconstructing_manifest_capacity}`
- `RecordAppendBatch`, `AdmittedRecordPlacementPolicy`, and
  `RecordPublicationStage`
- `ServingPhysicalRuntime::execute_scheduled_writeback`
- `PhysicalWritebackClaim::publish_clean`
- `ArtifactTreeMedia::{create_new_file, write_scheduled_exact_at,
  synchronize_file, replace, synchronize_directory}`
- `PublishedRecordBatch`, `UnpublishedRecordBatchFailure`, and
  `IndeterminateRecordPublication`

**Target APIs**

- independently borrowable `PhysicalRecordSubmission` or equivalently named
  mutation authority obtained without `&mut ServingPhysicalRuntime`
- append submission that consumes `RecordAppendBatch` and placement authority,
  then drives the canonical work topology synchronously or asynchronously
  without changing its outcome semantics
- capability-attached Signal hierarchy for genuinely asynchronous publication
  dependencies such as prepared data/manifest work becoming eligible before
  root publication; each dependency edge is advanced only from settled
  physical stage truth
- aspect-native publication facts for data materialization, manifest basis,
  catalog candidacy, root publication, and durability posture, lowered into
  Signal aspect/partition deltas only through admitted bindings
- narrow root-publication owner as the only serialized mutation boundary;
  preparation, payload materialization, and disjoint artifact work remain
  independently admissible
- physical publication outcome and recovery disposition that remain exactly
  correlated to the submitting `PhysicalWorkIdentity`, allowing a future
  adapter to correlate physical fate with its semantic mutation and Relational
  to settle its separately owned writer generation without exposing branch
  authority to Store
- current scheduled writeback folded into the general executor/settlement path;
  the special-case orchestration type is removed when no longer needed

**Warnings**

- Removing the global mutable borrow does not permit concurrent root
  publication without ordering; serialize the smallest true authority.
- Do not replace the global mutable borrow with a Store-owned branch lock table,
  branch-labelled submission lane, or semantic writer lease. Physical root,
  allocation, artifact, and scheduler coordination must remain named by the
  physical authority actually shared.
- Manifest and catalog ordering remains C.5 truth and cannot be inferred from
  Signal dependencies or scheduler completion.
- `RecordPublicationStage` remains physical publication typestate. Signal's
  hierarchical async graph derives readiness around it and must not become an
  alternative publication state machine.
- Publication packets, manifest facts, and completion evidence remain typed
  native artifacts. JSON documents and unvalidated aspect maps are never an
  internal staging representation.
- Every fallible precondition and resource admission occurs before the first
  effect whenever physically possible.
- Candidate bytes remain owned by residency until the Store-created exact
  effect receipt is accepted; scheduler or Signal completion cannot clean them.

**Test requirements**

- Two disjoint append preparations submitted through separate physical mutation
  capabilities must progress concurrently and converge through legal root
  publication order; overlapping publication must not lose either batch or
  expose a partial root.
- Crash or fault injection at every C.5 publication edge must preserve the
  existing exact `Published`, `Unpublished`, or `Indeterminate` world fate after
  migration.
- Focused direct regressions must make a bypass of Signal readiness, scheduler
  admission, executor dispatch, backend receipt, or physical settlement fail at
  the nearest causal boundary.
- Reorder or suppress completion of one child publication stage and prove the
  dependent async-capable node remains in the Signal-classified posture while
  the physical publication typestate independently forbids root publication.
- The old `records_mut` global-borrow path and special writeback path must be
  unreachable or removed before this phase closes.

**Engineering decisions**

- Public synchronous append remains acceptable DX if it drives the canonical
  internal work progression and exposes honest orchestration cost.
- Mutation concurrency stops at the narrowest real physical authority:
  artifact coordination, allocation frontier, or root publication.
- The future single active writer per branch generation at a time is a semantic
  admission rule above Store. C.5.1 deliberately permits multiple physical
  mutation capabilities because different branches, maintenance, writeback,
  and other admitted work must share one physical instance without global
  serialization.
- One canonical publication outcome remains the source for all derived Signal,
  scheduler, diagnostic, and future WAL consumers.

**Open questions**

- None.

### Phase 15: Delete Duplicate Work Machinery And Seal Dependencies

Remove every production substitute made obsolete by the canonical instance and
make its return mechanically difficult.

**Relevant subsystems**

- Store, buffer-pool, scheduler, backend, certification, and test-support
  feature graphs
- legacy background envelopes and S.2 models
- direct backend execution sessions and special-case work orchestrators
- source, visibility, manifest, and dependency checks

**Existing APIs**

- `legacy-s2-models` and `certification-test-authority` feature boundaries
- buffer-pool `background_work` envelope surfaces
- backend queue execution session and Store-owned ticket construction
- physical scheduled-writeback special case
- C.1 boundary-check and consolidated UI-test infrastructure

**Target APIs**

- one ordinary dependency graph in which `worth-store` alone composes
  `worth-signal`, scheduler, buffer pool, and owned backend facade
- one production work registry: Worth Signal owns generic resource lineage and
  Store owns only bounded command packets and the exact physical identity/
  effect-obligation binding required for dispatch and settlement
- manifest/feature gate proving ordinary Store products cannot activate
  `legacy-s2-models` or certification test authority
- source/visibility gate proving all backend queue tickets and raw owner
  constructors remain private to the admitted Store execution boundary
- aspect-boundary gate proving production physical runtime code cannot
  construct raw Signal aspect slots/masks outside
  `PhysicalSignalAspectBindingSet`, cannot collapse Foundational mask modes,
  and cannot accept raw aspect values before native validation/admission
- dependency/source gate forbidding `serde_json` and JSON-shaped carriers in
  ordinary physical runtime, scheduler, residency, backend, settlement, and
  evidence-construction modules; only explicitly named external compatibility
  ingress and terminal projection modules are allowlisted
- dependency/source gate forbidding Query, Relational, branch-head, MVCC, and
  semantic writer authority from the physical Store instance while preserving
  an adapter-consumable facade for submission identity, exact physical scope,
  settlement, and recovery disposition

**Warnings**

- Similar names are not proof of duplication. Delete based on competing
  authority and lifecycle, not string matching alone.
- Certification-only substrate may remain only when an active proof consumes it
  and the ordinary feature graph cannot; otherwise delete it.
- Do not leave deprecated aliases, wrappers, reexports, feature shims, or dead
  tests for unpublished internal APIs.
- Delete or forbid Store-local lifecycle enums, timer wheels, retry queues,
  timeout registries, cancellation/supersession tables, retained async history,
  policy registries, and direct `ResourceNodeDeclaration` construction. Their
  surviving meanings must come from Signal A-D APIs.
- No warning, dead code, unused dependency, or unreachable production branch is
  accepted at closeout.

**Test requirements**

- Cargo feature-tree and boundary tests must prove ordinary Store cannot enable
  legacy models, raw backend authority, certification constructors, or a second
  Signal/scheduler runtime.
- Aspect and JSON gates must fail on raw `Aspect::new`/`AspectMask::from_bits`
  production routing, Foundational mask substitution, `serde_json::Value`, JSON
  byte round-trips, or JSON-shaped maps introduced into any internal work phase.
- Dependency, compile-fail, and focused behavior tests must reject a duplicate
  pending registry, callback settlement route, Store-local timer/retry/policy
  registry, legacy resource-node construction, raw backend dispatch, or old
  special writeback route at the nearest owned boundary.
- A Store-local branch writer map, branch-labelled physical queue, or branch
  token accepted as physical disjointness proof must fail the dependency/source
  gates and a focused compile-fail or behavior test.
- Strict all-target/all-feature Clippy, dead-code/unused-dependency checks, and
  touched-file structural QA must be clean.

**Engineering decisions**

- The implementation change may cross crate boundaries as far as required to
  delete the real duplicate.
- Historical compatibility is not a constraint for unpublished internal APIs.
- The surviving tree must make the next correct physical work edit obvious
  without searching for competing runtimes.

**Open questions**

- None.

### Phase 16: Exercise The Joined Runtime And Seal The C.6 Handoff

Exercise the complete physical work topology with real files and processes, then
expose only the typed seams required for C.6 buffer-pool completion.

**Relevant subsystems**

- physical Store instance, Signal binding, scheduler, executor, settlement,
  shutdown, and observation
- C.5 production record journeys and independent offline walker
- C.6 frame load, dirty claim, candidate publication, and bounded residency
- direct process-scenario runner

**Existing APIs**

- `FrameLoadPort`, `CandidateFrameSet`, and
  `CandidateFramePublicationPort`
- `PhysicalResidencyPool`, `PhysicalFrameLease`, `PhysicalWritebackClaim`, and
  `OperationAllocationGrant`
- `RecordAppendBatch`, `PhysicalRecordReader`, and record scan sessions
- production fault schedule/yieldpoint support at the C.4 media boundary
- offline record walker and store-test-runner process gates

**Target APIs**

- sealed `C6PhysicalWorkHandoff` or equivalently responsibility-named handoff
  whose private construction requires canonical frame-load, dirty/writeback,
  candidate-publication, scheduler-demand, executor, settlement, and lifecycle
  contracts
- a narrow future-integration handoff through the ordinary physical Store
  facade: independently borrowable submission capability, generation-fenced
  physical work identity, exact physical concurrency scope, terminal settlement
  and recovery disposition, and no branch/MVCC/semantic writer vocabulary
- no public export of raw Signal nodes, resource completion constructors,
  scheduler plan construction, residency claims, executor commands, or backend
  receipts through the handoff

**Warnings**

- The handoff is not permission to predeclare fake C.6 owners or policies.
- Direct tests observe execution; they cannot construct authority or provide
  expected persisted bytes to reopen.

**Test requirements**

- **Lifecycle maelstrom:** on one real Store, concurrently run
  disjoint reads, two append preparations through independent physical mutation
  capabilities, exact writeback, resource denial,
  pre/post-dispatch cancellation, timeout/retry, completion reordering, and
  shutdown. Drive timeout and retry using `ClockAdvanceRequest`; attach the
  selected lifecycle/policy families through
  `AsyncNodeCapabilityDeclaration`; and scope unrelated work with aspects and
  partitions derived from admitted Foundational/Store-native aspect bindings.
  Apply one native aspect patch that should wake exactly one dependency slice
  while disjoint slices remain untouched. A serial authority model fixes legal
  outcomes before execution;
  exact media and work counters prove one effect per dispatch, zero global
  serialization of disjoint work, and no Store-local async or branch-writer
  state machine. A caller branch/lane label must be unable to widen the physical
  concurrency scope or alter scheduling.
- **Hostile physical truth:** a writer process is killed at named
  points before dispatch, during a short write, after exact write before
  scheduler settlement, during publication, and during shutdown. A fresh
  offline observer verifies exact bytes/prefix/residue; a distinct fresh Store
  process opens from root and configuration only. No Signal or scheduler state
  crosses the process boundary.
- **C.6 inheritance siege:** run a Store materially larger than
  its admitted residency budget with hot and cold reads, pinned frames, dirty
  pressure, denied over-pin, writeback, eviction/refault, cancellation, and
  close. Prove every fault and writeback enters the C.5.1 topology, memory stays
  bounded, dirty frames never become clean without the exact backend receipt,
  and no C.6-local scheduler or pending registry exists.
- Phase 16 remains a direct lifecycle and shutdown regression test. It does not
  require a mutation campaign, source manifest, or machine report.

**Engineering decisions**

- Direct boundary scenarios use production facades, real filesystem effects, independent
  expected truth, and at least one fresh process.
- Package-scoped checks are the normal implementation feedback loop; workspace
  formatting, constitutional gates, and affected all-feature verification are
  the final code gates.
- C.6 may add residency policy and frame lifecycle only by implementing the
  handoff; it may not add another async or I/O runtime.
- Part II may bind its Relational branch-write authority to submitted physical
  work only through the adapter's transactional join over the ordinary Store
  facade and exact settlement evidence; the adapter may correlate, but only
  Relational may advance, release, or fence branch-write authority. Part II may
  not push branch ownership into C.5.1 or replace the physical work topology.

**Open questions**

- None.
