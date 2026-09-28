# Provider Sessions And Decision Read-Sets

## What This Feature Is

Provider sessions give one running managed computation a sealed physical
execution context. Decision read-sets then capture every authoritative fact
that influenced proposed work and verify those facts are still current. Use
this pair when a graph-changing operation must not act on a caller-selected or
stale subset of its dependencies.

The application holds Query-minted phase values. The provider owns physical
session mechanics and fact observation; neither can fabricate the other's
authority.

## Why You Use It

- Bind provider work to the exact operation, basis, resources, and provider
  generation.
- Prevent prepared sessions from staging effects before reads are bound.
- Require every installed decision-fact family and exact fact count.
- Detect stale dependencies before staging a provisional change.
- Localize prepare, stage, abort, and provider-panic failures.

## Stable Entry Points

Consumer progression:

- `worth_query_host::facade::installed::domain_computation`
- `WorthQueryRunningDirectRun::admit_provider_execution_plan(...)`
- `WorthQueryRunningWorkflowRun::admit_stage_provider_execution_plan(...)`
- `WorthQueryAdmittedProviderExecutionPlan::readmit()`
- `WorthQueryPreparedProviderSession::prepare()`
- `WorthQueryPreparedProviderSession::bind_reads_and_effects()`
- `WorthQuerySessionReadAuthority::capture_decision_read_set(...)`
- `WorthQuerySessionReadAuthority::compare_decision_read_set(...)`
- `WorthQueryDecisionReadSetFreshnessOutcome`

Host provider integration:

- `worth_query_host::facade::installed::provider_session`
- `WorthQueryProviderSessionLifecycle`
- `WorthQueryDecisionFactProvider`
- `WorthQueryGraphParticipationProvider`

Provider traits belong in runtime assembly. Domain consumers should not invoke
them directly.

The higher application facade exposes a performed-publication terminal as:

- `WorthQueryApplicationUncommitted::SettlementDeferred(...)`, returned as the
  `Err` of `WorthQueryApplicationCommitOutcome::landed()` and carried by a
  request lane's `WorthQueryApplicationMutationOutcome::Commit(...)`
- `WorthQueryApplicationSettlementDeferred`
- `WorthQueryApplicationSettlementNextAction::RecoverDeferredApplicationSettlement`
- `WorthQueryPrimaryGraphApplicationRuntime::recover_deferred_application_settlement(...)`

The provider-session settlement carrier and raw Relational settlement remain
inside the owning execution boundary.

## Core Mental Model

The provider protocol is a type-level sequence:

```text
running managed run + installed graph authority
  -> admitted provider execution plan
  -> readmitted plan
  -> prepared provider session
  -> session-bound read and effect authorities
```

The sealed plan binds provider identity and generation, session token, operation
binding, resource attempt, Bridge basis, graph role, decision-fact families,
provisional dimensions, and installed invariant requirements.

Installed application queries and operations enter this protocol only after
one sealed obligation selection and graph-work plan. Read-only queries consume
the session-owned read port and terminate. Mutations continue through complete
decision facts, proposed state, real invariant execution, and
compare-and-commit. The lower managed run is the exact worker inside that
application session, not a competing session authority.

Compare-and-commit has a terminal that ordinary error handling must not erase:
the Relational branch movement can perform before durability or Query index
publication finishes. The provider then returns a typed settlement-deferred
carrier. The application layer wraps it in
`WorthQueryApplicationSettlementDeferred`; it does not turn the result into a
denial and does not expose the raw lower-runtime repair capability.

The basis is branch-qualified. A version ordinal has meaning only with its
typed branch and runtime identity; an equal ordinal from another branch cannot
satisfy the plan, session, read-set, proposal, invariant, commit, retry, or
publication chain.

Preparation does not authorize effects. Only
`WorthQuerySessionBoundReadsAndEffects` exposes separate read and effect
authorities. The read authority cannot stage effects; the effect authority
cannot observe decision facts.

A decision read-set is not an arbitrary collection of hashes. Each installed
fact family declares a kind and exact count. Query canonicalizes the requested
locators, calls the registered provider, checks returned evidence affinity, and
seals one complete receipt. Comparison asks the same provider to compare the
captured evidence against the same session basis.

```text
complete captured receipt
  -> compare every fact
  -> Fresh(fresh read-set authority)
     or Stale(stale evidence and counters)
```

Freshness is permission to continue into provisional lowering. It is not
mutation, publication, or commit authority. Stale means replan from a new
current basis.

## How It Executes

```text
running run admits installed graph participation
  -> Query seals the physical plan
  -> provider plan is readmitted against current run authority
  -> provider creates a fresh session token and generation
  -> Query binds separate read and effect authorities
  -> caller requests every installed decision fact
  -> provider returns bound evidence
  -> Query compares every captured fact
  -> fresh continues; stale replans
  -> mutation progression prepares and compares one branch-bound candidate
  -> committed, already committed, or performed-but-settlement-deferred
```

Provider rejection or panic is caught at the exact protocol stage. Abort and
cleanup outcomes retain whether physical recovery is complete or required.
Settlement-deferred is post-effect: the operation must be repaired, not
aborted, discarded, or rerun.

## Small Example

```rust
let staged = running
    .admit_provider_execution_plan(&graph)?
    .readmit()?
    .prepare()?
    .bind_reads_and_effects();
```

Each method consumes the preceding phase. A prepared session cannot be cloned
or used to skip directly to effects.

## Real Example

```rust
use worth_query_host::facade::installed::domain_computation as execution;

let captured = staged
    .read_authority()
    .capture_decision_read_set(decision_fact_requests)?;

let fresh = match staged
    .read_authority()
    .compare_decision_read_set(captured)?
{
    execution::WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh) => fresh,
    execution::WorthQueryDecisionReadSetFreshnessOutcome::Stale(stale) => {
        inspect_stale_facts(&stale);
        return replan();
    }
};

let program = staged
    .effect_authority()
    .lower_provisional_program(&fresh, effect_steps)?;
```

The installed operation determines which fact families are complete. The
caller supplies locators for that declared closure; it cannot omit a required
family or add an undeclared one.

If the later application commit returns `SettlementDeferred`, hand its opaque
carrier to `recover_deferred_application_settlement(...)`. That serialized
recovery boundary repairs durability, refreshes Query indexes, observes the
current branch head, proves the performed commit remains in its ancestry,
binds the current Bridge head, and validates the exact idempotency binding,
readmitting it when required. It preserves a later legal head and is safe to
call again.

## How It Relates To Other Features

- [Execution Resource Admission And Managed Runs](./execution-resource-admission-and-managed-runs.md)
  owns the running authority and resource attempt.
- [Provisional State And Invariant Execution](./provisional-state-and-invariant-execution.md)
  consumes the fresh read-set and lowered program.
- [Conditional Installed Operations](./conditional-installed-operations.md)
  owns Signal-backed eligibility. Decision facts describe the basis of a
  proposal, not conditional-node execution.
- [Basis Capability Lifecycle](../capabilities/basis-capability-lifecycle.md)
  explains basis affinity in ordinary Query surfaces.

## Inspection And Debugging

Inspect:

- provider identity and generation;
- provider session token identity and generation;
- operation, resource-attempt, graph-role, and Bridge-basis affinity;
- declared decision-fact families, kinds, and exact counts;
- read-set identity and requested, provider-call, compared, stale, and
  false-conflict counters;
- protocol failure stage and recovery posture.
- settlement-deferred stage, detail, counters, and optional Query publication
  failure without inspecting a raw settlement token.

If capture fails for incompleteness, no provisional provider call should have
occurred.

## Anti-Patterns

- Constructing a provider session token from an identity.
- Reusing a token, read-set, or fact receipt across sessions or generations.
- Calling provider lifecycle or fact traits directly from domain code.
- Capturing only the facts a caller happens to know about.
- Using raw hashes, callbacks, or caller-authored snapshots as decision proof.
- Letting effect authority capture facts or read authority stage effects.
- Treating a fresh read-set as mutation or commit authority.
- Continuing after `Stale` without replanning.
- Retrying, aborting, or discarding a mutation after its provider reports
  performed-but-settlement-deferred.
- Exposing the provider-session or Relational settlement carrier to application
  callers.

## Current Limits

- Provider sessions are runtime-local and generation-bound.
- A provider must implement the declared session and decision-fact capability
  set before runtime construction can preserve this lane.
- Freshness covers the installed fact closure, not undeclared application
  state.
- This document stops at the provider-session worker boundary. The higher
  installed application-operation progression owns compare-and-commit,
  idempotency, co-committed outbox meaning, typed settlement recovery, external
  dispatch, aftermath, and receipt-bound recovery.

## Related Docs

- [Canonical Graph Obligation Progression](./canonical-graph-obligation-progression.md)
- [Execution Resource Admission And Managed Runs](./execution-resource-admission-and-managed-runs.md)
- [Provisional State And Invariant Execution](./provisional-state-and-invariant-execution.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
- [Conditional Installed Operations](./conditional-installed-operations.md)
- [Basis Capability Lifecycle](../capabilities/basis-capability-lifecycle.md)
