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

Inbound external completion uses a separate installed resource profile on the
operation's source contract. `maximum_outstanding_dispatch_provenance` reserves
the committed outbox relationship before an effectful commit, including the
gap before World publication. `maximum_accepted_occurrences` and
`maximum_accepted_bytes` govern verified but still retained completions;
`maximum_concurrent_publications` bounds their World attempts. Envelope and
payload byte limits, replay window, discovery work, and cleanup work are finite
at installation. Exhaustion denies new admission before taking custody; it
does not evict pending or unpublished evidence.

The verifier is installed by the host for one declared operation; the request
cannot nominate another mechanism. Query rejects an oversized envelope before
calling it. The installed verifier's finite work ceiling is part of that
operation contract, and the trusted mechanism must stop with a typed denial
when its budget is spent. Bank's fixed v1 signature and decoder walk only the
bounded envelope. A byte ceiling alone would not prove arbitrary product
verifier CPU work, so hosts must install a budget-aware mechanism.

An outbound physical attempt holds a move-only claim on its original finite
outbox reservation. If a callback seals the terminal while that attempt is
still in flight, the reservation stays charged. A real `Completed` observation
transfers the claim into bounded transport recovery custody until an exact
World terminal or matching callback winner is confirmed. No capacity check
after the remote effect may discard that observation.

After World performs a completion, the owner keeps exact terminal history and
a compact lookup entry. Host-requested cleanup can reclaim an accepted payload
slot only after signed expiry, original dispatch handoff, and delivery
settlement. The exact lookup checks its retained World protection and sealed
Relational pairing under the declared cleanup work limit. A lost derived
terminal index is repaired with an explicit bounded history pass; ordinary
requests fail closed while that proof is incomplete. Runtime close must expose
unresolved obligations, and a
forced process exit does not preserve this process-local custody.

`observe_inbound_cost(&installed_source)` is a cumulative, read-only,
operation-scoped observation of the signed inbound lane. It reports verifier
input bytes, exact terminal and outbox key
probes, selected outbox records, completion candidate prepares, World
publication attempts/performed publications, current accepted count/charged
bytes, and outstanding dispatch provenance. These fields count their named
owner events; Bank separately owns HTTP/rail contacts and workflow separately
owns transitions. Compare snapshots around a fixed-size callback at small and
large unrelated populations. Duplicate volume may add verification and exact
lookup work, but must add no prepare, World publication, redispatch, workflow
transition or retained occurrence after terminal settlement.

Cleanup's `maximum_work` is a selected-candidate page bound, intersected with
the installed `maximum_cleanup_work`; each selected candidate gets exact
canonical revalidation before release. Pending and unpublished entries never
enter the expiry turnover index. Explicit index reconstruction has its own
bounded World-page, changed-record and ancestry budgets, with no hidden scan
on ordinary lookup. A blocked repair leaves the index unavailable rather than
answering from partial proof.

The trusted host enters repair through the runtime-issued verifier handle:

```rust
let complete = application.repair_completed_inbound_index(
    &installed_source,
    world_page,
    changed_records_per_commit,
    ancestry_commits_per_completion,
)?;
```

The installed discovery ceiling narrows `world_page`; the other two values are
explicit finite repair budgets. `Ok(false)` retains a cursor for another call.
The handle selects an installed runtime, while the repair itself rebuilds the
runtime's disposable terminal index and never runs from ordinary lookup.

The Bank rail installation owns one serial cue-driven maintenance task. Each
batch uses fresh request controls and the installed verifier handle, examines at
most the declared discovery work, rotates past blocked entries, and reclaims only
terminal entries that satisfy the declared cleanup bound. Orderly server
shutdown waits for the current batch; this task continues already accepted
custody after the source envelope expires without opening a raw correlation
selector to callers.

## Core Mental Model

### Ordinary work-limit ownership

Applications declare semantic result cardinality, effect permissions, and retained
representation. They must not reproduce Query's traversal, source-observation, or
invariant-closure cost formulas. Installed host profiles own operational safeguards;
explicit binding/request ceilings are deliberate restrictions. This policy does not
infer an arbitrary native algorithm's cost or make it preemptible.

The decisive proof uses a real nested query and composed output demand. Adding a
projected field or installed invariant requires no application cost algebra. A large
parent allowance admits a small child when its actual need fits its own ceiling.
Independent variants exhaust capacity, cancel during traversal, and recover demand
custody: they deny before publication and release transient reservations. Raising
fixture constants cannot satisfy this proof.

- Query bindings use `ApplicationQueryBindingLimits::results(n)` ordinarily;
  `bounded(n, work)` is an explicit cap. Installation retains the distinction,
  without zero or unlimited sentinels. The existing host query resource profile
  supplies a configurable finite work guard (default 1,048,576 metered units).
  All query modes resolve host/binding policy before request narrowing; lower
  admission also enforces the host guard. This is a runaway safeguard, not a
  throughput promise or guaranteed sufficiency for unbounded `Many` fan-out.
- Root traversal, materialization and source observation poll cancellation and
  deadlines internally. Opaque native calls stay bounded; this does not promise
  interruption inside those calls. Actual work remains metered.
- Output demands use installed defaults, not implicit `1/1` work/byte allowances.
  Child allowance intersects host, caller and artifact ceilings. Admission checks
  the child's required resources, not the parent's offered maximum against a
  child maximum. Framework currentness and provider algorithm work stay separate.
- Candidate validation has no aggregate work quota or descriptor-max prediction.
  Installed invariants retain their own algorithm controls and actual execution
  evidence. Exact closure remains checked before publication; effect shape,
  representation-byte and real concurrent-capacity admission remain intact.
- Local result-buffer and candidate-representation accounting is not a global
  heap reservation. Provider demand estimates are not aggregate memory proof.
  Managed reservations retain their consuming lifecycle.

```rust,ignore
const LIMITS: ApplicationQueryBindingLimits =
    ApplicationQueryBindingLimits::results(1);

// Ordinary output requests inherit this installation's policy, even when the
// host has configured different limits from the standard profile.
let controls = WorthQueryOutputDemandControls::default();

// Candidate representation is authored; actual invariant closure is installed.
let resources = ApplicationCandidateResourceCeiling::representation_bytes(4096);
```

`WorthQueryInMemoryApplicationLimits::with_output_demand_resources(...)` installs
`WorthQueryOutputDemandResourceProfile`. Its four independent dimensions are source
currentness work, producer work, producer retained bytes, and settlement attempts.
The standard profile allows 4,194,304 units in each work dimension, 4 MiB of producer
retained representation, and 64 advances per `settle` call. These are finite runaway
guards, not measured throughput guarantees or aggregate heap reservations. A host
can select its policy once instead of teaching every graph call those constants.

`WorthQueryOutputDemandControls::default()` (also `host_policy()`) carries no caller
overrides. An explicit `new(work, bytes)` narrows both work dimensions and producer
retention; source-currentness and settlement-attempt restrictions can be set
separately. Every start, child, and recovery intersects controls with the current
host. Artifact ceilings constrain producer work and retained bytes, not framework
currentness. A larger parent allowance does not itself consume resources or make a
small child inadmissible. `settle` returns `Pending` when its advance allowance is
spent; it does not manufacture completion.

Destination owners, under `workspaces/worth-query/crates` (existing paths revised):

```text
worth-query-declaration/src/
  application_query/binding/limits.rs       semantic bound and optional cap
  application_operation/candidate/         retained candidate representation
worth-query-installation/src/
  application_query/binding/limits.rs       installed/resolved limit types
  application_operation/contracts/         exact invariant closure
worth-query-execution/src/domain_computation/
  execution_runtime/                       installed host policies
  primary_graph/application_query/          admission and interruption
  primary_graph/application_contribution/producer/demand/
                                           phase-specific demand admission
  primary_graph/application_installation/program/
                                           child ceiling composition
  primary_graph/application_attempt/effect_program/
                                           consuming candidate reservation
  primary_graph/provider/invariant_execution/
                                           exact pre-effect closure (preserved)
worth-query-publication/src/application_entry/
  query/                                   all modes share resolution
  demand/                                  installed ordinary defaults
```

Implement query ownership/interruption first, demand composition and candidate
closure next, then migrate the real House consumer and verify recovery/checkpoint
behaviour. Default and restricted calls use the same admission/execution lane.
Resource tuning grants no authority, changes no source identity and cannot make
incompatible continuations/checkpoints valid. Existing explicit caps never silently
widen. Declaration compatibility changes require normal readmission.

This guide, facade examples and the Query AI readme must describe the same ordinary
path. Acceptance requires owner and public-request tests, the pinned House consumer,
narrow-only continuation/readmission tests, denial cleanup and boundary enforcement.
Compilation alone is not closure evidence.

### Managed resource lifecycle

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
