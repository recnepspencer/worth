# worth-store-buffer-pool

`worth-store-buffer-pool` is the lower physical owner for bounded frame
residency. It owns the frame table, residency identity, loading coalescence,
leases, pin state, dirty state, eviction eligibility, allocation accounting,
speculative frame accounting, and shutdown residue of one Store buffer pool.

The ordinary `worth-store` facade admits a complete
`PhysicalRecordResidencyPolicy`, constructs one pool for the physical Store
instance, and keeps pool-control capabilities private. Application code and
future physical adapters consume Store-owned record access and pressure
surfaces; they do not construct or control this crate directly.

## Authority Boundary

This crate may know:

- stable Store identity;
- physical generation, page, extent, segment, and frame coordinates;
- bounded resident bytes and metadata;
- frame, lease, dirty, operation-scope, and speculative-kind accounting;
- physical allocation, eviction, and shutdown residue.

This crate must not know:

- Query, branch, MVCC, relational, or semantic object meaning;
- Signal families, scheduler policy, executor policy, or retry timers;
- `worth-proof` authority or public transition outcomes;
- Foundational contracts, facts, patches, or semantic bases;
- aspect-native domain types;
- replay or reconstruction authority.

Those exclusions are architectural boundaries. A cache hit, miss, pin,
eviction, or pressure denial cannot create semantic truth or effect authority.

## Configuration

`PhysicalResidencyLimits::builder()` requires the complete lower declaration:

- total, resident, metadata, dirty-replacement, and operation bytes;
- frame entries, pinned frames, pin leases, and dirty frames;
- all seven `PhysicalOperationAllocationScope` ceilings;
- all three `PhysicalSpeculativeWorkKind` frame ceilings.

`admit(page_bytes)` is the only construction path for
`PhysicalResidencyLimits`. It rejects missing dimensions, invalid
category/global or scope/operation relationships, frame-count ceilings above
the frame table, and declarations that cannot hold one admitted physical page.

Production consumers should use
`worth_store::physical_runtime::PhysicalRecordResidencyPolicy`, which translates
the lower denial into the Store facade's typed denial-transition outcome.

## Runtime Ownership

One opened physical Store instance owns one pool. The pool does not spawn
workers, own queues, schedule retries, issue media effects, or settle
writebacks. Effectful misses and writebacks must travel through Store's
existing Signal, scheduler, executor, backend-receipt, and settlement path.
Signal is therefore used above this crate, not inside it.

The pool mints an opaque incarnation for that Store instance. Leases,
dirty-frame authority, and writeback claims carry it so an upper consuming
port can reject authority minted by a predecessor or different pool before it
changes state. `DirtyPhysicalFrame::pool_incarnation()` is a read-only equality
input for that comparison; it exposes no frame-table, claim, mutation,
writeback, or cleanup control. Store owns the comparison and the typed public
failure because this lower crate does not know lifecycle generations or
effect topology.

Store uses `worth-proof` for the public residency-policy admission transition,
then privately projects the admitted limits into this crate. The pool receives
neither proof authority nor a transition outcome. Physical pressure and frame
residency are not Foundational facts; Store's dedicated frame-writeback basis
remains above this boundary.

Physical operation scopes classify memory ownership only. They are not
priority, fairness, tenant, or semantic authority. Speculative kinds classify
bounded prefetch, read-ahead, and write-behind capacity; they do not authorize
background execution.

## Observation Contract

Residency counters describe executed pool transitions. Current values report
live authority; peak values report the highest live value reached; cumulative
values report named attempts, admissions, releases, denials, hits, faults,
loads, evictions, copies, dirty transitions, and writeback outcomes.
Allocation-event snapshots independently separate attempts, admissions,
releases, denials, allocator failures, and their exact units by dimension.

A denied pre-effect operation changes denial evidence but creates no fake
load, copy, or writeback completion. A resident hit changes hit and lease
evidence but creates no media-work count. Store projects these lower snapshots
into identity-bound public observation. No counter or snapshot can grant an
allocation, lease, eviction, retry, media effect, or clean transition.

## Frame Access Contract

`PhysicalResidencyPool::access_frame` decides residency before Store prepares
or executes a media read. Its exhaustive result is:

- `PhysicalFrameAccess::Hit`, which carries only a resident lease and exposes
  no source-loading method;
- `PhysicalFrameAccess::Fault`, which carries the sole move-owned
  `PhysicalFrameFaultOwner` allowed to execute one source load;
- `PhysicalFrameAccess::Coalesced`, which carries a
  `PhysicalFrameFaultWaiter` that can only await the existing loading
  identity.

The loading identity is reserved in the frame table before source work starts.
Fault ownership cannot be cloned, a waiter cannot load, and callers cannot
construct loading identities. Every participant in a failed load observes the
same typed `PhysicalFrameLoadTerminal`. Publishing any terminal wakes
participants that are already sleeping on the loading identity; terminal
retention and notification are one completion transition. Dropping the owner
or a waiter releases its exact reservation; a later access may fault again only
after the failed identity is fully reconciled.

Bounded loading identity includes the declared request limit. Only equal-limit
requests coalesce. If the same artifact is already loading under another
limit, access returns `BoundedLoadLimitConflict` with both the active and
requested limits before source work. A wider request therefore cannot inherit
a narrower owner's insufficient authority, and a narrower request is not
misreported as a physical length mismatch before length discovery. Either may
retry after the active identity resolves; resident hits are then judged
against the observed frame length.

Candidate publication uses `PhysicalCandidateFrameKey` to distinguish an
artifact fragment from a complete-artifact frame. A complete-artifact
declaration must be offset zero. Its artifact alias is reserved with the
candidate, so a concurrent bounded read is denied as
`CandidatePublicationActive` instead of starting source work. Clean
publication turns that same alias into a bounded hit; cancellation, discard,
eviction, and identity promotion remove or retarget it atomically. Fragment
candidates never acquire a whole-artifact alias.

Candidate declaration metadata and Store-side key projection are
operation-owned allocation, not free scratch. A physical owner asks
`PhysicalResidencyPool::candidate_batch_operation_bytes` for the conservative
demand of one batch. Composed owners call `begin_candidate_batch` with the
count before projecting keys, incrementing publication counters, or allocating
validation structures; its sealed admission then consumes the exact-cardinality
key slice and yields `PhysicalCandidateBatchReservation`. The one-shot
`reserve_candidate_frames` surface performs the same progression for callers
that already own a key slice. Independently live batches each consume their
own demand, including fixed validation-table overhead. The reservation borrows
the exact grant use and `reserve_next` accepts no second grant, so a
publication cannot switch scopes or spend the same grant bytes twice. Dropping
the admission or batch returns its named use; the operation-wide grant and its
global/scope accounting remain live until the owning operation ends.

Candidate declaration failures retain contract meaning:
`EmptyCandidateBatch`, `DuplicateCandidateIdentity`,
`CandidateCoverageConflict`, and `CandidateSequenceConflict` never impersonate
byte-length or residency facts. Admission against an occupied exact identity
also preserves the state it actually observed: retained failure returns its
exact `FrameLoadTerminated` terminal, loading or candidate reservation returns
`FrameIdentityOccupied`, and only a loaded frame returns
`FrameAlreadyResident`. A different complete or bounded identity for the same
artifact returns `ArtifactIdentityOccupied`.

Clean invalidation consumes only an unpinned, clean `Resident` frame. It cannot
erase a live loading identity, candidate reservation, or retained failure
terminal. Complete-artifact identity promotion likewise revalidates
offset-zero coverage and target alias availability before removing the source;
`CompleteArtifactRequiresOffsetZero` and
`ArtifactIdentityOccupied` are non-destructive denials. An exact target is
replaceable only when it is already clean resident. A retained failed load
returns its original `FrameLoadTerminated` terminal so its waiters keep one
outcome; a live loading or candidate-reserved target returns
`FrameIdentityOccupied`. These target-lifecycle denials preserve the source,
the target's authority, and all residency accounting.

## Shutdown Contract

`PhysicalResidencyShutdown` is a terminal snapshot, not a promise that handles
will reconcile later. `requires_inspection` is true when close observes any
live pool-owned allocation, load, pin lease, writeback claim, dirty or candidate
frame, or speculative grant. `has_cancellable_work_residue` is the narrower
diagnostic for operation allocations, loads, prefetch, and read-ahead. Store
uses the exhaustive inspection posture for its terminal outcome, so a surviving
facade or abandoned handle cannot make shutdown appear clean. Later handle
drops still release the live pool counters exactly once; they do not rewrite
the already-issued shutdown snapshot.

## Eviction Contract

Victim selection is deterministic for a fixed access trace. Selection skips
every pinned, dirty, loading, candidate-reserved, or writeback-claimed frame.
Only that checked selection can construct the private move-owned
`LegalEvictionVictim`; eviction and administrative clean draining must consume
that proof. No executor accepts a raw coordinate as eviction authority.

If every nominal victim is ineligible, access returns typed frame-entry
pressure before a new fault reservation or source load. If one legal victim
exists, its frame and resident-byte accounting are released exactly once. A
later access to that coordinate is a new fault and must obtain sole loading
ownership again.

## Store Composition

The Store facade owns the admitted residency policy and constructs one pool for
each serving physical runtime. Ordinary reads, candidate publication,
dirty-frame settlement, prefetch, read-ahead, and write-behind all compose this
same pool through Store-private capabilities. No parallel frame table,
snapshot-derived admission path, local worker, or compatibility facade remains.

Store publishes identity-bound residency counters, allocation events,
writeback counters, and pressure evidence. The snapshots produced by this crate
are lower mechanism inputs to that facade, not application authority.

Applications and successor features should not depend on this crate directly.
Use `worth_store::physical_runtime` for admitted policy construction, record
reads, bounded copies, pressure handling, and observation. See
[`bounded-physical-record-access.md`](../../../../plans/worth-store/bounded-physical-record-access.md)
for the current Store-facing contract.
