# Independent Creation Sites

## What This Feature Is

A compiled body can create independent peer records and attach a creation to a
live existing entity selected by a typed path. Every record belongs to one native
candidate, but an independent peer does not inherit another site's reuse or
disposal. This is useful for a disposable selection plus a retained parent audit.

## Why You Use It

Use peers when records share an atomic decision but have different ownership or
lifetimes. Use children when the new parent owns the newly created descendants.
Use an existing-reference input when selecting an already existing entity instead.

## Stable Entry Points

Through `worth_query_decl::facade::application_schema`:

- `creation.with_peer(entity, key, authoring_closure)`
- `creation.link_from_path(ApplicationQueryRootPath, relation, key)`
- `creation.link_to_principal::<Binding, Relation>(relation, key)`

The peer closure receives the existing typed creation builder, not a runtime
callback or another mutation executor. Initializers, native actor capture and
computed values use their ordinary APIs.

## Core Mental Model

The admitted scope remains the original existing entity. A path selects an exact
existing anchor from that scope; it does not turn a supplied key into authority.
Each creation has a candidate-local symbolic key, independent of business identity.
Root, peer and descendant creation keys must all be distinct.

A creator relationship has a different source from a scope/path link. Use
`link_to_principal::<Binding, _>` with a relation from the created entity to
`Binding::Principal`. Query captures the principal actually admitted for that
binding before candidate effects. The input supplies neither an actor ID nor an
organization-member selection. Two editors in the same scope therefore retain
distinct creator relationships. The link records provenance, not permission.

A snapshot can be reused while an independently declared audit is still created.
Snapshot replacement retires only its disposable predecessor. It does not delete
the peer audit or move that audit under the snapshot's ownership. Peers may own
their own children.

## How It Executes

Query validates the complete site inventory, source/read/effect closure and exact
installed codecs. Decision-making captures original inputs, native actor/source
values, checked computations and live path anchors before candidate effects.

Candidate assembly allocates all sites in the original admitted context. Existing
anchors and newly created parents are retained handles, not graph re-lookups.
After all ordinary effects, the signed root's deferred provenance slot reserves
the remaining representation budget. Authentication still seals the whole candidate.

Native authorization, invariants and publication currentness remain mandatory.
Missing or ambiguous anchors deny; competing adjacency edits can stale the whole
candidate. No partial scope write, snapshot or audit is published. Exact logical
request replay recovers the original outcome without creating another audit.

## Small Example

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Supply complete declared schema fields.
let body = Select::reference().body().in_scope(Row::reference())?
    .create_in_scope(Snapshot::reference(), "snapshot")?
    .initialize_input(SnapshotName::reference(), "name")?
    .with_peer(Audit::reference(), "audit", |audit| {
        audit.initialize_input(AuditName::reference(), "name")?
            .link_from_path(parent_path(), ParentAudits::reference(), "audit")
    })?
    .finish()?;
```

Declare Create/Write capabilities for both sites and Link for the audit relation.
The path's root and anchor require Read capabilities. The operation definition
derives the exact path reads and effect contracts. All required created fields
must have initializers; the example's names do not imply a universal audit schema.

## Real Example

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// next_revision is an admitted native checked value.
let body = SelectLot::reference().body()
    .assign_value_at_path(parent_path(), Revision::reference(), &next_revision)?
    .create_in_scope(LotSnapshot::reference(), "snapshot")?
    .initialize_input(SourceRow::reference(), "source_row")?
    .initialize_input(SourceRevision::reference(), "source_revision")?
    // Initialize every other declared snapshot field, including optional absence.
    .replace_and_retire_from_scope(CurrentLot::reference(), "current-lot")?
    .reuse_current_if_unchanged()
    .with_peer(SelectionAudit::reference(), "audit", |audit| {
        audit.initialize_input(AuditSourceRow::reference(), "source_row")?
            .initialize_value(AuditRevision::reference(), &next_revision)?
            .initialize_from_principal_identity::<SelectLotBinding, _, _, _, _, _>(
                AuditActor::reference(),
            )?
            .link_from_path(parent_path(), ParentAudits::reference(), "audit")
    })?
    .finish()?;
```

The native value owns arithmetic; admission owns the actor; the captured provider
input owns snapshot content. Full-content comparison includes provenance and
optional absence. Reuse skips only the reusable site's creation effects, not the
parent revision or independent audit. Whether a business operation should emit an
audit for a no-op request is supplier-authored meaning, not an implicit audit policy.

To retain a creator relationship as well as an actor scalar, the corresponding
root or peer builder can append the actual principal link:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// CreatedBy targets CreateBinding::Principal.
let created = created
    .link_to_principal::<CreateBinding, _>(CreatedBy::reference(), "creator")?;
```

Declare Link membership normally. Cold readmission checks the operation, binding,
principal identity locus and relation endpoints; installation also checks the
exact typed mutation binding. Native capture retains the actual observed target,
not a scalar identity for later lookup. The admitted principal's identity field
and entity are tracked reads, not arbitrary directory-read permission.

The operation declares authority for every creation instance, initializer,
scope/path write and link, including each peer's children; it does not forecast
candidate counts or bytes. Repeated entity kinds are not one creation instance.
This mechanism alone is not a complete lot-provider,
timestamp, business-audit or workflow-signature contract.

## How It Relates To Other Features

Root signing can compose with unsigned peers; all ordinary effects remain sealed
together. Peer/path links participate in dependency-scoped signature lifecycle
touch derivation. Existing-anchor direction matters: linking a fresh endpoint
does not falsely imply an old capture anchored at that fresh endpoint changed.

Use `with_child` for owned topology. Use `replace_relation_from_input` for
input-selected existing targets. Do not convert either into a custom host handler.

## Inspection And Debugging

Inspect `creation.peers()`, `creation.path_links()` and
`creation.initialization_sites()`. The inventory orders the root and its children,
then each peer and its children. Read committed attachment through a governed query;
echoing an input or observing only a field on an unattached record is insufficient.

New composition uses canonical body profile 28 and portable archive profile 38.
Older bodies keep their profiles. Program/evolution identity retains peer
initializers, link targets, anchor paths and descendant-parent correspondence.
Profiles do not advance schema compatibility versions or rewrite old releases.
Principal anchors add explicit canonical link meaning and select portable body
archive profile 40. Root, peer, path and child carriage retain the same meaning
within that record. An older-profile writer refuses principal anchors rather
than silently dropping metadata; ordinary scope links keep their original bytes.

## Anti-Patterns

Do not put a retained audit beneath a disposable snapshot, suppress it because
another site was reused, split atomic effects into two commits, copy a parent ID
or revision as authority, resolve an anchor in the host, or invalidate signatures
with a second changed-field planner.

## Current Limits

Paths must be guard-free and select one distinct live endpoint at each hop.
Path links append from an existing anchor; they do not replace an existing-path
selection. A reusable or retiring site cannot also have path links.
Principal links append from the created record to the existing admitted principal.
They are supported on roots and peers, not child sites or path-link wrappers.
Unsupported descriptions fail fresh validation. Reverse-path subjects anchored
at the existing principal can be affected by this new relation; a fresh endpoint
does not imply a preexisting forward-path subject changed.

Independent peers are unsigned, append-only creation trees. Nested peer
attachments, peer reuse/replacement, peer enrollment, peer signing and succession
with peers are refused, not silently discarded. Use repeated `with_peer` calls
on the primary builder for independent siblings. Existing owned-child profile
limits still apply. Creation outputs identify only the primary root; no peer
output is implied. These are fresh-admission constraints, not host conventions.

## Related Docs

- [Scope-bound bodies](scope-field-operation-bodies.md)
- [Atomic child creation](atomic-child-creation.md)
- [Complete-content reuse](complete-content-selection-reuse.md)
- [Computed path writes](computed-path-writes.md)
- [Graph-certified subjects](graph-certified-subjects.md)
