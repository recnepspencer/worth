# Physical WAL Group Append And Barrier

## What This Feature Is

This is the Store's internal write-ahead-log stage for durable mutation. It
writes one or more prepared mutations and makes the exact group durable with
one shared file barrier. Ordinary callers start a prepared mutation and let the
Store manage this stage. Direct group progression exists only for certification
and fault testing.

## Why You Use It

- Submit durable mutations without manually sequencing WAL, data, and root
  publication phases.
- Understand how the Store shares one WAL barrier without sharing mutation
  identity, fate, or acknowledgment.
- Observe WAL activity and diagnose inspection-required outcomes without
  gaining mutation authority.
- Phase-drive exact failure boundaries in certification tests.

## Stable Entry Points

- `PhysicalRecordSubmission::prepare_durable_append(...)`
- `PreparedPhysicalMutation::start()` or `PreparedPhysicalMutation::execute()`
- `PhysicalMutationHandle::poll()`, `request_cancellation()`, and `wait()`
- `PhysicalRecordSubmission::wal_observation()`
- `CompletedPhysicalCheckpoint::wal_reclamation()`

The phase-driving methods live on
`certification::CertificationPhysicalRecordSubmission`, obtained from
`ServingPhysicalRuntime::certification_record_submission()` when the
`certification-test-authority` feature is enabled. They are test authority, not
an ordinary application API. Certification group methods consume a
`worth_proof::NonEmpty<PreparedPhysicalMutation>`; even one member uses the
group contract.

## Core Mental Model

Each prepared mutation keeps its own mutation identity, idempotency identity,
request fingerprint, redo, WAL range, and eventual outcome. Group admission
adds an immutable group identity, ordered membership digest, member ordinal,
member count, and bounded admission facts. It does not turn the members into a
transaction.

For ordinary callers, `PreparedPhysicalMutation::start()` transfers the work
to the Store-owned lifecycle. The returned handle observes progress,
cancellation, and final fate; dropping it does not cancel the mutation. The
Store continues settlement and drains owned workers during close.

Idempotency binding is terminal-aware. A same-key same-fingerprint replay
returns the existing unresolved observation or the retained
`ProvenNoEffectPhysicalMutation`; a conflicting fingerprint is denied. Before
group seal, cancellation consumes one prepared value and either proves the
exact mutation had no effect or returns the exact prepared value with a typed
denial. Once any same-binding prepared value seals into a group, cancellation
is permanently closed. Once cancellation becomes terminal, every older
prepared value for that binding is rejected before WAL reservation.

`SealedPhysicalDurabilityGroupMembers` means every admitted member was appended
and the exact ordered membership was validated. It does not mean the WAL is
durable.

`WalDurablePhysicalMutationMembers` means one real file barrier completed for
the exact sealed group. The Store then derives a separate
`WalDurablePhysicalMutation` for every member. A member proof cannot be
substituted for another member, and the group itself is never acknowledged.

`PhysicalGroupAppendAmplificationObservation` is append-stage truth:
mutations admitted, groups formed, members per group, WAL frames and bytes
actually appended, and root publications planned. Root publication is
explicitly planning truth at this stage. After the shared barrier,
`PhysicalGroupBarrierAmplificationObservation` adds exactly one executed
barrier and the exact count of members proved WAL-durable. Data writes, root
effects, and acknowledgments are not reported before their later stages
execute.

## How It Executes

```text
NonEmpty<PreparedPhysicalMutation>
  -> bounded group admission and unique identity validation
  -> ordered per-member WAL reservation and append
  -> SealedPhysicalDurabilityGroupMembers
  -> one Signal/scheduler/executor WAL barrier
  -> one completion-bound group barrier settlement
  -> exact per-member WalDurablePhysicalMutation values
```

## Segment Placement And Reopen

`PhysicalWalPolicy` admits two independent bounds. `WalSegmentByteLimit` is
the maximum encoded byte prefix of one segment. `WalSegmentInventoryLimit` is
the maximum number of segment artifacts that rotation or reopen may retain at
once. Neither is the checkpoint policy's retained-WAL-tail limit; checkpoint
publication and retention begin in the next phase.

The Store plans every member in a group before any member effect begins. If
the complete group fits the active segment, all members stay there. Otherwise
the Store discards that no-effect plan and places the complete group in one
new empty segment. A group is never split across segments. A group that does
not fit an empty segment, or would require an artifact beyond the admitted
inventory, is denied before effect with a typed reservation cause.

Each reserved member carries `PhysicalWalFrameWriteDisposition`. The first
frame in a new segment carries `CreateSegment`; later frames in that segment
carry `AppendExistingSegment`. That distinction reaches command lowering and
the C4 media owner, so creating a segment is not disguised as an append to an
artifact that does not exist.

After a partial group append, the continuation retains the exact preplanned
suffix. The Store keeps the group in flight, rejects competing group
reservations, and resumes the suffix without allocating new LSNs, changing
artifact identity, or reconsidering segment placement. Group ownership is
released only after the full suffix completes. Before-effect failure of the
first member is different: because no member effect occurred, the reservation
may be released back to admitted member authority.

Reopen first enumerates file names under the admitted inventory bound. It then
requires canonical segment names and, for each file, checks nonzero metadata
length and the segment byte limit before allocating or reading that file.
Complete-segment inspection verifies framing, digests, LSN continuity, and
cross-segment topology. Reopen repairs only final write residue that can be
placed exactly. An incomplete final frame may be truncated to the complete
frames already verified in the active segment. An empty or strictly incomplete
first frame may be removed only when its artifact is the active, exact-next,
same-generation successor of a fully verified retained segment. The WAL parser
proves the incomplete frame shape; Store topology proves that it is the lawful
successor. Neither proof alone has cleanup authority.

A lone or internal partial segment, a noncontiguous or wrong-generation
successor, a malformed complete header, and a complete frame with digest
damage remain typed `PhysicalWalOpenFailure` values and are not changed.
Noncanonical names, oversized segments, allocation failure, gaps, overlaps,
and counter overflow are likewise denials rather than cleanup candidates.

A nonempty WAL reopened without a namespace-durable checkpoint cutoff remains
sealed for inspection. When binding-compaction reopen supplies a cutoff inside
the exact retained WAL range, the Store can reopen that checkpoint-certified
suffix without an inspection seal. A cutoff outside the retained range is a
typed `CheckpointCutoffOutsideRetainedWal` failure; a checkpoint is not trusted
merely because an artifact exists.

Checkpoint publication may reclaim only the exact obsolete prefix preceding
the canonical retained tail. The deletion owner joins namespace-durable
checkpoint identity, binding-compaction generation and digest, checkpoint
boundary, exact suffix inventory, and a private per-segment no-last-copy proof.
Each artifact removal uses the dedicated `WalReclamation` Signal/scheduler
route and the C4 scheduled durable-delete effect. Inventory truth advances
oldest-first only after the matching completed effect receipt.

`CompletedPhysicalCheckpoint::wal_reclamation()` reports `NotRequired`,
`Reclaimed`, `DeferredBeforeEffect`, or `InspectionRequired`. A before-effect
denial preserves the exact live inventory for a later checkpoint. A possible
delete effect or mismatched receipt seals the WAL for inspection and never
claims reclamation. `wal_observation()` also exposes cumulative reclaimed
segment and byte counters.

If append stops before the first member, `NotStarted` retains one consuming
continuation. If earlier members appended, `PartiallyAppended` retains the
already-appended and remaining authority together. Continue either value with
`continue_prepared_wal_group`; do not reconstruct the group.

An indeterminate append or barrier means an effect may have happened. It is
inspection authority, not retry authority.

## Small Example

```rust
use worth_proof::NonEmpty;
use worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission;
use worth_store::physical_runtime::{
    IndeterminatePhysicalWalGroupAppend, PhysicalWalAppendFailureCause,
    PhysicalWalGroupAppendContinuation, PhysicalWalGroupAppendFailureCause,
    PhysicalWalGroupAppendOutcome, PreparedPhysicalMutation, RejectedPhysicalDurabilityGroup,
    SealedPhysicalDurabilityGroupMembers, WalAppendedPhysicalMutation,
};
enum WalGroupAppendDecision {
    Appended(SealedPhysicalDurabilityGroupMembers),
    NotAdmitted {
        members: NonEmpty<PreparedPhysicalMutation>,
        cause: PhysicalWalGroupAppendFailureCause,
    },
    AdmissionRejected(RejectedPhysicalDurabilityGroup),
    Continue(PhysicalWalGroupAppendContinuation),
    Inspect(IndeterminatePhysicalWalGroupAppend),
}

fn append_wal_group(
    submission: &CertificationPhysicalRecordSubmission,
    members: NonEmpty<PreparedPhysicalMutation>,
) -> WalGroupAppendDecision {
    match submission.append_prepared_wal_group(members) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => {
            WalGroupAppendDecision::Appended(appended)
        }
        PhysicalWalGroupAppendOutcome::NotAdmitted { members, cause } => {
            WalGroupAppendDecision::NotAdmitted { members, cause }
        }
        PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected) => {
            WalGroupAppendDecision::AdmissionRejected(rejected)
        }
        PhysicalWalGroupAppendOutcome::NotStarted(continuation)
        | PhysicalWalGroupAppendOutcome::PartiallyAppended(continuation) => {
            WalGroupAppendDecision::Continue(continuation)
        }
        PhysicalWalGroupAppendOutcome::Indeterminate(indeterminate) => {
            WalGroupAppendDecision::Inspect(indeterminate)
        }
    }
}
```

This is a certification example for deterministic boundary tests. It keeps
every authority-bearing value in the branch that owns it. `NotAdmitted`
retains raw prepared members. `AdmissionRejected` retains an exact rejected
group. Retryable work retains a consuming continuation.

In certification code, call `synchronize_appended_wal_group` after `Appended`.
On `Durable`, iterate `into_members()` and drive the next certified boundary.
On `BarrierNotStarted`, retain the sealed group and retry only after its typed
cause is resolved. On `Indeterminate`, inspect; do not resubmit the barrier as
proven no-effect. Ordinary code should call `start()` or `execute()` instead of
performing these transitions.

## Real Example

Use a member projection and the WAL observation when recording operational
evidence:

```rust
fn inspect_wal_append(
    submission: &PhysicalRecordSubmission,
    appended: &WalAppendedPhysicalMutation,
) -> Result<(), &'static str> {
    let declaration = appended.reserved().declaration();
    let settlement = appended.settlement();

    assert_eq!(settlement.range(), declaration.artifact_range());
    assert_eq!(
        settlement.payload_digest(),
        declaration.payload_digest(),
    );

    let wal = submission
        .wal_observation()
        .ok_or("the serving Store has released its publication authority")?;

    assert!(wal.appended_frames() >= 1);
    assert_eq!(
        wal.last_lsn_end(),
        Some(declaration.lsn_range().end_exclusive().get()),
    );
    assert!(!wal.sealed_for_inspection());
    Ok(())
}
```

Obtain the member projection from
`sealed.members()[index].mutation()`. The declaration is the planned member
identity; the settlement is the completed append effect. The observation is a
bounded read-only summary and cannot append, retry, unseal, or advance any
member.

## How It Relates To Other Features

- Durable mutation preparation defines each member before grouping begins.
- The physical work runtime supplies Signal readiness, bounded scheduling,
  execution, and effect settlement without becoming WAL authority.
- Data dispatch consumes individual `WalDurablePhysicalMutation` values, never
  a group-level durability receipt.
- The group carries an explicit future root-publication plan, but current root
  replacement remains a separate feature.

## Inspection And Debugging

`wal_observation()` reports the current segment and generation, appended frames
and bytes, reclaimed segments and bytes, valid prefix, last LSN end, and whether
allocation is sealed for inspection.

For append failures, inspect `PhysicalWalGroupAppendContinuation::cause()` plus
its appended and remaining member counts. For indeterminate append, inspect the
group basis, appended count, uncertain member, and unstarted count. These
projections explain retained authority; they do not grant retry permission.

For barrier failures, `PhysicalWalGroupBarrierFailureCause` preserves the exact
declaration, work-admission, scheduler, command, or media cause. A successful
group settlement exposes one group identity, membership digest, member count,
work identity, effect identity, and physical barrier.

Call `SealedPhysicalDurabilityGroupMembers::amplification_observation()` after
append and
`WalDurablePhysicalMutationMembers::amplification_observation()` after the
barrier. These are immutable stage observations. They cannot execute the
planned root publication or collapse per-member fate into a group
acknowledgment.

## Anti-Patterns

- Do not acknowledge or settle the group as one mutation.
- Do not use certification phase-driving methods in ordinary product code.
- Do not use one member's durable proof for another member.
- Do not rebuild a continuation from member fields.
- Do not retry an indeterminate append or barrier as though it had no effect.
- Do not bypass the Store through a backend WAL planner or direct filesystem
  synchronization.
- Do not infer semantic transaction ordering from group order.
- Do not report planned root publication as an executed root effect.
- Do not fill future-stage data-write or acknowledgment counters with
  misleading zeroes.
- Do not delete from a checkpoint-existence, age, pressure, or file-count
  heuristic; none carries last-copy authority.
- Do not consume live inventory before the exact scheduled deletion receipt.
- Do not retry an indeterminate deletion as though it were proven no-effect.

## Current Limits

- Group append shares the WAL barrier only. Shared root publication is planned
  explicitly but is not executed by this surface yet.
- Data dispatch and later acknowledgment remain per member.
- Ordinary callers cannot phase-drive WAL append, barrier, or data settlement;
  the Store-owned mutation lifecycle performs those transitions.
- An opened Store containing pre-existing WAL bytes without a matching
  namespace-durable checkpoint cutoff remains inspection-bound until recovery
  establishes safe continuation.

## Related Docs

- [Bounded Physical Record Access](./bounded-physical-record-access.md)
- [Durable Publication Join](./physical-reconstruction-c7-durable-publication-join.md)
