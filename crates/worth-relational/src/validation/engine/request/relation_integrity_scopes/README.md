# Relation-law source scope

Storage owns the maintained witness: `kind_admission::maintained::KindMaintainer`
is the synchronous `Maintained` adopter for the current kind membership of both
persistent adjacency directions. Its `CommitChange` route updates only journal
slots; its independent reconstruction requires `ReconstructionLane`. Invariants
consume that witness through `InvariantStateView::try_kind_relation_candidates`.
There is no second graph catalog or ordinary cold reconstruction lane here.

A proposal needs the *affected source set*, not every visible source. The
physical candidate journal supplies changed identities/kinds/endpoints,
including generated writes. Explicit commands add revalidation and pre-apply
created references. For connectivity, incoming traversal in both images expands
those seeds through every indirect predecessor; successor traversal then reads
only their affected closure. Incidence is selected by relation kind even when
unrelated kinds share an endpoint. The changed planned edges are indexed once
for both directions rather than rescanned at every visited vertex. Acyclicity needs the forward closure of proposed
edge targets. Metadata checks kind, visibility, endpoint and generation; an
adjacency slot alone never establishes entity identity. Field-only writes with
unchanged identity/kind/endpoints do not dirty reachability.

The predecessor is required for removed/retargeted records and reused slots.
The applied candidate supplies surviving reachability. Planned created keys
remain logical before application; applied source selection uses the actual
issued identities. A refused proposal publishes neither its mutation nor its
source scope.

The existing branch basis has already satisfied the installed laws. Schema
transition admission requires unchanged executable contracts
(`runtime/schema_transition_admission.rs`); the source witness therefore remains
valid across an admitted transition. Explicit whole-state validation, which has
no semantic plan, still enumerates its requested source population and reads
successors on demand. It does not construct a cold whole-model adjacency map or
serve as an ordinary missing-witness fallback.

The original request charges preparation through `ChangeWork` under
`relation_source_witness`. One reservation travels with the immutable prepared
scope and covers its owned traversal buffers until the last packet drops it.
The inspection recorder is bounded diagnostic evidence and is never accepted
as graph or validation authority. SDK courts compare the actual selected source
and successor sets with an exhaustive edge-list oracle, including refusals,
retargeting, deletion and restoration; the scale court doubles unrelated
components, retained history and installed kinds independently.
