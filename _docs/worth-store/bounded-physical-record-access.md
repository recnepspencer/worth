# Bounded Physical Record Access

## What This Feature Is

Use bounded physical record access when a Store must serve records without
letting its live physical-memory use grow with the database. You declare one
memory envelope before opening the Store. Reads then return borrowed chunks or
copy into caller-provided buffers, and pressure is reported as typed evidence.

Physical residency answers which encoded Store bytes are currently in memory.
It does not mean a Query result, branch view, or MVCC snapshot is resident.

## Why You Use It

- Keep a Store instance inside a known physical-memory envelope.
- Give foreground, recovery, maintenance, verification, and blob work explicit
  operation ceilings.
- Reject incomplete or page-incompatible configuration before a buffer pool is
  constructed.

Use the canonical policy when its fixed envelope is appropriate. Supply an
explicit policy when deployment capacity or a physical adapter needs different
limits.

## Stable Entry Points

Most callers need these exports from `worth_store::physical_runtime`:

- `PhysicalRecordResidencyPolicy::builder()` to declare the envelope;
- `PhysicalDurabilityDeclaration::builder()` to declare durability after
  filesystem-media admission;
- `PhysicalRecordOpen::with_residency_policy(...)` or
  `PhysicalRecordInitialization::with_residency_policy(...)` to attach the
  admitted policy;
- `MediaOwnedPhysicalRuntime::open_record_store(...)` to consume the open
  request and produce the serving Store outcome;
- Fallible `ServingPhysicalRuntime::records()` to capture a protected root,
  then `PhysicalRecordReader::open(...)` to start a `RecordReadSession`;
- `RecordReadSession::{next_chunk, read_next}` for borrowed or bounded-copy
  access;
- `ServingPhysicalRuntime::physical_allocations()` for successor physical
  adapters that must charge Recovery, Scrub, Maintenance, Verification, or
  Blob operation bytes to this Store generation;
- `ServingPhysicalRuntime::residency_observation()` for read-only runtime
  inspection;
- `RecordReadError::pressure()` and `RecordAppendError::pressure()` for typed
  physical-pressure evidence.

The supporting public types include `PhysicalRecordChunkView`,
`PhysicalRecordChunkBasis`, `PhysicalRecordPressureEvidence`,
`PhysicalResidencyRetryPosture`, `PhysicalResidencyObservation`, and the
counter/allocation snapshots returned by that observation.

`PhysicalRecordInitialization::new(...)` and `PhysicalRecordOpen::new(...)`
require the admitted durability policy derived from the exact admitted
filesystem media. There is no implicit durability policy and no constructor
that lets a caller omit it.

This guide starts after physical runtime and filesystem-media admission.
`open_record_store` is the public handoff between the configured open request
and the `ServingPhysicalRuntime` used below; the linked C.5.1 runtime guide
covers that broader admission sequence.

Pool construction, frame tables, eviction controls, allocation grants, and
lower residency snapshots are not application APIs. Store owns them and
publishes only read-only Store evidence.

Prefetch, read-ahead, and write-behind controls are not ordinary application
entry points. Successor physical adapters may request one of the five exact
operation scopes through `physical_allocations()`. That surface charges
temporary bytes and binds them to the current Store runtime; it does not expose
pool operations or grant recovery, integrity, isolation, maintenance, or blob
policy.

## Core Mental Model

The physical record format owns page shape. The residency policy owns how much
live physical memory the Store instance may use. Store admits the policy
against the already-admitted format, then consumes both values when it creates
the instance's single buffer pool.

There are three forms:

1. `PhysicalRecordResidencyPolicy` starts a declaration.
2. `PhysicalRecordResidencyPolicyBuilder` is incomplete configuration and
   grants no runtime authority.
3. `AdmittedPhysicalRecordResidencyPolicy` proves that every required
   dimension was declared and that the declaration is internally consistent
   with the physical page size.

The admitted type is sealed. Callers cannot construct or forge it, and
initialize/open do not accept the raw builder. These are type boundaries, not
developer conventions.

After serving begins, `PhysicalResidencyObservation` reports the exact stable
Store identity, lifecycle generation, admitted policy, counter snapshot, and
allocation-event snapshot for that instance. The observation is evidence, not
authority: it cannot allocate, pin, evict, retry, write back, or alter limits.

Read and append pressure follows the same rule. `RecordReadDenial` and
`RecordAppendDenial` classify the result as `PhysicalPressure`; the enclosing
error exposes `PhysicalRecordPressureEvidence`. That evidence describes the
pre-effect world that rejected the work. It does not prove that a later retry
is safe and cannot be exchanged for runtime authority.

An operation scope identifies which physical activity owns temporary bytes. It
does not express priority, fairness, tenant identity, or semantic authority.
Speculative kinds distinguish prefetch, read-ahead, and write-behind capacity;
they do not create background workers or retry policy.

Physical adapters consume a deliberately narrow basis: borrowed validated
bytes, their logical range, stable Store identity, Store generation, physical
record identity, durable physical owner, frame coordinate, pressure, and retry
posture. They may also hold an exact successor-scoped allocation while their
temporary bytes remain live. Those values are enough for integrity, isolation,
and blob layers to bind later physical work to the correct Store generation.
They are not proof that bytes satisfy an integrity policy, belong to a stable
semantic snapshot, or form a complete blob. Those meanings remain owned by the
successor that defines them.

## How It Executes

### Physical Adapter Boundary

Live byte adapters are exported from
`worth_store::physical_runtime::stability`, not
`worth_store_physical_isolation`. Use `PhysicalByteGuard::from_record_chunk`
to bind an admitted lower read plan to a real Store chunk;
`StablePhysicalReadExecution` checks the guard and security-scope bindings and
owns the resulting byte-access counters and
`StablePhysicalReadReceipt`. Ordinary execution evidence is current, freshly
retained evidence, not replay-derived evidence.

The page security-scope adapter consumes a decoded page-header witness and a
manifest slot envelope. Before exposing bytes, execution checks the decoded
header and exact page coordinates/generation and manifest slot owner against
the guard scope. Equal generations alone are insufficient: a carrier from
another segment, page, or slot returns `LogicalDecodeScopeCarrierMismatch`.

Lower read planning remains in `worth_store_physical_isolation`.
`StablePhysicalReadHandle::complete_plan()` returns only
`PhysicalReadPlanCompletionReceipt`: local plan counters and release facts.
It cannot be converted into Store byte execution evidence. B-tree and lower
checkpoint/compaction interlocks consume that local completion without
inventing a zero-byte Store execution.

### Residency Admission

1. Admit the physical record format.
2. Declare every residency dimension with nonzero values.
3. Call `admit(format)`.
4. Handle the typed outcome.
5. Attach only the admitted policy to initialization or open.
6. Store constructs one lower buffer pool for that Store identity.
7. During serving, inspect only Store-owned residency and pressure evidence.

Admission rejects:

- any omitted dimension;
- a byte category above `total_bytes`;
- a scope above `operation_bytes`;
- pinned, dirty, or speculative frame counts above `frame_entries`;
- resident, operation, or dirty-replacement capacity smaller than one physical
  page.

## Small Example

After filesystem-media admission, admit the deployment durability policy
against that exact media. Pass the resulting move-owned `durability` value
when constructing the request:

```rust
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordOpen,
};

let format = AdmittedPhysicalRecordFormat::admit(
    PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
);
let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
let request = PhysicalRecordOpen::new(format, access, durability);
```

This is the minimal honest request construction. A policy admitted for a
different Store root or qualification basis is rejected before serving effects
can begin. There is no raw default policy that can bypass admission.

## Real Example

This example declares a complete instance envelope and attaches the admitted
result to an open request:

```rust
use std::num::{NonZeroU32, NonZeroU64};
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, PhysicalOperationAllocationScope as Scope,
    PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration,
    PhysicalRecordOpen, PhysicalRecordResidencyPolicy,
    PhysicalSpeculativeWorkKind as Speculation, RecordReadError,
    ServingPhysicalRuntime,
};

let bytes = |value| NonZeroU64::new(value).unwrap();
let frames = |value| NonZeroU32::new(value).unwrap();

let format = AdmittedPhysicalRecordFormat::admit(
    PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
);
let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();

let residency = PhysicalRecordResidencyPolicy::builder()
    .total_bytes(bytes(65_536))
    .resident_bytes(bytes(16_384))
    .metadata_bytes(bytes(8_192))
    .frame_entries(frames(8))
    .pinned_frames(frames(8))
    .pin_leases(frames(2))
    .dirty_frames(frames(4))
    .dirty_replacement_bytes(bytes(16_384))
    .operation_bytes(bytes(16_384))
    .scope_bytes(Scope::ForegroundRead, bytes(16_384))
    .scope_bytes(Scope::ForegroundWrite, bytes(16_384))
    .scope_bytes(Scope::Recovery, bytes(16_384))
    .scope_bytes(Scope::Scrub, bytes(16_384))
    .scope_bytes(Scope::Maintenance, bytes(16_384))
    .scope_bytes(Scope::Verification, bytes(16_384))
    .scope_bytes(Scope::Blob, bytes(16_384))
    .speculative_frames(Speculation::Prefetch, frames(8))
    .speculative_frames(Speculation::ReadAhead, frames(8))
    .speculative_frames(Speculation::WriteBehind, frames(4))
    .admit(format)
    .into_result()
    .expect("deployment residency policy must fit the admitted format");

let request =
    PhysicalRecordOpen::new(format, access, durability)
        .with_residency_policy(residency);

fn inspect_residency(serving: &ServingPhysicalRuntime) {
    let observation = serving.residency_observation();
    let metadata = observation
        .allocations()
        .for_dimension(
            worth_store::physical_runtime::PhysicalResidencyDimension::MetadataBytes,
        );

    assert_eq!(observation.store_identity(), serving.store_identity());
    assert_eq!(
        metadata.active_units(),
        observation.counters().metadata_bytes(),
    );
}

fn inspect_read_pressure(error: &RecordReadError) {
    if let Some(pressure) = error.pressure() {
        eprintln!(
            "residency pressure: scope={:?} dimension={:?} requested={} limit={}",
            pressure.scope(),
            pressure.dimension(),
            pressure.requested(),
            pressure.limit(),
        );
    }
}
```

The format remains authoritative for page size. The admitted residency value
proves configuration completeness and consistency. The durability value proves
that a complete durability declaration was admitted against this filesystem
media. The request retains both proofs until Store consumes them. The
application never receives frame-table, eviction, or allocation-grant
authority.

In production code, match the policy outcome and report
`PhysicalRecordResidencyPolicyDenial` rather than using `expect`.
After open, `inspect_residency` uses only Store-owned evidence. The pressure
handler reads the pre-effect basis without treating it as permission to retry.
Append failures expose the same evidence through `RecordAppendError::pressure`.

## Reading Records Without Owning Them

`records()` returns `Result<PhysicalRecordReader, PhysicalReadProtectionDenial>`.
It registers protection and selects the root under the same publication lock.
Later publications do not change that reader's membership: a record introduced
after capture is `RecordNotFound` to the old reader, and a scan consumes the
reader and continues against that exact root. This is physical root stability,
not a Query or MVCC snapshot.

Each acquisition reserves a slot, even when another reader protects the same
root. Point sessions share their reader's registration and can outlive the
parent; only the final reader/session release returns the slot. Ordinary
acquisition and release do not walk the root's descendants or read media.

```rust
let reader = serving.records().expect("read protection admission");
let root = reader.protected_root();
let mut session = reader.open(record, limits).expect("record read admission");
drop(reader); // The session still protects the captured root.
let chunk = session.next_chunk().expect("stream progress");
```

Production callers handle acquisition and record-read failures separately.
`ProtectionLimit` means all reader slots remain held; `RetainedRootLimit` means
another distinct protected root cannot fit. Retrying requires a new acquisition
after the blocking owners release. Configure both ceilings with
`PhysicalReadProtectionPolicy::new(...)` and `with_read_protection_policy(...)`
on initialization or open. Defaults are 64 acquisitions and 16 protected roots.
Metadata is preallocated before initialization effects; failure is reported as
`RecordBootstrapDenial::ReadProtectionUnavailable(MetadataUnavailable)`.

`read_protection_observer()` reports live counts and can look up an issued
`protected_root()` in the actual root index. Neither that observation nor a
zero blocker count authorizes reads, retirement, or deletion.

Close and abort stop new calls and drain already-admitted read calls through
the existing execution gate, including warm reads that issue no physical I/O.
Implicit runtime drop also revokes future access. Already borrowed chunk
memory remains valid until its borrow ends. Explicit shutdown reports
`RetainedUntilReadersRelease` when protection registrations survive shutdown;
it does not falsely report those resources as drained. The observer can track
their eventual release without retaining execution or publication authority.

`PhysicalRecordReader::open` and `open_external` return a
`RecordReadSession`. The session owns the read lifecycle, operation allocation,
logical cursor, and at most one current resident frame. It is not an owning
record buffer.

Borrow decoded payload directly when the consumer can finish with one chunk
before advancing:

```rust
use worth_store::physical_runtime::{
    RecordReadSession, RecordStreamFailure,
};

fn consume(mut session: RecordReadSession) -> Result<(), RecordStreamFailure> {
    while let Some(chunk) = session.next_chunk()? {
        let basis = chunk.basis();
        consume_payload(
            chunk.bytes(),
            chunk.logical_range(),
            basis.record(),
            basis.frame_coordinate(),
        );
    }
    Ok(())
}
```

An inline record yields one chunk. An extent yields one decoded payload chunk
per resident frame. `bytes()` excludes frame headers, extent metadata, and
neighboring inline slots. `logical_range()` names the exact range of the
logical record represented by those bytes.

The chunk mutably borrows its session. The compiler therefore rejects
advancing, copying from, moving, or dropping the session while either the
chunk or its borrowed byte slice remains live. The chunk and its basis have no
public constructors. A caller cannot fabricate them from bytes, coordinates,
generations, counters, or pressure evidence.

`PhysicalRecordChunkBasis` is observation only:

- `store_identity()` identifies the stable physical Store;
- `store_generation()` identifies its admitted serving lifecycle;
- `record()` identifies the logical physical record;
- `physical_owner()` identifies the durable generation owner that minted the
  record bytes;
- `frame_coordinate()` identifies the exact durable artifact range. The
  artifact identity carries its durable generation.

The basis does not expose the pool incarnation, frame lease, allocation grant,
pin, eviction, fault, writeback, retry, or mutation authority.

Use `read_next` when caller-owned storage is required:

```rust
fn copy_bounded(
    mut session: RecordReadSession,
    target: &mut [u8],
) -> Result<(), RecordStreamFailure> {
    assert!(
        !target.is_empty(),
        "a bounded-copy buffer must distinguish progress from end of record",
    );
    loop {
        let count = session.read_next(target)?;
        if count == 0 {
            break;
        }
        consume_copy(&target[..count]);
    }
    Ok(())
}
```

`read_next` never allocates an owning result. It copies only into the supplied
slice and records the actual nonzero copy operation, byte count, and maximum
copy width. `next_chunk` records no copy. The methods share one cursor, so
interleaving them neither repeats nor skips bytes.

Store deliberately provides no whole-record `read_to_end`, `to_vec`,
`into_bytes`, `Arc<[u8]>`, or compatibility conversion. A caller that chooses
to accumulate chunks owns and budgets that separate allocation.

Chunk projection adds no Signal family, Foundational fact, or `worth-proof`
authority. A real cold fault still uses the existing private `ReadFault`
topology; a resident hit and borrowing decoded bytes create no synthetic work.
The safety proof is the concrete `RecordReadSession` ownership boundary plus
the Rust borrow lifetime.

## Successor Physical Allocation

Recovery, scrub, maintenance, verification, and blob adapters sometimes need
temporary memory that must count against the same Store envelope. Admit those
bytes through the Store runtime, not through the buffer pool:

```rust
use std::num::NonZeroU64;
use worth_store::physical_runtime::{
    PhysicalScopedAllocationFailure, RecoveryPhysicalAllocation,
    ServingPhysicalRuntime,
};

fn admit_recovery_bytes<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    bytes: NonZeroU64,
) -> Result<RecoveryPhysicalAllocation<'runtime>, PhysicalScopedAllocationFailure> {
    runtime.physical_allocations().admit_recovery(bytes)
}
```

The returned allocation borrows the runtime, records the exact scope and byte
count, and releases the charge when dropped. It cannot outlive or close over
the runtime. Selecting Recovery does not grant recovery authority; the C.7-C.11
owner still supplies the real operation, policy, and downstream proof.
On denial, `PhysicalScopedAllocationFailure::{kind, reason, pressure}` exposes
the Store classification, exact cause, and available pressure evidence. The
failure is descriptive; retry still requires a fresh admission after the
reported condition changes.

## Allocation Admission And Materialization

Allocation admission and allocation materialization are different boundary
facts. Admission charges capacity and grants the right to attempt one bounded
allocation. Actualization records the concrete allocation produced under that
admission; it cannot create allocation authority after the fact.

Metadata admission is unscoped and charges the concrete metadata capacity.
Its actualization preserves both the requested table size and the actual
charged capacity. Resident and dirty-replacement admission is scoped and
charges the requested reservation before allocation; actualization may report
fewer concrete bytes but never more than that reservation.

The allocation trace preserves this order mechanically. Every metadata,
resident, and dirty-replacement actualization must consume a matching earlier
admission from the same dimension and scope. Final counter equality is not
enough: a reordered trace that materializes first and admits later is invalid
even when every aggregate balances.

Resident allocation is additionally sequenced by ownership. Store-private
fault admission produces the move-owned fault owner that alone can invoke the
allocator. Observation snapshots describe this progression but grant no
allocation, pool, retry, Signal, Foundational, or `worth-proof` authority.

## Concurrent Cold Reads And Fault Coalescence

When concurrent read sessions request the same nonresident frame, the Store
does not start two physical reads. The buffer pool first reserves one loading
identity. The first request receives move-owned fault authority; later
requests receive wait-only attachments to that same identity.

Only the fault owner can discover the source length, allocate the admitted
frame, or execute the source load. A waiter can only wait for the owner to
publish the admitted frame or its terminal failure. Successful readers
therefore receive two valid leases over one frame after exactly one fault and
one source load. If the owner fails, every waiter observes that same terminal
loading failure instead of starting a fallback read.

The one real miss still uses the private `ReadFault` Signal, scheduler,
executor, backend, and Store-settlement path. Attaching a waiter creates no
Signal request, physical-work identity, media read, or second allocation.
Fault coalescence is physical work sharing for one Store generation and frame
identity; it is not semantic request deduplication, a retry policy, or a
persisted cache contract.

Use `residency_observation().counters()` to inspect this behavior. One
overlapping pair should add one fault, one source load, one coalesced waiter,
one pinned frame identity, and two pin leases while both views remain live.
The leases still obey the ordinary session lifetime and release rules.

## Generation And Incarnation Fences

Every Store-owned physical-work capability captures the
`LifecycleGeneration` that admitted it. The first consuming boundary compares
that generation with the Store's current serving generation before allocation,
pool admission, gate admission, Signal declaration, scheduler admission, or
media work. A predecessor capability therefore fails exactly as
`PhysicalWorkExecutionFailure::PreEffect(PhysicalWorkAdmissionFailure::StaleGeneration)`;
it cannot become valid merely because the requested frame is already resident.

Pool-owned frame authority is fenced independently. Each opened Store owns one
opaque pool incarnation, and its leases, dirty-frame authority, and writeback
claims carry that incarnation. Dirty admission first requires the lease's exact
Store/runtime/generation binding before invoking the mutation closure or
changing dirty state; a mismatch fails as
`PhysicalDirtyTransitionFailure::StaleOrForeignFrame`. The frame-writeback port
then compares the dirty authority's carried pool incarnation with the consuming
Store before recording a writeback attempt, claiming a frame, declaring Signal
work, entering the scheduler, or attempting media. That mismatch fails exactly
as `PhysicalWritebackFailureCause::StaleOrForeignDirtyFrame`.

The fence consumes real authority, not caller-supplied coordinates, generation
numbers, counters, or observation snapshots. Those values may describe a
denial but cannot pass it. Rejected dirty admission releases the rejected lease;
rejected writeback returns the dirty-frame authority so the caller can discard
it honestly. `PhysicalRecordChunkBasis` still exposes neither pool incarnation
nor any pool-control capability.

These fences add no Signal family, Foundational fact, or `worth-proof`
authority. Lifecycle ownership and opaque pool incarnation are lower
mechanical identities; the existing Store work topology remains the only path
to an effect.

## How It Relates To Other Features

- Format admission happens first because residency must fit at least one page.
- Placement and access policies remain separate. They do not imply memory
  capacity and cannot substitute for residency admission.
- Scheduler capacity controls admitted physical work; the residency policy
  controls memory. Neither can substitute for the other.
- A cold miss and a dirty-frame writeback use Store's existing physical-work
  runtime. Hits and pre-effect pressure denials create no fake work.
- Recovery, scrub, maintenance, verification, and blob work have distinct
  memory scopes, but their policies belong to those successor features. This
  feature grants them no direct pool control.
- `worth-proof` governs policy admission above the pool. Foundational and
  Signal describe other boundaries above the pool. None of them turns cache
  state or counters into semantic truth.
- Integrity, isolation, durability, recovery, QoS, and blob completeness are
  separate guarantees. A chunk basis gives those features physical identity
  and generation context; it does not prove their policy.

## Inspection And Debugging

Before attaching the policy, inspect its public getters to confirm the admitted
values. If admission is denied, match the typed denial:

- `MissingRequiredDimension` names the omitted declaration.
- `CategoryExceedsTotal` identifies the byte category above the global
  declaration.
- `ScopeExceedsOperation` identifies the oversubscribed operation scope.
- `CountExceedsFrameEntries` identifies the frame-count relationship.
- `PageExceeds*` identifies which byte category cannot hold one admitted page.

During serving, call `residency_observation()`:

- `store_identity()`, `store_generation()`, and `admitted_policy()` identify
  the exact instance and envelope being observed.
- `counters()` exposes Store-owned current/peak byte and frame counts plus hit,
  fault, eviction, writeback, copy, denial, and speculative-work totals.
- `allocations().store_identity()` identifies the same Store.
- `allocations().for_dimension(dimension)` exposes exact attempts, admissions,
  releases, denials, allocator failures, admitted/released/denied units, and
  current active units for one dimension.

For a denied read or append, inspect `pressure()` on the error. The evidence
names the Store/optional record basis, Store generation, allocation scope,
pressure dimension, requested/admitted/limit values, retry posture, and
whether an effect may have started. `pressure_denial()` gives append callers
the unit classification without embedding evidence in the denial enum.

## Anti-Patterns

- Do not treat the raw builder as an admitted configuration.
- Do not choose `total_bytes` by summing independent maxima and assume all
  combinations are executable; it is one live envelope.
- Do not use scopes as priorities, tenants, or semantic categories.
- Do not request successor-scoped bytes from general application code or treat
  a scoped allocation as permission to perform successor work.
- Do not make a second pool for recovery, verification, or blob work.
- Do not import `worth-store-buffer-pool` from an ordinary application or
  future adapter.
- Do not use pressure evidence as a retry token or allocation grant.
- Do not infer mutable pool control from a residency observation.
- Do not start a fallback source read when an overlapping session is waiting
  on an existing cold fault.
- Do not infer semantic residency, data completeness, or durability from
  physical frame residency.

## Current Limits

- Stable: admitted instance envelopes, bounded ordinary reads, borrowed chunk
  views, explicit bounded copies, Store-owned observation, typed read/append
  pressure, dirty-frame settlement, and bounded speculative work.
- Stable: readers protect one captured physical root; `RecordReadSession`
  retains that registration and at most one current frame. There is no owning
  whole-record convenience or direct pool-control API.
- Stable for physical adapters: exact Recovery, Scrub, Maintenance,
  Verification, and Blob allocations borrow the Store runtime and grant only
  bounded temporary-byte ownership.
- Certification-only: direct speculative controls and fault-driving probes.
- Not provided here: WAL/checkpoint ordering, crash reconstruction, integrity
  admission, semantic stable reads, QoS policy, or blob protocol.

## Related Docs

- [Physical WAL Append](./physical-wal-append.md)
- [C.6 Buffer-Pool Runtime Join](./physical-reconstruction-c6-buffer-pool-runtime-join.md)
- [C.5.1 Physical Store Work Runtime](./physical-reconstruction-c5-1-physical-store-work-runtime.md)
- [Physical Foundation Reconstruction Roadmap](./physical-foundation-reconstruction-roadmap.md)
