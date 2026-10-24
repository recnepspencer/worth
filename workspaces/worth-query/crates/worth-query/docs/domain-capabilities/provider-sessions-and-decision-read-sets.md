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

The higher application facade exposes an unknown durable fate as
`WorthQueryApplicationUncommitted::Indeterminate(...)`, returned as the `Err`
of `WorthQueryApplicationCommitOutcome::landed()` and carried by a request
lane's `WorthQueryApplicationMutationOutcome::Commit(...)`. The raw Relational
unresolved-durability value remains inside the owning execution boundary.

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
Relational publishes the commit to its durable medium before the live cutover,
and that durable fate can be unknown. Nothing was installed live, but the
commit may be durable. The application layer surfaces this as `Indeterminate`
(commit recovery required); it does not turn the result into a pre-effect
denial. The Relational owner refuses later publications until it is reopened,
and reopening resolves the fate.

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
  -> committed, already committed, or durability unresolved (Indeterminate)
```

Provider rejection or panic is caught at the exact protocol stage. Abort and
cleanup outcomes retain whether physical recovery is complete or required.
An unresolved durable fate is not a pre-effect denial: the commit may be
durable, so the operation requires commit recovery and is not aborted,
discarded, or rerun.

### Completed native application reads

Ordinary installed application progression supplies its completed read directly
to the primary provider. Completion pairs the original fact body with checked
locator, ordinal and installed-scope routing. Qualified native completion
also retains its actual original snapshot lease and owning admission. Cold
reconstruction and unqualified reads retain full capture and comparison. This
avoids rebuilding every application fact and effect lookup on each warm
attempt; applications supply neither routing nor a reuse hint.

Every attempt still receives fresh principal, authorization and provider-session
observations. Capture retrieves the actual registered attempt. Initial comparison
and precommit comparison both check that registration's generation, session and
native observation. The physical prefix can close together only when its original
body, admission and live native source correspondence remain qualified. Fresh
source-role and authorization evidence is compared individually. A different
readmitted source requires funded capture and comparison of all original facts.
Unknown facts and source queries without original query permission use that full
path as well.

Effect lowering consumes routing paired with the same original body and retains
the first sealed observation for each target. Installed preimage demands select
only observations eligible for actual mutation records and demanded loci; scope,
ambiguity and before-image checks still apply. Normalized own-write postconditions
remain separate and cannot certify those original reads. Retained snapshot and
allocation custody ends with its final owner, including refusal and abort paths.

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
    .capture_decision_read_set(
        decision_fact_requests,
        execution_request.allocation_policy(),
        Some(&request_scope),
    )?;

let fresh = match staged
    .read_authority()
    .compare_decision_read_set(&execution_request, captured)?
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
family or add an undeclared one. `execution_request` and `request_scope` belong
to the current admitted host call; retained data does not reconstruct them.

If the later application commit returns `Indeterminate`, do not retry the
mutation. Recover in the direction its evidence names after the Relational
owner is reopened and the commit's fate is resolved.

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
- the `Indeterminate` commit evidence when the durable fate is unknown,
  without inspecting a raw Relational durability value.

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
- Retrying, aborting, or discarding a mutation after its commit reports an
  unresolved durable fate.
- Exposing the raw Relational unresolved-durability value to application
  callers.

## Current Limits

- Provider sessions are runtime-local and generation-bound.
- A provider must implement the declared session and decision-fact capability
  set before runtime construction can preserve this lane.
- Freshness covers the installed fact closure, not undeclared application
  state.
- Completed native physical-read custody is runtime-local. Restored data, equal
  results, equal content and shared backing do not issue it. This optimization
  does not make posting registration, collection membership or delivery sparse.
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
