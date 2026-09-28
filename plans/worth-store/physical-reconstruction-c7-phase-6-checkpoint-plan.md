# C.7 Phase 6 Checkpoint Implementation Plan

Phase 7 remains locked until every Phase 6 checkpoint and retention guarantee
is proved. This plan starts from the stable artifact boundary and advances in
vertical slices; it does not preserve the unreleased recovery checkpoint as a
compatibility API.

## Destination skeleton

```text
workspaces/worth-store/crates/worth-store-physical-format/src/checkpoint/
  mod.rs
  identity.rs
  source.rs
  dirty_basis.rs
  stream.rs
  tests/
    roundtrip.rs
    hostile.rs

workspaces/worth-store/crates/worth-store/src/physical_runtime/
  durability/admission/checkpoint_start.rs
  durability/mutation/idempotency/
    bootstrap.rs
    binding_compaction.rs
    binding_compaction/
      encoding.rs
      decoding.rs
    persisted_binding.rs
    persisted_binding/
      decoding.rs
    fate/
      persisted.rs
  durability/checkpoint/
    mod.rs
    capture.rs
    progress.rs
    publication.rs
    retained_wal_tail.rs
    handle.rs
    reopen.rs
    reopen/
      binding_compaction.rs
  durability/wal/
    inventory/
      reopen.rs
      reopened_member.rs
    reclamation/
      eligibility.rs
      authority.rs
      execution.rs
      inventory_transition.rs
  instance/
    durability_bootstrap.rs
    construction/
      work_runtime.rs
      record_serving.rs
  record_serving/work_semantics/durability/
    checkpoint_capture_basis.rs
```

A directory may initially contain one file when its semantic owner is expected
to grow. No file may become a generic checkpoint bag.

The destination names are authority claims. `idempotency/fate/` owns persisted
terminal outcomes, `wal/reclamation/` owns deletion eligibility and effect,
and `instance/durability_bootstrap.rs` owns the single fresh-process join. None
may be folded into a registry, inventory, or construction catch-all merely
because that is locally shorter.

## Slice 6A: bounded dirty source

Completed. The buffer pool freezes a dirty-generation frontier in O(1), then a
move-owned session visits each fixed frame slot once. Each advance consumes a
maintenance allocation grant and returns typed unfinished or completed
progress. Later generations cannot enter passed slots; an in-range dirty frame
cannot move or be evicted before completed writeback makes that version
durable. No whole-resident metadata vector or compatibility capture API
remains.

## Slice 6B: stable checkpoint stream

`worth-store-physical-format` owns path-independent checkpoint identity and a
versioned stream grammar. The stream is a checksummed header, zero or more
independently checksummed dirty-basis records, and one checksummed footer. The
header binds Store identity, checkpoint identity, admitted begin LSN, covered
end LSN, root generation and tree identity, dirty-generation frontier, and the
concurrent-mutation posture. The footer binds record count and an aggregate
digest over the exact ordered records.

The decoder is incremental and bounded. It rejects unknown schema or record
kind, nonzero reserved bytes, identity or source substitution, truncation,
trailing bytes, record-count mismatch, digest mismatch, duplicate footer, and
records after the footer. Physical format validates structure and identity; it
does not claim checkpoint publication, namespace durability, recovery, or WAL
retention authority.

## Slice 6C: exact background work route

Store constructs `PhysicalCheckpointCaptureBasis` only from the matching root,
WAL frontier, admitted checkpoint policy, and newly frozen dirty-source
frontier. The basis binds the existing `CheckpointCapture` Signal family and
exact Foundational policy receipt. Each bounded dirty-source slice produces a
distinct append authority; only the completed scan produces footer and
publication authority. No initiation or slice type can be widened into a
terminal publication capability.

Background admission consumes the existing foreground-preservation receipt,
checkpoint pressure shape, backend capability, finite idle and debt budgets,
and the lowered queue lease before the sole Store executor can reach C4 media.

Checkpoint writes are candidate creation, bounded record appends, candidate
sync, publication replacement, and required namespace sync. Every possible or
partial effect remains inspection-required; no foreground or raw-Signal lane
can invoke checkpoint media effects.

## Slice 6D: managed lifecycle and retention

The Store instance owns every admitted handle. A handle exposes exact identity,
source range, bounded progress, pre-publication cancellation, and typed
`Completed`, `ProvenNoEffect`, or `Indeterminate` terminal fate. Dropping a
caller view abandons observation only. Close enumerates and drains Store-owned
work; no fire-and-forget thread or detached task is permitted.

Pre-publication cancellation and failed continuation reconcile the exact
staging candidate through a scheduled C4 cleanup effect. Until cleanup is
proved, a created candidate is explicit inspection-required residue; it is
never forgotten, overwritten by a later identity, or treated as published.

Publication produces the authoritative binding compaction and a canonical
nonempty retained WAL tail. WAL deletion consumes namespace-durable checkpoint
publication, exact covered range, contiguous tail, binding compaction, and the
absence of unresolved last-copy obligations. Checkpoint existence, WAL age,
file count, disk pressure, or a copied range opens no deletion authority.

### Slice 6D1: whole-Store bounded-capture siege

Before lifecycle or retention can close, the ordinary Store route must prove
that the bounded buffer-pool capture remains bounded after it is joined to
Signal scheduling, C4 media, and continuing mutation. The siege starts from
real WAL-durable data effects that hold a nonempty dirty frontier, freezes the
checkpoint source, and pauses the first checkpoint media action. While the
checkpoint remains admitted, foreground mutation completes thirty-two times
the resident dirty-frame cardinality through the ordinary WAL route.

The proof must then advance one bounded capture slice at a time and establish:

- the active maintenance allocation never exceeds the admitted checkpoint
  memory limit and returns to its pre-capture value;
- checkpoint append and publication I/O equals the independently decoded
  header, dirty-record, and footer shape rather than Store cardinality;
- the completed artifact binds the frozen root, WAL range, and dirty frontier;
- mutations admitted after the frontier remain excluded while foreground
  progress continues; and
- no whole-Store vector, frame-table traversal escape, raw pool authority, or
  substituted source can satisfy the route proof.

This is a certification slice, not a second capture implementation. It reuses
the managed handle, deterministic execution gate, ordinary mutation facade,
public counters, and independent physical-format decoder.

### Slice 6D2: managed lifecycle finalization and drain

The checkpoint worker must finalize Store-owned lifecycle truth before waking
caller observers. Terminal handle state may never race ahead of owner counters,
publication retention, current-attempt classification, or inspection posture.
The owner retains the exact worker join handle and current attempt until a later
start joins and clears the terminal worker or close stops admission, requests
safe pre-publication cancellation, and joins it. Dropping or disposing a caller
view changes observation only; it cannot detach, cancel, or forget Store work.

The focused lifecycle proof must establish:

- same-key start joins one exact attempt while a distinct key is deferred;
- pending disposal reports observation abandonment and close still enumerates,
  cancels, reconciles, and drains the Store-owned attempt;
- terminal poll and disposal return the exact completed, proven-no-effect, or
  indeterminate fate after Store finalization;
- cancellation is accepted before publication, rejected once replacement may
  begin, and converted to typed cleanup or terminal observation accordingly;
- the worker records owner terminal truth before notifying handle waiters; and
- no detached spawn, omitted join, forgotten current attempt, or close-time
  admission/cancellation bypass can satisfy the lifecycle contract.

This slice adds no second checkpoint manager, background executor, compatibility
handle, or caller-owned cancellation-on-drop convention. The serialized current
attempt is the complete enumerable live set under the admitted one-checkpoint
policy; future concurrency must replace that typed policy and its enumeration
shape together rather than silently adding another lane.

### Slice 6D3: canonical retained WAL-tail authority

The WAL owner preserves the only live physical inventory of segment artifact
identity, complete observed LSN coverage, and physical byte count. At the
checkpoint cutover it may project one immutable, non-authorizing inventory
snapshot. The checkpoint owner consumes that snapshot together with the exact
published checkpoint identity and covered-end boundary to construct
`ContiguousRetainedWalTail`; neither copied ranges nor caller-assembled segment
facts may enter the authority path.

The retained tail is built from Worth Proof `NonEmpty<RetainedWalSegment>` and
then admitted in the WAL owner's existing order as
`CanonicalVec<RetainedWalSegment>`. The Store wrapper additionally proves:

- the first retained artifact contains or ends exactly at the checkpoint
  boundary, including the lawful zero-new-record case without an empty segment
  inventory;
- every later artifact is the exact next identity in one WAL generation;
- adjacent segment LSN ranges have neither a gap nor an overlap;
- the final retained artifact covers the exact durable WAL frontier observed at
  cutover;
- the summed physical retained bytes do not exceed the admitted checkpoint
  retained-tail limit; and
- the authority remains attached to the completed checkpoint outcome and the
  Store-owned namespace-durable publication.

The public contract exposes checkpoint identity, checkpoint boundary, durable
tail frontier, retained physical bytes, and read-only exact segment facts. It
does not expose constructors, mutable collections, copied-tail identity, replay
success, or any deletion/recycling operation. Recovery physics keeps its
existing `ContiguousWalTailProof` solely as source-precedence evidence; it is
not renamed or widened into Store retention authority.

Focused evidence must include a real multi-rotation checkpoint journey whose
independent filesystem oracle reconstructs artifact identities, LSN ranges,
and byte counts without using the Store projection. The adversarial contract
must reject empty input, reordered segments, identity gaps, generation
substitution, LSN gaps, overlaps, shifted checkpoint boundaries, truncated
durable coverage, copied-range construction, and retained-byte-limit escape.
This slice adds no binding compaction and no WAL deletion effect; those remain
locked behind the next two Phase 6 guarantees.

### Slice 6D4: namespace-durable binding compaction and reopen

Checkpoint publication streams the exact retained binding set directly from
the locked registry into the checkpoint encoder. It may encode one bounded
record at a time; it may not build `Vec<Box<[u8]>>`, retain a second encoded
copy, or make memory proportional to historical WAL. The same rule governs
fresh-process reopen: checkpoint header, compaction header, footer, and one
bounded binding record may be resident while the dirty-record body is skipped.

The idempotency policy carries two independent nonzero bounds:

- pending unresolved bindings, which bounds obligations that cannot yet be
  forgotten; and
- total live bindings, which bounds the complete authoritative registry,
  including terminal fates retained by lease and compaction law.

The latest namespace-durable `checkpoint.current` compaction and the retained
WAL suffix after its exact cutoff form one reopen basis. WAL physical framing
is verified once; idempotency alone decodes binding meaning from borrowed
verified payloads. Construction must install the rebuilt owner before any
idempotency, grouping, or binding-compaction authority can be obtained. The
pre-reopen owner and post-reopen owner are different compiler-visible types.

Rebuilt pre-WAL obligations are `ReopenedUnresolved`: they may be completed by
the exact retained WAL member but may not be cancelled, resealed, or treated as
a newly prepared request. Reopen rejects noncanonical compaction order,
duplicate keys, foreign Store or policy, invalid leases, discontinuous WAL,
incomplete or substituted groups, limit overflow, and any encoded fact that
does not round-trip canonically. Durable generation advances only through one
successfully namespace-durable checkpoint publication; staged, failed,
rename-only, copied, or repeatedly observed candidates do not advance it.

Persisted terminal meaning lives behind `idempotency/fate/`. Phase 8 may extend
that closed family without reopening registry storage or compaction grammar.
Expired terminal fate remains until it was present in at least one later
namespace-durable compaction; only a subsequent compaction may omit it.

### Slice 6D5: proof-gated WAL reclamation

`wal/reclamation/` is the sole destination for segment eligibility, the
proof-bearing deletion authority, the C4 removal effect, and the resulting
inventory transition. It consumes the exact namespace-durable checkpoint,
covered range, canonical retained tail, binding compaction, and live WAL
inventory. Eligibility proves the retained tail is the exact live suffix, the
candidate prefix ends at or before the checkpoint boundary, and the binding
compaction cutoff reaches that boundary. Only then does it mint a private
`ProvenNoLiveBindingLastCopyObligation` for each exact segment. The WAL
inventory may report physical facts but may not decide deletion.

Reclamation is forbidden without that proof. Each eligible segment enters the
ordinary mutation work route under the dedicated `WalReclamation` operation
and Signal family, the `store.physical.durability.wal-reclamation-basis`
dependency/output aspect, and a named Foundational background policy. The sole
executor schedules durable removal of the canonical `families/wal` artifact
and records an exact segment/generation recovery target. Only its immutable
completed receipt permits the oldest live-inventory entry to be consumed.

The checkpoint typestate is deliberately incomplete after namespace sync:
`NamespaceDurableCheckpointPublication` becomes a final checkpoint publication
only through `with_wal_reclamation(...)`. Callers observe `NotRequired`,
`Reclaimed`, `DeferredBeforeEffect`, or `InspectionRequired` through
`CompletedPhysicalCheckpoint::wal_reclamation()`. Before-effect denial leaves
the exact inventory eligible for a later checkpoint. Any possible effect,
stale receipt, or inventory mismatch seals the WAL for inspection and cannot be
reported as reclaimed. Checkpoint existence, age, pressure, segment count, a
copied cutoff, or terminal expiry without the required later compaction opens
no deletion authority.

## Proof and cleanup

Required evidence includes incremental hostile-format tests, exact allocation
and I/O counters, the 32-times-resident mutation siege, continued foreground
progress, cancellation and close-drain seams, fresh-process compaction-plus-tail
reopen, unresolved and expired binding worlds, and causal retention mutants.

Reopen evidence additionally reports checkpoint artifact bytes, exact bytes
read, dirty-body bytes skipped, binding records read, WAL members read, and WAL
peak segment buffer bytes. Tests must prove those counters from independent
media inspection and must attack source shapes that reintroduce whole-
compaction or all-history materialization.

WAL reclamation evidence additionally includes exact-prefix filesystem
inspection, deterministic fail-before and indeterminate-after-effect deletion
faults, and a fresh-process reopen proving the reclaimed artifact stays absent
while the checkpoint-certified retained suffix reopens without an inspection
seal. The adversarial contract must kill checkpoint-only, cutoff-only,
tail-only, boundary-crossing, direct-inventory, unscheduled-delete,
unbound-recovery, completion-bypass, failure-fate-collapse, and unsafe-reopen
mutants.

Cleanup deletes the sharp/demo checkpoint path, duplicate retention decisions,
unbounded capture helpers, stale proof anchors, and superseded recovery-format
checkpoint ownership. It also deletes generation-zero production shortcuts,
memory-shaped compaction/reopen containers, direct WAL initialization that
bypasses durability reopen, terminal-fate encoding embedded in the registry,
and any deletion decision outside `wal/reclamation/`. Because the product is
unreleased, no legacy alias, adapter, or parallel checkpoint authority is
retained. Reclamation cleanup also removes no temporary inventories, copied
artifact lists, direct filesystem delete helpers, compatibility recovery
records, or duplicate retry queues: the live WAL inventory and scheduled C4
effect remain the only physical truth and mutation lane.
