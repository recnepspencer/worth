# Retiring Selection Replacement

## What This Feature Is

An authored creation can replace a current snapshot and delete its observed
predecessor in the same native candidate. This is an explicit disposal policy,
not a change to ordinary `replace_from_scope`, which preserves the old entity.

## Why You Use It

Use this for a disposable provider-selection snapshot whose replacement should
not leave an unreferenced old copy alive. Do not use it for approval rounds,
immutable signatures or document revisions that must remain historical.

## Stable Entry Points

Through `worth_query_decl::facade::application_schema`:

- `creation.replace_and_retire_from_scope(relation, key)?`;
- `ApplicationOperationBodyCreationLinkPosture::ReplaceAndRetireSingle` for
  descriptive transported meaning;
- ordinary `replace_from_scope` when the predecessor must survive.

The typed call requires scope/selected entity Read membership, relation
Read/Link/Unlink membership and selected entity Delete membership. As usual,
creation and initialization require their declared Create/Write membership.
The relation must have `source_max = Some(1)` and `target_max = Some(1)`.

## Core Mental Model

The existing native selector observes the complete current-pointer adjacency,
including an empty selection, and retains its live predecessor mutation target.
Retirement uses that target; no host ID, post-commit lookup or second observation
can substitute for it. Declaration grants no mutation permission.

Incoming exclusivity is local to the declared relation. It does not prove that
no other relation refers to the snapshot. Existing deletion invariants remain
authoritative. All ordinary principal, operation, policy and publication checks
still apply.

## How It Executes

Declaration derives the Link, Unlink and Delete contracts and the same reads
used by replacement. Installation requires worst-case predecessor delete and
unlink capacity even when the first selection is empty. One candidate owns scope
or parent writes, creation, old-edge unlink, predecessor deletion, the new link
and native signature consequences.

A concurrent first selection or replacement stales the losing candidate. None
of its creation, field or deletion effects publishes. Exact original-intent
replay returns the performed result, even if a later operation has retired that
result's snapshot. Preserve the original input, source posture and logical key.

## Small Example

For a row-owned selection with the required one-to-one relation contract:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let body = SelectSnapshot::reference().body()
    .in_scope(Row::reference())?
    .create_in_scope(Snapshot::reference(), "snapshot")?
    .initialize_input(SnapshotSourceRow::reference(), "source_row")?
    .initialize_input(SnapshotValue::reference(), "value")?
    .replace_and_retire_from_scope(CurrentSnapshot::reference(), "current")?
    .finish()?;
```

Attach the body to the operation and install it through the normal
`setup.operation_body::<Binding>()?` lane. Candidate capacity includes one
create, one delete, one link, one unlink and every field initialization/write.

## Real Example

Combine a captured snapshot with an authoritative parent revision update, using
the existing checked `next_revision` value rather than host arithmetic:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let body = SelectSnapshot::reference().body()
    .assign_value_at_path(parent_path(), ParentRevision::reference(), &next_revision)?
    .create_in_scope(Snapshot::reference(), "snapshot")?
    .initialize_input(SnapshotSourceRow::reference(), "source_row")?
    .initialize_input(SnapshotSourceRevision::reference(), "source_revision")?
    .initialize_input(SnapshotValue::reference(), "value")?
    .replace_and_retire_from_scope(CurrentSnapshot::reference(), "current")?
    .finish()?;
```

The operation scope stays the row. Parent, row and snapshot are distinct kinds.
Resolve and revision-admit provider input through its installed native binding
before invoking the action. A real query-backed revision calculation also needs
its original row-affine observed source; a public result DTO is not that source.
Provider freshness at admission does not imply a distributed provider/graph commit.

## How It Relates To Other Features

Use [owned-selection retirement](./owned-selection-retirement.md) when a source
change clears selections across preserved holder rows. Use regular replacement
when retaining the predecessor as history. Native Delete/Unlink touches still
feed existing signed-subject lifecycle rules.

This feature changes selection identity and retires a source. It does **not**
declare two snapshots equivalent because their contents match. Identity-sensitive
graph certificates may therefore be superseded on an identical reselection.
A product requiring no-op signature preservation needs an explicit native
equivalence/reuse rule; do not add a host comparison or restore signatures.

## Inspection And Debugging

Missing Delete/Unlink, nonexclusive topology and unsupported profile combinations
are typed authoring or fresh-admission refusals. Insufficient candidate capacity
refuses body installation. Stale publication remains the native stale outcome,
not an invitation to automatically retry a lifecycle action.

The new posture participates in canonical schema/program meaning and portable
body v34. Its wire tag is refused under older body versions. Append and ordinary
replacement retain their existing identity and format paths.

## Anti-Patterns

Do not delete after committing the replacement, infer global ownership from one
relation, clear history, supply a predecessor ID as authority, or treat Delete
membership as permission to delete any entity. Do not claim this is a no-op or
that it proves an application endpoint, browser parity or restart durability.

## Current Limits

One unsigned creation link, outgoing one-to-one selection, and distinct scope
and selected entity kinds. Signed creation is refused. Related Preserve outputs
and path writes targeting the selected kind are refused, as are overlapping
owned-selection retirement rules deleting that kind. This conservative profile
prevents retained-target and duplicate-retirement conflicts; it is not a general
cascade or arbitrary graph-deletion API. Scope/other-kind path writes and
new-creation output correspondence remain available through their ordinary lanes.

## Related Docs

- [Scope-bound operation bodies](./scope-field-operation-bodies.md)
- [Computed graph-target writes](./computed-path-writes.md)
- [Owned-selection retirement](./owned-selection-retirement.md)
- [Graph-certified subjects](./graph-certified-subjects.md)
