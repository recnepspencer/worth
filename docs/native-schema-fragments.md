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
owning record prefix. Tuple arities for membership, cases and record accessors are 1?8;
empty membership tuples are supported.

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
