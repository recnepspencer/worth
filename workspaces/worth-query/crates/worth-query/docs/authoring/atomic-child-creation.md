# Atomic Child Creation

## What This Feature Is

An operation body can create a root record and explicitly declared child records
in one native candidate. `with_child` extends the existing creation funnel;
it is not a runtime callback, second executor or multi-commit orchestration loop.

## Why You Use It

Use it when a new record must never be published without its default related
records—for example, a lot row with Physical and Electronic verification cells,
or a new revision with its initial review records. A late failure publishes none
of the tree or its other declared writes.

## Stable Entry Points

Through `worth_query_decl::facade::application_schema`:

- `creation.with_child(relation, key, |child| ...)?`;
- `child.initialize`, `initialize_input`, `initialize_optional_input`;
- nested `child.with_child(...)` for another explicitly authored level.
- `child.link_from_parent(relation, key)` for another association from that
  child's same freshly created parent.

Child kinds require `OperationCreates`; fields require `OperationWrites` and
relations require `OperationLinks`. The relation must forbid cross-context links
and have incoming `target_max = Some(1)`. This is relation-local exclusivity,
not a claim that no other relation can ever refer to the child.

An additional parent association has the same endpoint, context and incoming
exclusivity rules. It adds a link, not another child or another parent slot.
The original ownership link stays first and unchanged. Different parents,
duplicate relations/keys and missing Link membership are refused.

## Core Mental Model

The authoring closure builds inert native meaning and is then gone. Declaration
retains one flat preorder site inventory with exact parent keys, kinds, fields
and links. Parents must precede children; keys are unique across all sites.
Caller values and matching labels do not supply parent authority.

The installed native operation captures and validates original input once. Query
allocates every site in the **original admitted Existing scope's context**,
then links the actual retained created handles. A new parent does not become a
new context-authority source.

## How It Executes

1. Finish the body and freshly admit its complete field, codec and graph closure.
2. Install parent-slot correspondence and full instance capacity. Two children
   of one kind count as two creates and two field/link groups, even though their
   permission contracts are deduplicated.
3. Prepare inputs in scope/root/child-preorder/path-guard order. All child input
   codecs run before candidate effects, including optional absence.
4. Retain request-funded space for actual created handles and assemble the tree
   through existing Create/Initialize/Link owners.
5. Publish all sites, independent writes and native consequences together.

Stale publication lands no losing subtree. Exact original-intent replay returns
the performed result without creating more children or repeating revision writes.

## Small Example

Assume the shown initializers cover the entities' required fields:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let body = CreateRecord::reference().body()
    .in_scope(Organization::reference())?
    .create_in_scope(Record::reference(), "record")?
    .initialize_input(RecordIdentity::reference(), "record_id")?
    .link_from_scope(OrganizationRecords::reference(), "organization-record")?
    .with_child(RecordChecks::reference(), "check", |child| {
        child.initialize_input(CheckIdentity::reference(), "check_id")?
            .initialize(CheckKind::reference(), Physical)
    })?
    .finish()?;
```

For one initial review occurrence referenced by both current selection and
retained history, add the second link inside the same child declaration:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
creation.with_child(CurrentReview::reference(), "initial-review", |review| {
    review.initialize(ReviewOrdinal::reference(), first_ordinal)?
        .link_from_parent(ReviewHistory::reference(), "review-history")
})?;
```

Both relations reach the same retained created handle. A native invariant or
publication failure lands neither association nor a partial child.

Attach the completed body to the ordinary operation definition and install it
through `setup.operation_body::<Binding>()?`. Creation output correspondence, if
declared, still names the root site—not an implicit child output.

## Real Example

Root computations stay available while child fields use scalar initializers:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled. Domain rules produce the checked native values.
let body = AddRow::reference().body()
    .assign_value(Revision::reference(), &next_revision)?
    .create_in_scope(Row::reference(), "row")?
    .initialize_input(RowIdentity::reference(), "row_id")?
    .initialize_value(RowOrder::reference(), &next_order)?
    .link_from_scope(RecordRows::reference(), "record-row")?
    .with_child(RowChecks::reference(), "physical", |cell| {
        cell.initialize_input(CheckIdentity::reference(), "physical_id")?
            .initialize(CheckKind::reference(), Physical)?
            .with_child(CheckNotes::reference(), "instructions", |note| {
                note.initialize_input(NoteText::reference(), "instructions")
            })
    })?
    .with_child(RowChecks::reference(), "electronic", |cell| {
        cell.initialize_input(CheckIdentity::reference(), "electronic_id")?
            .initialize(CheckKind::reference(), Electronic)
    })?
    .finish()?;
```

The root and all three descendants share one publication. Collection bounds,
ordering, default-row policy, assignment readiness and signing meaning remain
explicit domain declarations; this feature does not choose those business rules.

## How It Relates To Other Features

This uses ordinary scope-bound bodies and their checked root values. It does
not add a source-selection engine or command continuation. Existing protected
signed-evidence and supersession-kind laws apply to every derived Create/Write
contract. Child creation cannot manufacture protected evidence unsigned.

## Inspection And Debugging

Fresh admission refuses missing/forward parents, duplicate keys, endpoint drift,
missing field/program closure and nonexclusive or cross-context ownership.
Insufficient site capacity refuses native body installation. Program meaning
includes exact parent correspondence even when the field footprint is unchanged.
Canonical schema body v26 and portable body v36 identify this opt-in topology;
older bodies keep their historical identity and wire paths.
Children with multiple parent links select canonical body v29 and portable v39.
Earlier portable profiles reject that shape, including inside an independent
peer tree. Program meaning includes every association, not just ownership.

## Anti-Patterns

Do not create the root and children in separate commits, install a callback that
assembles effects, supply parent IDs as authority, deduplicate instance capacity
by entity kind, or allocate in the newly created parent's context. Do not claim
that generic child creation proves an application's signing, audit or UI parity.

## Current Limits

Children support Literal, Input and OptionalInput initializers. Root initializer
capabilities, including checked computed values, remain. This profile is unsigned
append topology: no signing, workflow enrollment/succession, root replacement or
content reuse. Child-output references are unsupported; root outputs keep their
exact original contract. It is not arbitrary existing-target graph editing,
dynamic runtime fan-out, global ownership proof or restart-durable execution.

## Related Docs

- [Scope-bound operation bodies](./scope-field-operation-bodies.md)
- [Graph-certified subjects](./graph-certified-subjects.md)
- [Complete-content selection reuse](./complete-content-selection-reuse.md)
