# Type-owned application schema fragments

Import `worth_query_decl::facade::application_schema::fragment::*` for declarations
and `worth_query_host::facade::primary_graph::*` for runtime views.

An installation root also implements the existing `ApplicationSchema` trait. Its
header defaults to the platform namespace and version 1.0; shape identities and
aspect revisions remain derived. Only a root needs this forwarding implementation:

```rust
impl ApplicationSchema for Fixture {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}
```

The declaration and runtime types are named by `Fixture`; no inferred traversal
witness escapes into a public type. Nested fragments need only `Fragment`.

An entity lists its aspects and outgoing relations. An aspect supplies a row type;
it does not repeat its entity. A fragment lists entities, nested fragments and
operation/query/capability registrations in `Members`. `Registration<Schema>`
retains those registrations' existing definition builders.

```rust
pub struct Item;
impl Entity for Item {
    const NAME: &'static str = "item";
    type Aspects = (Metrics, Option<Label>, Exclusive<Form>);
    type Relations = (Next,);
}
pub struct Metrics;
impl Aspect for Metrics {
    const NAME: &'static str = "metrics";
    type Row = Measurements;
}
```

`Option<A>` permits whole-aspect absence. `Exclusive<G>` lists a one-of whose
`OneOf::Cases` tuple owns case membership. The enum owner supplies `From<CaseRow>`
for each distinct case row type. `Exclusive` has a separate name because Rust
places traits and struct types in the same namespace.

A fragment's `BindingSet` supplies the original checked-constructor context through
`Work<'a>` and its scalar admission callback through `with_admission`. Implement
`Scalar<T>` on the set for each scalar value and `Record<T>` once for each row.
Bindings retain their units, equality and write postures. A record field defaults
to the binding's write posture. `.read_only()` handles existing field exceptions;
`.unique()` requires exactly one lowered scalar.

```rust
impl Record<Interval> for Bindings {
    fn record() -> impl Shape<Interval, Self> {
        field("start", |row: &Interval| row.start)
            .and("end", |row: &Interval| row.end)
            .build(|start, end, _: &mut ValueDecodeAdmission<'_>| {
                Interval::checked(start, end)
            })
    }
}
```

Arrays lower to `components[0]`, `components[1]`, and so on. A nested interval
lowers to `interval.start` and `interval.end`. These remain ordinary scalar fields.
Native scalar decoding borrows Query's original admission separately from domain
work. Decode invokes the owner's checked constructor and retains its typed error.
Scalar refusals identify the full leaf path; constructor refusals identify their
owning record prefix. Membership and case tuples support arities 1 through 8, with nested membership
tuples and empty membership tuples. Records support up to 16 constructor arguments.

During schema authoring, select a field from that same record:

```rust
let coordinate: RecordField<_, _, u64> =
    record_field(Fixture, Item, Metrics, "components[1]")?;
let schema = declare(Fixture)?;
```

Selection checks the actual lowered path, value type and binding type. Schema
finalization checks registered field expectations against installed declarations.
This is a checked path lookup: stable Rust closures do not expose named typed
field handles without a second declaration or a declaration macro.

`DecisionReader::fragment(Fixture)` and `CandidateWriter::fragment(Fixture)` lend
`View<F, Reading>` and `View<F, Writing>` respectively. Fragment code names its
fragment and its real authority phase; it has no installed-schema type parameter.
The existing projection and candidate owners retain snapshot, operation, attempt
and installation affinity. Acquisition uses the fragment catalog carried by the
existing installed value binding catalog. No Store or Relational owner changes.

The [generic fixture](../workspaces/worth-query/crates/worth-query-testkit-decl/src/fragment_fixture.rs)
and [declaration courts](../workspaces/worth-query/crates/worth-query-court-decl/tests/worth_query_court_decl/decl/fragment_records.rs)
exercise the declarations and round-trip every reached binding with owner-supplied
samples, including a foreign checked value with borrowed owner context.

One-of reads observe every case's actual Native parent presence before decoding.
A required group needs exactly one case; an optional group also permits none.
Two present cases refuse as corruption, even if one case would independently
fail its checked constructor. Schema build refuses repeated case row types:
Rust permits one `From<T>` implementation to satisfy two cases with row `T`.

Declare whole-record operation access from the same membership:

```rust
builder
    .operation_read_record(operation, Fixture, Item, Form)
    .operation_write_record(operation, Fixture, Item, Measured)
```

A whole case write owns the sibling removals derived from that group. A single
field write grants no parent removal. Repeated case writes replace only that
transition's own pending clears; separately authored clears remain conflicts.

Principal-binding requirements accept `from_record` handles and check write,
equality and binding postures. Bootstrap entity seeds accept `record_field`.
Selected products resolve those handles with `resolve_record_entity`, preserving
the original phase, product and request. For completed candidate programs,
`application.fragment_candidate(&mut candidate, Fixture)` lends a write view
after checking the candidate's runtime and schema affinity. Handler writers
continue to use `writer.fragment(Fixture)`.

A scalar binding set can attach an immutable content format through
`Scalar<T>::content_format()`. Format-bearing value types use
`ApplicationContentReferenceBinding` to carry the Native reference through their
owned scalar codec. `content_field(selection, Format)` requests that installed
interpretation; it does not register storage. Different formats use distinct
value forms, and installation checks the actual ContentRef representation.

`TypedSchemaRoot` and `worth_query_schema!` remain query-surface declarations.
They describe exposure to a canonical query, including ordering-only, hidden and
workflow fields and traversal limits. They install no rows, identities or value
codecs. Application storage membership belongs to fragments.

History queries accept `aspect(fragment, entity, aspect)` and
`relation_aspect(fragment, source_entity, relation, aspect)`. Both selectors
prove membership and check the installed fragment owner before resolving the
portable contract, including when another fragment reuses native names.
