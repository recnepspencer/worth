# Runtime-Preflight Workflow Contributions

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Feature Is

Runtime-preflight workflow contributions are the workflow category's stronger
runtime-bound path. They let a domain attach confirmation-required workflow
meaning to a real `ExecutionPreflightBundle`.

## Why You Use It

- you need workflow semantics tied to real runtime-preflight authority
- you want to avoid the weaker snapshot-token-only surrogate when real preflight
  evidence exists
- you need runtime authority preserved for geometry operations that are too
  dangerous for declaration-only planning

## Stable Entry Points

Proof-facing authoring:

- `WorthQueryWorkflowContributionAuthoring::confirmation_required_query_inspection_from_preflight(...)`
- `WorthQueryWorkflowContributionAuthoring::confirmation_required_mutation_reconciliation_from_preflight(...)`
- `WorthQueryWorkflowContributionAuthoring::confirmation_required_writeback_projected_state_diff_from_preflight(...)`

Canonical materialization:

- `materialize_query_workflow_declaration(...)`

## Core Mental Model

This is not the same thing as preview-session planning.

Here, the runtime preflight bundle is the authoritative basis for workflow
binding. Query scopes that real preflight identity by the contribution target so
different domain contributions over the same preflight do not collapse.

## How It Executes

1. start with a real `ExecutionPreflightBundle`
2. author confirmation-required workflow semantics from that preflight
3. bind to the appropriate contribution target
4. materialize a Query workflow declaration that preserves real preflight
   authority and scoped contribution identity

## Small Example

```rust
let requested =
    WorthQueryWorkflowContributionAuthoring::confirmation_required_query_inspection_from_preflight(
        "worth.spatial.workflow.inspect",
        "runtime preflight is required before inspecting this writeback boundary",
        preflight,
        "authority-binding:preflight",
    );
```

## Real Example

```rust
let requested =
    WorthQueryWorkflowContributionAuthoring::confirmation_required_writeback_projected_state_diff_from_preflight(
        "worth.spatial.workflow.writeback_diff",
        "runtime preflight must preserve the authoritative basis for this edge-split writeback",
        preflight,
        "authority-binding:preflight",
    );

let declaration = materialize_query_workflow_declaration(ready_contribution)?;
```

For geometry work, use this when an operation depends on real runtime authority
or writeback basis, not just declaration-time preview identity.

## How It Relates To Other Features

- Preview Inspection And Mutation Planning
  is the ordinary declaration-preview path
- Workflow Lanes: Common, Checked, Proof, And Raw
  explains why this surface lives lower than the ordinary preview lane

## Inspection And Debugging

- real preflight authority should survive into the materialized workflow
  declaration
- contribution target scoping should still keep distinct domain contributions
  from collapsing across one preflight bundle

## Anti-Patterns

- treating snapshot-token-only workflow authoring as equal to real preflight
  binding
- teaching this proof-lane surface as the first ordinary API for product code
- collapsing contribution target scope into ordinary runtime preflight identity

## Current Limits

- this is intentionally a sharper lane than ordinary preview planning
- if you only have declaration preview semantics, stay on the common lane
- if you need runtime authority, use the real preflight path instead of the
  snapshot-token surrogate

## Related Docs

- Preview Inspection And Mutation Planning
- Workflow Lanes: Common, Checked, Proof, And Raw
