# Composite history contract

Runtime World history has two separate meanings:

- `CompositeBasisKey` binds one Runtime World owner to the exact
  owner-issued Relational, Signal, and Bridge admission identities. It is the
  composite equivalence key; descriptors are descriptive only.
- `CompositeCommitIdentity` names one immutable commit occurrence.

Equal bases may therefore appear in distinct commits. A commit carries its
owner-issued identity, one `CompositeCommitParent::Root` or `Ordinary`, the exact admitted
composite basis, Relational and Signal change postures, changed-owner
publication identities, the admitted Bridge basis, root/publication
provenance, and optional descriptive caller correlation. Caller correlation
does not authorize a commit.

`ProductBranchIdentity`, lifecycle incarnation, reference generation, and
selected commit remain separate meanings. `ProductBranchObservation` compares
all of them together with the exact owner-issued composite admission. A branch
name, commit id, generation, digest, or descriptor alone is not a product-head
observation. The product reference selects its retained exact tuple even if a
component owner has since advanced. Only a requested component mutation checks
that owner's current exact basis; unchanged components require no owner contact.

The same exact owner-issued tuple has the same `CompositeBasisKey`; no separate
lookup index authorizes equivalence. Equal descriptors with distinct component
admission identities do not compare as the same composite basis.

Root bootstrap is the only operation that can establish the initial product
reference. Later history is single-parent in this milestone. The mutable
product reference is not the immutable commit, and history insertion alone is
not a product-reference movement.

Ordinary publication reserves a canonical performed envelope beside its history
entry before component effects. Its logical metadata charge includes the
preallocated facts and a conservative per-envelope charge for the retained
shared branch name; it is not a measurement of unique allocator-resident bytes.
The immutable commit remains separate from the later movement. The envelope
retains the exact old/new snapshots, full component results, transfer receipt,
late cancellation, and final publication counters only after the cell commits.
It is reclaimed with its entry, with final evidence destructors outside the
history lock. A live delivery claim carries explicit history protection; the
envelope stored inside the entry does not protect itself.

Admission allocates stable pending entry and reachability slots and indexes
them in owner-local hash maps. Reservations carry those slots into promotion;
installation never searches an ordered tree or inserts/grows a storage index.
Keyed admission, lookup, and removal have expected/amortized O(1) indexing cost,
not a worst-case hashing guarantee. Hash-table growth happens during admission
before component effects; table capacity is bounded by peak admitted occupancy.
The metadata ledger charges the slot payloads, shared controls and carried
reservation handles; it remains a logical charge, not allocator-resident bytes.
The `reserved_entry_writes` counter measures direct slot installations. Logical
lookup counters do not measure hash collisions or resizing; profile timings
include that physical work. Pending slots count once against capacity but remain hidden
from lookup, protection, parent admission, and reclamation. Installation fills
those slots in place, preserving the parent dependency acquired at reservation.
Reservation Drop removes both pending slots and releases its charge. Ordinary
publication performs promotion inside the branch's final comparison lock;
a stale comparison leaves the reserved storage uninstalled.

Exact branch reuse installs the source observation's existing commit while its
source reference still matches, without a new commit or owner effect. Explicit
fork plans create the requested component branches and a single-parent composite
commit; performed forks remain recoverable if the destination cannot install.

Retirement accepts an existing `ProductBranchObservation` as proof of the
installed occurrence. It compares owner, name, and incarnation, allowing later
head movement within that occurrence. An older occurrence cannot retire a
recreated name. The live registry needs no historical retired-name set.

## Public history and maintenance

`inspection_port().trace_ancestry(commit, maximum)` returns a bounded managed
traversal. It protects its selected start and ancestry while live; drop it before
requesting reclamation. History is an explicit inspection lane, not an ordinary
publication scan. Head generation is readable through `reference_generation().get()`;
the scalar cannot manufacture a branch observation or advance a reference.

An intermediate owner effect can install a retained **unpublished** successor
occurrence without changing any product reference. Its parent and exact component
bases remain inspectable, but history presence is not performed authority. If pin
binding was denied, the partial instead retains its reserved capacity and need not
have an installed successor. Cleanup releases protection; explicit bounded history
reclamation removes eligible unreachable occurrences. Reachable ancestry and live
observation/attempt/partial protections are never pruned to make space.
