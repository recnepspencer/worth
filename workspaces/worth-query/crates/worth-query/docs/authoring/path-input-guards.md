# Compare Captured Input with a Field on a Related Entity

## What This Feature Is

`expect_input_from_path` is an executable operation-body guard. It requires the
present field at one selected graph path to equal a scalar from the operation's
original captured input. Query owns path selection, comparison, dependency
retention and publication. The operation's scope stays at the path's start;
the related entity does not become the mutation scope.

## Why You Use It

Use it when editing a row must agree with shared truth on its parent, such as
requiring a provider-selected lot's company to equal its NCR's company. The
parent field stays authoritative. Do not copy it onto every affected row merely
to make a scope-only guard possible, or compare it in a host before submitting.

## Stable Entry Points

Declare through `worth_query_decl::facade` using the existing query-path type:

```rust,ignore
body.expect_input_from_path(
    ApplicationQueryRootPath::from(AffectedLot::reference())
        .reverse(NcrAffectedLots::reference()),
    Company::reference(),
    "company",
)?
```

Finish and install the normal operation body. No special host handler is needed.
`body.path_input_expectations()` returns descriptive meaning, not selected
entities or authorization. `ApplicationOperationBodyPathInputExpectation` and
its untrusted carriage constructors grant no execution authority.

## Core Mental Model

The typed path starts at the admitted scope and ends at the field's entity.
Each hop must select one distinct live endpoint. Forward, reverse, multi-hop
and identity paths use the same native path vocabulary as certified captures.
The scope and terminal entity, plus the field, need typed Read capabilities.
Registration derives exact installed entity/relation/field reads; do not add
those same reads again. Scope identity resolution still needs its read when
not already derived by a body clause.

Both Required and Optional stored fields can be compared, but the input must be
a present flat scalar. Absent optional graph data cannot match; neither empty
text nor a default is substituted. An Option-valued input is not this contract.
The exact installed field codec admits the captured scalar.

## How It Executes

After ordinary authorization, input admission selects guard members from the
single canonical input capture. Guard slots follow existing scope and creation
input slots without shifting their correspondence. Query never serializes the
input again or asks a provider for a replacement value.

The native path reader seals complete adjacency, including negative knowledge,
and endpoint liveness. The terminal field comparison retains its value/absence
dependency without cloning a selected graph value into host custody. It uses
the same decision observation as the candidate's other facts. Candidate
construction writes only the body's declared effects, not the guard fields.
Normal request work/memory and projection budgets bound traversal.

Another session changing the selected field, parent liveness or relevant
ownership adjacency can stale the whole candidate. Row edits and unsigned audit
creation then publish nothing. An unrelated parent's edit does not by itself
invalidate this guard. Exact committed replay recovers its original outcome
before selecting or comparing today's parent; it does not repeat the write.

## Small Example

```rust,ignore
let body = SaveRow::reference().body()
    .expect_input_from_path(
        ApplicationQueryRootPath::from(Row::reference()).reverse(ParentRows::reference()),
        ParentCompany::reference(), "company",
    )?
    .assign_input(RowValue::reference(), "value")?
    .finish()?;
```

This mutates the Row, not the Parent. The input binding opts into
`RETAIN_CANONICAL_INPUT`, and each selected field codec supports native input
admission, just as for `expect_input` and input assignments.

## Real Example

After the installed provider port has captured and revision-admitted a lot:

```rust,ignore
let body = SelectLot::reference().body()
    .expect_input_from_path(
        ApplicationQueryRootPath::from(AffectedLot::reference())
            .reverse(NcrAffectedLots::reference()),
        Company::reference(), "company",
    )?
    .create_in_scope(LotSnapshot::reference(), "lot")?
    .initialize_input(SnapshotCompany::reference(), "company")?
    .initialize_input(SnapshotLot::reference(), "lot")?
    .replace_from_scope(AffectedLotSelection::reference(), "selection")?
    .finish()?;
```

This is an illustrative supplier declaration, not completion of the NCR product
integration. Declare all snapshot initializers, ordinary candidate ceilings,
domain requirements and authorization. A captured snapshot company is provider
data, not a second mutable authority for the parent's company. Declare exclusive
NCR ownership separately in relation integrity; a runtime single-selection guard
does not create an ownership invariant.

Retain the original admitted input, observed source and logical key through the
owning action. Provider freshness at admission is not atomic with graph
publication. This guard checks graph correspondence; it does not strengthen the
external provider's transaction guarantee.

## How It Relates To Other Features

`expect_input` compares a field directly on the admitted scope. Use it when no
path is needed. `capture_from_path` copies a tracked field into a creation and
may participate in native signing; this guard compares an input instead. It
creates no signature and supplies no approval authority. Existing signed-subject
and supersession owners retain their separate responsibilities.

Use declared requirements for readiness guidance. A path-input guard is an
execution precondition, not a new Met/Unmet assessment finding. Assessment does
not turn an input comparison into a mutation permit.

## Inspection And Debugging

A missing, duplicate, nested, wrong-family or codec-invalid input member produces
`InputMismatch { member }`. A different or absent terminal field produces
`SourceMismatch { entity, aspect, field }`. Missing path selection and multiple
distinct endpoints remain native `MissingAuthoritativeFact` and
`AmbiguousRelation` execution denials. Resource refusal, cancellation, stale and
publication recovery preserve their existing typed outcomes; do not collapse or
automatically reissue them with another key.

Path, member, field binding and presence participate in canonical schema/program
meaning. Canonical body v22 and portable body v32 apply only when these guards
are used. Empty guard collections retain old identity and encoding paths. Fresh
readmission checks the exact scope, topology, field metadata and read closure.

## Anti-Patterns

Do not select the first/latest parent, infer identity from equal company labels,
copy parent state onto every row, replace missing data with a default, cache a
successful comparison as permission, or accept a host-built parent record.
Do not change scope to the parent while claiming to edit an existing child.

## Current Limits

This profile accepts guard-free native paths only. Filtered `where_equal` paths
are rejected, never silently weakened. Every hop selects one distinct endpoint;
it is not a collection quantifier or join over multiple parents. Same-endpoint
parallel edges do not create a second endpoint, but their complete adjacency
still participates in currentness. No Option-input comparison or readiness
assessment finding is supplied by this method.

## Related Docs

- [Scope-bound bodies and captured-input guards](./scope-field-operation-bodies.md)
- [Graph-certified subjects](./graph-certified-subjects.md)
- [Owned-selection retirement](./owned-selection-retirement.md)
- [Owning application front door](../foundations/ordinary-application-front-door.md)
