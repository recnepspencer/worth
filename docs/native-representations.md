# Choose a native representation

Start with independently mutable facts, names and links. A serialized Rust
structure does not make its members native fields or its identifiers relations.
An application imports `worth-query-decl` and `worth-query-host`; here `decl::`
means `worth_query_decl::facade::` and `host::` means
`worth_query_host::facade::`.

## Representation table

| Meaning | Representation and current qualified entry |
|---|---|
| Independently referenced or edited object | A unit marker implementing `decl::application_schema::fragment::Entity`, listed in its fragment’s `Members`; a record field becomes unique through `.unique()`. Native owns the occurrence, partition and generation. |
| Link between objects | A unit marker implementing `fragment::Relation`, listed once in the source entity’s `Relations`. A key or `EntityRef` hidden in another value does not provide relation topology or dependency tracking. |
| Fixed tuple of references | Separate named typed relations for each semantic role; no stored position is needed. |
| Flat facts | An `Aspect` with its `Row`, listed in the entity’s `Aspects`. `Record<T>` and `Scalar<T>` implementations on the fragment’s binding set lower that row to ordinary native scalar columns. |
| Alternative record classes | An `Exclusive<Group>` member of the entity’s `Aspects`. `Group::Cases` lists the case aspects; owner `From<CaseRow>` implementations construct its enum. SDK writes clear siblings atomically and reads refuse multiple cases. |
| Keyed set or graph | Named entities, unique keys and typed membership/adjacency relations. No artificial position; declare traversal and negative dependencies. See [source subgraphs](source-subgraphs.md). |
| Cycle with Native orbit meaning | `decl::application_schema::ApplicationRelationIntegrity::orbit`, the ordinary typed relation declaration, and `decl::application_query::ApplicationQueryOrbitRef`/query builder `orbit`. Host decision/invariant `orbit` reads retain the actual native cycle contract. |
| True ordered sequence | Classify it first. This lane qualifies no positioned-sequence helper or relation-field carrier. Report the actual ordering/edit/inverse requirement before choosing an unqualified representation. |
| Immutable, identity-free ordered numeric content, including splines at every size | A format-bound `ContentRef` field in a flat Struct aspect, declared through the typed content binding. Native owns admission, integrity, retention and reconstruction; Host content reads and candidate writes use that owner. Frames above 16 MiB refuse before effects. No whole-aspect content shape exists. |
| Identity-free, reference-free leaf with no platform scalar | An owner-admitted, named/versioned atom. Today the qualified fallback is an explicit canonical String field or original-text envelope, with its domain policy and reason recorded. The separate inline binary value-binding work is not qualified here. |
| Compiled pack-owned parameters awaiting a finer carrier | An explicit canonical atom exception with owner, wire shape, bounds/policy and reason. Its guard must reject hidden independently mutable objects or references; it is not a graph container. |
| External input/output document | JSON at a named ingress or terminal projection boundary; admit ingress before native effects. Terminal text grants no native authority. |

The declaration traits in the table belong to
`decl::application_schema::fragment`.
`BoolApplicationValueBinding`, `I64ApplicationValueBinding`,
`U64ApplicationValueBinding` and `StringApplicationValueBinding` live in
`decl::application_schema`. Other scalar families require an actual entry-owned
`ApplicationScalarValueBinding` and readable binding, or the qualified
`decl::worth_query_value_binding!` scalar bridge. There is no generic built-in
Rational binding asserted here. The numerical owner still admits exactness,
normalization, capacity, units, frames, tolerances and certificates; a valid
native scalar alone does not prove those domain laws.

No new list, variant or nested `AspectShape` form is implied by this table.

## Exclusive class aspects

An entity lists `Exclusive<Form>` in `Aspects`; `Form::Cases` lists the case
aspects once. The enum owner supplies `From<CaseRow>` for every distinct row.
Use `Option<Exclusive<Form>>` when absence is lawful.

A whole-case write through `CandidateWriter::fragment` clears its sibling cases
in that transaction. Reading the group uses actual aspect presence: exactly one
case is required, or zero for an optional group. Multiple present cases are a
typed corruption refusal before any case constructor runs. Applications do not
maintain a discriminator. The SDK owns any indexing scalar it requires.

The [fragment courts](../workspaces/worth-query/crates/worth-query-court-decl-host-native-fragments/tests/worth_query_court_decl_host_native_fragments/native_fragments/mod.rs)
exercise actual sibling removal, optional absence and corrupt multiple presence.
See [native schema fragments](native-schema-fragments.md) for the declaration and
operation-record grants.

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

For callers that already know which class to read, absence work is flat.
Discovering a case by scanning its siblings remains linear. The SDK owns
exclusivity and any index it needs; applications author neither a tag nor a
second exclusivity rule.

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
atom at its enclosing native field; there is no inner-field patch. Records lower the qualified canonical binding to an ordinary scalar field
inside the existing Struct aspect shape.

Record the owner, binding identity/version, shape, domain bounds/policy, reason
and no-hidden-reference guard for each exception. Use existing owner policy;
do not invent arbitrary ceilings. The [Store grain contract](../plans/worth-store/storage-foundation-aspect-native-gate.md#current-value-grain-contract)
states the actual read/patch, touch, conflict, invalidation and serialization
boundaries, including conservative widening and reader-specific decode controls.
