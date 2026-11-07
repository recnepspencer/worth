# Incremental SDK maintenance

This is an SDK maintainer contract. Applications continue to use Query's
declaration and host facades. The Relational SDK namespace is
`worth_relational::facade::storage::maintenance`.

`CommitChange` is constructed once during canonical root preparation from the
storage owner's partition journals and that publication's canonical envelope.
It carries before/after record identity, kind, lifecycle and retirement facts;
created and retired identities; canonical field, posting and endpoint touches;
exact required symbols and their retained meanings; and the schema-change fact.
The prepared root and candidate share one `Arc<CommitChange>`. Candidate
consumption does not turn this descriptive value into publication authority.
Readmission does not invent a change when the predecessor journal is absent.

A `Maintained` owner declares its install-time semantic routes and consistency.
Its persistent state is carried by `Arc`. `apply_change` refuses a missing
predecessor and returns the same Arc when no route intersects the change.
Dispatchers keep their installed reverse routing tables and offer only the
owners reached by the change. They do not scan every maintained object.
The `ChangeWork` scope admits work and memory through the original SDK request.
There is no ordinary fallback to reconstruction.

A cold implementation requires a sealed `ReconstructionLane` and a
`ReconstructionWork` scope. Only the exact trust-boundary files registered in
`tools/boundary-check/config/reconstruction-callers.txt` may reference the
`enter_reconstruction` constructor. There is no crate-wide exemption:
`BC7014_RECONSTRUCTION_LANE_SEAL` refuses ordinary callers, including aliases.
The registry covers cold restore/readmission, checkpoint import/export,
first-region initialization and the generic certification oracle. Native unit
oracles exist only under `cfg(test)`. Possessing a lane permits
`lane.recover(execution, operation)`, which retains original cancellation,
deadline, work and memory admission. Every full kind rebuild, projection,
history-support capture and commitment reconstruction requires that lane by
type. A maintainer's retirement input remains its owner-specific horizon;
the contract invents no retention policy or authority to prune history.

`DigestIndex` is the existing storage Patricia commitment, generalized from
partition-content maintenance. Its tree shape and digest depend only on keys
and values. Cold construction rejects duplicate or unordered keys. Ordinary
updates copy only the affected persistent paths. Existing partition-content
commitment bytes are unchanged by this relocation.
Like the other storage primitives, it delegates admission to its owning
maintainer. The owner prepares the affected paths through `ChangeWork`, or the
full reconstruction through `ReconstructionWork`, before using the primitive.

## Kind commitment adopter

Each resident region carries persistent head and historical kind trees. A
relation leaf binds its kind and separate outgoing and incoming slot trees.
Publication uses only journal-selected adjacency differences and changed
record-history cells. Shared storage owners prune untouched subtrees.
Publication with no relation or adjacency changes retains the same kind-state
Arc. Unchanged kind trees stay shared when historical support counts change.

Persistent support counts distinguish repeated historical incidence from its
last supporting row. Retirement, suffix replacement and restoration update
only changed metadata cells; deleting one support cannot remove another row's
incidence. These counts accelerate maintenance and are not commitment material.

The historical tree's previous persistent root is its carried frontier. A new
history cell extends that state without serializing or hashing its old prefix.
This uses a keyed Merkle frontier rather than a linear hash of successive
commits: the canonical commitment must remain a function of the projected set,
independent of the sequence of edits that produced it. Record-content history
continues to use its existing `HistoryAppendFrontier` for omitted cold prefixes.

Cold restore reconstructs local historical incidence from authenticated record
metadata and checks peer claims against the selected region commitment. It
checks independently of the incremental kind trees. Checkpoint/recovery code
keeps the full reconstruction as the oracle.

The kind encoding uses `region-kind-authority.v3` and region frames use version
7. Version 6 frames are unsupported and are refused; their bytes are never
reinterpreted as the new commitment. This is the current-format-only window of
the undeployed Relational persistence format.

## Courts

`worth-testkit-relational::maintenance::MaintainedCourt` compares an applied
state with its independent cold reconstruction. The same equality comparison
rejects a dropped route or a corrupted carried state. Its growth assertion
compares charged work across independent model, history and unrelated-routing
axes with headroom, rather than pinning exact counts. Each adopter supplies
real owner inputs and its actual cold implementation.
Inspection can call `facade::inspection::observe_maintenance_work(maximum, operation)`
around just the measured operation. It returns bounded `{ phase, charged_work,
accounted }` observations from actual original-request Execution meters.
`root_admission` encloses canonical root preparation, regions and digest work;
`kind_commitment` identifies the first adopter. Future owners use
`ChangeWork::run_phase(execution, "owner_phase", operation)` for the same seam.
Totals are inclusive: nested phases overlap their enclosing phase, so use the
request report for whole-operation work. Native-only rows have `accounted: false`.
The production inspection recorder is thread-local, isolated under nested
captures, bounded by the requested maximum, and cleared on return or unwind.

The public scale journey also guards the real SDK commit's inclusive
`root_admission` charged work, separately doubling the relation roster and the
retained history of one relation while keeping the measured endpoint edit
fixed. It uses `assert_change_bounded`; timings remain telemetry. A full
rebuild inserted after the production kind update fails this court, even
though the isolated incremental engine itself remains unchanged.

Composite initial-region and checkpoint image/member capture doors also require
both a `ReconstructionLane` and the original `ExecutionRequest`. They do not mint
lanes. Their projection traversals run through `lane.recover` and the same
controlled admission engine as cold kind reconstruction; request refusals remain
typed request failures. Only the no-predecessor publication branches, explicit
checkpoint import/export, and closed corruption recorders mint at those callers.
