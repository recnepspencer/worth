# Cold materialization: actual backing owner handoff

Status: IN PROGRESS. FUNDING is not landed. No requested owner below has been
edited. The ten granted residency files retain their work in progress.

## Boundary correction

The approved aggregate design is insufficient. `lease.rs::into_owned_contents`
returns/moves/clones PartitionState and drops ResidentRegion._memory. The
partition pages can survive in `full_edit`, `current_edit`, or the expanded
checkpoint converter. Keeping a ticket through the last ResidentRegion Arc does
not keep it through the last native backing owner. This is a design flaw in this
slice, not an unrelated consumer defect.

Inspection: `rg -n 'into_owned_partition|into_owned_contents'
crates/worth-relational/src` finds the Full edit, Head edit, and expanded
checkpoint extraction consumers. `storage/substrate/stored_value.rs` also
exposes payload extraction through StorageAllocation::unwrap_or_clone.
`storage/substrate/shared_column/bulk_bootstrap.rs` explicitly funds nodes only,
not the owned incoming payload. Native checkpoint conversion currently uses
ordinary constructors. Existing Extra/Metadata, ColumnNode and MapNode owners
already carry request tickets; no shared aggregate carrier was found in the
reviewed storage owners. ExecutionMemoryReservation is not Clone/splittable.

## Required actual-owner repair

Keep the concrete source/wire/verification scratch preflight through conversion.
Native arena conversion must reject inconsistent independent scalar-column
lengths before conversion, or quote each actual column length; generation count
alone does not bound malformed created_at/retired_at backing. The current
Full quote assumes that invariant although the converter checks only selected
columns. This is also in the exact arena converter handoff below.
Construct native backing with request-aware node/page/map constructors, with
payload funding reserved before each actual payload allocation. Remove the
corresponding aggregate native reservation instead of admitting it twice.
Retain only fixed ResidentRegion backing funding at the region wrapper.
No per-maintenance exception, source replay, wire change, new feature, new
public door, or request-budget increase. Root serving admission remains unchanged.

Illustrative internal signatures, subject to owner review:

```rust
fn partition_from_image_for_request(
    image: PartitionCheckpointImage,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
    request: &ExecutionRequest<'_, '_>,
) -> Result<PartitionState, ImageReadmissionFailure>;

impl EntityExtra {
    fn from_funded_parts(
        fingerprint: Option<StructuralFingerprint>,
        lineage: Option<LineageId>,
        state: Option<AuthoritativeRecordAspectState>,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Self;
}
// Corresponding RelationExtra construction; metadata already has fund_for_request.

impl<T> StorageAllocation<T> {
    fn from_reserved(value: T, memory: Option<ExecutionMemoryReservation>) -> Self;
}
```

The last is a new private native backing capability. Its ticket must live with
its actual shared allocation and survive handle clones. A raw unwrap, COW copy,
or StoredValue extraction must not make a surviving native allocation ticket
free. Receiving wire conversion has its independently admitted ticket. Audit
those closed consumers; do not claim a bare Arc wrapper alone settles that edge.
No change to lease.rs, PartitionState, current_edit.rs or full_edit.rs is proposed.

## Exact additional owner scope requested

All paths below are relative to crates/worth-relational/src. This is a coordinated
owner handoff, not permission inferred from the grant of the ten loader files.

| File | Required responsibility |
| --- | --- |
| durability/checkpoints/images.rs | Route the same native conversion algorithm through original request admission and preserve classified stops. |
| durability/checkpoints/images/arena_images.rs | Carry request admission to all native columns and metadata/extra conversion. |
| durability/checkpoints/images/arena_images/entity.rs | Admit entity payload ownership and metadata before allocation. |
| durability/checkpoints/images/arena_images/relation.rs | Corresponding relation payload/metadata admission. |
| durability/checkpoints/aspect_state_images.rs | Admit state conversion before readmission allocation. |
| storage/substrate/record_arena/values/extra.rs | Accept the original admitted payload ticket into its actual owner. |
| storage/substrate/shared_column/bulk_bootstrap.rs | Dense construction from already funded carriers, preserving their actual payload tickets. |
| storage/substrate/shared_column.rs | Sparse default construction from an already funded carrier. |
| storage/substrate/storage_allocation.rs | Per-allocation custody for diagnostics and history openings, with clone/extraction discipline. |
| storage/substrate/stored_value.rs | Closed payload-extraction integration if needed by the actual shared allocation change; no new blanket impl. |
| storage/substrate/record_arena/arena/slot_directory.rs | Request-funded restore of directory columns and map nodes. |
| storage/substrate/record_arena/arena/pin_counts.rs | Request-funded restore of nonzero pin nodes. |
| storage/partition/bitsets.rs | Request-funded restoration of the word map. |
| durability/checkpoints/images/bitset_image.rs | Route restoration through its funded word owner. |
| storage/partition/adjacency/request.rs | Funded checkpoint reconstruction and current-kind indexing; historical indexing already exists. |
| storage/residency/kind_projection/reconstruction.rs | Request-funded native COW/kind-map reconstruction after authenticated projection. |

Exact new private leaves requested, if needed to preserve responsibility/caps:
- durability/checkpoints/images/arena_images/readmission.rs
- durability/checkpoints/aspect_state_images/readmission_funding.rs
- storage/residency/region_load/materialization_funding/history_custody.rs
- storage/residency/region_load/materialization_funding/current_image.rs

The current-image leaf replaces the unmetered allocation_inventory() prewalk in
current(). Quote concrete owned wire fields using borrowed cardinalities and
charge each visited payload/membership before inspecting it. Do not compute a
whole native inventory and charge the result afterwards; already shared native
backing is neither new wire storage nor a new-allocation quote. The aggregate
native ownership repair above does not excuse this before-work contract.

The custody leaf funds the independent dependency closure before roster growth
using borrowed directory.bases, request-funded ordered collections, and a
separately funded final member array. Charge navigation and every visited edge.
Call existing RelationalDurableMemberCustody::capture / retain_members_for_request,
which funds provider custody and validates its actual issuing source. Never
fall back to legacy retain_members. The fixed RecordHistoryCustody header ticket
is separate. No graph-depth cutoff or unsupported acyclic graph reinterpretation.
The existing members() expression is still unfunded in current WIP; it is not
claimed fixed. The new leaf is requested rather than overfilling the existing
400-line modules or scattering custody into region serialization.

## Current corrections, awaiting functional proof

- Restored the backing guard on lease_resident_full; capture uses the existing
  ordinary demand_full path before same-cell DATA emission.
- Raw scalar preflight accounts InlineBytes Vec backing and explicitly lists
  the fixed scalar cases. No scalar wildcard asserts unknown ownership is inline.
- Projection preflight includes retained prefix claims and grouping scratch.
  Full quotes now include actual historical incidences, including independent
  peer-owned relations, plus local-row reconstruction before possible refusal.
  The owned Head capture is preflighted before kind_projection::capture allocates.
- FundedRows exposes its actual collection length; production fold construction
  uses it, preserving the existing refusal court's observation unchanged.

No runtime accounting proof is claimed for those source corrections yet.
The retained-wrapper lifetime, Head dependency custody, unmetered current-image
inventory walk and independently sized scalar-column quote remain open.

## Cost and proof contract

For selected Home Complete's flat regions, native allocation and reconstruction
are O(S+T) plus existing ordered-container navigation, where S includes actual
peer historical memberships, aspects and intrinsic retained metadata, and T is
the bounded uncovered tail. No covered canonical-body reads or historical replay.
General independently admitted region dependency graphs retain their separately
charged traversal/prefix work and original legality; they are not flattened by
inventing new semantics.

Before expensive qualification, add public-observation controls for InlineBytes
raw Head/history/struct denial and retry; independent dependency-closure growth;
unique/shared Head and Full extraction/edit after all original owners drop;
retired peer history with fixed local/current state; and separate root serving
admission after physical Full materialization. Then run the exact Native3,
Host2, Store recovery, cold filesystem zero-covered-read, CDC/recovery selection,
and Root/Store Clippy requested in the brief. No budget raises.

The last accepted instrumented sample remains 16,240,012 units versus old
16,236,875 at head41/covered0 (funding-state-attribution.md). This turn has no new
instrumented runtime sample. Neither number proves selected cold recapture.
