# Complete-Content Selection Reuse

## What This Feature Is

An exclusive disposable selection can reuse its current entity when every
declared field equals its validated creation initializer. Query derives the
comparison from the same initializer list that would create the replacement.
This is not a command-wide no-op: independent scope/path writes and emissions
still execute.

## Why You Use It

Use this when selecting the same provider snapshot should preserve its native
identity. A change to provenance, an optional value or its absence still replaces
and retires the predecessor. Do not use it to equate different business entities
or to preserve signatures while changing their declared subjects.

## Stable Entry Points

Through `worth_query_decl::facade::application_schema`, call
`creation.reuse_current_if_unchanged()` before `finish()`, together with
`replace_and_retire_from_scope(relation, key)?`. Attach and install the completed
body through the ordinary operation-definition and native-body setup funnel.

All retiring-replacement Read/Create/Write/Delete/Link/Unlink and relation
cardinality requirements remain. Declaration additionally derives a Field read
for each initializer. Candidate ceilings retain the full replacement capacity;
reuse reduces actual effects, not permissions or worst-case reservation.

## Core Mental Model

The current-pointer relation and committed fields are authoritative. Original
input is captured and codec-admitted once. The native decision compares that
admitted content with the retained predecessor through Relational's canonical
field equivalence, including absent optional fields. No caller supplies an
equality flag, content hash, predecessor ID or replacement decision.

Every declared selected-entity field must have exactly one supported initializer.
Adding an optional field without defining its initialization is an admission
error. Do not add a fresh allocation UUID or clock value to the content tuple;
native entity identity is already separate from provider/business content.

## How It Executes

1. Author and freshly admit the complete unsigned exclusive-replacement body.
2. Observe the current selection and validate all original input initializers.
3. Observe every comparison field, including absence; reserve comparison work
   and memory before canonical encoding. Missing required predecessor data denies.
4. Carry the sealed reuse/create disposition and native selection through decision.
5. On reuse, consume the same prepared initializer slots but suppress only the
   selection Create/Unlink/Delete/Link effects. Otherwise use ordinary retirement.
6. Publish through the existing authorization, invariant and currentness owners.

Concurrent field or pointer changes stale the prepared action. No automatic retry
or host recomparison occurs. Exact original-intent replay returns the original
performed outcome and does not repeat independent effects.

## Small Example

Assume these are **all** fields of `Snapshot`:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let body = Select::reference().body()
    .in_scope(Row::reference())?
    .create_in_scope(Snapshot::reference(), "snapshot")?
    .initialize_input(SourceRow::reference(), "source_row")?
    .initialize_input(Value::reference(), "value")?
    .initialize_optional_input(Note::reference(), "note")?
    .replace_and_retire_from_scope(CurrentSnapshot::reference(), "current")?
    .reuse_current_if_unchanged()
    .finish()?;
```

The first selection creates normally. Repeating the complete tuple retains the
actual selected entity, not merely the displayed source-row label.

## Real Example

Keep an affected row's selection stable while updating its parent's revision:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled. Every Snapshot field appears below.
let body = Select::reference().body()
    .assign_value_at_path(parent_path(), ParentRevision::reference(), &next_revision)?
    .assign_input(RowQuantity::reference(), "row_quantity")?
    .create_in_scope(Snapshot::reference(), "snapshot")?
    .initialize_input(SourceRow::reference(), "source_row")?
    .initialize_input(SourceRevision::reference(), "source_revision")?
    .initialize_input(SelectedQuantity::reference(), "selected_quantity")?
    .initialize_optional_input(SupplierLot::reference(), "supplier_lot")?
    .replace_and_retire_from_scope(CurrentSnapshot::reference(), "current")?
    .reuse_current_if_unchanged()
    .finish()?;
```

`next_revision` is an existing native checked value, not host arithmetic. Provider
input still enters through its normal installed lookup/admission binding; this
feature does not upgrade provider freshness into a distributed transaction.

## How It Relates To Other Features

Ordinary retiring replacement always creates a successor. This explicit option
keeps identical content and its entity untouched. Identity-sensitive graph
certification therefore sees no selection change on reuse; unrelated effects
can still invalidate evidence whose subjects they change. This does not prove a
particular application's signature workflow or user interface is correct.

## Inspection And Debugging

Typed finish rejects unsupported combinations. Fresh schema admission additionally
checks complete field coverage and comparison-read membership. Canonical schema
body v25, program meaning and portable body v35 identify the reuse policy only
when used; existing bodies retain their prior identity and wire profiles.

## Anti-Patterns

Do not compare DTOs in the host, normalize after request capture, omit optional
absence, bypass input validation on a presumed no-op, restore invalidated
signatures, or discard stale/indeterminate outcomes. Do not confuse provider row
identity with a new snapshot allocation identity.

## Current Limits

One unsigned exclusive retiring-replacement link with distinct scope/selected
kinds. Initializers are Literal, Input or OptionalInput only. No signing,
workflow enrollment/succession, created-slot outputs or graph/principal/computed
initializers in this profile. Existing retirement fences also refuse writes,
preserved outputs or owned-selection rules that overlap the predecessor kind.
Scope/other-kind path effects and external emissions retain their ordinary rules.
This does not add preview, persistent Store recovery or application parity claims.

## Related Docs

- [Retiring selection replacement](./retiring-selection-replacement.md)
- [Computed graph-target writes](./computed-path-writes.md)
- [Graph-certified subjects](./graph-certified-subjects.md)
- [Owned-selection retirement](./owned-selection-retirement.md)
