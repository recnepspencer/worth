# Storage Foundation S.2 Engineering Spec: Buffer Pool, Memory Budgets, And Zero-Copy Record Access

> **Status:** Historical — superseded by
> [C.6 Buffer Pool And Bounded Physical Access Join](./physical-reconstruction-c6-buffer-pool-runtime-join.md)
>
> **Current developer contract:**
> [Bounded Physical Record Access](./bounded-physical-record-access.md)
>
> **Current roadmap:**
> [Physical Foundation Reconstruction Roadmap](./physical-foundation-reconstruction-roadmap.md)
>
> **Primary prerequisite:** `S.1 Physical Page, Segment, And Extent Substrate`
>
> **Follow-on storage-foundation sequence:** `S.3 Physical Integrity, Scrub, Quarantine, And Corruption Localization`
>
> **Primary architectural driver:** make physical memory residency explicit,
> bounded, and machine-checkable before integrity, recovery, isolation, I/O,
> blob, or certification work can claim to run on stores larger than memory.

This document preserves the isolated S.2 design and its original reasoning.
Its authority types, feature graph, snapshot admission, frame table, record
views, and background-work models are not current APIs or implementation
targets. C.6 replaced them with the Store-owned bounded residency contract
linked above.

## Goal

Make Worth Store operate through a bounded physical buffer-pool regime where
page residency, pinning, dirty state, foreground allocation, maintenance
allocation, and hot record views are admitted before execution and measured
after execution.

S.2 turns memory residency into proof-bearing database law instead of cache
folklore.

S.2 is complete when the platform-grade Store path can read, write, plan
recovery, plan compaction, and stream large physical records against an
S.1-formatted store whose persisted footprint exceeds the configured memory
budget, without whole-store heap materialization, unbounded allocation, hidden
pin leaks, unbounded dirty-page growth, or semantic-domain reconstruction on
physical hot paths.

## Why This Sequence Exists

S.1 proved that Store bytes have physical shape. That shape is not enough for a
database claim if every operation can still pull too many pages, records, or
diagnostics into memory.

S.2 exists between S.1 and S.3 because corruption localization, WAL recovery,
physical isolation, I/O pacing, native blobs, and certification all need an
honest answer to a prior question: which bytes may be resident, pinned, dirty,
copied, streamed, or materialized at any point in an operation?

## Governing Summaries

- `MENTALITY.md`
  protects hard-problem-first design. S.2 therefore treats stores larger than
  memory as the starting condition, not a later stress test.
- `arch_laws.md`
  protects proof-bearing lifecycle transitions. S.2 must make lease, pin,
  dirty, eviction, admission, and zero-copy view states phase-typed rather than
  convention-managed counters around a cache.
- `composition_laws.md`
  protects named semantic steps. S.2 must not collapse budget admission,
  resident selection, frame loading, pinning, view construction, dirty
  publication, eviction, and evidence materialization into one buffer-manager
  god path.
- `domain_structure_laws.md`
  protects responsibility boundaries. Residency, leases, dirty state, eviction,
  allocation scopes, streaming windows, policy, diagnostics, and certification
  fail differently and need separate Store modules even though they all live
  near the buffer pool.
- `perf_laws.md`
  protects visible cost. S.2 must expose resident-byte, pinned-page,
  dirty-page, allocation, copy, eviction, hit/miss, read-ahead, write-behind,
  and materialization counters at the boundary being claimed.
- `physical-database-roadmap.md`
  places S.2 after physical page/segment/extent structure and before physical
  integrity. The roadmap requires bounded resident memory and exact allocation
  behavior for stores larger than memory.
- `runtime-integration-roadmap.md`
  keeps canonical semantic truth above physical storage. S.2 may make physical
  memory behavior explicit, but it must not become a semantic runtime arena or
  replace `worth-relational` ownership of in-memory truth.
- `worth_foundational_roadmap.md`
  protects shared meaning without stealing local representation. S.2 must use
  Foundational performance, layout, profile, provenance, diagnostic, and
  receipt vocabulary when exporting evidence, comparing claims, supporting
  certification, or explaining materialization posture. Foundational must not
  become Store's buffer-pool implementation, allocator, page table, or lease
  runtime.

## Adversarial Constraint

S.2 must survive this hostile condition:

> A store whose persisted physical footprint is larger than the configured
> resident-memory budget receives foreground reads and writes while recovery,
> compaction planning, scrub planning, import/export, and large-record streaming
> request page and extent access. Every admitted operation must stay within
> declared resident-byte, pinned-page, dirty-page, allocation, copy, and
> materialization envelopes, or deny/defer before it constructs expensive
> domain objects or unbounded buffers.

If an operation can satisfy its result by loading the whole store, if a pinned
page can survive beyond its admitted lifetime, if dirty pages can exceed budget
without publication or denial, if a "zero-copy" view outlives the lease that
protects its bytes, or if Foundational evidence is constructed from planned
work rather than executed counters, S.2 is not closed.

## Product Decision Lock

- S.2 owns physical memory residency, not semantic runtime memory.
- S.1 physical references and header witnesses are the only admissible physical
  byte entry points for ordinary S.2 work.
- Buffer-pool admission is a proof-bearing lifecycle, not a cache hint.
- Zero-copy means lease-scoped physical byte access, not unchecked borrowing of
  backend-private buffers.
- Foundational vocabulary is mandatory at evidence boundaries and forbidden as
  a substitute for Store-owned residency authority.
- S.2 residency, lease, budget, dirty-publication, record-view, and S.3 handoff
  evidence must be typed aspect-native Store/Foundational surfaces. JSON,
  serde-shaped objects, debug strings, display names, raw byte slices, copied
  payload views, and producer-private names may not satisfy budget admission,
  lease admission, zero-copy proof, dirty-publication proof, or handoff
  readiness.

## Planned Directory Skeleton

`workspaces/worth-store/crates/worth-store-buffer-pool/src/`

- `lib.rs`
  aggregates the public S.2 facade and re-exports only proof-bearing boundary
  types.
- `budget.rs`
  owns resident-byte, pinned-page, dirty-page, allocation, copy, and streaming
  budget declarations.
- `admission.rs`
  owns budget admission requests, admitted work envelopes, denial/defer
  outcomes, and admission counters.
- `residency.rs`
  owns resident-frame table vocabulary, residency identity, page-frame
  generation, hit/miss reports, and resident-byte accounting.
- `lease.rs`
  owns page-lease, pin, unpin, and lease-scope typestates.
- `dirty.rs`
  owns dirty-page state, dirty publication plans, write-behind eligibility, and
  dirty-budget accounting.
- `eviction.rs`
  owns eviction candidates, protected pages, eviction plans, and eviction
  denial reports.
- `allocation_scope.rs`
  owns foreground, maintenance, recovery, scrub, import/export, and streaming
  allocation envelopes.
- `record_view.rs`
  owns bounded-copy and zero-copy framed-record view admission.
- `streaming_window.rs`
  owns extent-backed large-record and future blob-style streaming windows.
- `read_ahead.rs`
  owns read-ahead plans, admission rules, and counters.
- `write_behind.rs`
  owns write-behind plans, admission rules, and counters.
- `counters.rs`
  owns executed S.2 counter snapshots and exact counter assertions.
- `diagnostics.rs`
  owns typed memory-pressure, pin-leak, over-budget, and materialization
  denials.

`workspaces/worth-store/crates/worth-store-certification/src/`

- `buffer_pool_scenario_definitions.rs`
  defines S.2 scenario grammar over the existing Roadmap 2 harness rather than
  inventing sequence-local setup APIs.
- `buffer_pool_scenario_plans.rs`
  lowers S.2 definitions into required capabilities, drivers, observers,
  counters, denial boundaries, artifact policies, and transcript identity.
- `buffer_pool_evidence.rs`
  maps executed S.2 evidence into Roadmap 2 scenario lanes.
- `buffer_pool_observers.rs`
  registers resident-set, pin-lifecycle, dirty-set, allocation-envelope,
  copy-boundary, eviction, and streaming-window observers.
- `buffer_pool_oracles.rs`
  owns proof judgments for bounded residency, no whole-store materialization,
  protected-frame eviction denial, dirty-budget denial, and lease-scoped views.
- `buffer_pool_counter_receipts.rs`
  materializes Foundational performance receipts from executed Store counters.
- `buffer_pool_story_lanes.rs`
  defines memory-pressure, zero-copy, dirty-budget, eviction, recovery,
  maintenance, and streaming lanes.
- `buffer_pool_transcripts.rs`
  emits replay-comparable story, counter, denial, allocation, copy-boundary,
  eviction, and streaming traces.

`workspaces/worth-store/crates/worth-store-test-support/src/`

- `memory_pressure.rs`
  provides reusable pressure devices and large-store fixture mechanics without
  owning certification meaning.
- `allocation_sentinels.rs`
  detects allocation-envelope violations before RSS-style evidence can hide
  path-local allocation mistakes.
- `resident_pressure_fixtures.rs`
  creates persisted stores whose physical footprint exceeds the configured
  resident-memory budget.
- `large_record_streams.rs`
  drives extent-backed large-record and future blob-style streaming windows.
- `background_work_drivers.rs`
  supplies recovery, compaction, scrub, import/export, and streaming pressure
  mechanics while certification keeps the pass/fail meaning.

## Roadmap 2 Harness Test Plan

S.2 must piggyback on the S.1 `PhysicalScenarioQualityHarness`. It must not
create a second harness, a local scenario runner, or test-support-owned proof
meaning. The S.2 contribution is the `buffer_pool` lane family plus its
scenario definitions, lowered plans, drivers, observers, proof oracles,
transcripts, counter receipts, and evidence bundles.

The inherited proof pipeline remains:

`PhysicalScenarioDefinition` -> `PhysicalScenarioPlan` ->
`PhysicalScenarioExecution` -> `ObservedPhysicalTrace` ->
`PhysicalProofOracleVerdict` -> `PhysicalStoryTranscript`

S.2 scenario definitions must be readable physical-law stories. Required
definition vocabulary includes:

- `given_s1_physical_store_larger_than_resident_budget`
- `given_resident_byte_budget`
- `given_pinned_page_budget`
- `given_dirty_page_budget`
- `given_allocation_envelope`
- `when_page_is_leased_and_pinned`
- `when_zero_copy_record_view_is_open`
- `when_dirty_budget_is_exhausted`
- `when_eviction_pressure_targets_protected_frames`
- `when_large_record_stream_crosses_many_extents`
- `then_denied_before_materialization`
- `then_counter_trace_matches_declared_envelope`
- `then_story_transcript_replays_with_same_identity`

S.2 lane families must be registered under the S.1 harness extension mechanism:

- `budget_admission_lane`
  proves admission, defer, and denial happen before expensive construction.
- `resident_frame_lifecycle_lane`
  proves page residency, generation, hit/miss, and resident-byte accounting.
- `pin_lifecycle_lane`
  proves pin, unpin, lease scope, and protected-frame behavior.
- `dirty_budget_lane`
  proves dirty growth, publication planning, and dirty-budget denial.
- `eviction_protection_lane`
  proves eviction scans resident candidates and excludes protected frames.
- `allocation_envelope_lane`
  proves foreground, maintenance, recovery, scrub, import/export, and streaming
  allocation budgets remain separate.
- `zero_copy_view_lane`
  proves view lifetimes are lease-scoped and copies are exact or forbidden.
- `speculative_read_write_lane`
  proves read-ahead, prefetch, and write-behind are budget-admitted work.
- `large_store_pressure_lane`
  proves the ordinary certification case is a persisted store larger than the
  resident-memory budget.
- `background_envelope_lane`
  proves later recovery, compaction, scrub, import/export, and streaming work
  consumes envelopes without stealing foreground budgets.
- `foundational_evidence_lane`
  proves Foundational receipts derive from executed Store counters.
- `s3_handoff_lane`
  proves S.3 receives bounded protected-byte access, not raw buffers.

Required drivers extend the S.1 driver set without changing harness shape:

- existing `PlatformBackendDriver`, `PersistedFileDeviceDriver`,
  `AdversarialByteDeviceDriver`, and `CrashInterposerDriver`
- S.2 `MemoryPressureDriver`, `AllocationSentinelDriver`,
  `LargeRecordStreamingDriver`, and `BackgroundMaintenanceDriver`

The `CrashInterposerDriver` may be used only to interrupt S.2 memory and
publication pressure. It must not let S.2 claim S.4 recovery physics.

Required observers extend the S.1 observer set:

- existing `CounterObserver`, `StorageBoundaryObserver`,
  `MaterializationObserver`, `RuntimeLayoutObserver`,
  `DenialBoundaryObserver`, and `EvidenceExportObserver`
- S.2 `ResidentSetObserver`, `PinLifecycleObserver`, `DirtySetObserver`,
  `AllocationEnvelopeObserver`, `CopyBoundaryObserver`, `EvictionObserver`,
  and `StreamingWindowObserver`

Required proof oracles:

- `NoWholeStoreMaterializationOracle`
- `ResidentBytesNeverExceedBudgetOracle`
- `PinnedPagesNeverEvictedOracle`
- `DirtyPagesNeverExceedBudgetWithoutDenialOracle`
- `ZeroCopyViewCannotOutliveLeaseOracle`
- `AllocationEnvelopePrecedesMaterializationOracle`
- `EvictionScansResidentSetNotWholeStoreOracle`
- `StreamingMemoryBoundedByWindowOracle`
- `FoundationalReceiptsUseExecutedCountersOracle`
- `TestSupportCannotOwnBufferPoolCertificationMeaningOracle`

Required transcripts and traces:

- `buffer_pool_story_transcript`
- `resident_memory_counter_trace`
- `pin_lifecycle_trace`
- `dirty_budget_trace`
- `allocation_envelope_trace`
- `copy_boundary_trace`
- `eviction_pressure_trace`
- `streaming_window_trace`
- `buffer_pool_denial_trace`
- `foundational_performance_receipt_trace`

Production-grade S.2 tests must include three layers:

- Contract tests for narrow type and module boundaries, including compile-fail
  or visibility tests where invalid lifecycle states must be unrepresentable.
- Harness certification tests where readable S.2 scenario definitions lower
  into plans and run through drivers, observers, oracles, and transcripts.
- Large-store pressure tests using persisted fixtures larger than the configured
  memory budget, exact counter assertions, allocation sentinels, and
  materialization observers.

Unit tests that only show a cache hit, a cache miss, or a successful read are
insufficient. Each phase must include at least one replay/parity/convergence
test and at least one denial/leakage/boundary-localization test, and the S.2
closeout must include at least one failed synthetic shortcut proving the test
suite rejects fake harnesses, proof meaning inside test support, and whole-store
materialization disguised behind a passing workload.

## Phases

### Phase 1: Consume S.1 Physical Substrate Readiness And Freeze Residency Vocabulary

Phase 1 closes the entry boundary: S.2 consumes typed S.1 physical readiness and
defines what residency words are allowed to mean before any page is cached.

**Relevant subsystems**

- `worth-store-buffer-pool`
- `worth-store-physical-format`
- `worth-store-readiness`
- `worth-store-contracts`
- `worth-store-certification`
- `worth-foundational` performance and profile vocabulary

**Relevant APIs**

- `S2PhysicalSubstrateReadiness`
- `PhysicalReference`
- `PhysicalHeaderDecodeWitness`
- `PhysicalPayloadViewAdmission`
- `BufferPoolBudget`
- `ResidentMemoryBudget`
- `PinnedPageBudget`
- `DirtyPageBudget`
- `AllocationEnvelope`
- `FoundationalPerformanceBoundary`

**Warnings**

- Do not accept raw page ids, backend handles, or byte slices as S.2 entry
  authority.
- Do not name modules after `S.2`; use responsibility names.
- Do not let Foundational profile names or performance labels become Store
  memory-budget authority.

**Test requirements**

- Adversarial parity: two independently constructed S.1 readiness handoffs
  lower into the same S.2 residency vocabulary and Foundational performance
  basis at the evidence boundary.
- Adversarial denial: raw page ids, raw payload views, compatibility backend
  handles, or local string profile names cannot enter S.2 admission APIs.
- Compile-fail: external callers cannot synthesize S.2 readiness or resident
  budget witnesses from raw fields.

**Engineering decisions**

- S.2 begins with vocabulary because every later phase consumes resident,
  pinned, dirty, copied, and materialized meanings.
- Foundational performance/profile vocabulary is attached only to boundary
  reports and receipts.
- Store keeps the actual resident-frame and budget implementation local.

**Open questions**

- None.

### Phase 2: Define Resident Frame Table And Page Residency Authority

Phase 2 creates the resident-frame table as the only authority for whether an
S.1 physical page or frame is in memory.

**Relevant subsystems**

- `worth-store-buffer-pool`
- `worth-store-physical-format`
- `worth-store-physical-backend`
- `worth-store-certification`

**Relevant APIs**

- `ResidentFrameTable`
- `ResidentFrameSlot`
- `ResidentFrameGeneration`
- `ResidentFrameAdmission`
- `ResidentFrameHitMissReport`
- `PhysicalPageId`
- `PhysicalFrameHeader`

**Warnings**

- Do not let backend-private maps or OS page cache state count as resident
  frame authority.
- Do not treat semantic artifact identity as residency identity.
- Do not hide resident-byte accounting inside debug-only observer state.
- Resident Frame Authority Law: after this phase, no platform-grade S.2 lane
  may claim a page, frame, or payload is resident unless that fact comes from
  `ResidentFrameTable` or a stronger admitted residency witness.
- Generation Domain Separation Law: S.1 physical generations protect durable
  placement reuse. S.2 resident-frame generations protect in-memory frame-slot
  reuse. Neither generation domain proves validity in the other domain.

**Test requirements**

- Adversarial parity: loading the same admitted physical page through two
  equivalent S.1 references produces the same resident-frame identity and
  hit/miss counter sequence.
- Adversarial denial: backend-private residue, stale frame generation, and
  physical pages outside the S.1 root manifest cannot become resident frames.
- Generation separation: a valid physical generation cannot keep a stale
  resident view alive after resident-frame reuse, and a valid resident
  generation cannot prove persisted physical-reference validity after S.1
  physical reuse.
- Performance proof: resident-byte and frame-table lookup counters are exact at
  the buffer-pool facade.

**Engineering decisions**

- Residency is a Store-owned physical state, not a semantic cache.
- Resident-frame generations are separate from S.1 physical cell generations
  because they describe in-memory lifecycle, not on-media reuse.
- The resident table may change strategy later, but it must preserve the same
  authority boundary.

**Open questions**

- None.

### Phase 3: Define Lease, Pin, Unpin, And View Lifetime Typestates

Phase 3 makes resident bytes borrowable only through lease-scoped proof types.
It closes the pin-leak and use-after-unpin holes before zero-copy views exist.

**Relevant subsystems**

- `worth-store-buffer-pool`
- `worth-store-physical-format`
- `worth-store-certification`

**Relevant APIs**

- `PageLease`
- `PinnedPageLease`
- `UnpinnedPageReceipt`
- `LeaseScope`
- `LeaseEpoch`
- `PinnedFrameView`
- `LeaseLeakReport`

**Warnings**

- Do not expose a borrowable byte view without a live lease proof.
- Do not make unpin a best-effort destructor side effect only.
- Do not allow pinned pages to be evicted or dirty-published behind the lease.
- Lease-Scoped Byte Access Law: after this phase, ordinary S.2 consumers may
  access resident bytes only through live lease and pin proofs.
- Explicit Pin Closure Law: explicit unpin produces the authoritative lifecycle
  receipt. `Drop` may perform defensive cleanup, but destructor-only cleanup is
  certification evidence only in scenarios that explicitly test panic/drop
  behavior. Leaked pins remain protected and produce typed leak evidence at
  scope or pool closeout.

**Test requirements**

- Adversarial replay: repeated pin/unpin of the same resident frame produces
  stable lease receipts, pin counters, and no residual pinned state.
- Adversarial denial: evicting, writing behind, or reusing a frame while a live
  pin exists fails with a typed denial before backend I/O.
- Compile-fail: a `PinnedFrameView` cannot outlive or be constructed without a
  `PinnedPageLease`.
- `pin_drop_and_leak_honesty_suite`: explicit unpin produces stable receipts;
  panic/drop cleanup does not fabricate normal lifecycle evidence; leaked pins
  prevent eviction and publication; pool closeout reports leaked pin state; and
  the harness can force leak scenarios without exposing ordinary silent-leak
  APIs.

**Engineering decisions**

- Lease lifecycle is encoded in types before policy is optimized.
- Pin state is a residency proof, not a boolean on a broad page descriptor.
- The test harness may force leak scenarios, but ordinary APIs may not create
  leaked pins silently.

**Open questions**

- None.

### Phase 4: Define Dirty Page State And Publication Planning

Phase 4 separates clean residency from dirty residency and makes dirty-page
growth an admitted, measured lifecycle.

**Relevant subsystems**

- `worth-store-buffer-pool`
- `worth-store-physical-format`
- `worth-store-physical-backend`
- `worth-store-certification`

**Relevant APIs**

- `DirtyPageState`
- `DirtyPageBudget`
- `DirtyPublicationPlan`
- `DirtyPublicationReceipt`
- `WriteBehindEligibility`
- `DirtyBudgetDenial`

**Warnings**

- Do not let dirty pages accumulate as unbounded resident state.
- Do not claim S.4 recovery or fsync correctness here; S.2 only plans and
  accounts for dirty residency.
- Do not publish dirty state while a conflicting pin or lease scope is active.
- Dirty State Authority Law: after this phase, no page may be treated as dirty,
  clean, publishable, write-behind eligible, or eviction-safe through broad
  metadata. It must consume `DirtyPageState`, `DirtyPublicationPlan`, or a
  stronger dirty-state witness.
- Dirty Publication Honesty Law: S.2 dirty publication plans and receipts prove
  memory-budgeted dirty-state admission, write-scheduling eligibility,
  executed write-attempt accounting, and dirty-budget release facts only. They
  do not prove WAL ordering, pageLSN correctness, checkpoint correctness, fsync
  durability, crash recovery, semantic commit visibility, or post-crash source
  precedence.

**Test requirements**

- Adversarial parity: equivalent write workloads produce the same dirty-page
  budget reports and publication-plan identity.
- Adversarial denial: dirtying a page beyond budget, dirtying an unadmitted
  page, or publishing a page still protected by a conflicting lease denies
  before backend write scheduling.
- Performance proof: dirty-page, dirty-byte, publication-plan, and
  write-behind-eligible counters are exact.
- `dirty_state_shutdown_honesty_suite`: clean close cannot silently discard
  dirty pages; unflushed dirty pages cannot be reported as durable; dirty
  publication plans cannot become recovery claims; shutdown either publishes
  through admitted policy, denies closeout, or emits typed dirty-residency
  evidence; and close/reopen tests do not treat dirty resident memory as
  persisted physical truth.

**Engineering decisions**

- Dirty state is a physical residency state, not a WAL or recovery claim.
- Publication plans are lowered work; executors may not re-decide dirty
  eligibility.
- Later S.4 WAL/pageLSN work consumes dirty publication evidence rather than
  retrofitting dirty state.

**Open questions**

- None.

### Phase 5: Define Eviction Admission And Protected-Frame Denials

Phase 5 turns eviction from a heuristic into an admitted physical plan with
named protections and denials.

**Relevant subsystems**

- `worth-store-buffer-pool`
- `worth-store-certification`
- `worth-store-physical-format`

**Relevant APIs**

- `EvictionCandidateSet`
- `EvictionPlan`
- `EvictionReceipt`
- `ProtectedFrameDenial`
- `EvictionPressureReport`
- `ResidentFrameTable`

**Warnings**

- Do not evict pinned, dirty-unpublished, verifier-protected, streaming, or
  recovery-protected frames.
- Do not let eviction scan the entire store when only resident frames are
  relevant.
- Do not hide eviction choice behind backend cache behavior.
- Eviction Candidate Authority Law: after this phase, eviction may consider
  only resident frames admitted by the resident-frame table, and it must exclude
  protected, pinned, dirty-unpublished, verifier-protected, recovery-protected,
  and streaming-protected frames before policy ranking.

**Test requirements**

- Adversarial parity: two resident-frame tables with the same residency,
  protection, and policy facts produce the same eviction plan identity.
- Adversarial denial: all-protected, dirty-unpublished, pinned, and
  verifier-protected resident sets deny eviction with precise protection
  reasons.
- Performance proof: eviction candidate scan counters are bounded by resident
  frame count, not persisted store size.

**Engineering decisions**

- Eviction plans consume resident-frame facts and protection proofs.
- The phase may define simple policy first, but the policy must be explicit and
  measured.
- S.2 proves physical memory boundedness, not I/O optimality.

**Open questions**

- None.

### Phase 6: Define Allocation Scopes For Foreground, Maintenance, Recovery, And Streaming

Phase 6 makes allocation a first-class admitted resource rather than ambient
heap behavior.

**Relevant subsystems**

- `worth-store-buffer-pool`
- `worth-store-budgets`
- `worth-store-certification`
- `worth-foundational` performance vocabulary

**Relevant APIs**

- `AllocationScope`
- `ForegroundAllocationEnvelope`
- `MaintenanceAllocationEnvelope`
- `RecoveryAllocationEnvelope`
- `ScrubAllocationEnvelope`
- `StreamingAllocationEnvelope`
- `AllocationAdmission`
- `AllocationDenial`

**Warnings**

- Do not treat Rust heap success as allocation admission.
- Do not let maintenance or scrub steal foreground envelopes.
- Do not allocate rich diagnostics on hot paths unless the resolved profile
  admits that materialization.
- Allocation Admission Law: after this phase, no platform-grade operation may
  allocate unbounded buffers, copied payloads, rich diagnostics, materialized
  record sets, or background work memory outside an admitted allocation
  envelope, except for explicitly declared fixed-size control metadata.
- Fixed Metadata Allocation Rule: explicitly bounded fixed-size control
  metadata may be exempt from operation allocation envelopes only when the
  exemption is declared, counted separately, and proven independent of store
  size, page count, record count, diagnostic richness, and payload size.
  Variable-size diagnostics, copied payloads, decoded records, page/record-id
  vectors, and background buffers are never exempt.

**Test requirements**

- Adversarial parity: equivalent foreground and maintenance operations resolve
  the same allocation envelopes and Foundational performance basis when
  exported.
- Adversarial denial: a maintenance operation that would exceed its envelope
  denies or defers before allocating buffers or materializing diagnostics.
- Performance proof: allocation counters distinguish requested, admitted,
  allocated, copied, and denied bytes per scope.
- Exemption proof: fixed-metadata exemptions remain constant-size across store
  scale, payload scale, page-count scale, and profile richness changes.

**Engineering decisions**

- Allocation scope is separate from resident page budget because callers can
  allocate without changing page residency.
- Foundational performance reports describe allocation evidence; they do not
  allocate memory.
- S.2 defines recovery envelopes so S.4 can later consume them instead of
  inventing recovery memory policy.

**Open questions**

- None.

### Phase 7: Build Zero-Copy And Bounded-Copy Physical Record Views

Phase 7 creates the physical record-access regime that prevents hot paths from
deserializing or copying unbounded record sets.

**Relevant subsystems**

- `worth-store-buffer-pool`
- `worth-store-physical-format`
- `worth-store-certification`
- `worth-foundational` layout vocabulary

**Relevant APIs**

- `ZeroCopyRecordView`
- `BoundedCopyRecordView`
- `FramedRecordView`
- `PinnedFrameView`
- `RecordViewAdmission`
- `RecordCopyCounterSnapshot`
- `MaterializationProfile`

**Warnings**

- Do not expose semantic domain objects from physical record views.
- Do not call a view zero-copy if it copies payload bytes into a heap-owned
  buffer on the hot path.
- Do not let a zero-copy view outlive the lease, pin, header witness, or
  physical reference that admitted it.
- View Admission Law: after this phase, zero-copy and bounded-copy views may
  only be constructed from an admitted physical reference, admitted header
  witness, live lease/pin proof, and admitted copy/materialization envelope
  where applicable.
- View/Mutation Compatibility Law: a zero-copy view must declare immutable or
  mutable access. Mutable access requires exclusive lease authority. Dirtying or
  publishing a resident frame with live incompatible views must deny before
  mutation/publication or route through an explicitly admitted bounded-copy or
  copy-on-write path with copy counters and dirty-state receipts.

**Test requirements**

- Adversarial parity: locating the same framed record through equivalent S.1
  references yields the same view admission basis and exact copy counter.
- Adversarial denial: stale references, invalid header witnesses, expired
  leases, or profile-forbidden rich materialization deny before view
  construction.
- Compile-fail: a `ZeroCopyRecordView` cannot be constructed from raw bytes or
  stored without its lease-scoped proof.
- `conflicting_view_and_dirty_mutation_suite`: immutable zero-copy views block
  conflicting mutation; mutable views require exclusive lease authority;
  dirtying denies before mutation when incompatible views are live; publication
  denies while protected incompatible views exist; admitted bounded-copy or COW
  fallback is counted if supported; and no semantic domain materialization
  resolves the conflict.

**Engineering decisions**

- Zero-copy is a lease-scoped physical property, not a semantic API promise.
- Bounded-copy views are allowed only when the copy envelope is admitted and
  counted.
- Foundational layout vocabulary may describe AoS/SoA/packed posture at export
  boundaries but may not force one Store internal representation.

**Open questions**

- None.

### Phase 8: Define Speculative Physical Work Memory Admission

Phase 8 gives speculative physical work memory admission before execution so
background help cannot secretly break memory, dirty-page, pin, or allocation
budgets.

**Relevant subsystems**

- `worth-store-buffer-pool`
- `worth-store-physical-backend`
- `worth-store-io-scheduler`
- `worth-store-certification`

**Relevant APIs**

- `ReadAheadPlan`
- `ReadAheadAdmission`
- `PrefetchWindow`
- `WriteBehindPlan`
- `WriteBehindAdmission`
- `SpeculativeResidencyDenial`

**Warnings**

- Do not let read-ahead or write-behind bypass foreground resident or dirty
  budgets.
- Do not infer policy inside the executor after admission.
- Do not hide speculative work under cache-hit metrics.
- Speculative Work Admission Law: after this phase, read-ahead, prefetch, and
  write-behind may not execute unless admitted against foreground-safe resident,
  dirty, pin, and allocation budgets.
- Speculative Work Honesty Law: S.2 read-ahead, prefetch, and write-behind
  plans prove only that speculative physical work was admitted, bounded,
  counted, denied, or deferred under memory and dirty-budget rules. They do not
  prove I/O optimality, queue-depth correctness, backend-specific pacing, fsync
  policy, fairness, or throughput improvement.

**Test requirements**

- Adversarial replay: the same scan or streaming hint lowers into the same
  read-ahead or write-behind plan under the same budget state.
- Adversarial denial: read-ahead, prefetch, or write-behind requests deny when
  they would evict protected foreground pages, exceed dirty budget, or exceed
  admitted allocation scope.
- Performance proof: speculative-read, prefetch-admitted, prefetch-denied,
  write-behind-admitted, and write-behind-denied counters are exact.
- No-inflation proof: speculative work cannot claim success by inflating cache
  hit metrics while hiding resident-byte growth, dirty-budget growth, protected
  eviction pressure, or foreground allocation interference.

**Engineering decisions**

- Speculation is useful only after foreground budgets are protected.
- Plans are lowered before execution; executors consume plans and record facts.
- S.6 may later refine I/O pacing, but S.2 owns the memory-admission side.

**Open questions**

- None.

### Phase 9: Build Large-Store Memory-Pressure And OOM-Avoidance Lanes

Phase 9 makes the adversarial condition executable: persisted footprint exceeds
memory budget while ordinary operations continue or deny honestly.

**Relevant subsystems**

- `worth-store-buffer-pool`
- `worth-store-certification`
- `worth-store-test-support`
- `worth-store-physical-format`

**Relevant APIs**

- `LargeStoreMemoryPressureScenario`
- `MemoryPressureDriver`
- `AllocationSentinel`
- `ResidentBudgetObserver`
- `MaterializationObserver`
- `AllocationEnvelopeObserver`
- `BufferPoolScenarioPlan`
- `PhysicalStoryTranscript`
- `MemoryPressureDenialTrace`

**Warnings**

- Do not satisfy this phase with small fixtures that fit in memory.
- Do not let test support own pass/fail meaning; certification lanes own it.
- Do not use process RSS as the only proof. RSS can support investigation, but
  S.2 proof must use Store counters and sentinels.
- "Larger than memory" is not a single fixture. This phase must certify
  pressure classes that expose boundary precision, normal pressure, scale
  independence, fragmentation, protection, and streaming behavior.

**Test requirements**

- Adversarial scale parity: the same logical workload over a store larger than
  budget completes with resident-byte, pinned-page, allocation, copy, and
  materialization counters inside the declared envelope.
- Adversarial denial: a workload that cannot fit admitted page, dirty, or
  allocation envelopes denies before domain-object construction or unbounded
  heap allocation.
- Harness proof: memory-pressure lanes emit stable story transcripts, counter
  traces, and denial traces through the Roadmap 2 scenario harness.
- Shortcut rejection: a test that bypasses scenario-plan lowering, fabricates
  resident counters without an observer trace, or keeps pass/fail meaning inside
  `worth-store-test-support` fails certification.
- Replay proof: the same persisted pressure fixture replays into the same
  scenario plan identity, counter trace identity, denial trace identity, and
  transcript identity across two executions.
- Pressure-class proof: `barely_over_budget`, `moderately_over_budget`,
  `far_over_budget`, `fragmented_pressure`, `protected_pressure`, and
  `streaming_pressure` fixtures each run through the Roadmap 2 harness with
  deterministic counters and transcript identity.

**Engineering decisions**

- Large-store pressure is a first-class S.2 lane, not a late benchmark.
- Allocation sentinels are test-support mechanics; certification interprets
  the evidence.
- Denial is success when the requested work cannot fit the declared envelope.

**Open questions**

- None.

### Phase 10: Define Background Work Memory Envelope Contracts

Phase 10 proves S.2 is not only a foreground read cache. It defines the memory
contracts non-foreground physical programs will consume without implementing
the recovery, scrub, blob, import/export, compaction, or repair semantics owned
by later Roadmap 2 sequences.

**Relevant subsystems**

- `worth-store-buffer-pool`
- `worth-store-recovery-physics`
- `worth-store-maintenance`
- `worth-store-physical-integrity`
- `worth-store-blob-chunks`
- `worth-store-certification`

**Relevant APIs**

- `RecoveryMemoryEnvelope`
- `CompactionPlanningMemoryEnvelope`
- `ScrubPlanningMemoryEnvelope`
- `ImportExportMemoryEnvelope`
- `LargeRecordStreamingEnvelope`
- `BackgroundMemoryInterferenceReport`

**Warnings**

- Do not claim S.3 scrub, S.4 recovery, S.7 blobs, or S.10 repair behavior
  merely because their memory envelopes exist.
- Do not let background envelopes borrow from foreground budgets without
  explicit admission.
- Do not make streaming depend on whole-object residency.
- Background Envelope Honesty Law: S.2 background envelopes prove only memory
  admission, foreground reservation protection, bounded planning access,
  allocation/copy accounting, streaming-window bounds, and typed denial/defer
  behavior. They do not prove scrub correctness, corruption localization, WAL
  recovery, checkpoint safety, compaction validity, import/export semantic
  correctness, blob lifecycle completion, or repair behavior.

**Test requirements**

- Adversarial parity: recovery planning, compaction planning, scrub planning,
  import/export, and large-record streaming each consume explicit envelopes and
  report exact memory counters without exceeding foreground reservations.
- Adversarial denial: background work that would steal foreground residency,
  pin pages indefinitely, or require whole-object memory denies or defers with
  typed interference evidence.
- Performance proof: streaming memory remains bounded by window size, not
  record or blob size.
- Boundary proof: recovery, scrub, compaction, import/export, and streaming
  scenarios can consume S.2 envelopes without producing S.3 corruption, S.4
  recovery, S.7 blob, or S.10 repair claims.

**Engineering decisions**

- This phase creates envelope contracts for later sequences, not their full
  behavior.
- Background work is admitted against budget before it touches resident frames.
- Large-record streaming is S.2's bridge to S.7 without pre-claiming blob
  chunk-tree semantics.

**Open questions**

- None.

### Phase 11: Materialize Foundational Performance, Layout, Profile, Provenance, And Receipt Evidence

Phase 11 turns executed S.2 facts into shared boundary evidence without moving
Store's buffer-pool authority into Foundational.

**Relevant subsystems**

- `worth-store-buffer-pool`
- `worth-store-certification`
- `worth-foundational`

**Relevant APIs**

- `BufferPoolCounterSnapshot`
- `ResidentMemoryPerformanceReceipt`
- `AllocationEnvelopePerformanceReceipt`
- `ZeroCopyLayoutPostureReport`
- `MaterializationProfileReport`
- `BufferPoolProvenanceAttachment`
- `CompletedResidencyBoundaryReceipt`

**Warnings**

- Do not build Foundational receipts from plans alone. Receipts require executed
  facts.
- Do not collapse resident-memory, allocation, copy, layout, and profile
  evidence into one generic "performance report."
- Do not let profile-rich diagnostics change the admitted operation result.
- Evidence Source Law: Foundational S.2 performance, layout, profile,
  provenance, and receipt artifacts may describe Store-executed residency
  facts, but they may not substitute for `ResidentFrameTable`,
  `DirtyPageState`, `PageLease`, `AllocationEnvelope`, or
  `RecordViewAdmission` in any Store operation.

**Test requirements**

- Adversarial equivalence: the same executed counter snapshot materializes the
  same Foundational performance/layout/profile/provenance basis through two
  independent report constructors.
- Adversarial denial: planned counters, raw strings, local profile labels, and
  copied report fields cannot satisfy the S.2 evidence APIs.
- Profile proof: reduced-richness profiles remove optional descriptive
  diagnostics while preserving authority result, counters, and denials.
- Authority denial: a Foundational receipt, provenance attachment, layout
  posture report, or profile report cannot be passed back into Store as
  resident-frame, dirty-state, lease, allocation, or view-admission authority.

**Engineering decisions**

- Foundational is mandatory for shared evidence meaning.
- Store-owned counters are the source of evidence; Foundational receives the
  boundary materialization.
- Profile-driven richness controls explanation cost, not operation truth.

**Open questions**

- None.

### Phase 12: Close Bounded Residency And Publish S.3 Integrity Readiness

Phase 12 closes S.2 by running the named bounded-memory suite and publishing a
typed handoff for physical integrity work.

**Relevant subsystems**

- `worth-store-buffer-pool`
- `worth-store-readiness`
- `worth-store-certification`
- `worth-store-physical-format`
- `worth-foundational`

**Relevant APIs**

- `BoundedMemoryResidencySuite`
- `S3PhysicalIntegrityReadiness`
- `BoundedMemoryCloseoutReport`
- `BufferPoolCertificationBundle`
- `ResidentMemoryCounterReceipt`
- `AllocationCounterReceipt`
- `ZeroCopyAccessReceipt`
- `BufferPoolScenarioPlanLoweringReport`
- `BufferPoolStoryTranscriptReplayReport`
- `SyntheticHarnessShortcutRejectionReport`

**Warnings**

- Do not close S.2 on cache-hit behavior alone.
- Do not claim corruption localization, WAL recovery, blob-scale completion,
  I/O QoS, or aerospace-grade readiness from S.2 evidence.
- Do not leave S.3 with raw buffer handles or untyped page views as its entry
  point.
- `S3PhysicalIntegrityReadiness` must contain concrete entry evidence, not a
  symbolic handoff token.

**Test requirements**

- Adversarial closeout: a store larger than configured memory budget completes
  admitted reads, writes, recovery planning, compaction planning, and large
  record streaming within exact resident-byte, pinned-page, dirty-page,
  allocation, copy, and materialization envelopes.
- Adversarial denial: over-budget page residency, pin leaks, dirty overflow,
  whole-store materialization, whole-object streaming, and forged view access
  fail at named S.2 boundaries.
- Handoff proof: S.3 receives typed integrity-readiness evidence that includes
  protected resident-page access and bounded verifier/scrub planning
  envelopes, not raw buffers.
- Harness closeout: every S.2 acceptance suite runs through the inherited
  `PhysicalScenarioQualityHarness` pipeline from definition to transcript, and
  the closeout bundle names the lane, driver, observer, oracle, and transcript
  families used by each suite.
- Synthetic-test rejection: suites that prove behavior only through logs,
  same-run self-comparison, small in-memory fixtures, or test-support-owned
  oracle meaning fail before S.2 can close.
- S.3 payload proof: the handoff includes admitted protected page/extent view
  capability, maximum verifier resident envelope, maximum scrub planning
  allocation envelope, corruption-inspection view lifetime law,
  no-whole-store-materialization witness, relevant resident/copy/allocation
  counters, denial behavior for over-budget integrity inspection, S.1 physical
  format/reference/header authority recap, and S.2 lease/pin/residency
  authority recap.

**Engineering decisions**

- The closeout suite proves bounded physical residency, not full database
  certification.
- S.3 readiness is a typed handoff because integrity must inspect bytes without
  reopening the memory-budget problem.
- Aerospace-grade remains later Roadmap 2 certification, not an S.2 claim.

**Open questions**

- None.

## Must Ship

- typed S.1-to-S.2 physical readiness consumption
- resident-frame table authority with resident-byte and hit/miss accounting
- resident/physical generation domain separation law
- lease, pin, unpin, and view-lifetime typestates
- explicit pin drop, panic, and leak closeout behavior
- dirty-page state and publication planning with dirty-budget accounting
- dirty publication honesty and dirty shutdown honesty
- eviction plans that reject protected, pinned, dirty-unpublished, verifier,
  recovery, and streaming frames
- allocation envelopes for foreground, maintenance, recovery, scrub,
  import/export, and streaming work
- fixed metadata allocation exemption policy
- zero-copy and bounded-copy physical record views tied to admitted leases and
  header witnesses
- view/mutation compatibility rules for dirtying, publishing, immutable views,
  mutable views, and admitted bounded-copy/COW fallback
- read-ahead, prefetch, and write-behind admission policies
- speculative physical work honesty limits that reserve I/O QoS, pacing,
  fairness, queue-depth, and throughput claims for S.6
- large-store memory-pressure certification lanes
- large-store pressure fixture classes for barely-over-budget,
  moderately-over-budget, far-over-budget, fragmented, protected, and streaming
  pressure
- Roadmap 2 `buffer_pool` lane family registered through the S.1
  `PhysicalScenarioQualityHarness`
- S.2 scenario definitions, lowered plans, drivers, observers, proof oracles,
  stable transcripts, and evidence bundles for every budget and lifetime claim
- persisted large-store fixtures and allocation sentinels owned by
  `worth-store-test-support`, with all proof meaning retained by
  `worth-store-certification`
- background-work envelope contracts for later Roadmap 2 sequences
- background envelope honesty limits that reserve corruption, recovery, blob,
  compaction, import/export, and repair semantics for later sequences
- Foundational performance, layout, profile, provenance, and receipt evidence
  materialized from executed Store counters
- Foundational evidence source law preventing evidence receipts from becoming
  Store runtime authority
- concrete `S3PhysicalIntegrityReadiness` payload definition

## Must Preserve

- S.1 physical byte authority remains the only ordinary source for physical page
  and frame access.
- `worth-relational` continues to own semantic runtime memory, transaction
  meaning, and truth visibility.
- Store buffer-pool state is physical residency state, not semantic authority.
- Foundational standardizes boundary meaning and evidence vocabulary, not
  Store's internal page table, allocator, cache strategy, or zero-copy runtime.
- Background maintenance, recovery, scrub, import/export, and streaming cannot
  steal foreground budgets by convention.
- Rich diagnostics and support materialization are profile-controlled and must
  not change admitted operation truth.

## Acceptance Evidence

- `bounded_memory_residency_suite`
  proves a store larger than resident budget completes admitted operations
  within exact resident-byte, pin, dirty-page, allocation, copy, and
  materialization envelopes.
- `s1_harness_extension_suite`
  proves S.2 registers `buffer_pool` lanes, drivers, observers, oracles, and
  transcript families through the S.1 Roadmap 2 harness without forking
  definition, planning, execution, observer, oracle, transcript, or evidence
  architecture.
- `buffer_pool_scenario_plan_lowering_suite`
  proves S.2 scenario definitions lower into plans that expose required
  capabilities, driver requirements, observer requirements, resident footprint,
  allocation envelope, expected counters, denial boundaries, artifact policy,
  and transcript identity before execution.
- `pin_lifecycle_and_view_scope_suite`
  proves zero-copy views cannot outlive leases and protected frames cannot be
  evicted or published behind live pins.
- `pin_drop_and_leak_honesty_suite`
  proves explicit unpin is the authoritative normal lifecycle receipt, panic or
  drop cleanup cannot fabricate normal proof, leaked pins keep frames protected,
  and pool closeout emits typed leak evidence.
- `dirty_budget_and_publication_plan_suite`
  proves dirty growth is admitted, bounded, and denied before backend write
  scheduling when it exceeds budget.
- `dirty_state_shutdown_honesty_suite`
  proves dirty resident memory is never silently discarded, reported durable,
  or promoted to S.4 recovery evidence during close or reopen behavior.
- `eviction_protection_suite`
  proves pinned, dirty-unpublished, verifier-protected, recovery-protected, and
  streaming-protected frames are not eviction candidates.
- `allocation_envelope_suite`
  proves foreground, maintenance, recovery, scrub, import/export, and streaming
  allocation scopes are separately admitted and counted.
- `fixed_metadata_allocation_exemption_suite`
  proves declared fixed metadata exemptions are constant-size and independent
  of store size, page count, record count, payload size, and diagnostic
  richness.
- `materialization_forbidden_suite`
  proves whole-store domain materialization, backend-private buffer borrowing,
  and unbounded copy paths are rejected even when the final operation result
  would otherwise be correct.
- `zero_copy_compile_fail_suite`
  proves view handles cannot outlive leases, pins cannot be released through
  the wrong lifecycle state, and protected byte access cannot be fabricated
  outside the admitting authority.
- `generation_domain_separation_suite`
  proves S.1 durable physical generation validity and S.2 resident-frame
  generation validity cannot substitute for each other.
- `conflicting_view_and_dirty_mutation_suite`
  proves immutable views block conflicting mutation, mutable views require
  exclusive lease authority, dirtying and publication deny while incompatible
  views are live, and any supported bounded-copy/COW fallback is admitted and
  counted without semantic materialization.
- `resident_memory_budget_property_suite`
  generates hostile page access, pin/unpin, dirtying, prefetch, streaming, and
  background-pressure sequences while asserting exact resident, allocation,
  copy, eviction, and denial counters.
- `large_store_pressure_suite`
  proves data size greater than memory budget is the normal certification case,
  not an optional benchmark, across barely-over-budget, moderately-over-budget,
  far-over-budget, fragmented, protected, and streaming pressure classes.
- `speculative_work_memory_admission_suite`
  proves read-ahead, prefetch, and write-behind are admitted, bounded, counted,
  denied, or deferred under S.2 memory and dirty-budget law without claiming S.6
  I/O QoS, pacing, or throughput quality.
- `background_envelope_honesty_suite`
  proves recovery, scrub, compaction, import/export, and streaming envelopes
  expose only memory, allocation, copy, denial/defer, and foreground reservation
  facts without becoming S.3, S.4, S.7, or S.10 semantic proof.
- `production_grade_large_store_ci_suite`
  runs persisted large-store fixtures through the Roadmap 2 harness with
  allocation sentinels, materialization observers, replay-comparable
  transcripts, and deterministic counter assertions.
- `synthetic_test_rejection_suite`
  proves shortcut tests that bypass lowered scenario plans, place oracle
  meaning in test support, rely on logs as proof, or compare a run only to
  itself fail certification.
- `foundational_boundary_evidence_suite`
  proves Store counters materialize into Foundational performance, layout,
  profile, provenance, and receipt vocabulary only at evidence boundaries.
- `foundational_evidence_source_suite`
  proves Foundational receipts cannot be passed back into Store as
  resident-frame, dirty-state, lease, allocation, or view-admission authority.
- `test_support_cannot_own_buffer_pool_certification_meaning`
  proves `worth-store-test-support` can supply mechanics but cannot own
  scenario meaning, proof-lane meaning, oracle verdicts, counter receipt
  interpretation, or evidence-bundle authority.
- `s3_readiness_handoff_suite`
  proves physical integrity work receives typed protected-byte access and
  bounded planning envelopes rather than raw buffers, including the concrete
  verifier resident envelope, scrub allocation envelope, inspection lifetime
  law, no-materialization witness, counters, denial behavior, and S.1/S.2
  authority recaps.

## Allowed Debt

S.2 may reserve advanced replacement policies, adaptive read-ahead tuning, and
backend-specific I/O pacing as later Roadmap 2 work when the basic admission,
counter, and denial boundaries are already complete.

S.2 may not mark these as debt:

- resident-byte accounting
- pin/unpin lifecycle enforcement
- explicit pin drop/leak honesty
- zero-copy view lifetime enforcement
- view/mutation compatibility enforcement
- dirty-page budget enforcement
- dirty publication and shutdown honesty
- allocation-envelope admission
- fixed metadata allocation exemption proof
- whole-store materialization denial
- large-store memory-pressure proof
- large-store pressure classes
- Foundational evidence materialization at boundary reports
- Foundational receipts barred from Store runtime authority
- concrete S.3 readiness payload

## Sequencing Notes

S.2 belongs immediately after S.1 because a physical page/segment/extent
substrate without resident-memory law still permits heap-shaped database
behavior. S.2 belongs before S.3 because physical integrity cannot honestly
inspect damaged bytes if the only way to inspect them is to materialize too much
state or use unprotected raw buffers.

Later sequences consume S.2 as follows:

- S.3 consumes protected byte views and scrub planning envelopes.
- S.4 consumes dirty publication and recovery memory envelopes.
- S.5 consumes lease, pin, reachability, and protected-frame vocabulary.
- S.6 consumes read-ahead/write-behind memory admission before adding I/O QoS.
- S.7 consumes streaming windows before native blob chunk trees expand them.
- S.12 consumes bounded-memory evidence as a certification lane, not as an
  aerospace-grade claim by itself.

## Required Self-Check

- Does S.2 solve a real structural problem? Yes: it turns memory residency into
  proof-bearing database law instead of cache folklore.
- Is the adversarial constraint precise and load-bearing? Yes: the store must
  operate larger than memory while respecting exact resident, pin, dirty,
  allocation, copy, and materialization envelopes.
- Does the roadmap justify this milestone now? Yes: Roadmap 2 places S.2 after
  S.1 and before physical integrity because byte shape must be bounded in
  memory before corruption, recovery, and certification can be honest.
- Does the spec preserve crate authority boundaries? Yes: Store owns physical
  memory behavior; Foundational owns shared evidence vocabulary; Relational
  owns semantic memory and truth visibility.
- Are the phases carrying most of the design information? Yes: the top sections
  frame the problem, while the phases define the structural boundaries and
  proof obligations.
- Is each phase centered on one conceptual detail or boundary? Yes.
- Does each phase contain adversarial tests? Yes: every phase includes parity
  or replay pressure plus denial, leakage, or boundary-localization tests.
- Could a competent engineer map this into honest types, modules, and tests?
  Yes: each phase names subsystems, APIs, warnings, proof obligations, and
  ownership boundaries.
- Does the milestone belong in this roadmap sequence? Yes: bounded memory is
  the required bridge from S.1 physical structure to S.3 physical integrity.
