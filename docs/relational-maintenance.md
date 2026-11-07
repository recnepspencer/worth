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
`ReconstructionWork` scope. `ReconstructionLane::recover` is the explicit
recovery entrance. It keeps original cancellation, deadline, work and memory
admission. No public constructor manufactures a lane. A maintainer's retirement
input is its associated owner-specific horizon type; the contract does not
invent a retention policy or authorize pruning authoritative history.

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
