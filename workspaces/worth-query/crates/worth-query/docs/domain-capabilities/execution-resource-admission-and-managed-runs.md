# Execution Resource Admission And Managed Runs

## What This Feature Is

Execution resource admission and managed runs let Query govern domain work that
is too large, interruptible, or stateful for one executor call. Use this when
work needs real capacity reservation, bounded provider steps, safe-point
cancellation, explicit cleanup, yield, same-runtime readmission, or coordinated
convergence.

The caller holds a move-only run authority. Query retains the exact resource,
Bridge, Relational, Signal, provider, and artifact relationships required to
advance or clean it up.

## Why You Use It

- Reject work before provider allocation when capacity is unavailable.
- Prove saturation through competing reservations and release.
- Bound each provider contact and observe cancellation at installed safe
  points.
- Yield retained work without pretending the operation completed.
- Preserve exact retry or recovery authority when cleanup cannot finish.
- Coordinate convergence without turning convergence into approval.

## Stable Entry Points

Resource declaration and admission:

- `worth_query_host::facade::declaration::domain_computation`
- `WorthQueryExecutionResourceRequest`
- `worth_query_host::facade::admission::resource_admission`
- `admit_execution_resource_plan(...)`
- `reserve_execution_resource_plan(...)`
- `WorthQueryAdmittedExecutionResourcePlan`
- `WorthQueryCapacityReservedExecutionResourcePlan`

Managed execution:

- `worth_query_host::facade::installed::domain_computation`
- `WorthQueryExecutionRuntime::start_direct_resource_attempt(...)`
- `WorthQueryExecutionRuntime::managed_run_admission(...)`
- `WorthQueryManagedRunAdmission::admit_direct(...)`
- `WorthQueryManagedRunAdmission::admit_workflow(...)`
- `WorthQueryAdmittedDirectRun::start()`
- `WorthQueryRunningDirectRun`
- `WorthQueryRunningWorkflowRun`
- `WorthQueryYieldedDirectRun`
- `WorthQueryYieldedWorkflowRun`

Convergence enters through `worth_query_host::facade::convergence_epoch`.

## Core Mental Model

Resource admission is a lifecycle, not a comparison against a descriptive
snapshot:

```text
installed resource contract + request + current support
  -> admitted plan
  -> atomic capacity reservation
  -> operation-bound resource attempt
  -> managed-run admission
  -> running attempt
  -> terminal cleanup or yielded retained package
  -> reservation release or transfer
```

The request declares scale, memory, concurrency, queue, chunk, deadline,
safe-point, and cleanup needs. The selected strategy and envelope remain
immutable after admission.

Managed-run admission joins the exact:

- installed-operation phase proof;
- reserved resource attempt;
- Query runtime and installation generation;
- Runtime Bridge execution basis;
- Relational read basis;
- Signal request generation and pressure state;
- provider session and installed step contract.

Query owns the phase progression. Bridge owns the causal binding between the
request and lower execution basis. Relational owns authoritative state
mechanics. Signal owns cancellation and pressure state. Providers own physical
work and retained memory.

A provider advances only through bounded steps. Pending output must be consumed
before another step or yield. Provider rejection and panic are contained as
typed failures that retain the remaining cleanup posture.

A successful yield terminates the current attempt but not the logical
operation. The yielded capability owns the checkpoint, retained artifacts,
capacity, applied-effect evidence, and affinity needed for same-runtime
readmission. Readmission mints fresh attempt and session generations.

Convergence consumes managed lifecycle outcomes. It distinguishes completed,
yielded, cancelled, failed, cleanup-pending, and recovery-required work. It
does not grant decision, invariant, publication, or resolution authority.

## How It Executes

```text
declare resource contract and safe-point family
  -> admit request against current support
  -> reserve capacity
  -> bind reservation to installed operation
  -> join Query, Bridge, Relational, Signal, and provider authority
  -> start direct or workflow run
  -> advance bounded provider steps
  -> complete, fail, cancel, or yield
  -> explicitly clean up or readmit the retained yielded capability
```

Every terminal reports what happened to each owner. A cleanup failure returns
the authority needed to retry or recover; it does not hide behind `Drop`.

## Small Example

```rust
let resource_attempt = runtime.start_direct_resource_attempt(
    &operation,
    admitted_plan,
)?;

let admitted = runtime
    .managed_run_admission(&bridge, &relational)
    .admit_direct(&operation, resource_attempt, truth_read_request)?;

let running = admitted.start();
```

The Bridge and Relational arguments are owning lower-runtime authorities, not
configuration markers. A foreign source or basis is rejected before provider
work.

## Real Example

```rust
let active = running.begin_graph_execution(&graph, graph_request)?;

match active.advance() {
    execution::WorthQueryDirectGraphStepOutcome::Continue(paused) => {
        match paused.yield_run() {
            execution::WorthQueryDirectYieldOutcome::Yielded(yielded) => {
                retain_for_readmission(yielded);
            }
            execution::WorthQueryDirectYieldOutcome::Denied(denied) => {
                continue_from_paused(denied);
            }
            execution::WorthQueryDirectYieldOutcome::RecoveryRequired(recovery) => {
                recover_yield(recovery);
            }
        }
    }
    execution::WorthQueryDirectGraphStepOutcome::ChunkReady(chunk) => {
        consume_rows(chunk.chunk());
        continue_from_step(chunk.acknowledge());
    }
    execution::WorthQueryDirectGraphStepOutcome::Completed(completed) => {
        continue_running(completed.into_running());
    }
    execution::WorthQueryDirectGraphStepOutcome::Cancelled(terminal)
    | execution::WorthQueryDirectGraphStepOutcome::TimedOut(terminal)
    | execution::WorthQueryDirectGraphStepOutcome::Exhausted(terminal)
    | execution::WorthQueryDirectGraphStepOutcome::Degraded(terminal)
    | execution::WorthQueryDirectGraphStepOutcome::Failed(terminal) => {
        handle_terminal_cleanup(terminal.cleanup());
    }
}
```

Workflow graph execution has the same discipline through its stage-specific
entry. Pending work, yield, terminal, and cleanup authorities remain typed and
move-only.

## How It Relates To Other Features

- [Installed Computation Artifact Contracts](./installed-computation-artifact-contracts.md)
  declares retained-memory and safe-point meaning.
- [Managed Artifact Ownership And Native Access](./managed-artifact-ownership-and-native-access.md)
  carries artifacts owned by the run.
- [Provider Sessions And Decision Read-Sets](./provider-sessions-and-decision-read-sets.md)
  starts from a running managed run.
- [Conditional Installed Operations](./conditional-installed-operations.md)
  owns installed eligibility; cancellation does not replace it.

## Program Adoption Resource Evidence

Program adoption reports resource evidence through the owners that perform the
work rather than through a second accounting registry:

| Evidence | Public source |
| --- | --- |
| Target branches and explicit order | owner-issued `WorthQueryProgramAdoptionCoverage` and ordered coverage |
| Affected existing state | `selected_entity_count()` on prepared/performed/unpublished adoption; `selection_work_units()` on preparation |
| Migration and continuation custody | `migration()` and `custody().dispositions()` |
| Owner contacts and publications | performed adoption's `relational_owner_contacts()`, branch, and composite commit; branch-set progress cardinality |
| Cumulative broader-scope work | branch-set `progress()` and `total_selection_work_units()` |
| Retained support memory | retirement inventory `retained_program_bytes()` plus interpretation/custody counts |

Local adoption scans only its selected branch and affected installed kinds.
Broader adoption pays once for the exact issued branch inventory and preflights
all ordered targets before the first effect. Unchanged components make no
mutation contact. Retained support bytes are stable logical installation
accounting, not allocator or heap introspection.

## Inspection And Debugging

Inspect:

- request identity, selected strategy, and admitted envelope;
- reservation scope, occupancy, retained bytes, and release counters;
- operation binding, attempt identity, and session identity;
- bounded-step contacts, safe-point observations, cancellation state, and
  backpressure;
- yielded checkpoint and retained-resource evidence;
- old and fresh generations during readmission;
- terminal kind and per-owner cleanup or recovery dispositions.

Denials before reservation should show zero capacity consumption. Denials
before provider admission should show zero provider calls.

## Anti-Patterns

- Treating a support snapshot as a reservation.
- Starting a provider session before capacity is reserved.
- Letting callers supply capacity, safe-point, or work-completion reports.
- Polling cancellation in an application-owned retry loop.
- Advancing while a pending chunk remains unconsumed.
- Reconstructing yield authority from checkpoint bytes or identities.
- Treating yielded work as completed, published, or executable without
  readmission.
- Using convergence as approval or conflict resolution.
- Relying on `Drop` as the only cleanup evidence.

## Current Limits

- Yield readmission is same-runtime and runtime-affine.
- Providers must cooperate with installed bounded-step and safe-point
  contracts.
- Multi-provider work has compensation and reconciliation semantics unless one
  genuine shared atomic authority is installed.
- Convergence observes managed terminals but cannot manufacture missing
  participant authority.

## Related Docs

- [Managed Artifact Ownership And Native Access](./managed-artifact-ownership-and-native-access.md)
- [Provider Sessions And Decision Read-Sets](./provider-sessions-and-decision-read-sets.md)
- [Runtime-Installed Domains](./runtime-installed-domains.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
