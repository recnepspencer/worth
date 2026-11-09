# Source as a native subgraph

A source can be several independently named entities joined by typed relations.
Declare the subgraph's boundary before declaring its projection or fingerprint.
An anchor aspect alone does not describe the graph reachable from that anchor.

Here `decl::` means `worth_query_decl::facade::` and `host::` means
`worth_query_host::facade::`. The
[small public fixture](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/aspect_patterns/source_subgraph/mod.rs)
uses these two application facades. Its source is a root with two named members.
Its generated result is another root with two members. Both topologies are fixed;
ordinary installed operations can change values, but cannot change membership.

## Declare and observe the complete source

Use `decl::application_schema::fragment::{Entity, Aspect, Relation, Record, Scalar}`.
Fragments list entities, entities list aspects and outgoing relations, and binding
sets supply each row’s columns.
Unique semantic keys name the fixture's nodes; Native retains their physical
occurrence and generation. A key stored inside a value does not replace a link.

The family's source contract specifies:

- Its root selector and ownership/context boundary.
- The entity kinds, field meanings and relation kinds/directions it observes.
- Its membership closure, including absent fields, missing edges and empty
  adjacency lists on which a decision depends.
- Cardinality, duplicate-name and dangling/foreign-target refusals.
- Canonical ordering for identity-free projection output, without inventing a
  stored position for a graph or keyed set.

The fixture declares its read using
`decl::application_query::ApplicationQueryResultRelationRef::forward_many` and
`ApplicationQueryResultShapeBuilder::relation`. Its Host projection reads
`host::primary_graph::WorthQueryApplicationProjectionRow::many` and each member's
native key/value, then orders the transient rows by unique key. The projection
is a DTO for this declared one-level graph, not durable source text and not a
general recursive graph walk.

For another topology, extend the declared read closure and its checks. A field
on the root cannot certify omitted descendants. A topology-changing command must
observe the affected membership and negative facts through the native reader,
then publish its typed relation changes atomically. Do not infer those facts from
a cached projection or a digest.

## Edit and retain previous state

Declare only the required field/entity/relation grants with
`decl::worth_query_operation!`. Read through
`host::primary_graph::DecisionReader`, retain its typed mutation targets, and
write through `host::primary_graph::CandidateWriter`. The fixture's member edit
writes one `SourceValue`; its sibling, root and relation records remain intact.

`host::application_entry::WorthQueryApplicationRequest::retain_current_read`
and `request.at(&observation)` retain a genuine previous native observation.
The query declares `decl::application_query::ApplicationQueryBasisSupport::current_and_pinned()`.
Reading the complete source at that observation gives the previous graph while
a fresh current query gives the edited graph. Retaining the DTO alone does not
retain native read or mutation authority.

The recorded inverse uses
`decl::application_aftermath::ApplicationCorrectionBinding`,
`DeclaredPreImageDemand` and `DeclaredPreImageLocus::from_field`. The owner retains
the actually observed, mutated member value; the inverse restores that value
through the real correction preparation/publication path. One UInt64's logical
retention width is nine bytes, including its scalar tag.

This qualifies a member edit and its undo. The current recorded-field demand
requires present mutated fields on one common record. It is not an arbitrary
multi-record graph snapshot, absent-field retention or a general topology inverse.
Declare a supported inverse for each edit meaning; refuse unsupported meanings
instead of reconstructing missing prior truth from current state.

## Fingerprint meaning and retain provenance separately

The fixture fingerprints a transient canonical value containing a versioned
format, node/field meanings, the directed membership meaning, and the complete
`GraphRow`. `decl::application_schema::ApplicationScalarValueBinding::encode`
uses the existing bounded canonical codec; `decl::identity::hash_parts` hashes
that single canonical frame through the existing SHA-256 implementation. It
stores neither this text nor this hash as graph authority. Root key/value and
the ordered member key/value roster participate. A member-only change changes
the content fingerprint; equal graphs in independent installations match.

The identity supplied to
`host::application_contribution::WorthQueryApplicationProducerProvider::idempotency_key`
is a native source-epoch identity, not that portable content fingerprint. The
court observes distinct native identities for equal graphs in separate runtimes.
The producer's complete typed `GraphRow` also passes through Query's canonical
operation-input identity machinery. Neither identity replaces the native reads.

A family with an application-owned content fingerprint must frame and version
its complete meaning: semantic names, field identities and canonical values,
relation kinds/directions/endpoints, and any meaningful absence. Sort sets by
their declared canonical keys; preserve true sequence order when that is part of
the meaning. Include units, frames and other domain meaning when present. Use
the owning canonical codec/digest service, with its admitted controls and typed
refusals; do not invent a second serialization authority. The small fixture's
descriptive hash call is a test-side bounded calculation, not a qualified
original-request-controlled production fingerprint service.

A content identity is descriptive. It neither grants mutation authority nor
replaces native observations of actual occurrences. The fixture checks the
producer's projected input against independent native decision reads before
building its candidate. Equal content across two runtime placements can have
the same content identity while retaining distinct native provenance.

## Cover generated members explicitly

The parent declares three exactly-one entity roles using
`decl::application_operation::WorthQueryApplicationOutputRole` and
`decl::application_operation::ApplicationMutationOutputContract`. The candidate
binds the root and both children with `CandidateWriter::create_output` and
authors the actual typed membership links. Omitting a child's required role
refuses even when that child and its link were staged.

Native's output witness covers every installed aspect of each declared entity
role. It does not discover descendants or cover relation/adjacency facts merely
because one role names a root. In this fixture, the generated topology is closed:
only the generating operation can create/link Generated records, it can obtain
only freshly created Generated handles, and no installed operation can relink,
unlink or delete them. A separate operation can change a Generated value, but
holds no topology grants. This is why the fixed topology plus the three entity
witnesses cover this generated graph.

The court retains a genuine parent output operation, changes only a child value,
and requires `deliver_current` to refuse while the source and readiness-selected
root remain unchanged. A literal graph roster readback is independent of the
generator's correspondence and input fingerprint.

For mutable or discovered generated topology, retain actual membership/negative
observations or install a complete topology rule with the corresponding scope.
Every live/retired member must participate in the declared output contract.
An optional output family with minimum zero is not proof that an omitted member
does not exist. The fixed fixture does not qualify arbitrary reachability,
mutable relation currentness or graph-wide recorded undo.

## Variable authored membership

The same source-subgraph host also declares a variable authored member kind with
`ApplicationAuthoredCorrection::owned_members`. Its exactly-one member-to-anchor
relation is the sole live membership authority. The existing authored identity,
recorded field/relation pre-image, Native retirement event and restoration owners
carry the complete inverse. No producer publication is required for an authored
roster, including an empty one. See the [authored retirement
walkthrough](build-an-application.md#source-owned-producer-publications) for output
families, total byte bounds and the typed exact-roster refusals. The recorded
pre-image is historical inverse material, never a second live graph.
