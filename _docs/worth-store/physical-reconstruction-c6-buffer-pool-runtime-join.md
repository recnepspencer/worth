# C.6: Buffer Pool And Bounded Physical Access Join

## Goal

Make bounded physical residency the only ordinary access path for the C.5
file-backed Store inside the C.5.1 physical work runtime. Real record reads and
writes must fault, pin, lease, decode, dirty, write back, evict, and retry under
one physical-instance-owned budget and one Signal/scheduler/executor/settlement
topology. The public read surface must offer lease-scoped zero-copy views and
explicitly bounded copies without exposing pool mutation authority or inventing
semantic residency.

C.6 is not an isolated buffer-pool exercise. It is the cutover that turns the
existing residency mechanisms into the ordinary Store contract and deletes the
temporary handoff and legacy S.2 model graphs.

## Current Verification Policy

C.6 behavior is protected by current direct Cargo tests and repository gates.
Generated proof ledgers, source bindings, mutation catalogs, evidence bundles,
and report pipelines are retired and must not be recreated. Git preserves the
implementation history.

## Why This Milestone Exists

C.5.1 proved that a bounded frame path can inherit the canonical physical work
runtime. It did not yet make that path the finished product boundary:

- ordinary `PhysicalRecordReader` sessions expose copy-only streaming;
- temporary `C6*` production types expose low-level residency and writeback
  operations;
- operation scopes are observable, but they do not yet carry independently
  admitted scope ceilings inside one hard total envelope;
- prefetch, read-ahead, and write-behind have pool counters and grants but are
  not fully joined to the ordinary C.5.1 progression;
- legacy S.2 frame tables, snapshot admissions, record views, feature flags,
  fixtures, and downstream certification models still describe a parallel
  physical world.

If those seams survive C.6, C.7 recovery and durability work can accidentally
join the temporary handoff, C.9 and C.10 can build on fake leases, and every
later physical owner can add another queue or allocation truth. C.6 therefore
finishes the join, proves it under hostile memory pressure, and removes every
superseded route in the same milestone.

## Governing Constraints

- `MENTALITY.md` requires the all-victims-unavailable world to shape the
  residency authority before WAL, recovery, integrity, isolation, and blob
  work increase its surface.
- `arch_laws.md` requires fault, lease, dirty, writeback, and settlement
  progression to consume proof-bearing predecessors. A counter, coordinate,
  generation number, or scheduler receipt is not authority for the next phase.
- `composition_laws.md` requires access, lease projection, pressure admission,
  dirty transition, speculative work, and writeback to remain named semantic
  responsibilities. C.6 may not replace the temporary handoff with a
  `residency_manager.rs`, `helpers.rs`, or another god file.
- `domain_structure_laws.md` requires the physical tree to expose current and
  committed future distinctions. A directory with one implementation file is
  correct when it names a roadmap-backed family expected to grow. Empty
  placeholder modules for unimplemented successors are forbidden.
- `perf_laws.md` requires admission before allocation, exact structural
  accounting, bounded copy widths, hot-hit avoidance of media work, and
  separate ordinary and reconstructive costs.
- `testing_laws.md` requires the real Store facade, real filesystem effects,
  independent allocation and media observation, fresh-process verification,
  and controlled defects that fail at their causal boundary.
- `dx_laws.md` requires the intended lease and pressure experience to be
  designed as code before implementation makes accidental APIs permanent.
- The reconstruction roadmap reserves `C.*` vocabulary for planning,
  specification, and evidence. Production modules, types, functions, tests,
  counters, and errors must use responsibility names.

## Inherited Truth From C.5.1

C.6 starts from, and must not weaken, these closed guarantees:

1. One `ServingPhysicalRuntime` owns one physical Store instance, one private
   physical Signal runtime, one scheduler, one executor, one settlement owner,
   and one physical residency pool.
2. Signal derives readiness and owns generic request lifecycle. It does not own
   physical effect truth.
3. The scheduler admits resource demand. It does not perform media effects or
   settle Store work.
4. The executor is the only ordinary route to the media port.
5. Store settlement classifies every possibly started effect and revokes
   health when effect fate is indeterminate.
6. Ordinary serving reads use the canonical frame source. Direct media reads
   are confined to bounded bootstrap/admission work.
7. Candidate-frame publication and residency writeback already have exact
   receipt validation and canonical settlement paths.
8. A physical Store generation fences stale submission, residency, retry, and
   completion authority.
9. Query, branch, MVCC, and semantic residency authority are absent from the
   physical runtime.
10. The C.5.1 C6 inheritance siege proved the handoff can carry a bounded
    workload. That evidence is inherited substrate, not proof that C.6 itself
    is complete.

Any implementation that changes one of these guarantees must rerun the focused
owner and integration tests affected by that change.

## Adversarial Constraint

The completed milestone must survive this joined condition:

> A producer process creates and closes a real store whose persisted record
> bytes exceed the configured resident-byte budget by at least 32 times. A
> fresh serving process opens that store with exactly twelve frame entries,
> eight resident-frame slots, six pin leases, four pinned-frame identities, two
> dirty frames, one write-behind grant, and scope ceilings whose admitted sum
> is larger than—but whose live total can never exceed—the global operation
> envelope. It pins every legal foreground victim, saturates dirty capacity,
> delays the sole writeback after media dispatch, starts duplicate cold faults
> for one frame, drives a hot-set loop, streams a disjoint cold extent, submits
> prefetch and read-ahead beyond their ceilings, attempts a seventh lease,
> cancels one request before dispatch, detaches one consumer after dispatch,
> closes a record session while a borrowed view would still be live, and
> begins shutdown while the delayed writeback still has possible effect.
> No request allocates before admission. Duplicate faults cause one source
> load. A hot hit causes no media command. No pinned or dirty frame is evicted.
> Foreground progress is either admitted or denied with exact retry posture;
> background work cannot steal ungranted foreground bytes. The delayed
> writeback remains dirty until an exact backend receipt is consumed by Store
> settlement. Every cancellation and shutdown obligation reaches one terminal
> fate. A second fresh process reconstructs the producer's expected records
> from files alone.

The direct hostile scenario and focused owner tests must detect regressions
that would:

1. copy the complete store into an unaccounted buffer;
2. evict a pinned frame;
3. allocate a frame before resident and metadata admission;
4. make duplicate faults perform duplicate source loads;
5. mark a dirty frame clean from scheduler completion rather than an exact
   backend receipt;
6. let speculative work bypass its kind ceiling or the total envelope;
7. let one operation scope consume another scope's ungranted allowance;
8. route a fault or writeback around Signal, scheduler, executor, or Store
   settlement;
9. accept a stale-generation lease or writeback claim;
10. add a residency-local queue, pending map, callback registry, or executor;
11. use a legacy S.2 frame table or snapshot-derived admission as the subject;
12. make a record view outlive the lease that pins its frame.

Each regression must fail at the nearest owned boundary: unaccounted copies at
allocation observation, pinned eviction at lease/eviction truth, duplicate
loads at fault coalescence, and premature clean transitions at dirty/writeback
settlement. A generic final-data mismatch is insufficient localization.

## Authority Topology

| Responsibility | Sole owner | Explicit non-authority |
| --- | --- | --- |
| Store lifecycle, generation fence, physical operation identity, and final effect fate | `worth-store` physical runtime | pool ids, Signal ids, scheduler receipts, counters |
| Resident bytes, metadata, frame identity, pin leases, dirty state, eviction eligibility, operation allocations, and speculative grants | `worth-store-buffer-pool` | media effects, work readiness, scheduler policy, semantic residency |
| Fault and writeback dependency readiness plus generic cancellation/retry/timeout lifecycle | private C.5.1 `worth-signal` instance | bytes, frame truth, resource admission, effect settlement |
| Memory/I/O/queue admission and dispatch order | `worth-store-io-scheduler` | residency truth, filesystem truth, final effect fate |
| Exact filesystem reads and writes | `worth-store-physical-backend` | frame lifecycle, decode truth, Store settlement |
| Physical coordinates, byte formats, decode, and integrity fields | `worth-store-physical-format` | residency, scheduling, media ownership |
| Record lease/view projection and adapter-safe pressure evidence | `worth-store` record-serving facade | pool mutation, semantic residency, branch truth |
| Future WAL ordering and durable publication | C.7 durability owners | current C.6 dirty state alone |
| Future recovery reconstruction | C.8 recovery owners | live pool contents or cached runtime state |
| Future protected access and stable-read execution | C.9/C.10 physical owners composed by `worth-store` | direct pool control or legacy S.2 models |

## Product Decision Lock

1. `ServingPhysicalRuntime` remains the sole physical Store runtime. C.6 does
   not add a second runtime, pool owner, work registry, scheduler, or executor.
2. `PhysicalRecordResidencyPolicy` remains part of initialize/open admission,
   but evolves to admit the full budget declaration before the pool exists.
3. One hard admitted-memory envelope covers resident bytes, bounded metadata,
   active operation allocations, and any other pool-owned live allocation
   named by the implementation. No category may disappear from the total by
   being called scratch, staging, speculative, or test support.
4. Resident bytes, metadata bytes, frame entries, pin leases, pinned frame
   identities, dirty frames, and speculative grants remain separately bounded
   dimensions because none can be honestly inferred from another.
5. Foreground read, foreground write, recovery, scrub, maintenance,
   verification, and blob scopes receive typed per-scope operation ceilings
   inside one global hard envelope. C.6 provides accounting and denial
   isolation, not C.10 fairness, QoS, or stable-read policy.
6. A scope ceiling is not a reservation unless explicitly admitted as one.
   Unused allowance does not authorize the total envelope to be exceeded.
7. Ordinary record access always uses `ResidencyFaultOnly` semantics: a hit
   consumes existing frame authority without fake I/O work; a miss lowers one
   canonical fault through the C.5.1 topology.
8. Concurrent misses for the same store generation and frame coordinate share
   one loading identity and one source load. Waiters receive the same admitted
   frame or the same terminal failure; they do not create a second queue.
9. Bootstrap may use a direct source only while no serving pool can yet exist,
   under explicit bootstrap byte and operation limits. The direct source is
   private and mechanically absent from ordinary serving call graphs.
10. `RecordReadSession` becomes the public lease-bearing record access object.
    It may own one frame at a time for extent streaming; it does not materialize
    the complete record.
11. `PhysicalRecordChunkView<'session>` borrows a `RecordReadSession`. Its byte
    slice, logical range, physical basis, and generation cannot outlive that
    borrow. The name states honestly that an extent view is not a whole record.
12. Contiguous inline payloads and the current extent chunk may be exposed
    zero-copy. Cross-frame consumers iterate lease-scoped views or request an
    explicitly bounded copy into caller-owned storage.
13. No public convenience returns `Vec<u8>`, `Bytes`, `Arc<[u8]>`, or another
    owning whole-record value from an unbounded record read.
14. Copy accounting occurs at the actual copy boundary and records exact
    operations, bytes, and maximum width. A decode that borrows bytes is not a
    copy; a fixture cloning bytes is.
15. Pool frame leases and writeback claims remain move-owned typestate. A
    coordinate, id, generation, or counter snapshot cannot forge them.
16. A clean frame becomes dirty only through an admitted mutation/candidate
    transition with its replacement allocation already charged. Consuming a
    lease is not permission to allocate an unbounded replacement `Vec`.
17. A dirty frame becomes clean only after an exact backend write receipt is
    validated and consumed by Store settlement.
18. A frame with any live pin, live load, active candidate publication, dirty
    state, or outstanding writeback claim is not an eviction candidate.
19. Eviction selection is pool-owned and deterministic for a fixed trace. It
    cannot invoke the backend, decode bytes, settle work, or mutate generation.
20. Prefetch, read-ahead, and write-behind use distinct pool grants and
    counters. Effectful read misses and writebacks lower through the existing
    physical work declaration, Signal readiness, scheduler demand, executor
    command, and Store settlement. Hits and pre-effect denials do not create
    fake work.
21. Speculative denial is a normal typed outcome. It may not revoke serving
    health or silently fall back to unaccounted work.
22. Pressure failures expose basis, Store generation, requested/admitted
    dimensions, denial cause, and retry posture through Store-owned evidence.
    They do not expose `PhysicalResidencyPool`, mutable counters, eviction
    candidates, or semantic cache membership.
23. `C6PhysicalWorkHandoff`, every `C6*` production type, the
    `c6_handoff/` module, and `c6_*` production test/function names are
    temporary C.5.1 scaffolding. They are replaced by responsibility-named
    surfaces with no aliases or deprecation bridge.
24. `legacy-s2-models`, `legacy-certification-models`,
    `S2PhysicalResidencyEntry`, `ResidentFrameTable`, snapshot-derived
    admission, legacy zero/bounded-copy record views, and their feature/dependency
    graph are deleted after consumers move to the canonical lease contract.
25. Old certification fixtures are either rewritten to drive the production
    Store boundary or deleted. They cannot be kept as “algorithm tests” when
    their authority model contradicts production.
26. C.6 creates no empty C.7-C.11 modules. It leaves named insertion points in
    the directory plan and creates each directory only when its first real
    responsibility lands.
27. Cleanup is part of the phase that makes a predecessor obsolete and part of
    the final closeout gate. “Follow-up cleanup” is not an accepted phase
    outcome.

## Normative API Contract

The names and visibility in this section are architectural decisions. An
implementation plan may refine private mechanics, but it may not rename these
concepts, merge their authorities, expose a lower owner, or add an owning
whole-record shortcut without first revising this specification.

### Configuration admission

Raw configuration and admitted configuration are different types:

```rust
pub struct PhysicalRecordResidencyPolicy;
pub struct PhysicalRecordResidencyPolicyBuilder;
pub struct AdmittedPhysicalRecordResidencyPolicy;
pub enum PhysicalRecordResidencyPolicyDenial;

pub type PhysicalRecordResidencyPolicyOutcome =
    worth_proof::DenialTransitionOutcome<
        AdmittedPhysicalRecordResidencyPolicy,
        PhysicalRecordResidencyPolicyDenial,
    >;
```

The declaration API is:

```rust
impl PhysicalRecordResidencyPolicy {
    pub fn builder() -> PhysicalRecordResidencyPolicyBuilder;
}

impl PhysicalRecordResidencyPolicyBuilder {
    pub fn total_bytes(self, bytes: NonZeroU64) -> Self;
    pub fn resident_bytes(self, bytes: NonZeroU64) -> Self;
    pub fn metadata_bytes(self, bytes: NonZeroU64) -> Self;
    pub fn frame_entries(self, entries: NonZeroU32) -> Self;
    pub fn pinned_frames(self, frames: NonZeroU32) -> Self;
    pub fn pin_leases(self, leases: NonZeroU32) -> Self;
    pub fn dirty_frames(self, frames: NonZeroU32) -> Self;
    pub fn dirty_replacement_bytes(self, bytes: NonZeroU64) -> Self;
    pub fn operation_bytes(self, bytes: NonZeroU64) -> Self;
    pub fn scope_bytes(
        self,
        scope: PhysicalOperationAllocationScope,
        bytes: NonZeroU64,
    ) -> Self;
    pub fn speculative_frames(
        self,
        kind: PhysicalSpeculativeWorkKind,
        frames: NonZeroU32,
    ) -> Self;
    pub fn admit(
        self,
        format: AdmittedPhysicalRecordFormat,
    ) -> PhysicalRecordResidencyPolicyOutcome;
}
```

`PhysicalRecordInitialization::with_residency_policy` and
`PhysicalRecordOpen::with_residency_policy` accept only
`AdmittedPhysicalRecordResidencyPolicy`. Default construction must build a
declaration and pass through the same admission; it is not a privileged bypass.
The admitted type proves internal consistency and page-size sufficiency. It
does not prove that runtime allocation will succeed or authorize media work.

`Option` is not an acceptable configuration outcome. Every invalid
relationship—category greater than total, scope greater than operation
envelope, page larger than resident/operation allowance, dirty replacement
larger than its envelope, or zero/missing required dimension—has a distinct
`PhysicalRecordResidencyPolicyDenial`.

### Ordinary product API

The only ordinary entry into resident record bytes remains the Store facade:

```rust
impl ServingPhysicalRuntime {
    pub fn records(&self) -> PhysicalRecordReader;
    pub fn record_submission(&self) -> PhysicalRecordSubmission;
    pub fn residency_observation(&self) -> PhysicalResidencyObservation;
}

impl PhysicalRecordReader {
    pub fn open(
        &self,
        record: PhysicalRecordId,
        limits: RecordReadLimits,
    ) -> Result<RecordReadSession, RecordReadError>;

    pub fn open_external(
        &self,
        locator: ExternalPhysicalRecordLocator,
        limits: RecordReadLimits,
    ) -> Result<RecordReadSession, RecordReadError>;
}

impl RecordReadSession {
    pub fn next_chunk(
        &mut self,
    ) -> Result<Option<PhysicalRecordChunkView<'_>>, RecordStreamFailure>;

    pub fn read_next(
        &mut self,
        target: &mut [u8],
    ) -> Result<usize, RecordStreamFailure>;

    pub fn observation(&self) -> RecordReadObservation;
}
```

`RecordReadSession` is a logical record-read lease. It owns the operation
allocation, lifecycle permit, cursor, and at most one current frame lease. It
is not a byte container. `OpenedPhysicalRecord` is deleted rather than retained
as an alias because two names for the same authority obscure the public model.

`PhysicalRecordChunkView<'session>` deliberately says **chunk**, not record:
an inline record produces one chunk, while an extent produces one chunk per
resident frame. Its API is:

```rust
impl<'session> PhysicalRecordChunkView<'session> {
    pub fn bytes(&self) -> &'session [u8];
    pub fn basis(&self) -> PhysicalRecordChunkBasis;
    pub fn logical_range(&self) -> Range<u64>;
}

impl PhysicalRecordChunkBasis {
    pub fn store_identity(self) -> StableStoreIdentity;
    pub fn store_generation(self) -> LifecycleGeneration;
    pub fn record(self) -> PhysicalRecordId;
    pub fn physical_owner(self) -> PhysicalGenerationOwner;
    pub fn frame_coordinate(self) -> RecordFrameCoordinate;
}
```

The view borrows the session mutably used to advance the stream. Therefore the
caller cannot retain one chunk while advancing to or evicting its successor.
`bytes()` returns only the payload range admitted by physical-format decode;
headers, neighboring slots, and unvalidated bytes are not exposed.
`PhysicalRecordChunkBasis` has no public constructor and is observation only.
Its `physical_owner` is minted from the exact durable inline slot or top-level
record-extent generation cell that produced the chunk; callers cannot pair a
different owner with the basis. Its coordinate, owners, and generations cannot
create a session, lease, fault, writeback, retry, or semantic-residency claim.

The isolation adapter derives its reference from that Store-minted owner:

```rust
impl CurrentGenerationPhysicalReference {
    pub fn for_record_chunk(
        chunk: &PhysicalRecordChunkView<'_>,
    ) -> CurrentGenerationPhysicalReference;
}

impl PhysicalByteGuardScope {
    pub fn for_record_chunk(
        chunk: &PhysicalRecordChunkView<'_>,
    ) -> PhysicalByteGuardScope;
}
```

There is deliberately no overload accepting a separately selected physical
reference. A caller cannot pair reference A with Store chunk B and ask later
runtime validation to notice the substitution.

`read_next` is the explicit copy API. It copies only into caller-owned storage,
never allocates a result, and advances the same cursor as `next_chunk`.
Interleaving the two methods is legal and preserves one monotonic logical
offset. No `read_to_end`, `to_vec`, `into_bytes`, `Arc<[u8]>`, or equivalent
whole-record convenience is provided by C.6.

Ordinary writes continue through `PhysicalRecordSubmission` and its existing
prepared append/publication progression. C.6 adds no public `mark_dirty`,
`claim_writeback`, `execute_writeback`, executor-command, or retry-binding
method. Dirty and writeback APIs are Store-private because callers own record
intent, not frame lifecycle or media effects.

### Pressure and observation API

Pressure is returned as evidence, never as pool authority:

```rust
pub struct PhysicalRecordPressureEvidence;

pub enum RecordReadDenial {
    // existing non-pressure variants remain
    PhysicalPressure,
}

pub enum RecordAppendDenial {
    // existing non-pressure variants remain
    PhysicalPressure,
}

pub enum PhysicalResidencyRetryPosture {
    AfterLeaseRelease,
    AfterAllocationRelease,
    AfterWritebackSettlement,
    AfterGenerationReadmission,
    AfterConfigurationChange,
    Terminal,
}

impl RecordReadError {
    pub fn pressure(&self) -> Option<PhysicalRecordPressureEvidence>;
}

impl RecordAppendError {
    pub fn pressure(&self) -> Option<PhysicalRecordPressureEvidence>;
}

impl PhysicalRecordPressureEvidence {
    pub fn basis(&self) -> PhysicalRecordPressureBasis;
    pub fn store_generation(&self) -> LifecycleGeneration;
    pub fn scope(&self) -> PhysicalOperationAllocationScope;
    pub fn dimension(&self) -> PhysicalResidencyDimension;
    pub fn requested(&self) -> u64;
    pub fn admitted(&self) -> u64;
    pub fn limit(&self) -> u64;
    pub fn retry_posture(&self) -> PhysicalResidencyRetryPosture;
    pub fn effect_may_have_started(&self) -> bool;
}

impl PhysicalResidencyRetryPosture {
    pub fn may_retry(self) -> bool;
}

impl PhysicalRecordPressureBasis {
    pub fn store_identity(&self) -> StableStoreIdentity;
    pub fn record(&self) -> Option<PhysicalRecordId>;
    pub fn frame_coordinate(&self) -> Option<RecordFrameCoordinate>;
    pub fn work_identity(&self) -> Option<PhysicalWorkIdentity>;
}
```

`PhysicalRecordPressureBasis` identifies the record, locator, or physical work
scope that was actually admitted far enough to exist. Read and append errors
lower the same physical pressure vocabulary rather than exposing pool denials.
It cannot be constructed from a record id alone and cannot be used to retry.
Retry consumes the original Store request or a Store-issued retry capability,
never the evidence object.

The current public
`ResidencyUnavailable(worth_store_buffer_pool::PhysicalResidencyDenial)`
variants are removed. A product caller learns physical dimension and retry
posture through Store vocabulary; it does not pattern-match the lower pool's
state machine.

`PhysicalResidencyObservation` is Store-owned and read-only. It carries stable
Store identity, lifecycle generation, admitted limits, and an exact counter
snapshot. It exposes no pool reference, drain method, eviction candidate,
mutable counter, allocation grant, frame lease, or writeback claim.

The current naked methods
`ServingPhysicalRuntime::{residency_counters, drain_clean_residency,
physical_residency_writeback_command,
bind_physical_residency_writeback_retry}` are not ordinary product APIs:

- counters are replaced by `residency_observation`;
- clean draining moves behind a Store-private or explicitly admitted
  Maintenance-scoped capability;
- command construction and retry binding become Store-private work-runtime
  operations.

They are deleted from the public facade without aliases.

No ordinary record-I/O API accepts or returns a Signal request handle, Signal
aspect/mask, `PhysicalWorkSemanticBasis`, Foundational fact/patch, scheduler
grant, backend capability/receipt, pool lease, dirty frame, or `worth-proof`
raw transition packet. The Store facade consumes those internally and returns
record/session/outcome vocabulary at the caller's altitude.

### Store-private composition API

`worth-store` composes lower owners through one private
`PhysicalResidencyWorkPort`. It replaces the residency portion of
`C6PhysicalWorkHandoff`; it is never re-exported from `physical_runtime`.

```rust
pub(in crate::physical_runtime) struct PhysicalResidencyWorkPort;

impl PhysicalResidencyWorkPort {
    fn load(
        &self,
        request: AdmittedPhysicalFrameFault,
    ) -> Result<LoadedPhysicalFrame, PhysicalFrameLoadFailure>;

    fn publish_candidate(
        &self,
        candidate: AdmittedPhysicalFrameCandidate,
    ) -> Result<DirtyPhysicalFrame, PhysicalDirtyTransitionFailure>;

    fn prepare_writeback(
        &self,
        dirty: DirtyPhysicalFrame,
    ) -> Result<PreparedPhysicalWriteback, PhysicalWritebackPreparationFailure>;
}
```

The signatures express three distinct authorities:

- `AdmittedPhysicalFrameFault` proves current Store generation, exact
  coordinate, read semantic basis, security scope, operation allocation, and
  fault ownership;
- `AdmittedPhysicalFrameCandidate` proves current generation, exact coordinate,
  fully admitted replacement bytes, mutation scope, and exclusive/copy-on-write
  posture;
- `DirtyPhysicalFrame` proves bytes are resident, dirty, not evictable, and
  owned by exactly one pool transition.

`PreparedPhysicalWriteback` contains the Store work intent and dirty claim
needed to request Signal readiness. It does not contain a scheduler grant,
backend capability, or settlement authority. Each later progression consumes
the predecessor type already owned by the C.5.1 runtime.

The lower `worth-store-buffer-pool` facade may expose concrete pool types to
`worth-store` because they are separate crates, but workspace dependency gates
must prevent Part II and unrelated product crates from importing that facade.
The pool API accepts/returns concrete leases, grants, candidates, dirty frames,
and claims; it never accepts a generic authority bound, raw Signal handle,
Foundational aspect, Store semantic basis, or scheduler/backend receipt.

## Dependency Semantics: Signal, Proof, And Foundational

C.6 uses the inherited dependencies at exact boundaries. It does not spread
them through the buffer pool.

| C.6 action | Worth Signal | `worth-proof` | `worth-foundational` / aspect-native |
| --- | --- | --- | --- |
| Residency-policy declaration/admission | Not used | `DenialTransitionOutcome` distinguishes admitted policy from typed denial | Not used; byte limits are physical policy, not aspect meaning |
| Pool construction and close | Not used directly; Store lifecycle already owns Signal construction/disposal separately | Existing Store `ProofOutcome` preserves success/denied/stale/rebind/inspection categories | Not used |
| Resident hit and pin | Not used; no asynchronous/effectful work exists | Concrete `PhysicalFrameLease` is the proof-bearing result; no generic marker | Not used |
| Borrowed chunk view or explicit copy | Not used | Rust lifetime plus concrete lease authority; no new outcome wrapper | Physical-format decode determines the exposed payload; no aspect value crosses the API |
| Cold fault or effectful read-ahead/prefetch miss | Existing `PhysicalWorkSignalFamily::ReadFault` derives dependency readiness and owns cancellation/retry/timeout request lifecycle | Existing submission/admission/readiness/resource-admission/dispatched/settled types; `TransitionOutcome` preserves typed submission categories | Exact Store-native projection basis for root, artifact, frame, or scan, bridged through `worth-store-aspect-native` into `PhysicalWorkSemanticBasis` |
| Prefetch/read-ahead hit or pressure denial | Not used; no fake Signal request is emitted | Concrete hit/grant/denial types | Reuses no new aspect; speculative posture is resource policy, not semantic meaning |
| Clean-to-dirty transition | Not used; this is pool state, not dependency readiness | Concrete candidate and dirty typestates | Not used; dirty is physical lifecycle truth, not an aspect value |
| Effectful dirty writeback/write-behind | Existing `PhysicalWorkSignalFamily::ExactWriteback` derives readiness and owns generic request lifecycle | Existing move-owned work progression and exact settlement outcome | Dedicated Store-native frame-writeback mutation basis, bridged through aspect-native |
| Existing record publication | Existing `PhysicalWorkSignalFamily::Publication` | Existing publication progression | Existing publication-stage mutation bases |
| Pool pressure/counters | Not used and never encoded as Signal aspects | Evidence is explicitly non-authoritative | Not used and never encoded as authoritative aspect state |
| Store shutdown | Existing `PhysicalWorkSignalFamily::Lifecycle` and inherited disposal join only | Existing close/abort `ProofOutcome` and typed terminal results | No new C.6 aspect |

### Worth Signal law

C.6 adds no Signal runtime and no C.6-specific Signal family. It reuses the
four installed C.5.1 families:

- `ReadFault` for a real cold range/metadata read, including an effectful
  prefetch or read-ahead miss;
- `ExactWriteback` for an admitted dirty-frame media write;
- `Publication` for the existing record-publication stages;
- `Lifecycle` for the inherited work-runtime shutdown join.

Signal is not called for a hit, pin, unpin, lease borrow, copy, decode, dirty
mark, victim selection, eviction, counter read, or pre-effect pool denial.
Those operations have no dependency-readiness question for Signal to answer.
Creating Signal requests for them would make derived async state masquerade as
physical work and would corrupt hit/media accounting.

On a miss, the Store first owns a bounded fault request and exact physical work
intent. Signal consumes the admitted work, evaluates the installed
aspect/partition dependency, and yields `ReadyPhysicalWork` or
`BlockedPhysicalWork`. Signal does not allocate the frame, choose a victim,
perform I/O, admit scheduler capacity, install bytes, clean a frame, or settle
effect fate.

### `worth-proof` law

`worth-proof` is used for governed outcome categories and consuming
progression, not as decorative vocabulary:

- configuration admission returns an admitted policy through
  `DenialTransitionOutcome`;
- initialize/open/close/abort retain the inherited `ProofOutcome` categories;
- submission retains `TransitionOutcome` so denied, deferred, stale, rebind,
  and failed states cannot collapse into `Result`;
- the physical work path continues to consume concrete
  `AdmittedPhysicalWork`, `ReadyPhysicalWork`,
  `ResourceAdmittedPhysicalWork`, `DispatchedPhysicalWork`, and
  `SettledPhysicalWork`;
- pool transitions use concrete move-owned lease/grant/candidate/dirty/claim
  types where they are stronger than wrapping a local state change in a generic
  proof container.

An id, coordinate, generation, digest, counter, observation, or
`AuthorityMarker` bound opens no governed path. C.6 does not add a direct
`worth-proof` dependency to `worth-store-buffer-pool`; Store owns the governed
cross-owner outcomes, while the pool owns concrete physical typestate.

### `worth-foundational` and aspect-native law

Foundational defines admitted Store-native meaning for physical work; it does
not model cache contents.

C.6 preserves the existing distinct projection bases:

- `store.physical.record.root-read-basis`;
- `store.physical.record.artifact-read-basis`;
- `store.physical.record.frame-read-basis`;
- `store.physical.record.scan-read-basis`.

Prefetch and read-ahead reuse the basis of the exact physical read they perform.
They do not introduce “prefetch” or “cache” aspects because speculation changes
resource posture, not the meaning of the bytes.

C.6 splits exact frame writeback from the current broad publication binding.
It installs a dedicated
`store.physical.record.frame-writeback-basis` contract/fact/patch for
`PhysicalWorkSignalFamily::ExactWriteback`, admitted with exact projection and
mutation masks and `PhysicalSignalAspectRole::DependencyAndOutput`. The
Store-owned mutation patch becomes the `PhysicalWorkSemanticBasis` carried by
the writeback intent. The existing
`store.physical.record.publication-basis` remains bound only to
`PhysicalWorkSignalFamily::Publication`. This prevents a publication-stage
patch from authorizing arbitrary dirty-frame writeback or a writeback receipt
from claiming record publication.

The frame-writeback Signal aspect is still derived orchestration state. It
does not say that a frame is dirty or clean; only `DirtyPhysicalFrame` and the
pool's exact receipt-consuming transition own that truth. Destroying the
Signal graph and reconstructing the Store must not reconstruct dirty state from
the aspect.

`worth-store-aspect-native` remains the only bridge from Foundational admitted
facts/patches into `PhysicalWorkSemanticBasis`. Raw `AspectValue`,
`AspectMask`, Foundational collections, Signal aspect slots, JSON, strings, or
maps never enter the pool API, record-view API, scheduler demand, or backend
command. Resident, pinned, dirty, evictable, pressured, prefetched, and hot are
never authoritative Foundational aspects.

## Authority Ownership

| Type | Constructed only by | Proves | Authorizes | Explicitly cannot authorize | Consumed by |
| --- | --- | --- | --- | --- | --- |
| `AdmittedPhysicalRecordResidencyPolicy` | Store configuration admission | all limits are nonzero, mutually consistent, and sufficient for the admitted format | construction of one pool for the matching Store admission | allocation, media work, retry, semantic residency | physical Store instance construction |
| `OperationAllocationGrant` | buffer-pool pressure admission | exact bytes and one physical scope are charged inside category, scope, and total envelopes | the named bounded allocation only | frame residency, I/O, another scope, retry | loader, candidate builder, or bounded scratch owner |
| `AdmittedPhysicalFrameFault` | Store-private residency work port | current Store generation, exact coordinate, read basis, security, operation allocation, and single fault ownership are joined | submission of one canonical read-fault intent | a hit, arbitrary coordinate read, backend access, settlement | canonical frame loader |
| `LoadedPhysicalFrame` | exact settled read plus pool admission, or a real pool hit | one validated frame identity is resident and leased; attached work trace says whether a fault occurred | physical-format decode and Store-private session projection | mutation, publication, semantic residency, ownership beyond its lease | record decoder / `RecordReadSession` |
| `PhysicalFrameLease` | buffer pool | one frame is pinned in the current pool identity | immutable frame borrow and explicit bounded copy | eviction, dirty transition without candidate admission, media work | Store-private loaded frame |
| `RecordReadSession` | `PhysicalRecordReader` | caller/read/access limits, lifecycle permit, operation allocation, logical cursor, and current frame lease are joined | `next_chunk`, `read_next`, observation | pool control, whole-record ownership, use after Store generation loss | ordinary caller or physical adapter |
| `PhysicalRecordChunkView<'session>` | mutable borrow of `RecordReadSession` after successful decode | exposed bytes are the validated payload for one logical range and one physical basis | read-only use for the borrow lifetime | advancing/dropping the session, mutation, retaining bytes, semantic cache claims | caller or future integrity/isolation adapter |
| `AdmittedPhysicalFrameCandidate` | Store-private candidate admission | exact replacement bytes, generation, coordinate, scope, and immutable-view compatibility are admitted | one clean-to-dirty transition | unbounded allocation, writeback, publication | buffer-pool dirty transition |
| `DirtyPhysicalFrame` | buffer pool consuming an admitted candidate or admitted mutation transition | exact resident bytes are dirty and non-evictable under one frame identity | prepare one writeback claim or retain dirty state | clean state, eviction, backend write, publication | Store-private writeback preparation |
| `PreparedPhysicalWriteback` | `PhysicalResidencyWorkPort` consuming dirty authority | exact dirty claim, Store work intent, writeback basis, security, and generation are joined | request `ExactWriteback` Signal readiness | scheduler admission, backend write, clean transition | inherited C.5.1 readiness progression |
| `ReadyPhysicalWork` | private Signal owner consuming admitted work | the installed dependency is ready for this request/attempt | scheduler demand construction | memory grant, media effect, settlement | I/O scheduler admission |
| `ResourceAdmittedPhysicalWork` | physical scheduler consuming ready work and exact demand | required queue/I/O resources are admitted for the bound work | executor command construction | changing scope/bytes/basis, final settlement | physical executor |
| `DispatchedPhysicalWork` | executor dispatch | media effect may have started under exact backend binding | exact receipt matching and Store settlement | retry erasure, frame clean transition by itself | Store settlement |
| `SettledPhysicalWork` | Store settlement consuming backend outcome | exact terminal effect fate and recovery disposition | clean transition when exact success matches, or typed retry/inspection progression | pretending an indeterminate effect was clean/no-effect | residency writeback outcome |
| `PhysicalRecordPressureEvidence` | Store lowering a real denial | what was denied, against which admitted basis/limit, and the retry posture | observation and caller policy selection only | retry, allocation, eviction, generation readmission | caller / adapter |
| `PhysicalRecordResidencyFailureReason` | exhaustive Store translation of one lower denial | the exact causal failure and small actionable parameters, independently of its broad policy class | caller classification only | pool mutation, retry, proof, scheduling, backend access, semantic residency | caller / adapter |
| `PhysicalResidencyObservation` | Store observation facade | identity-bound snapshot of admitted limits and executed counters | diagnostics only | any state transition or effect | caller / evidence projector |

Every row is a compile-time or sealed-constructor requirement. If
implementation replaces a row with an id lookup, boolean flag, generic marker,
counter threshold, raw coordinate, or public constructor, it has changed the
architecture and fails C.6.

## Type And Work Progression

Ordinary fault progression is:

```text
record access intent
  -> admitted Store generation + caller limits + operation scope
  -> pool operation-allocation grant
  -> hit: proof-bearing frame lease
  -> miss: loading identity + physical read work declaration
  -> Signal-derived readiness
  -> scheduler resource grant
  -> executor media read
  -> Store settlement
  -> exact frame admission
  -> proof-bearing frame lease
  -> lease-scoped decoded record chunk view
```

Dirty/writeback progression is:

```text
admitted mutation allocation + current frame lease
  -> candidate frame authority
  -> dirty frame authority
  -> writeback speculative grant + Store work intent
  -> Signal-derived readiness
  -> scheduler memory/I/O reservation
  -> executor media write
  -> exact backend receipt
  -> Store effect settlement
  -> clean frame authority | retryable dirty authority | inspection obligation
```

No arrow may be implemented as a boolean, counter comparison, coordinate
reconstruction, callback, or lookup in a parallel registry.

## Intentional Developer Experience

The intended ordinary read experience is:

```rust
let records = runtime.records();
let mut session = records.open(record_id, RecordReadLimits::new(maximum_payload))?;

while let Some(chunk) = session.next_chunk()? {
    consume(
        chunk.bytes(),
        chunk.logical_range(),
        chunk.basis(),
    )?;
}

let observation = session.observation();
```

The explicitly bounded-copy path remains available without constructing an
owning whole-record result:

```rust
let mut session = runtime.records().open(record_id, limits)?;
let mut scratch = [0_u8; 16 * 1024];

loop {
    let copied = session.read_next(&mut scratch)?;
    if copied == 0 {
        break;
    }
    consume_copy(&scratch[..copied])?;
}
```

A borrowing view must make this shape fail to compile:

```compile_fail
let escaped = {
    let mut session = runtime.records().open(record_id, limits)?;
    session.next_chunk()?.unwrap()
};
consume(escaped.bytes());
```

Pressure handling is Store-facing and retry-honest:

```rust
match runtime.records().open(record_id, limits) {
    Ok(session) => consume_record(session),
    Err(error) => match error.pressure() {
        Some(pressure) if pressure.retry_posture().may_retry() => {
            defer(pressure.basis(), pressure.store_generation())
        }
        _ => fail(error),
    },
}
```

These examples fix the names, shape, authority, and lifetime of the public API.
Changing them is an architectural revision to this specification, not an
implementation-detail substitution.

## Required Destination Directory And Module Plan

The following is the committed destination, not an instruction to create empty
files. `[C.6]` entries exist by C.6 closeout. Successor entries identify where
already-committed growth belongs but are not created until that milestone has
real code.

```text
workspaces/worth-store/crates/
├── worth-store-buffer-pool/
│   └── src/
│       ├── lib.rs
│       └── physical_residency/
│           ├── mod.rs                         # narrow crate facade only
│           ├── limits.rs                      # admitted hard dimensions
│           ├── denial.rs                      # typed residency denials
│           ├── observation.rs                 # exact executed counters
│           ├── pool.rs                        # owner composition only
│           ├── pool/
│           │   ├── public_api.rs
│           │   ├── frame_admission.rs
│           │   ├── pin_lifecycle.rs
│           │   ├── eviction/
│           │   │   ├── mod.rs                    # narrow eviction facade
│           │   │   ├── order.rs                  # deterministic order maintenance
│           │   │   ├── legal_victim.rs           # checked proof issuance only
│           │   │   └── release.rs                # token-consuming exact release
│           │   ├── dirty_transition.rs
│           │   ├── candidate_admission.rs          # batch/frame lifecycle facade
│           │   ├── candidate_admission/
│           │   │   └── declaration.rs             # cardinality, coverage, allocation demand
│           │   ├── operation_accounting.rs
│           │   └── identity_transition.rs
│           ├── lease.rs                       # narrow typestate facade
│           ├── lease/
│           │   ├── candidate.rs               # candidate batch/frame progression
│           │   ├── frame.rs                   # clean frame lease + borrowed view
│           │   ├── dirty.rs                   # dirty frame authority
│           │   └── writeback.rs               # claim and receipt transition
│           ├── pressure/
│           │   ├── mod.rs
│           │   └── admission.rs               # global + per-scope envelopes
│           └── speculation/
│               ├── mod.rs
│               └── admission.rs               # prefetch/read-ahead/write-behind
│
├── worth-store/
│   └── src/physical_runtime/
│       ├── instance/
│       │   ├── residency_owner.rs             # constructs/closes the one pool
│       │   └── signal_owner/                  # inherited private Signal runtime
│       ├── work/
│       │   ├── profile/
│       │   │   └── capability.rs              # inherited four Signal families
│       │   └── progression/                   # admitted -> ready -> settled types
│       └── record_serving/
│           ├── mod.rs
│           ├── work_semantics/
│           │   ├── mod.rs                     # installs exact Store-native bases
│           │   ├── read_basis.rs              # root/artifact/frame/scan projection
│           │   ├── publication_basis.rs       # publication mutation only
│           │   ├── frame_writeback_basis.rs   # ExactWriteback mutation only
│           │   └── security_admission.rs      # authority-bound physical scope
│           ├── access/
│           │   ├── locate/
│           │   │   ├── session.rs             # lease-bearing record session
│           │   │   ├── inline.rs
│           │   │   └── extent.rs
│           │   ├── record_chunk_view.rs       # public borrowed chunk view
│           │   └── read_observation.rs         # copy/work/pressure evidence
│           ├── residency/
│           │   ├── mod.rs                     # Store-private composition facade
│           │   ├── capability.rs              # generation-fenced narrow port
│           │   ├── frame_ports.rs
│           │   ├── failure.rs                 # broad class + exact Store-owned reason
│           │   ├── failure/
│           │   │   └── tests.rs               # causal projection evidence
│           │   ├── certification/                # feature-gated proof access
│           │   │   ├── mod.rs                    # narrow certification facade
│           │   │   ├── probe.rs                  # runtime-bound fault-driving probe
│           │   │   ├── resident_frame.rs         # certification lease projection
│           │   │   └── failure.rs                # stable certification failures
│           │   ├── frame_loading/
│           │   │   ├── bounded_loader.rs
│           │   │   ├── loaded_frame.rs
│           │   │   └── read_source/
│           │   │       ├── canonical.rs
│           │   │       └── direct.rs          # bootstrap-only, mechanically sealed
│           │   ├── dirty/
│           │   │   ├── mod.rs
│           │   │   ├── admitted_frame.rs
│           │   │   ├── writeback.rs
│           │   │   └── outcome.rs
│           │   ├── speculation/
│           │   │   ├── mod.rs
│           │   │   └── work_submission.rs
│           │   └── pressure/
│           │       ├── mod.rs
│           │       └── evidence.rs
│           └── publication/                   # existing record publication owner
│
├── worth-store-wal/                           # C.7 consumes dirty/writeback truth
├── worth-store-recovery-physics/              # C.8 consumes reconstruction ports
├── worth-store-physical-integrity/             # C.9 consumes borrowed physical views
├── worth-store-physical-isolation/             # C.10 consumes leases + pressure evidence
└── worth-store-blob-chunks/                    # C.11 consumes Blob-scoped allocations

workspaces/worth-store/tools/store-test-runner/
└── src/
    ├── arguments.rs                             # direct scenario CLI
    └── bounded_residency_siege/                 # focused process scenario [C.6]
        ├── execution.rs
        ├── schedule.rs
        └── observation.rs
```

This plan intentionally permits a directory to begin with one semantic
implementation file plus its module facade. `pressure/` and `speculation/` are
not collapsed into `limits.rs` or `pool.rs` merely because they begin small:
the roadmap already commits recovery pressure, stable-read isolation, scrub,
maintenance, and blob consumers to those families. Conversely, no empty
`recovery/`, `integrity/`, `isolation/`, or `blob/` subtree is created inside
`worth-store`; their existing owner crates remain the future insertion points.

The listed filenames are responsibility boundaries, not a demand for tiny
files. Existing cohesive files may remain where they already satisfy the
boundary. Moves and splits are required where the current `lease.rs`,
`c6_handoff/`, `record_work_admission.rs`, or another file owns multiple
independently changing semantic responsibilities or approaches the workspace
line cap.

`work_semantics/` is the only C.6 location that constructs Foundational
contracts/facts/patches or lowers them through aspect-native. The private
`instance/signal_owner/` is the only C.6 path that calls Worth Signal.
`physical_residency/` in the buffer-pool crate contains neither dependency.

### Successor Insertion Rules

- C.7 consumes admitted dirty-frame and exact writeback progression at the
  Store composition boundary. WAL policy does not enter the buffer pool.
- C.8 receives explicit Recovery-scoped allocation and reconstruction ports.
  Recovery does not reopen a hidden unlimited pool mode.
- C.9 receives borrowed physical byte views before decode/integrity admission.
  It does not receive the pool, eviction APIs, or frame-table constructors.
- C.10 receives record leases, pressure evidence, and generation/basis
  observations through Store composition. It owns stable-read policy and QoS;
  C.6 does not guess those policies.
- C.11 receives Blob-scoped bounded allocations and streaming frame views. It
  does not regain whole-blob materialization through a blob-specific helper.

## Budget And Pressure Contract

### Hard dimensions

The admitted residency policy names at least:

- total live pool-owned bytes;
- resident payload bytes;
- metadata bytes;
- frame-entry count;
- pinned frame identities;
- simultaneous pin leases;
- dirty frame count and dirty replacement bytes;
- active operation-allocation bytes;
- prefetch frames;
- read-ahead frames;
- write-behind frames;
- per-operation-scope byte ceilings.

The implementation may split a dimension further when independent exhaustion
is possible. It may not merge dimensions merely because one current workload
happens to correlate them.

### Admission order

For every load, copy, candidate, decode scratch allocation, or speculative
operation:

1. validate Store and lifecycle generation;
2. calculate the exact or conservative upper-bound demand;
3. acquire the typed pool and scope grants;
4. allocate;
5. perform work;
6. record executed deltas;
7. release grants on every terminal path.

An allocator failure after a grant is a typed allocation failure and releases
the grant. An allocation that occurs before the grant is a constitutional
failure even if the process remains below its final RSS target.

### Scope law

`ForegroundRead`, `ForegroundWrite`, `Recovery`, `Scrub`, `Maintenance`,
`Verification`, and `Blob` are physical allocation purposes, not scheduling
priorities. Each request carries exactly one scope. Counters reconcile each
scope to the global active/peak operation bytes. C.6 proves that one scope
cannot spend another scope's ceiling and that all scopes together remain
inside the global envelope. Fairness, latency classes, and starvation policy
remain C.10 work.

### Retry posture

Every pressure denial is classified as one of:

- retry after a named lease/allocation/writeback release;
- retry only after Store generation readmission;
- retry only after operator/configuration change;
- terminal for the declared request.

The evidence names the denied dimension, requested amount, current admitted
amount, limit, physical basis, Store generation, operation scope, and whether
any media effect may have started. A bare `BudgetExceeded` without this basis
cannot serve downstream adapters.

## Lease, View, And Eviction Contract

1. A frame identity is `(stable store identity, Store lifecycle generation,
   physical artifact coordinate, frame generation)`.
2. A frame may be `loading`, `clean`, `dirty`, or `writeback-claimed`; these are
   typed states, not overlapping flags that allow impossible combinations.
3. Pin leases are counted separately from pinned frame identities. Two readers
   of one frame consume two lease grants but one pinned-frame identity.
4. A loading frame reserves its entry, metadata, and maximum resident bytes
   before media work. Concurrent identical faults attach to that loading
   authority.
5. A frame is evictable only when clean, fully loaded, unpinned, unclaimed, and
   not the subject of active candidate publication.
6. Eviction releases resident and metadata authority atomically with removal
   from the identity index. No stale lookup can recover the released bytes.
7. `PhysicalRecordChunkView<'session>` borrows the session holding the
   underlying frame lease. It has no constructor from raw bytes, coordinates,
   or a counter snapshot.
8. Advancing an extent session requires mutable access to the session and is
   therefore impossible while a prior chunk view is borrowed.
9. Closing or dropping a session releases all frame, pin, operation, and
   lifecycle grants exactly once.
10. Store close stops new leases, waits for or classifies outstanding work, and
    reports leaked pins/dirty/writeback state as an inspection requirement
    rather than silently discarding it.

## Fault, Hit, And Decode Contract

- A hit increments hit and lease counters but causes zero source loads, zero
  executor media commands, and zero synthetic read-work settlements.
- A miss increments one fault identity. A successfully coalesced waiter does
  not increment source loads.
- Every real source load is backed by one admitted physical read operation and
  one terminal Store settlement.
- Decode receives a borrowed frame view and a physical format declaration. It
  cannot keep the bytes, mutate pool state, or perform media I/O.
- Projection or integrity failure rejects the current frame use with exact
  evidence. It does not turn corrupt bytes into a cache miss and retry forever.
- A refault after eviction returns the same independently expected record bytes
  while producing a new source load and a new valid lease.

## Dirty And Writeback Contract

- Publication allocates candidate bytes under `ForegroundWrite` before the
  candidate exists.
- A mutation of an existing frame either holds exclusive admitted mutation
  authority or creates a separately admitted copy-on-write candidate. It never
  mutates bytes visible through an immutable view.
- Dirty admission checks Store generation, frame identity, candidate size,
  dirty-frame limit, live bytes, and publication state.
- Dirty order and durability order are not synonymous. C.7 will add WAL and
  checkpoint ordering; C.6 preserves the seam explicitly.
- Write-behind is speculative work and may be denied without losing dirty
  authority.
- Once dispatched, writeback remains an effect obligation even if the original
  consumer cancels or shutdown begins.
- Exact backend receipt mismatch, partial effect, or indeterminate effect does
  not clean the frame. The Store returns retryable dirty authority only when no
  unsafe ambiguity remains; otherwise it revokes the affected serving boundary
  for inspection.
- An evicted dirty frame, a clean transition without settlement, and a second
  simultaneous writeback claim are mechanically impossible.

## Speculative Work Contract

Prefetch, read-ahead, and write-behind are distinct work kinds with distinct
grants and exact attempt/admission/completion/denial counters. They share the
ordinary C.5.1 runtime only when an effect is required:

```text
typed prefetch/read-ahead intent
  -> scope + kind grant
  -> pool probe
     -> hit: consume/release grant, exact completion, no Signal request
     -> miss: physical read-fault declaration
              -> ReadFault Signal readiness
              -> scheduler admission
              -> executor media read
              -> Store settlement
              -> pool admission

typed write-behind intent over DirtyPhysicalFrame
  -> scope + kind grant
  -> PreparedPhysicalWriteback
  -> ExactWriteback Signal readiness
  -> scheduler admission
  -> executor media write
  -> Store settlement
  -> clean | retryable dirty | inspection-required
```

There is no buffer-pool worker, timer, pending map, callback list, or local
retry loop. A speculative hit may settle without media only when it consumes
real hit authority; it emits no Signal request and cannot emit fake source work
to make counters move.
Prefetch/read-ahead may be dropped on pressure. Write-behind denial preserves
dirty truth and exposes retry posture.

## Exact Observation Contract

Executed observation must reconcile at least:

- current and peak total admitted bytes;
- current and peak resident, metadata, operation, and dirty replacement bytes;
- current and peak frame entries, pinned frame identities, pin leases, dirty
  frames, candidate frames, active loads, and writeback claims;
- per-scope active and peak operation bytes;
- per-kind speculative attempts, admissions, completions, denials, active, and
  peak grants;
- faults, coalesced waiters, source loads, hits, evictions, and eviction
  candidate inspections;
- explicit copy operations, copied bytes, and maximum copy width;
- dirty transitions, candidate publications, writeback attempts, exact
  writeback receipts, retryable writebacks, and indeterminate writebacks;
- denials by dimension and retry posture;
- identity transitions, generation rejections, administrative drains, and
  shutdown residue.

Counters are observations of executed transitions. They cannot grant a lease,
prove an effect, settle work, or substitute for independent allocation/media
instrumentation.

## Cleanup And Cutover Contract

Cleanup is an architectural deliverable. Git history, current callers, Cargo
metadata, and the surviving dependency graph are the record of cutover. A
replacement is not complete while its predecessor still compiles; no separate
removal ledger is maintained.

Implement the cutover in the owning source:

- move responsibilities still needed by ordinary callers into the
  responsibility-named residency, access, work, and writeback owners;
- migrate every current caller in the same change, then delete superseded
  C.6/S.2 handoff types, directories, constructors, aliases, feature edges,
  fixtures, and dependencies;
- keep `RecordReadSession` as the single public logical lease and expose
  Store-owned pressure and observation types rather than buffer-pool internals;
- keep publication and exact frame writeback on distinct admitted bases; and
- retain a test only when it directly proves continuing behavior through the
  canonical Store composition boundary.

At closeout, repository-wide source and manifest checks must prove:

- no production identifier or test name begins with `C6` or `c6_`;
- no `c6_handoff` path exists;
- no `legacy-s2-models` or `legacy-certification-models` feature exists;
- none of the deleted S.2 authority types remains;
- no ordinary route constructs `DirectFrameReadSource`;
- no pool-local executor, scheduler, timer, callback registry, queue, or pending
  map exists;
- `worth-store-buffer-pool` has no direct `worth-signal`,
  `worth-foundational`, `worth-proof`, or aspect-native dependency, import, or
  API type; lower physical-owner dependencies may retain their own governed
  transitive proof dependencies without granting that authority to the pool;
- `ExactWriteback` is absent from the publication aspect family and is present
  only on the dedicated frame-writeback basis;
- no public record-read convenience owns an unbounded whole record;
- every remaining direct buffer-pool consumer is an explicitly admitted
  physical owner boundary, not Part II or a test-support shortcut.

Text search is necessary but not sufficient. Dependency graph checks,
compile-fail tests, ordinary feature-graph inspection, and direct production-
path scenarios must prove the absence claims.

## Documentation Deliverables

Documentation is required because C.6 changes the ordinary read experience,
configuration model, pressure handling, and successor integration contract.
The phase that stabilizes each public surface writes or revises:

1. `_docs/worth-store/bounded-physical-record-access.md`
   - explain admitted residency policy construction, `RecordReadSession`,
     `PhysicalRecordChunkView`, bounded copies, extent iteration, pressure
     evidence, Store-owned observation, and retry posture;
   - include working read and error-handling examples;
   - state plainly that physical residency is not Query/MVCC residency.
2. `workspaces/worth-store/crates/worth-store-buffer-pool/README.md`
   - state the crate's owner boundary and forbidden authorities;
   - document admitted dimensions and exact counter semantics;
   - state that Signal, `worth-proof`, and Foundational remain outside the pool
     and link to the Store composition contract;
   - point callers to the Store facade rather than teaching direct pool use.
3. `_docs/worth-store/storage-foundation-s2.md`
   - mark the isolated S.2 design and its authority types as historical and
     superseded by C.6;
   - preserve useful historical context without presenting deleted APIs as the
     current implementation target.
4. focused C.6 owner and integration tests
   - replace useful fake/model coverage with direct checks of the joined
     production boundary; delete redundant scaffolding.
5. `_docs/worth-store/physical-foundation-reconstruction-roadmap.md`
   - link this engineering spec now;
   - at closeout, record the successor handoff and current verification
     commands.
6. `workspaces/worth-store/tools/store-test-runner/README.md`
   - document the hostile residency scenario and any user-facing seed needed to
     replay a failed focused test;
   - keep the normal command direct and avoid generated reports or mutation
     registries.

The feature-facing guide must be written with the `feature-doc-writer` skill
after the public API is real. Documentation claims are verified against
compiled examples and executed counters; prose may not describe planned APIs
as shipped.

## Non-Fake Acceptance Setup

### Production subject

The production subject is:

- `PhysicalStore::{initialize, open}` admitting
  `PhysicalRecordResidencyPolicy`;
- `ServingPhysicalRuntime::records`;
- `PhysicalRecordReader::open` and external-locator readmission;
- the lease-bearing `RecordReadSession` zero-copy and bounded-copy methods;
- the ordinary record append/publication facade;
- the responsibility-named internal residency/writeback capability replacing
  the temporary C.5.1 handoff;
- `ServingPhysicalRuntime::{close_plan, close, abort_with_evidence}`.

The real call path includes `worth-store`, `worth-store-buffer-pool`,
`worth-store-io-scheduler`, `worth-store-physical-backend`,
`worth-store-physical-format`, the private physical `worth-signal` runtime,
`worth-store-aspect-native`, `worth-foundational`, and `worth-proof` where the
inherited work topology already requires them.

Expected physical artifacts are the C.5 namespace marker, format declaration,
segment/page/extent files, and root manifest required by the chosen record
placements. C.6 adds no durability artifact, replay artifact, persisted cache,
or serialized pool state.

Allowed test-only layers are:

- a deterministic workload/expectation generator that shares only stable
  format declarations;
- process-level allocation instrumentation at the named allocation boundary;
- the existing production storage interposer and declared physical yieldpoints;
- an independent read-only filesystem/format verifier;
- direct scenario observation that does not manufacture runtime authority.

### Initial world

1. A separately identified producer executable starts with an absent root,
   initializes the Store through the production facade, writes deterministic
   mixed inline and multi-frame extent records, closes cleanly, and terminates.
2. Persisted logical payload bytes are at least 32 times the serving process's
   resident-byte budget and at least 16 times its total live pool-owned byte
   envelope.
3. The deterministic workload seed fixes:
   - a hot set that fits in two frames;
   - a cold random set larger than the frame-entry budget;
   - a sequential extent set larger than the resident budget;
   - a pinned set that can saturate both pin dimensions;
   - append payloads that saturate dirty capacity;
   - duplicate-fault coordinates;
   - expected record bytes generated independently from Store decode.
4. The direct scenario fixes the backend profile, format version, page size,
   exact global and per-scope budgets, speculative ceilings, workload seed, and
   schedule seed needed for deterministic execution.
5. No replay artifact, persisted heap image, pool snapshot, decoded expected
   record file, or legacy S.2 fixture exists before serving begins.

### Execution topology

The fresh serving executable:

1. opens only from the store root and admitted configuration;
2. warms the fixed hot set, then proves repeated hits;
3. holds views that saturate the legal pin limits;
4. issues a denied over-pin without releasing those views;
5. starts two concurrent cold reads for the same nonresident coordinate;
6. streams the sequential extent set while cold random reads force eviction;
7. submits admitted and denied prefetch/read-ahead work;
8. appends through the ordinary publication facade until dirty capacity is
   saturated;
9. delays one dispatched writeback at the storage boundary and attempts a
   competing writeback and eviction;
10. exercises every allocation scope with an admitted request and a
    scope-ceiling denial, without claiming C.10 scheduling fairness;
11. cancels one request before dispatch and one after possible dispatch;
12. begins close while the delayed writeback is unresolved, then releases or
    faults it according to the scheduled case;
13. emits the terminal observations asserted by the direct test and terminates.

A second fresh verifier executable receives only the root, stable format
declaration, and workload seed. It does not receive a
runtime, pool, leases, registries, decoded records, or expected-state buffer
from either earlier process.

### Independent observation

- The allocation observer records every admitted pool-owned allocation and
  release with category, scope, requested bytes, actual bytes, process, and
  operation identity when one exists.
- The media observer records exact backend read/write operations and receipts.
- Buffer-pool counters and C.5.1 work receipts are compared against those
  observers; neither certifies itself.
- The verifier enumerates and parses the physical artifacts independently and
  compares decoded records with the seed-derived model.
- Process RSS and OS I/O counters may be included as supporting evidence only.
  They are not the memory or effect oracle.

### Required assertions

The closure lane proves:

1. total and category-specific admitted bytes never exceed their declared
   limits at any sample;
2. each scope remains within its ceiling and all scopes reconcile to the global
   operation total;
3. pin leases and pinned frame identities each reach their exact configured
   ceiling; the next request is denied before allocation;
4. dirty frames and write-behind grants reach, but never exceed, their limits;
5. repeated hot reads produce exact hits, zero additional Signal requests, and
   zero additional media reads;
6. the duplicate cold fault produces one fault identity, one source load, and
   two valid leases;
7. forced eviction never chooses a pinned, dirty, loading, candidate, or
   writeback-claimed frame;
8. a refault after eviction performs one real media read and returns the
   expected bytes;
9. extent streaming keeps live bytes bounded independently of total record
   size;
10. explicit copy counters equal the caller-observed copy operations and bytes;
11. zero-copy views add zero copy events;
12. speculative attempts, admissions, denials, completions, and live grants
    reconcile exactly by kind;
13. a speculative hit and every pre-effect pool denial create zero Signal
    requests, while every effectful speculative miss selects the exact inherited
    Signal family and Foundational basis;
14. every fault and writeback that reaches media has one physical work identity
    and one terminal Store effect fate;
15. the delayed writeback remains dirty until exact receipt settlement;
16. pre-dispatch cancellation reaches media zero times; post-dispatch
    cancellation preserves the Store settlement obligation;
17. stale Store/frame generation authority is rejected at the first consuming
    boundary;
18. close reports no unclassified loading, pin, dirty, speculative, allocation,
    or writeback residue;
19. the second fresh verifier observes the complete expected record set and no
    forbidden artifacts.

### Schedule perturbation and replay

The staged schedule above remains the canonical adversarial proof. It is not
the only lawful interleaving.

The hostile residency scenario may derive a deterministic
`SchedulePerturbationPlan` from a
schedule seed that is distinct from the workload seed. The plan may choose
only among actions that are simultaneously admissible at the existing sealed
certification gates: worker start order, equivalent contender identity, gate
release order, and independent ready-work selection. It may not reorder a
causal prerequisite, weaken a hostile condition, change workload truth,
introduce sleeps or timing jitter, call production scheduling policy, or
create another scheduler.

Direct C.6 tests print the schedule seed when a seeded scenario fails. Supplying
that seed to the focused test replays the same admissible decisions. No mutation
report or courtroom report is required.

CI may run a bounded set of deterministic schedule seeds derived from stable
scenario identity and lane index. A failure prints its exact seed so the same
direct test can be replayed without a report artifact.

### Regression sensitivity

When a real C.6 defect is fixed, add the smallest direct regression that reaches
the causal boundary when that test provides durable value. Mutation catalogs,
source replacement, binary identity reports, and append-only regression
registries are not part of the current verification model.

### Mechanical anti-substitution checks

The milestone adds:

- compile-fail coverage for view escape, forged frame/dirty/writeback authority,
  stale-generation consumption, and access to sealed pool internals;
- dependency and feature-graph checks rejecting legacy S.2 features,
  certification authority, replay authority, and Part II imports from ordinary
  Store graphs;
- dependency checks proving `worth-store-buffer-pool` imports no Signal,
  `worth-proof`, Foundational, or aspect-native crate;
- profile/source checks proving `ExactWriteback` is bound only to the dedicated
  frame-writeback basis and publication is bound only to publication;
- causal checks proving hit, pin, view, copy, eviction, dirty marking, and
  pre-effect pressure denial create no Signal request;
- compile and dependency checks rejecting whole-record owning read helpers,
  direct ordinary media sources, and local residency work machinery;
- runtime call-path evidence binding faults/writebacks to Signal, scheduler,
  executor, backend receipt, and Store settlement;
- allocation-boundary instrumentation that fails unadmitted or misclassified
  allocation;
- fresh-process identity checks that reject live-runtime or cache reuse;
- current call-site review showing former fake/model surfaces are deleted or no
  longer reachable.

### Evidence and rerun

The current verification surface is the focused Cargo tests, the direct hostile
scenario, ordinary feature and dependency checks, and the repository boundary
gates. A failing seeded test prints the seed needed to replay that same direct
test. Runtime diagnostics may expose identities and counters needed to explain
a failure, but no checked-in bundle, digest, source binding, or report is a
completion requirement.

## Phase Plan

### Phase 1: Freeze The Real Boundary And Cutover

**Telos:** inspect the current production path, inherited guarantees,
destination owners, and obsolete graph before moving authority.

**Must ship:**

- review of the live call path from `PhysicalRecordReader::open` and ordinary
  publication through pool, Signal, scheduler, executor, backend, and
  settlement;
- review of current direct media and legacy consumers using Git, Cargo metadata,
  and scoped source search;
- deletion of obsolete C.6 handoff, legacy S.2, and legacy feature edges in the
  phase that removes their last real consumer;
- compile/dependency gates that fail if a second residency pool or local work
  runtime is introduced.

**Preserve:** all C.5.1 ordinary read/write, cancellation, settlement, health,
and shutdown evidence remains green before authority moves.

**Proof:** direct compilation, dependency checks, focused tests, and the current
surviving graph reject a legacy feature edge or local pending map.

**Cleanup:** delete evidence or fixtures already proven redundant by C.5.1;
do not migrate dead scaffolding merely because it exists.

### Phase 2: Admit The Complete Residency And Scope Budget

**Telos:** make every pool-owned live allocation illegal until the physical
Store instance has admitted its exact global, category, kind, and scope
envelopes.

**Must ship:**

- an admitted residency policy with all hard dimensions in the Budget And
  Pressure Contract;
- the exact declaration/builder/admitted-policy API and typed
  `DenialTransitionOutcome` fixed by the Normative API Contract;
- typed per-scope operation ceilings inside one global live-byte envelope;
- preflight validation against physical format/page requirements;
- one pool construction in the Store instance and exhaustive
  initialize/open/abort/close propagation;
- exact current/peak counters and allocation-boundary events for every
  dimension and scope;
- typed denial evidence with requested, current, limit, basis, generation,
  scope, and retry posture.

**Preserve:** format admission owns page shape; scheduler admission remains
separate from memory ownership; scope does not become priority or semantic
authority.

**Proof:** constructor UI tests break every incomplete Store construction
site. Boundary tests attempt every dimension at limit, one past limit, allocator
failure after grant, and cross-scope overspend. Independent allocation events
reconcile to pool counters.

**Cleanup:** remove old scalar/default paths that can construct a pool without
the full admitted policy. Remove budget vocabulary that is not consumed by
execution.

### Phase 3: Complete Fault, Hit, Coalescence, And Eviction

**Telos:** make one generation-fenced pool identity the only truth for ordinary
frame residency.

**Must ship:**

- exact loading identity reservation before media work;
- one-source-load coalescence for concurrent identical faults;
- hit progression with zero fake media work;
- `ReadFault` Signal use only for a real miss, with the exact existing
  root/artifact/frame/scan projection basis and no cache/residency aspect;
- deterministic legal-victim selection and exact eviction release;
- refault through the canonical C.5.1 work path;
- corrupt/projection failure handling that cannot loop as a cache miss;
- Store-private responsibility-named residency capability replacing the read
  portion of the temporary handoff.

**Preserve:** direct source remains bootstrap-only; physical format remains the
decoder; backend remains the only media effect owner.

**Proof:** hot/cold/duplicate/refault tests assert exact media and pool deltas.
Hostile tests make every nominal victim pinned, dirty, loading, candidate, or
claimed and require typed denial before allocation. Focused regressions for
pinned eviction, duplicate source load, and direct-source bypass fail locally.

**Cleanup:** remove duplicate loader branches, fallback source reads, and
temporary handoff read types as their responsibility-named replacements land.

### Phase 4: Install Lease-Scoped Chunk Views And Bounded Copies

**Telos:** let callers and future physical adapters consume real resident bytes
without copying by default, owning a whole record, or acquiring pool authority.

**Must ship:**

- `RecordReadSession` as the lease-bearing public record access object;
- `PhysicalRecordChunkView<'session>` and `PhysicalRecordChunkBasis` with the
  exact methods and visibility in the Normative API Contract;
- view basis, Store/frame generation, logical range, and borrowed bytes;
- zero-copy extent iteration one frame at a time;
- retained `read_next(&mut [u8])` with exact bounded-copy accounting;
- pressure evidence on read denial without exposing pool internals;
- compile-time sealed constructors and lifetime constraints.

**Preserve:** caller maximum-payload policy, access scratch limits, streaming
extent behavior, external-locator readmission, stale-placement checks, and
record observation remain honest.

**Proof:** compile-pass consumers use inline and extent views; compile-fail
consumers attempt to return a view, advance/drop the session while a view is
live, construct a view from bytes, and reach the pool. Runtime tests prove zero
copy for views and exact copy counts/widths for bounded copies on records much
larger than the resident budget.

**Cleanup:** delete legacy record-view/materialization APIs and fixtures once
their useful consumers compile against the canonical borrowed contract. No
owning compatibility conversion is added.

### Phase 5: Join Dirty State And Writeback To Ordinary Publication

**Telos:** make real ordinary writes consume admitted candidate/dirty authority
and become clean only through exact C.5.1 effect settlement.

**Must ship:**

- pre-allocation admission for candidate and copy-on-write bytes;
- exclusive or separately admitted mutation semantics while immutable views
  exist;
- typed clean-to-dirty, dirty-to-claimed, claimed-to-settled, retryable-dirty,
  and inspection-required progression;
- exact backend receipt validation;
- cancellation, retry, timeout, close, and partial/indeterminate-effect
  behavior;
- responsibility-named writeback capability replacing the remaining temporary
  handoff surface.
- a dedicated `store.physical.record.frame-writeback-basis` Foundational
  contract/fact/patch bound only to `ExactWriteback`;
- removal of `ExactWriteback` from the broader
  `store.physical.record.publication-basis` family set.

**Preserve:** C.5 publication/root truth remains distinct from pool
cleanliness; C.7 WAL and checkpoint order are not invented here.

**Proof:** real append/publication drives dirty pressure against real files.
Tests saturate dirty state, delay writeback after dispatch, attempt eviction and
a second claim, retry a safe no-effect outcome, and verify final records after
fresh reopen. Focused premature-clean and bypass-settlement regressions fail at
the dirty transition.

**Cleanup:** delete raw `Vec` replacement paths that allocate before admission,
duplicate writeback helpers, temporary `C6*` dirty/writeback types, and obsolete
tests/selectors in the same phase.

### Phase 6: Lower Speculative Work Through The Canonical Runtime

**Telos:** make prefetch, read-ahead, and write-behind bounded consumers of the
existing physical work topology rather than pool-local background models.

**Must ship:**

- distinct typed intent and grant for each speculative kind;
- exact attempt/admission/completion/denial/live/peak observation;
- scope and total-envelope admission before work or allocation;
- Signal readiness, scheduler demand, executor dispatch, and Store settlement
  for every effectful speculative operation;
- drop/defer posture for denied read speculation and retained dirty authority
  for denied write-behind;
- shutdown classification of every speculative grant and possible effect.
- reuse of `ReadFault` for effectful prefetch/read-ahead misses and
  `ExactWriteback` for effectful write-behind, with no new Signal family.

**Preserve:** C.10 still owns QoS, fairness, and stable-read policy. Signal
still owns generic retry/timeout lifecycle; the pool owns no timer.

**Proof:** each kind reaches its exact limit, is denied one past it, and cannot
exceed the total envelope in combination. Tests prove a speculative hit causes
no media read and a speculative miss uses the canonical path. Focused tests
reject a local worker, kind-ceiling bypass, and scope theft.

**Cleanup:** delete isolated S.2 speculative/background models and any pool
worker, retry loop, queue, callback, or timer exposed by the inventory.

### Phase 7: Cut Over Every Ordinary Consumer And Future Adapter

**Telos:** leave one honest product boundary for current Store callers and one
narrow lease/pressure boundary for committed physical successors.

**Must ship:**

- all ordinary inline, extent, scan, manifest, publication, and writeback
  routes on canonical bounded residency;
- adapter compile specimens for C.9 integrity, C.10 isolation, and C.11 blob
  composition using borrowed views or scoped allocations rather than the pool;
- exact Recovery, Scrub, Maintenance, Verification, and Blob scope admission
  specimens without implementing successor policy;
- feature graphs proving Part II and ordinary consumers cannot import pool
  internals or certification authority;
- responsibility-named public exports only.

**Preserve:** lower owner crates stay Signal-agnostic; Store remains
branch/MVCC-agnostic; future adapters gain observation, not cache-control
authority.

**Proof:** positive UI cases compile through the intended adapters. Negative UI
cases attempt pool construction, eviction, dirty mutation, generation forgery,
and semantic-residency inference. Ordinary journey tests exercise each current
route and reconcile exact fault/hit/copy/writeback behavior.

**Cleanup:** delete `ServingPhysicalRuntime::c6_physical_work_handoff`,
`record_serving/c6_handoff/`, remaining `C6*` symbols, temporary re-exports,
and consumer shims. The product graph must fail if any returns.

### Phase 8: Delete The Parallel S.2 World

**Telos:** make it impossible for certification or future milestones to prove
physical residency against a model the production Store does not use.

**Must ship:**

- migration or deletion of every consumer in the Phase 1 legacy inventory;
- removal of all legacy S.2 feature declarations and dependency edges;
- deletion of snapshot admission, frame-table, legacy view, background,
  speculative, and materialization authority graphs;
- replacement of useful owner tests with narrow canonical-pool tests or real
  Store-boundary tests;
- removal of dependencies and feature branches made dead by the cutover;
- current Cargo metadata, compilation, and caller review showing no legacy edge
  remains.

**Preserve:** mathematical or policy tests with independent value may remain
only after they stop claiming production authority and stop constructing the
deleted physical world.

**Proof:** the workspace builds and tests without the deleted features, Cargo
metadata contains no legacy edge, and current callers use only the canonical
owners.

**Cleanup:** this phase is the cleanup. It cannot close with a quarantine,
deprecated alias, disabled-by-default feature, copied fixture, or “remove
after C.10” note.

### Phase 9: Publish The Current Contract

**Telos:** make the real C.6 behavior usable and prevent historical documents
from directing later work back to deleted surfaces.

**Must ship:**

- the bounded physical record access guide with compiled examples;
- the buffer-pool owner README;
- the S.2 supersession notice;
- roadmap closeout and C.7-C.11 handoff links;
- generated API documentation for public lease/view/pressure types.

**Preserve:** documentation distinguishes executed guarantees from future
successor responsibilities and never promotes physical residency into semantic
residency.

**Proof:** documentation examples compile and run against the ordinary facade,
links resolve, and the described owners match the current public facade and
Cargo graph.

**Cleanup:** remove stale examples, diagrams, paths, and vocabulary that teach
the deleted handoff or S.2 authority model.

### Phase 10: Hostile Direct Tests And Successor Handoff

**Telos:** prove the joined system under the adversarial condition, then hand
C.7 a single dirty/writeback truth and later milestones a single bounded read
truth.

**Must ship:**

- owner checks for pool laws and Store composition;
- developer smoke for hot/cold/refault/view/copy/dirty/speculative behavior;
- CI coverage for the larger-than-memory real-store scenario and a bounded set
  of stable, replayable schedule seeds;
- release coverage for the canonical hostile schedule;
- hardware qualification where filesystem or allocation instrumentation claims
  depend on a named platform;
- explicit C.7, C.8, C.9, C.10, and C.11 handoff entries naming the authority
  each successor may consume and the authority it may not acquire.

**Preserve:** no counter, test helper, diagnostic output, or successor adapter
becomes production authority.

**Proof:** the Non-Fake Acceptance Setup passes from clean source; each selected
CI schedule seed names a deterministic decision trace and is exactly replayable;
a clean rerun reproduces the behavior;
constitution, line-cap, dependency, feature, UI, and focused behavioral gates
are green.

**Cleanup:** delete temporary scenario-only production hooks. Retain only
named storage interposer/yieldpoint capabilities whose production boundary is
already admitted and whose test authority is mechanically sealed.

## Milestone Must Ship

C.6 is incomplete without all of the following:

- one physical-instance-owned pool with complete hard budget admission;
- the exact raw-policy -> admitted-policy `worth-proof` outcome API;
- exact per-scope and per-speculative-kind allocation authority;
- canonical hit, fault coalescence, eviction, and refault behavior;
- `RecordReadSession`, lease-scoped `PhysicalRecordChunkView`, and explicit
  bounded-copy streaming with no owning whole-record shortcut;
- Store-owned pressure and residency observation APIs that expose no pool
  control;
- ordinary write candidate, dirty, writeback, and settlement progression;
- prefetch/read-ahead/write-behind on the C.5.1 runtime;
- exact Signal-family reuse and the dedicated Foundational frame-writeback
  basis, with no direct Signal/Foundational/`worth-proof` dependency, import,
  or API type in the pool;
- adapter-safe basis/generation/bytes/pressure/retry evidence;
- exact counters reconciled to independent allocation and media observers;
- removal of the temporary `C6*` handoff surface;
- removal of the complete legacy S.2 model and feature graph;
- updated docs, focused tests, and roadmap handoff;
- the hostile larger-than-memory fresh-process scenario and bounded replayable
  schedule coverage.

## Explicit Non-Goals

C.6 does not:

- add WAL records, checkpoint rules, or durable root publication order (C.7);
- perform crash reconstruction or replay (C.8);
- define checksum/encryption/compression admission (C.9);
- define stable semantic reads, admission fairness, or QoS (C.10);
- implement the final artifact-family graph or blob protocol (C.11);
- add Query, Relational, branch, MVCC, or semantic cache authority;
- persist pool contents, leases, counters, Signal state, or scheduler state;
- guarantee that an arbitrary caller can materialize a whole record in memory.

## Closeout Gate

C.6 closes only when all ordinary C.5 file-backed access is mediated by the one
bounded physical residency owner inside the C.5.1 work topology; lease
lifetimes, memory admission, scope isolation, fault coalescence, dirty truth,
writeback settlement, and exact counters are mechanically falsifiable; a real
store at least 32 times larger than resident memory remains operational through
fresh processes; any seeded hostile schedule failure is replayable directly;
future physical
adapters can consume bounded views and
pressure evidence without pool or semantic-residency authority; the temporary
handoff and legacy S.2 worlds no longer compile; documentation describes the
real surface; and cleanup has no deferred entries.
