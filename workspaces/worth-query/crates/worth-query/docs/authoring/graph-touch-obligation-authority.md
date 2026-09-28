# Graph Touch Obligation Authority

## What This Feature Is

Graph obligation authority records the graph work required by an installed
application query or operation. Query derives that sealed meaning from typed
application contracts and carries it through selection, planning, execution,
and terminal evidence. Application callers do not register obligations or
attach callbacks to a graph mutation.

Use this feature when you need to inspect why an installed query or operation
requires graph reads, authorization observations, mutation touches, effects,
or invariant execution.

## Why You Use It

- Keep required graph work attached to the installed query or operation.
- Make the required lower owners and terminal evidence inspectable.
- Prevent selection or a support declaration from masquerading as execution.
- Preserve resource, effect, and warm-work accounting across the whole
  application progression.

## Stable Entry Points

- Declare application queries and operations through `worth_query_decl::facade`.
- Inspect `installed_query.graph_obligations()` or
  `installed_operation.graph_obligations()` through
  `worth_query_host::facade::domain`.
- Inspect an operation's exact declared graph ceiling through
  `installed_operation.contracts().touches().scopes()`.
- Inspect its declared in-process effect emissions through
  `installed_operation.contracts().emissions().emissions()`; this does not by
  itself grant external dispatch authority.
- Produce downstream read-only evidence with
  `worth_query_host::facade::inspect_installed_graph_obligations(...)`.
- Execute through the ordinary application runtime, which owns selection,
  planning, session progression, and terminal construction.

There is no public obligation registry, executor, manual invariant callback,
or caller-created dispatch envelope. An emitted application effect is installed
separately from graph touches, and only an explicit external-effect declaration
opens the escaping dispatch lane.

## Core Mental Model

Every installed query or operation owns one non-empty sealed obligation set.
Each row says:

- what kind of work is required;
- which lower owner must provide evidence;
- which installed contract selected it;
- its resource and effect posture; and
- which terminal proves completion.

The current kinds are:

- `GraphRead`;
- `AuthorizationObservation`;
- `MutationTouch`;
- `EffectApplication`; and
- `InvariantExecution`.

Selection is deterministic installed lookup. It carries no execution
authority. A terminal exists only after the selected row reaches its real
owner and the session consumes that owner's evidence.

Application-operation touches are structural values, not strings. The public
`WorthQueryOperationTouchScope` variants are `CreateEntity`, `DeleteEntity`,
`WriteField`, `LinkRelation`, and `UnlinkRelation`. Each carries its exact
schema binding and locus; field writes also carry the installed aspect contract
and canonical field path. `DeclaredDomain` belongs to portable domain-operation
contracts and is not emitted by application-operation compilation.

## How It Executes

```text
installed query or operation
  -> sealed obligation set
  -> intent-specific selection
  -> requirement, cost, budget, and capacity admission
  -> branch- and basis-bound session
  -> Relational / Runtime Bridge / Signal / installed-provider work
  -> read or commit terminal
  -> read-only publication and inspection
```

Relational owns graph truth and exact invariant mechanics. Runtime Bridge owns
installed neutral correspondence and crossing evidence. Signal owns policy
evaluation evidence. Query owns their legal composition and the typed
progression; it does not recreate the lower decisions.

## Small Example

```rust
use worth_query_host::facade::inspect_installed_graph_obligations;

let proof = inspect_installed_graph_obligations(
    "audit-service",
    installed_query.graph_obligations(),
)?;

for row in proof.rows() {
    println!("{:?} requires {:?}", row.kind(), row.required_owners());
}
```

The returned proof is an inspection snapshot. It cannot select, plan, execute,
validate, commit, or publish.

Inspect application touch meaning by matching the typed variants:

```rust
use worth_query_host::facade::domain::WorthQueryOperationTouchScope;

for scope in installed_operation.contracts().touches().scopes() {
    match scope {
        WorthQueryOperationTouchScope::CreateEntity(entity) => {
            inspect_entity_create(entity.schema(), entity.entity());
        }
        WorthQueryOperationTouchScope::DeleteEntity(entity) => {
            inspect_entity_delete(entity.schema(), entity.entity());
        }
        WorthQueryOperationTouchScope::WriteField(field) => {
            inspect_field_write(
                field.schema(),
                field.entity(),
                field.contract(),
                field.field_path(),
            );
        }
        WorthQueryOperationTouchScope::LinkRelation(relation) => {
            inspect_link(relation.schema(), relation.relation(), relation.from(), relation.to());
        }
        WorthQueryOperationTouchScope::UnlinkRelation(relation) => {
            inspect_unlink(relation.schema(), relation.relation(), relation.from(), relation.to());
        }
        WorthQueryOperationTouchScope::DeclaredDomain(_) => unreachable!(
            "application operation compilation does not create domain-only scopes"
        ),
    }
}
```

This contract says what execution may touch. Only the exact Relational
performed-touch evidence says what execution did touch. Query compares the
two; callers must not promote the declaration into a completion claim.

## Real Example

An application query with one installed graph read produces one `GraphRead`
row. That row names `RelationalGraph` as its required owner and
`GraphReadProduct` as its terminal requirement. Query binds the row's installed
graph to the specialized graph-read review, reserves capacity, opens one
managed session, and consumes the read through the session-owned port.

The executable public proof is
`worth-query-host/tests/canonical_graph_progression.rs`.

An application mutation may install several rows. Query must obtain the
complete authorization facts, build proposed state, execute every selected
installed invariant through its real provider, and compare-and-commit before a
commit publication receipt can exist.

An emit-only operation has no graph touch merely because it emits. Ordinary
typed emission targets remain in the sealed installed emission contract; the
one escaping external-effect correlation family is a separate installed
aftermath and dispatch contract.

## How It Relates To Other Features

- [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
  owns the complete cross-surface model and migration map.
- [Graph Read Access Planning](graph-read-access-planning.md) owns specialized
  access requirements, budgets, and plan review for a `GraphRead` row.
- [Provisional State And Invariant Execution](../domain-capabilities/provisional-state-and-invariant-execution.md)
  owns real installed invariant execution.
- [Graph Composition Authoring](graph-composition-authoring.md) remains a
  generic workspace mutation feature. It does not select or execute installed
  application obligations.

## Inspection And Debugging

Inspect:

- set identity and installed subject;
- row slot, kind, owners, selection basis, and terminal requirement;
- installation row count and fixed selector-index size;
- selection probes and exact-zero warm canonical work;
- admitted plan and session identities; and
- actual terminal work and cleanup receipts.

If a selected row has no actual owner evidence, execution is incomplete. Do
not infer completion from a selected kind, support row, callback return, or
formatted receipt.

## Anti-Patterns

- Consumer-owned obligation registries or validator tables.
- Manual invariant packs beside installed provider execution.
- Local graph walks used to replace a typed denial.
- Selection-backed fake executor verdicts.
- Public constructors for requirement, review, plan, session, or terminal
  products.
- Treating branch, preview, and current evidence as interchangeable.
- Encoding `write:Entity/Aspect/Field` or `link:Relation:From->To` and parsing
  that private grammar in a compiler or consumer.
- Treating `Emit` as graph mutation or a declared touch as performed evidence.

## Current Limits

- Read-only adoption is inspection, not an extension or execution API.
- Generic workspace read and graph-composition engines remain supported for
  their existing responsibilities.
- Durability, restart recovery, multiple branch heads, and concurrent branch
  writers are outside this feature.

## Related Docs

- [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
- [Graph Read Access Planning](graph-read-access-planning.md)
- [Graph Composition Authoring](graph-composition-authoring.md)
- [Provider Sessions And Decision Read-Sets](../domain-capabilities/provider-sessions-and-decision-read-sets.md)
