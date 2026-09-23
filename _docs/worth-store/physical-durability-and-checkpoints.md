# Physical Durability And Checkpoints

## What This Feature Is

WORTH Store turns one admitted physical record mutation into a typed physical
fate. A completed mutation has crossed the WAL append and durability barrier,
settled its data effects, published one root candidate, made the root namespace
durable, and advanced the Store-owned current root. Only that completed path can
produce `PhysicalMutationAcknowledgment`.

This is physical durability, not semantic commit. The acknowledgment says that
the named physical mutation completed under one qualified backend profile and
one Store durability policy. It does not decide transaction legality, branch
visibility, Query meaning, or business-level commit.

Checkpoints are managed Store work. A checkpoint captures a bounded dirty
generation, publishes a namespace-durable checkpoint artifact, binds the exact
contiguous retained WAL tail, compacts eligible idempotency bindings, and then
reports any lawful WAL reclamation. The existence of checkpoint bytes alone
never authorizes WAL deletion.

## Why You Use It

Use this surface when a caller must:

- request platform-durable physical record publication;
- retry the same request without creating a second identity;
- distinguish completed, proven-no-effect, and indeterminate outcomes;
- observe or cancel Store-owned work without owning its execution;
- publish bounded fuzzy checkpoints and inspect their retained WAL tail; or
- close a Store only after managed mutation and checkpoint work is drained.

Do not use lower WAL, recovery, backend, or filesystem crates as an ordinary
write API. They own meaning, persisted observations, or mechanism—not the final
Store mutation contract.

## Stable Entry Points

| Entry point | Responsibility |
| --- | --- |
| `ServingPhysicalRuntime::record_submission()` | Borrow the ordinary mutation submission facade. |
| `PhysicalRecordSubmission::issue_idempotency_key(...)` | Issue the exact Store- and policy-bound retry identity. |
| `PhysicalMutationDeadline::after_milliseconds(...)` | Build a nonzero caller deadline without importing Signal. |
| `PhysicalRecordSubmission::prepare_durable_append(...)` | Admit payload, placement, request equivalence, and current runtime authority. |
| `PreparedPhysicalMutation::start()` | Start Store-owned asynchronous execution and return an observation handle. |
| `PreparedPhysicalMutation::execute()` | Start and wait synchronously for the same managed execution. |
| `PhysicalMutationHandle::{progress,poll,request_cancellation,wait}` | Observe or request cancellation without acquiring progression authority. |
| `ServingPhysicalRuntime::physical_mutation_observation()` | Read aggregate mutation, cancellation, panic, and unobserved-completion counters. |
| `ServingPhysicalRuntime::checkpoints()` | Borrow the serialized checkpoint submission facade. |
| `PhysicalCheckpointDeadline::after_milliseconds(...)` | Build a nonzero checkpoint deadline without importing Signal. |
| `PhysicalCheckpointSubmission::start(...)` | Admit or join one managed fuzzy-checkpoint attempt. |
| `PhysicalCheckpointHandle::{progress,poll,request_cancellation,wait,dispose}` | Observe checkpoint progress or explicitly abandon observation. |
| `PhysicalRecordSubmission::rewrite_selected_inline_segment(...)` | Prepare a record-preserving rewrite of the selected tail segment. |
| `PhysicalRecordSubmission::rewrite_selected_inline_pages(...)` | Prepare a rewrite of an exact live page span into a compact destination generation. |
| `PhysicalRecordSubmission::{plan_inline_artifact_rewrites,rewrite_planned_inline_artifact}` | Plan a bounded set of inline artifacts, then prepare one planned rewrite. |
| `PhysicalRecordSubmission::rewrite_selected_extent_record(...)` | Prepare a copy-on-write rewrite of one extent record (at most 256 KiB) into extent generation g+1. |
| `ServingPhysicalRuntime::retire_displaced_segment()` | Durably retire one displaced inline or extent generation that no live reader protects. |
| `ServingPhysicalRuntime::close_plan()` | Drain checkpoints and mutations, dispose Signal, close residency, and release media in order. |

## Core Mental Model

Four different things participate, and none substitutes for another:

1. C.4 filesystem qualification produces the sealed
   `PhysicalDurabilityAdmissionBasis`. It binds a Store and media generation to
   exact file-sync, directory-sync, and durable-rename capability claims.
2. Worth Proof supplies the typed `Success`, `Denied`, `Deferred`, `Stale`,
   `RebindRequired`, and `Failed` transition topology. That topology makes
   admission posture explicit; it does not establish filesystem durability.
3. Signal owns dependency scheduling, deadlines, cancellation observation, and
   lifecycle progression. A Signal phase is not physical truth.
4. Store-owned execution carries the concrete WAL, data, root, checkpoint, and
   acknowledgment facts. Only completed Store typestate opens the final
   acknowledgment constructor.

The ordinary mutation order is:

```text
admit request -> seal group -> append WAL -> settle WAL barrier
-> dispatch data -> settle data -> prepare and sync root candidate
-> replace root -> sync root namespace -> advance current root -> acknowledge
```

Failure after an effect may have happened is never rewritten as safe retry.
`IndeterminatePhysicalMutation` names the exact stage and completed-effect
count. Only a pre-group-seal denial, deadline, cancellation, or unavailable
worker can become `ProvenNoEffectPhysicalMutation`.

## Request Equivalence And Attempt Identity

The caller supplies two related but separate values:

- `PhysicalMutationIdempotencyMaterial` asks the Store to issue one leased
  idempotency key. Reusing the same material in the same admitted generation
  returns the same key identity and lease.
- `PhysicalMutationRequestFingerprint` is derived from canonical request
  meaning. It excludes deadline, allocated mutation identity, group identity,
  WAL range, and persisted attempt binding.

Allocation happens after request equivalence. The Store persists the allocated
attempt and WAL binding separately. A retry therefore cannot change request
meaning merely by receiving a new deadline, and copied attempt or WAL fields
cannot impersonate request equivalence.

Preparation can return a newly `Prepared` mutation or the already-known
`Completed`, `ProvenNoEffect`, or `Indeterminate` fate for the same identity.
Conflicting material is denied. An expired key is denied rather than silently
reissued.

## Mutation Example

This example uses only the ordinary facade. The compile suite builds this exact
block as an external consumer.

```rust
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, PhysicalMutationHandle,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, RecordAppendBatch,
    ServingPhysicalRuntime,
};

fn submit_platform_durable_mutation(
    runtime: &ServingPhysicalRuntime,
    batch: RecordAppendBatch,
    placement: AdmittedRecordPlacementPolicy,
    idempotency_material: [u8; 32],
) {
    let submission = runtime.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(
            idempotency_material,
        ))
        .expect("the Store must still be accepting mutation identities");
    let deadline = PhysicalMutationDeadline::after_milliseconds(1_000)
        .expect("the deadline must be nonzero");
    let request = PhysicalMutationRequest::platform_durable(key, deadline);

    match submission
        .prepare_durable_append(batch, placement, request)
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            observe_mutation(prepared.start());
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(
            completed,
        )) => {
            let acknowledgment = completed.into_acknowledgment();
            let _persisted_records = acknowledgment.persisted_records();
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::ProvenNoEffect(
            fate,
        )) => {
            let _diagnostic = fate.diagnostic_evidence();
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Indeterminate(fate)) => {
            let _diagnostic = fate.diagnostic_evidence();
        }
        TransitionOutcome::Denied(_)
        | TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => {}
    }
}

fn observe_mutation(handle: PhysicalMutationHandle) {
    match handle.wait() {
        PhysicalMutationOutcome::Completed(completed) => {
            let acknowledgment = completed.into_acknowledgment();
            let _cost = acknowledgment.performance_evidence();
        }
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            let _diagnostic = fate.diagnostic_evidence();
        }
        PhysicalMutationOutcome::Indeterminate(fate) => {
            let _diagnostic = fate.diagnostic_evidence();
        }
    }
}

fn main() {}
```

## Handles, Deadlines, Cancellation, And Close

`PhysicalMutationHandle` is an observer, not the owner of the mutation. Dropping
it abandons observation only; it does not request cancellation and does not
detach the Store-owned worker. A completed mutation whose last handle vanished
is counted as completed-but-unobserved and remains part of persisted fate.

Cancellation is an explicit request with exact outcomes:

- `AcceptedBeforeEffect` means the request crossed the cancellation gate before
  group seal and can settle as proven no effect.
- `SettlementAlreadyEffectful` means cancellation arrived after the safe cut;
  the Store continues to a truthful terminal fate.
- `AlreadyTerminal`, `StaleHandle`, and `RuntimeClosing` report the actual
  lifecycle posture without changing fate.

A deadline follows the same rule. It may prevent pre-effect work, but it cannot
rewrite a post-seal or possibly-effectful operation as safe to retry.

`close_plan()` stops admission, drains the current checkpoint, requests only
safe pre-effect cancellation, waits for dispatch settlement, disposes Signal,
closes residency, and releases media. `PhysicalStoreCloseOutcome::InspectionRequired`
is returned if mutation or checkpoint work is indeterminate, a worker panicked,
Signal still has in-flight nodes, residency requires inspection, or media was
not released cleanly.

## Idempotency Retention And Compaction

An idempotency lease is measured in namespace-durable checkpoint generations,
not wall-clock time. Its expiry generation is the issuance generation plus the
admitted retention count.

Expiry does not erase uncertainty:

- unresolved and indeterminate bindings remain authoritative and bounded by
  the pending-unresolved policy;
- terminal bindings become compactable only after expiry and a later
  namespace-durable checkpoint records the compaction cutover;
- a checkpoint compacts bindings through the same fenced publication that
  establishes its retained WAL tail; and
- reopen rebuilds retained terminal and every unresolved binding before new
  issuance proceeds.

This prevents a stale retry from becoming a new mutation merely because time
passed or a checkpoint file exists.

## Group Commit

Group commit shares mechanism, never identity. Each member retains its own
idempotency key, request fingerprint, mutation identity, WAL member binding,
and terminal fate. The Store validates every member before atomically sealing
the group, performs one WAL append plan and one WAL barrier for the group, and
plans one shared root publication. A cancelled or terminal member cannot be
blended into a new group.

Use `PhysicalGroupAppendAmplificationObservation` and
`PhysicalGroupBarrierAmplificationObservation` for exact group cardinality,
WAL bytes, barrier executions, and shared-root planning. Do not infer completed
data writes or acknowledgments from an earlier-stage group observation.

## Checkpoint Lifecycle

Checkpoint work is serialized and Store-owned. Starting the same active
idempotency key joins the existing attempt; a distinct key is deferred while a
capture is active. The Store retains both the exact attempt and its worker and
joins them during close.

Progress exposes the current phase, dirty frames captured, bytes encoded,
current capture allocation, peak capture allocation, and cancellation posture.
The phases are candidate creation, capture, candidate synchronization, cleanup,
publication replacement, namespace synchronization, and terminal.

Cancellation before publication can produce proven no effect if the candidate
never existed or its deletion is proved. After publication becomes effectful,
cancellation cannot claim no effect. `dispose()` explicitly distinguishes a
terminal result from `ObservationAbandoned`; dropping a checkpoint handle has
no execution side effect.

## Checkpoint Example

```rust
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, ServingPhysicalRuntime,
};

fn publish_fuzzy_checkpoint(runtime: &ServingPhysicalRuntime, idempotency_key: [u8; 32]) {
    let deadline = PhysicalCheckpointDeadline::after_milliseconds(5_000)
        .expect("the deadline must be nonzero");
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(idempotency_key),
        deadline,
    );

    let TransitionOutcome::Success(handle) = runtime.checkpoints().start(request).into_raw()
    else {
        return;
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(completed) => {
            let _bytes = completed.encoded_bytes();
            let _tail = completed.retained_wal_tail();
            let _reclamation = completed.wal_reclamation();
        }
        PhysicalCheckpointOutcome::ProvenNoEffect(no_effect) => {
            let _cause = no_effect.cause();
        }
        PhysicalCheckpointOutcome::Indeterminate(indeterminate) => {
            let _failure = indeterminate.failure();
        }
    }
}

fn main() {}
```

## WAL Retention And Reclamation

A completed checkpoint carries `ContiguousRetainedWalTail`, built only from one
immutable WAL-owner inventory snapshot while the checkpoint publication cutover
fence is held. Admission proves nonempty canonical segment order, contiguous LSN
coverage from the checkpoint boundary through the durable tail, one generation,
exact physical bytes, and the configured retained-tail byte limit.

The fence spans checkpoint replacement and namespace synchronization. Only
after that boundary may the Store evaluate reclamation. The completed outcome
reports the retained tail, binding compaction, and WAL reclamation observation;
none of those descriptive values exposes delete or recycle authority.

## Record-Preserving Rewrite And Retained Storage

A rewrite is an ordinary managed mutation. It keeps record identities and bytes
and writes canonical rewrite redo before candidate data. Preparation records
the current root generation from the root owner's snapshot. The build takes a
fresh snapshot and refuses when the generation moved, and the single pending
publication admission below keeps another publication from changing the pages
it copies. It does not hold a reader protection registration. Before the first
WAL byte, the publication owner admits:

- **Pending publication.** One root-changing publication may be pending. A
  WAL-durable predecessor ends a later attempt as `ScopeConflict` without a
  second reservation, and a published predecessor ends a prepared rewrite as
  `SourceChanged`.
- **Retained-storage growth.** The WAL group, candidate pages and root metadata
  are charged against hard byte and obligation ceilings. Progress headroom
  inside those ceilings is reserved for one checkpoint or cleanup cycle. The
  byte ceiling is widened by the declared retained WAL tail, because each
  sealed WAL group stays charged until its segment is reclaimed. The Store
  default is an 8 MiB byte ceiling and 4,096 obligations, with 64 KiB and 8
  obligations withheld as headroom. Growth may therefore use 8 MiB − 64 KiB
  plus the checkpoint policy's retained WAL tail limit, and 4,088 obligations.
  Only the tail term follows caller checkpoint policy. A request that does not fit ends as
  `RetentionPressure` with no effect.

A span rewrite publishes a compact destination generation starting at frame 0.
The source generation stays readable while the current root or any protected
root still reaches it. A completed rewrite whose displaced source has not yet
been retired is a successful publication, not a partial one.

An extent rewrite (`ExtentRewrite` mutation family) keeps the record identity,
extent identity and bytes and writes the payload as extent generation g+1 with
a new manifest. It changes neither the record count nor the inline tail. Its
redo names the extent id as both placements and digests the full payload, so
recovery can prove or rebuild the successor from the verified source.
Generation g stays readable until retired.

`retire_displaced_segment()` is the only deletion path for displaced data, and
it retires a displaced extent generation's data and manifest together. It
denies `Protected` while a live reader holds the source and `Unresolved` while
an earlier publication is still unsettled. Otherwise it claims the generation,
makes the retirement intent WAL-durable, checkpoints the successor root, then
deletes and synchronizes the namespace. The growth charge is released only after
namespace durability. Each stage names its failure (`WalWrite`, `WalSync`,
`Checkpoint`, `Delete` and so on). The intent survives WAL rotation and
checkpoint until the completion record is durable. Reopen charges retained WAL
bytes plus the overhead of the newest publications whose frames remain, so a
fresh process cannot recover growth it has not reclaimed. Displaced extent
generations are recharged by walking only the routing blocks each same-count
maintenance root wrote.

## Backend Profiles And Admission

The ordinary platform-durable policy currently admits exactly:

- `BackendTargetProfile::PosixFileFsyncDirSync`; and
- `BackendTargetProfile::WindowsFlushFileBuffers`.

Each required capability claim must come from
`EstablishedByFilesystemAdmission` and match the same profile. Raw profile
labels, probe rows, terminal reports, simulated profiles, mmap flush, and
adversarial lost/reordered-flush profiles cannot admit ordinary durability.
Rebinding also checks the exact Store and current qualified-media generation.

## Cost And Amplification Surfaces

The contract exposes costs at the stage that actually incurred them:

- mutation acknowledgment carries requested/completed bytes, transfer count,
  explicit copy count, copied bytes, and peak scratch bytes;
- group observations carry members, groups, WAL frames and bytes, one barrier,
  and shared-root planning;
- checkpoint progress carries dirty-frame count, encoded bytes, current capture
  bytes, and peak capture bytes;
- completed checkpoints carry encoded bytes, dirty-record count, retained WAL
  bytes and segments, binding-compaction counts, and WAL reclamation results;
  and
- reopen observation separates checkpoint artifact bytes, bytes actually read,
  dirty-body bytes skipped, binding records read, and WAL members read.

Ordinary publication does not hide reconstruction or full-store scanning in a
hot-path convenience call.

## Foundational, Worth Proof, Signal, And Evidence

`FoundationalPolicyAdmissionReceipt` records the scheduler resource budget
admitted for record, checkpoint, or reclamation work. It is policy evidence,
not filesystem capability, Store progression, or acknowledgment authority.

Worth Proof outcomes make every transition posture explicit. Signal schedules
the admitted work and carries its dependency and time basis. Neither can mint
WAL durability, data settlement, namespace durability, or current-root truth.

Completed, proven-no-effect, and indeterminate Store outcomes can produce
one-way executed-boundary, performance, or diagnostic evidence. Those
projections omit execution authority and are never accepted back into
admission, progression, execution, or settlement.

## Inspection And Debugging

Operators should inspect whenever any of these is nonzero or present:

- mutation `indeterminate()` or `worker_panics()`;
- checkpoint `indeterminate()` or `worker_panics()`;
- `completed_unobserved()` when callers expected to observe every completion;
- a close outcome whose `requires_inspection()` is true; or
- an indeterminate mutation/checkpoint stage whose persisted fate has not yet
  been reconciled.

Preserve the mutation identity, idempotency identity, request fingerprint,
indeterminate stage, completed-effect count, checkpoint identity, retained WAL
tail, durability policy identity, and backend admission-basis identity. Do not
retry with fresh idempotency material merely to make the alert disappear.

## Anti-Patterns

- Treating physical acknowledgment as semantic commit.
- Calling WAL, recovery, backend, or filesystem execution directly.
- Treating a deadline or cancellation request as proof of no effect.
- Cancelling by dropping a handle.
- Merging group-member identities because the group shares a barrier.
- Deleting WAL because checkpoint bytes exist.
- Deleting a displaced segment outside `retire_displaced_segment()`, or
  treating an absent file as completed retirement.
- Retrying `RetentionPressure` without checkpointing or retiring first.
- Using a raw backend profile or Foundational receipt as durability authority.
- Feeding evidence projections back into Store progression.
- Treating dirty or written-back memory as platform durable.

## Current Limits And Recovery Handoff

This owner establishes ordinary physical mutation fate, record-preserving
rewrite, retirement and namespace-durable checkpoint publication. It does not
perform fresh-process source precedence, redo, root selection, or
indeterminate-operation reconciliation; recovery owns those.

Rewrite and retirement are caller-invoked. No Store producer schedules
compaction automatically, and the retained-storage profile is not a public
configuration surface.

C.8 independently reopens sealed persisted facts: current and previous root
bases, the latest namespace-durable checkpoint and covered LSN range, the
contiguous WAL tail, retained terminal and every unresolved idempotency binding,
persisted barrier evidence, and classified partial residue. Static
configuration, backend profile, and Recovery-scoped allocation are freshly
admitted in the new process and must match those facts. The C.7
in-memory closeout handoff describes that contract for orderly closeout and
certification; it is not serialized or accepted by fresh-process recovery. C.8
does not receive the live runtime, buffer pool, Signal graph, scheduler queues,
decoded artifact graph, or ordinary execution authority.

The normative C.8 architecture, public outcome model, source precedence,
cleanup law, and courtroom are defined in
[C.8 fresh-process recovery and reopen](./physical-reconstruction-c8-fresh-process-recovery-and-reopen.md).

## Related Documentation

- [Physical WAL append](./physical-wal-append.md)
- [Bounded physical record access](./bounded-physical-record-access.md)
- [Storage foundation aspect-native gate](./storage-foundation-aspect-native-gate.md)
- [Physical reconstruction roadmap](./physical-foundation-reconstruction-roadmap.md)
