# Graph Composition Authoring

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Graph composition lets one workspace mutation create, update, retarget,
supersede, or retire related graph records as one ordered batch. Use it when
later steps need symbolic handles for records created earlier in the same
mutation, or when existing graph identities must remain explicit in the
receipt.

Graph composition is a generic workspace mutation feature. It does not
register, select, or execute installed application obligations. Installed
application invariants run only through the canonical provider-session path.

## Why You Use It

- Refer to a newly created entity from a relation in the same batch.
- Preserve existing entity or relation identity during updates and rewires.
- Record lifecycle, lineage, resolution, and breadth evidence for one logical
  graph mutation.
- Commit the batch atomically through the configured backend.

## Stable Entry Points

- `WorthQueryWorkspace::compose_graph(...)`
- `WorthQueryGraphCompositionBuilder` methods for entity and relation
  insertion, follow-up mutation, retargeting, supersession, and retirement
- `WorthQueryBatchWriteReceipt` graph-composition inspection fields
- `workspace.inspections()?.inspect(&receipt)`

There is no `compose_graph_with_invariant_pack(...)`. A caller-authored
callback cannot become invariant authority.

## Core Mental Model

The closure builds an ordered mutation program. Symbolic handles are local to
that program and resolve before the backend writes. The resulting receipt
retains the authored-to-resolved mapping and the exact lifecycle outcomes.

```text
composition closure
  -> typed ordered program
  -> symbolic target resolution
  -> mutation-contract and existing-truth admission
  -> one backend graph batch
  -> write receipt + resolution/lifecycle/lineage evidence
```

The receipt explains what the generic batch did. It is not an application
obligation terminal and cannot stand in for authorization or installed
invariant execution.

## How It Executes

1. Query creates an empty `WorthQueryGraphCompositionBuilder`.
2. The closure adds typed steps and receives symbolic handles where needed.
3. Builder finalization validates names, target collections, ordering, and
   symbolic references.
4. Existing-truth and mutation-contract admission runs.
5. The runtime submits one atomic graph batch.
6. The receipt records resolved identities, ordered steps, breadth, lifecycle,
   assumptions, and lineage when applicable.

## Small Example

```rust
let receipt = workspace.compose_graph(|graph| {
    let task = graph.insert_entity("new-task", "Task", |entity| {
        entity.set_aspect(touch("title.value"), authored_text("Ship it"))
    })?;

    graph.insert_symbolic_relation("dependency", "TaskEdge", |relation| {
        relation
            .symbolic_entity_identity(touch("edge.source_identity"), &task)
            .existing_entity_identity(
                touch("edge.target_identity"),
                existing_target,
            )
    })?;
    Ok(())
})?;

assert_eq!(receipt.write_receipts().len(), 2);
```

The symbolic handle never escapes the closure. The receipt carries the
resolved identity.

## Real Example

A lifecycle composition can insert an entity, insert a relation that points to
it, update the new entity, and retire the new relation in one batch:

```rust
let receipt = workspace.compose_graph(|graph| {
    let draft = graph.insert_entity("draft-task", "Task", draft_values)?;
    let edge = graph.insert_symbolic_relation("draft-edge", "TaskEdge", |relation| {
        relation
            .symbolic_entity_identity(touch("edge.source_identity"), &draft)
            .existing_entity_identity(touch("edge.target_identity"), existing_target)
    })?;
    graph.update_entity(&draft, published_values)?;
    graph.delete_relation(&edge, retired_edge_touches)?;
    Ok(())
})?;
```

The complete compiling journey is
`worth-query/tests/graph_composition_public_bridge.rs`. It verifies final rows,
ordered program kinds, resolution mapping, and identity-preserving lifecycle
outcomes through the public facade.

## How It Relates To Other Features

- [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
  owns installed application selection, planning, session work, invariants,
  terminals, and publication.
- [Graph Touch Obligation Authority](graph-touch-obligation-authority.md)
  explains installed application graph-work meaning. It is not a hook inside
  generic graph composition.
- Existing-truth bindings and verified operations preserve current identity
  when the backend supports them.
- Generic workspace read composition may inspect the resulting committed graph;
  it does not see an application proposal before commit.

## Inspection And Debugging

Inspect:

- `graph_composition_program()` for exact step order;
- `graph_composition_resolution_map()` for symbolic-to-resolved identities;
- `graph_composition_lifecycle_outcomes()` for created, updated, retargeted,
  superseded, or retired posture;
- breadth and lineage summaries; and
- the ordinary write receipts for backend consequences.

Typed graph-composition denials distinguish duplicate symbols, unresolved
references, collection mismatches, unsupported existing-target operations, and
failed current-truth assertions. Preserve those denials rather than retrying
through scalar writes.

Contributed domain-capability inspection may attach a non-executable invariant
denial summary. It explains why a contributed capability rejected a shape; it
does not run an application invariant or authorize a commit.

## Anti-Patterns

- Splitting one identity-dependent graph change into unrelated scalar commits.
- Resolving same-batch identities in caller-owned maps.
- Passing a manual invariant callback beside the batch.
- Treating a contributed-capability denial summary as an invariant verdict.
- Using generic graph composition to bypass installed application operation
  admission.
- Reconstructing lifecycle or lineage from final rows when the receipt already
  carries it.

## Current Limits

- Only explicitly admitted entity, relation, existing-target, and lineage
  families are supported.
- Generic graph composition does not provide application authorization,
  selected installed invariants, or application publication receipts.
- Unsupported backend verification or identity-preserving operations deny
  rather than falling back.

## Related Docs

- [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
- [Graph Touch Obligation Authority](graph-touch-obligation-authority.md)
- [Existing Truth](../capabilities/existing-truth.md)
- [Writes And Intent Boundaries](../execution/writes-and-intents.md)
