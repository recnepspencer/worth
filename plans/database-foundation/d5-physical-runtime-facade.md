# D.5 Physical Runtime Facade

> Reviewed 2026-10-06 for the fast track, which the [Database Foundation
> Roadmap](roadmap.md) replaced. Still governing: home, entry, fence, token
> identity, fate arms, capacity, checkpoint cadence and retry window, and the
> concurrency owners. Replaced: "Discovery: a named anchor" and the record ports
> become D.4's root table and a tree port, and anchor locks become per-root
> preparation. The first D.5 slice revises this note to match.

## Design
Originally slice 2 of the fast track. `S/` is `workspaces/worth-store/crates/worth-store/src/physical_runtime/`.
- **Home.** New crate `worth-store-physical-runtime`, above `worth-store` and `worth-store-recovery-runtime`, exports
  only `WorthStorePhysicalRuntime` and the ports. The old entries live in other crates, so `initialize_record_store`,
  `open_record_store` and `WorthStoreRecovery::recover` move behind a `facade-owner` feature; tests use `test-support`.
- **Entry.** `WorthStorePhysicalRuntime::open(PhysicalRuntimeConfiguration) -> Result<Self, PhysicalOpenRefusal>`. The
  configuration holds the root, format, placement, access, durability declaration, recovery limits and declared anchor
  names. Media admission returns `Empty | Store | Unrecognized`. `Empty` initializes; `Store` runs `recover`, then
  `open_record_store` with the recovered custody; `Unrecognized` is refused. Mid-tail WAL damage also refuses.
  `PhysicalOpenRefusal` and causes are `#[non_exhaustive]`, own their values; rejoin diagnostics cross as causes.
- **Ports.** Each port is owned, `'static`, Clone + Send + Sync, and holds only `Weak` references to its owner, as
  `PhysicalRecordSubmission` does today; after drain, a call returns `Released`. The runtime is Send + Sync, not
  Clone. The borrowed blob and layout facades are not exported. No port takes `&mut`; reads run on the caller's
  thread; submit prepares under anchor locks, joins the slice 5 shared owners, and waits on its own attempt.
  - Reads (`RecordReads`), each minting a `PhysicalRecordReader` on the current root lease: `anchor(&self,
    RecordAnchorName) -> Result<Option<AnchoredRecord>, ReadDenial>`; `point(&self, PhysicalRecordId,
    RecordReadLimits) -> Result<RecordReadSession, RecordReadError>`; `range(&self, RecordScanRequest) ->
    Result<PhysicalRecordScanSession, RecordScanError>`. Streaming is these Send, non-Sync sessions.
  - Submission (`RecordSubmissions`). `fence(&self) -> IncarnationFence`;
    `submit(&self, Submission, CapacityReservation) -> TransitionOutcome<SubmissionHandle, SubmissionDenial, ..>`;
    `Submission::new(&IncarnationFence, IdempotencyToken, RecordAppendBatch, AnchorAdvances, DurabilityRequest)`.
    The fence is (`StableStoreIdentity`, `RuntimeIdentity`, `LifecycleGeneration`) with no public constructor.
    `RuntimeIdentity` is random per open (`S/admission.rs:116`), so a fence kept across drain and reopen is a
    pre-effect `Stale`; `LifecycleGeneration` restarts at 1 per process. Store derives scope from `AnchorAdvances`.
    `DurabilityRequest` is `#[non_exhaustive]`, one arm `PlatformDurable { deadline }`; `SubmissionHandle::wait(self)
    -> SubmissionFate` returns `Indeterminate(RecoveryHandle)` at the deadline while the attempt continues.
  - Capacity (`CapacityPort`). `reserve(&self, SubmissionDemand) -> Result<CapacityReservation, CapacityDenial>`
    denies before allocation (RI:598); `pressure(&self) -> PressureEvidence`. `CapacityReservation { proof:
    LinearResource<ReservationId, ReservationSpent, CapacityAuthority>, lease: PhysicalInstanceForegroundReservation,
    demand: SubmissionDemandDigest }` (Send + Sync, not Clone, `#[must_use]`) is the one budget for allocation and
    `record_write` (`S/record_serving/mutation_work_port/admission.rs:61`); `submit` checks the digest before effect.
  - Fate (`FatePort`). `fate(&self, &IdempotencyToken) -> TransitionOutcome<RecordedFate, FateDenial, InFlight,
    OutsideRetention, ..>`. `readmit_token(&[u8])` checks store and policy identity and refuses a future epoch.
  - Lifecycle. `capabilities`, `require -> Result<CapabilityGrant, CapabilityAbsent>`; `drain(self)` stops admission,
    waits until every attempt is terminal, checkpoints if the tail is above the floor, and closes; Drop aborts.
- **Token identity.** Today's key hashes only the lease (store, policy, checkpoint issuance) and caller material
  (`S/durability/mutation/idempotency/registry.rs:156-166`, `key.rs:36-46`), so it can be minted twice. It now also
  mixes `RuntimeIdentity` and a per-open issuance counter, both in the token bytes. Format consequence: key, token
  and binding encodings change; the binding record version bumps; older stores are refused (disposable-store policy).
- **Fate arms.** `SubmissionFate` and `RecordedFate` are both `{ Completed, NoEffect, Indeterminate(RecoveryHandle) }`,
  carry the token identity, and have private constructors. `RecoveryHandle` holds the token and the stage (DX 11).
  - **Open terminalizes rebuilt record-append bindings** in the serving registry after rejoin: `DurableUnacknowledged`
    becomes `Completed` (a test proves redo supplies every `CompletedPhysicalMutationFact` field; otherwise it stays
    `Indeterminate`), `RebuiltUnresolved { prior: Unsealed }` becomes `NoEffect`, and every other non-materialized
    binding (GroupSealed, or a persisted `Terminal(Indeterminate)` with an Unsealed basis) stays `Indeterminate`.
  - **The Unsealed rule is sound.** `seal_group` (`S/durability/wal/port/group.rs:182`) precedes every WAL append;
    compaction snapshots the registry under its lock after reading the cutoff (`publication_cutover.rs:98`); no path
    returns a binding to Unsealed. `reserve_group` (`group.rs:153`) runs before seal, but `durable_lsn_end` advances
    only contiguously up to the appended frontier (`S/durability/wal/runtime_owner/durable_barrier.rs:11-24`), so the
    cutoff never passes a reserved, unwritten range (test: pause between reserve and seal, checkpoint, crash).
  - **Record append only.** The new binding version carries an operation kind; there is no defaulted spelling.
    Rejoin keeps today's evidence, so `selected_rejoin/wal_fate.rs:104` never sees `NoEffect`. The new test
    `rebuilt_non_record_bindings_keep_recovery_fates`, with `selected_history_tests`, pins the blob and release paths.
  - **Unknown tokens.** Fresh admission needs the key's `RuntimeIdentity` to be this incarnation's, so an earlier
    incarnation's unknown token never completes later and is `NoEffect` until it expires. `fate()` on an issued,
    unbound token of this incarnation records a terminal `NoEffect` binding under the registry lock; the counter never
    repeats, so that identity is never minted again. A bound, non-terminal token answers `Deferred(InFlight)`. Hence
    `Unresolved` never crosses: terminalizing, drain waiting and abort mapping to `Indeterminate` remove its sources.
- **Discovery: a named anchor.** An authoritative `RecordAnchorTableV1` in `worth-store-physical-format` holds
  (name, `PhysicalRecordId`, generation), sized by the declared names, referenced only by a new root-manifest rung
  (`durable_root.rs:25-29,143-159`, decode tests per rung; not in `derived_family_directory`, Arch 18). Advances
  publish in their batch's root; WAL redo carries them.
  - The expected generation is checked under the anchor lock, held from preparation until root publication or a
    pre-effect rejection (`AnchorConflict`) releases it, so two advances on one anchor serialize and never share a
    group; group seal only `debug_assert!`s that. After a post-append publication failure or abort, the lock is
    released with the anchor unsettled: advances get `AnchorUnsettled` and reads `ReadDenial::Unsettled` until reopen.
  - A restart takes three bounded reads (root, table, record), meeting RI:378 and RI:573. Undeclared durable names
    refuse the open. `PhysicalArtifactFamilyRegistry` stays closed (C.11:598); Part II data is opaque anchored records.
- **Checkpoint cadence and retry window.** `PhysicalCheckpointPolicy` gains a start threshold strictly below
  `RetainedWalTailLimit` plus declared capture headroom, since capture refuses a tail over the limit (`S/durability/
  checkpoint/retained_wal_tail.rs:145`, `publication_cutover.rs:186-191`); open refuses a configuration without that
  headroom, and capacity defers submissions that would pass the limit during a capture. Cadence starts only above
  the threshold; slice 6's K tail (fast-track:339-343) uses a `test-support` setting that disables it. Maintenance
  checkpoints such as retirement's (`S/record_serving/publication/director/retirement_progression.rs:48-57`) still
  run, since a waiting retirement could stall reclamation. Expiry already counts checkpoint generations
  (`idempotency/lease.rs:32-37`, `bootstrap.rs:211,236`); it now counts a retention epoch in the checkpoint record,
  which a below-floor checkpoint copies from its predecessor. The window is at least retention × floor WAL bytes.
- **Bug-class contracts.** Second open path: `facade-owner` gate plus the slice 7 fence. Reservation leaked or spent
  twice: the `LinearResource` moves into `submit`. Fate lost across reopen: sealed fates plus terminalizing. Unfenced
  submission: `Submission::new` needs the fence. Fence reused after drain: `RuntimeIdentity`. Generation N expected
  twice in one group: the anchor lock, `debug_assert!`ed at seal. Token identity minted twice: identity plus counter.
- **Compile-fail tests**, each with a passing counterpart: a `Submission` without a fence; a reservation used after
  `submit`; a fence or fate built outside the crate; the old entries reached through the facade dependency alone;
  `SubmissionHandle: Sync`; `submit` on `RecordReads`. **Runtime tests:** a reservation for another demand is refused;
  a pre-drain fence is `Stale` after reopen; two advances on one anchor serialize; an unsettled anchor denies reads
  and advances; a threshold without headroom is refused; one material issued twice gives two identities; a withdrawn
  token stays `NoEffect`; a below-floor checkpoint copies the epoch; the RebuiltUnsealed and reserve-to-seal cases.
- **Slice 5 concurrency.** Shared owners: WAL group commit; root publication with the anchor table; the checkpoint
  worker; the registry; and allocation, funded by the reservation, which takes `allocation_frontier`, `arenas`,
  `recovered_retiring_arena` and `recovered_copy_destination` (`director/mod.rs:161-166`). The allocation and
  registry locks are never held across a pause point or I/O. Never shared: preparation on disjoint scope, split
  into per-anchor locks taken in name order; anchorless batches take none.
- **Implementation order.** (1) In `worth-store`: gate, media probe, key identity and token bytes, fence, `Weak`
  read parts, `wait` deadline. (2) Capacity budget; allocation owner and lock. (3) Anchor table and rung, decode
  tests, redo, anchor locks, seal `debug_assert!`, unsettled anchors. (4) After slice 1: binding version, terminalizing
  with redo, non-record and reserve-to-seal tests, threshold and headroom, checkpoint-record epoch, drain checkpoint.
  (5) Facade crate (`open`, ports, refusals, compile tests), per-port tests, a three-read restart test, agent docs.
