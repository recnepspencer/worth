# Actual native backing funding implementation

Boundary: cold Head/Full and folded history are original-request DATA. Serving
root admission stays unchanged. The granted handoff replaces wrapper-only
native funding with existing per-page/node/Extra/Metadata tickets. lease.rs,
PartitionState and both edit owners are unchanged. No wire/version/budget change.

1. Keep one native image conversion algorithm, parameterized by optional
   original request. Existing ordinary image input retains its existing contract;
   both residency loaders take the request-funded entrance. Preserve request
   denials separately from content corruption. Validate every SoA column length
   before constructing native columns, including all scalar/pin/extra arrays.
2. Fund slot directories, bitsets, pin maps, SharedMap/Column nodes and conversion
   scratch before allocation. Dense construction accepts already funded carriers
   so payload tickets are not lost through from_value reconstruction. Extra and
   Metadata carry their actual payload/fixed tickets. Aspect readmission quotes
   concrete state and contract clones before invoking foundational validation.
3. StorageAllocation gains a private ticket beside T inside its actual Arc
   allocation for diagnostics and RecordHistoryOpening. Clone retains that Arc.
   Existing node/Extra/Metadata own their existing tickets, not a second outer
   ticket. Audit raw extraction/COW: funded diagnostics are shared/replaced by
   runtime edits and exported only into pre-funded wire projections; openings
   are immutable retained prefix owners. No edit extracts those payloads into a
   surviving unfunded native owner. Preserve existing ordinary operations and
   allocation identity observations; no generic authority or request side door.
4. Both projection rebuild and its native COW/kind-map writes use request-aware
   existing table/map operations. Charge verification traversal incrementally.
   Quote temporary expected/grouping sets independently, including peer history.
5. Head dependency closure uses borrowed directory bases and request-funded
   ordered collections/member output, then existing provider custody capture.
   Replace the late allocation_inventory walk with concrete incrementally charged
   current-image fields. Keep source/wire/fold scratch until conversion finishes.
6. Remove the replaced aggregate native ticket, retaining only actual fixed
   ResidentRegion control. Verify raw decode, valid extraction/edit/freeze and
   last-owner refund using external Native courts, plus independent malformed
   scalar, peer-membership and dependency closure denial/retry cases.
7. One coherent owner compilation before the batched requested proofs. Repair
   failures honestly at the owner, no fixture budget increase. Full proof set,
   phase gates, independent review and FUNDING LANDED commit. Merge integration
   with master only afterwards, rerun required affected tests/proof set, review
   and FUNDING MERGED. Then complete the ordered queue.
