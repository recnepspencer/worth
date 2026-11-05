# Choose a native representation

Start with independently mutable facts, names and links. A serialized Rust
structure does not make its members native fields or its identifiers relations.
An application imports `worth-query-decl` and `worth-query-host`; here `decl::`
means `worth_query_decl::facade::` and `host::` means
`worth_query_host::facade::`.

## Representation table

| Meaning | Representation and current qualified entry |
|---|---|
| Independently referenced or edited object | `decl::worth_query_entity!` and `application_schema::ApplicationSchemaDeclarationBuilder::entity`; declare a unique semantic key with `unique_field`. Native owns the occurrence, partition and generation. |
| Link between objects | `decl::worth_query_relation!` and `ApplicationSchemaDeclarationBuilder::relation`. A key or `EntityRef` hidden in another value does not provide relation topology or dependency tracking. |
| Fixed tuple of references | Separate named typed relations for each semantic role; no stored position is needed. |
| Flat facts | `decl::worth_query_aspect!`, `decl::worth_query_field!`, then builder `aspect` and `field` for flat Struct fields. For a qualified primitive whole-aspect scalar, use `scalar_aspect_field`. These are distinct layouts. |
| Alternative record classes | A required scalar class field and one `optional_aspect` per class, with required payload fields and an installed commit invariant. See the candidate-only pattern below. |
| Keyed set or graph | Named entities, unique keys and typed membership/adjacency relations. No artificial position; declare traversal and negative dependencies. See [source subgraphs](source-subgraphs.md). |
| Cycle with Native orbit meaning | `decl::application_schema::ApplicationRelationIntegrity::orbit`, the ordinary typed relation declaration, and `decl::application_query::ApplicationQueryOrbitRef`/query builder `orbit`. Host decision/invariant `orbit` reads retain the actual native cycle contract. |
| True ordered sequence | Classify it first. This lane qualifies no positioned-sequence helper or relation-field carrier. Report the actual ordering/edit/inverse requirement before choosing an unqualified representation. |
| Immutable, identity-free ordered numeric content, including splines at every size | A format-bound `ContentRef` field in a flat Struct aspect, declared through the typed content binding. Native owns admission, integrity, retention and reconstruction; Host content reads and candidate writes use that owner. Frames above 16 MiB refuse before effects. No whole-aspect content shape exists. |
| Identity-free, reference-free leaf with no platform scalar | An owner-admitted, named/versioned atom. Today the qualified fallback is an explicit canonical String field or original-text envelope, with its domain policy and reason recorded. The separate inline binary value-binding work is not qualified here. |
| Compiled pack-owned parameters awaiting a finer carrier | An explicit canonical atom exception with owner, wire shape, bounds/policy and reason. Its guard must reject hidden independently mutable objects or references; it is not a graph container. |
| External input/output document | JSON at a named ingress or terminal projection boundary; admit ingress before native effects. Terminal text grants no native authority. |

The builder names in the table belong to
`decl::application_schema::ApplicationSchemaDeclarationBuilder`.
`BoolApplicationValueBinding`, `I64ApplicationValueBinding`,
`U64ApplicationValueBinding` and `StringApplicationValueBinding` live in
`decl::application_schema`. Other scalar families require an actual entry-owned
`ApplicationScalarValueBinding` and readable binding, or the qualified
`decl::worth_query_value_binding!` scalar bridge. There is no generic built-in
Rational binding asserted here. The numerical owner still admits exactness,
normalization, capacity, units, frames, tolerances and certificates; a valid
native scalar alone does not prove those domain laws.

No new list, variant or nested `AspectShape` form is implied by this table.

## One class, one optional aspect

The [one-of fixture](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/aspect_patterns/one_of/mod.rs)
declares a required class field in a required flat aspect and three optional
class aspects. Each class aspect's fields remain Required. An installed
`host::application_invariants::WorthQueryApplicationInvariantRule` checks the final
candidate: exactly the named class aspect must be present. Required content is
also enforced by the installed native contract validator.

Resolve invariant fields with
`host::application_invariants::WorthQueryApplicationInvariantSchemaResolver::typed_field`.
In the candidate, a Required field under an absent Optional aspect reads `None`;
a missing Required field in a present aspect refuses. The invariant typed field
reader currently supports Struct children; this does not qualify whole-scalar
aspect reads through that invariant view.

Prepare scope from the class field **and every class-aspect field** with
`host::application_invariants::WorthQueryApplicationInvariantReadView::touched_entities`.
The current planner
reports touched entities by kind, including aspect removal; it does not promise
field-filtered scope. Keep the explicit complete field inventory so additions
cannot silently escape the rule. Checked preparation/evaluation must retain the
original work and cancellation checks.

The operation declares its reads, writes and aspect-removal grants using
`decl::worth_query_operation!`. Through `host::primary_graph::DecisionReader`
and `CandidateWriter`, change the class, remove the old aspect and initialize
the complete new aspect in one transaction. `CandidateWriter::remove_aspect`
and `write_field` participate in the same final candidate. Removing the named
aspect alone, adding another alone, changing only the class, or omitting a
required payload all refuse through the real commit path.

This is **candidate-only**. SDK revision `c478117afa` refused the lawful fixture
because invariant reads did not distinguish Optional aspect absence from
missing Required content. The candidate fixes that private reader and composes
the entity's aspect effects. The pattern requires the planned repin; there is
no weakened at-pin variant. Courts also cover actual correction, retirement,
native recovery, original cancellation and an expired original deadline.

The directional correction retains the class plus old First value: 18 logical
bytes including scalar tags. It does not qualify arbitrary absent-field or
graph-wide preimage capture.

## Absence cost and the class scalar

The [cost court](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/cost/aspect_absence/court.rs)
installs real schemas with 3, 6 and 12 Optional class aspects and no class tag.
It uses the existing `execute_in_program_report` decision projection counters
and independently reads the actual committed Sink value.

| Captured decision projection work | 3 classes | 6 classes | 12 classes |
|---|---:|---:|---:|
| Known present class field, including Sink observation | 2 | 2 | 2 |
| Known absent class field | 1 | 1 | 1 |
| Exhaustive discovery, including Sink observation | 4 | 7 | 13 |

Every mode records one equality lookup and one index candidate. Other captured
projection categories are zero. These are acquisition counters, not physical
I/O, total retained native facts, invalidation breadth or per-read latency.

For callers that already know which class to read, absence work is flat and
**the class scalar could go**. Discovering the active class by reading every
aspect remains linear. Retain or remove the tag according to that actual access
pattern and an installed exclusivity rule, not a guessed absence penalty.

## Canonical atoms retain whole-value grain

Use `decl::application_schema::WorthValue` through its explicit derive facade
path for an application-owned value. Adopt its generated checked seed with
`decl::worth_query_value_binding! { ... scalar: String, canonical, ... }` only
when the complete value is an intentional atom. `native_envelope: Envelope`
delegates a pure owner's existing original-text document codec. A pure domain
crate may own the underlying value without importing Query.

The installed grammar can describe recursive structures, enums, arrays,
sequences, pairs and opaque delegates. Those grammar paths are **not native
field paths**. Read, replace, touch, invalidate and serialize the whole canonical
atom at its enclosing native field; there is no inner-field patch. Current
installation refuses this structured canonical carrier in
`ApplicationSchemaDeclarationBuilder::scalar_aspect_field`; use a Struct field
for the qualified canonical binding path.

Record the owner, binding identity/version, shape, domain bounds/policy, reason
and no-hidden-reference guard for each exception. Use existing owner policy;
do not invent arbitrary ceilings. The [Store grain contract](../plans/worth-store/storage-foundation-aspect-native-gate.md#current-value-grain-contract)
states the actual read/patch, touch, conflict, invalidation and serialization
boundaries, including conservative widening and reader-specific decode controls.
