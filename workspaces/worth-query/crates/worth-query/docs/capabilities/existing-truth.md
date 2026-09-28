# Existing Truth

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Existing-truth surfaces are the typed runtime capabilities for working against
already authoritative entities or relations instead of creating new truth from
scratch.

They cover:

- constructing a typed existing authoritative target binding
- updating, retargeting, superseding, or retiring that target through graph composition
- retaining or backend-verifying assertions as graph-composition evidence
- probing current authoritative values without mutating through probe intent

This is one capability family, not four unrelated feature surfaces. Probe
intent and verified graph-composition lanes still belong to existing truth;
they delegate through the shared admission lattice before bridge-backed work
executes.

## Stable Entry Points

- `WorthQueryExistingTruthTargetBinding::from_entity_target(...)`
- `WorthQueryExistingTruthTargetBinding::from_relation_target(...)`
- `workspace.compose_graph(...)`
- `runtime.probe_existing_intent(...)`
- `workspace.probe_existing_intent(...)`

Installed application invariants are not an existing-truth callback. They run
through the managed provider session after proposed state exists.

Direct workspace binding, update, assertion, verification, delete, and probe
helpers are sealed from consumers. They are not the public existing-truth
authoring model.

## Core Mental Model

Existing truth starts with a typed binding artifact, not a caller-owned identity
story and not a workspace helper seam. Once a target is bound, graph composition
or probe intent owns the canonical evidence path.

There are three distinct lanes:

- existing-target graph mutation
- existing-target retained or backend-verified assertion evidence
- existing-target probing without mutation

Do not flatten them together. They answer different questions and produce
different evidence.

## Existing-Target Mutation

Construct a typed binding artifact first, then pass it into graph composition:

```rust
let existing_task = WorthQueryExistingTruthTargetBinding::from_entity_target(
    WorthQueryExistingEntityTarget::new(existing_authority, task_identity)?
        .in_target_collection("Task")?,
)?;

let receipt = workspace.compose_graph(|graph| {
    graph.update_existing(existing_task, |task| {
        task.aspect("title.value", "Updated title")
    })
})?;
```

Admitted relation rewrites stay on the graph-composition surface instead of
pretending an update is a delete-plus-recreate disguise:

```rust
let existing_relation = WorthQueryExistingTruthTargetBinding::from_relation_target(
    WorthQueryExistingRelationTarget::new(existing_authority, relation_identity)?
        .in_target_collection("TaskRelation")?,
)?;

let receipt = workspace.compose_graph(|graph| {
    graph.update_existing(existing_relation, |relation| {
        relation
            .aspect("kind.value", "blocks")
            .aspect("status.value", "closed")
    })
})?;
```

## Verification And Probing

Use graph-composition verified lanes when the backend must prove asserted
values now and deny typed and early on mismatch or missing truth:

```rust
let receipt = workspace.compose_graph(|graph| {
    graph.update_existing_verified(
        existing_task,
        |verify| verify.aspect("status.value", "open"),
        |update| update.aspect("status.value", "closed"),
    )
})?;
```

Use `workspace.probe_existing_intent(...)` when the caller needs current
authoritative values for a bound existing target without executing a mutation:

```rust
let request = WorthQueryExistingTruthProbeRequest::new(
    existing_task,
    ["identity.id", "title.value", "status.value"],
)?;

let probe = workspace
    .probe_existing_intent(request)
    .execute()?
    .probe()
    .clone();
```

Use probing when the domain needs current authoritative truth as an input. Use
graph composition when the domain is staying in the mutation lane.
