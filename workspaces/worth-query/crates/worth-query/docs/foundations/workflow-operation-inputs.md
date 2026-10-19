# Workflow operation inputs

## What This Feature Is

`WorthQueryWorkflowOperationInputs` captures a finite set of original typed
operation descriptions. A native `RequiredWorkflowOperation` selects the exact
installed action from that set and transfers its input to the existing native
workflow-operation owner. Registration order does not decide execution order.

## Why You Use It

Use this when one user command needs several differently typed business
operations, and the authored graph—not a host match on stage or binding name—must
decide which operation is currently owed. Capture inputs before any effects,
then keep the actual owners while execution, refusal, recovery and cleanup proceed.

## Stable Entry Points

Import these through `worth_query_host::facade::application_entry`:

- `WorthQueryWorkflowOperationInputs::new(program_runtime)`.
- `inputs.with::<InstalledAction, _>(original_description)`.
- `inputs.select(workflow_vocabulary, &required)`.
- Selected owner: `required`, `advance`, `close`, `into_operation`.
- Completed owner: `required`, `close`, `into_operation`.
- Terminal owner: `close`, `into_operation`; no execution entrance.

The selected action must have an owning `ApplicationSourceCompletion` delivery
and a supported native admission binding. Required-output action deliveries do
not enter this pack. Registration also checks installed action membership,
runtime ownership and guarded, non-control binding applicability. Successful
registration is not execution authorization.

## Core Mental Model

The graph owns order, occurrence and approval obligations. The original funded
description owns input, preconditions, idempotency key and branch. The input pack
is only custody bookkeeping: one entry per binding, transferred at most once in
this finite submission. A sealed heterogeneous roster preserves actual Rust
types without erasure or a consumer-provided dispatcher.

Binding identity narrows the candidate. Native `for_workflow_operation` still
checks the exact input identity, branch, requirements and issued occurrence;
native `retain_operation` still constructs the real owning operation. Neither
an inspected binding string nor an input pack grants permission.

## How It Executes

1. Capture each original intent using `idempotency_owned` and register its actual
   complete installed action before running any effects.
2. Advance the authored graph through its native control surface. When it returns
   `AwaitingOperation(required)`, pass that requirement to `select`.
3. Advance the selected owner with a fresh authenticated request on its original
   branch. Native admission, publication, recovery and settlement remain in the
   existing `WorthQueryWorkflowOperation`.
4. Retain `Progress` and `Refused` owners unchanged. `Completed` retains real
   result/recovery evidence and cleanup; `Terminal` retains stopped custody, not
   a successful effect. No outcome is automatically retried or closed.
5. Freshly advance the graph for its next need. Close actual completed owners
   under fresh authority; a refused close returns the same owner.

`with` refusal returns the original pack, incoming description and typed reason.
An effect-free selection refusal restores the exact description. Once transferred,
selection returns `AlreadyTransferred`; an execution refusal never puts the input
back. Resume the returned operation, not a reconstructed description.

## Small Example

Given original funded descriptions and complete installed actions:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let mut inputs = WorthQueryWorkflowOperationInputs::new(program_runtime)
    .with::<SubmitAction, _>(original_submit)?
    .with::<BeginAction, _>(original_begin)?;
let selected = inputs.select(&workflow, &required)?;
let progress = selected.advance(&fresh_request);
```

The `?` paths here require caller errors that preserve the returned original
owners. `with` prepends internally: Begin is physically first, but a Submit
requirement still selects Submit. Neither the order of registration nor the host
names the next action.

## Real Example

Suppose a graph orders Submit then BeginReview. Submit publishes a Submitted
posture and its audit; BeginReview publishes Review and another audit. These are
two business commits, not an atomic transaction. Capture both original descriptions
before the first commit. Keep the completed Submit owner while the graph issues
the BeginReview requirement and the pack selects its original input.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let begin = inputs.select(&workflow, &begin_requirement)?;
let begin = match begin.advance(&request) {
    WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
        report(&denial);
        operation // Preserve the actual pending owner; do not recapture input/key.
    }
    other => return retain_progress(other),
};
let next = begin.advance(&freshly_authenticated_request);
```

A cancellation before Begin's effect leaves the committed Submit intact. Resuming
the same Begin owner under fresh authority does not execute Submit again. If
closing Submit also refuses, retain that actual completed owner independently.
Cleanup completion and workflow terminality are separate facts.

The input pack does not derive logical command identities. Suppliers define
stable native binding idempotency namespaces. If two distinct bindings use the
same scalar caller key, their namespaces must distinguish those operations.
Do not add a local execution counter or mint a new key on resumption.

## How It Relates To Other Features

- [Workflow operations](workflows.md#operations) own occurrence authority and
  the original operation's complete source/recovery lifecycle.
- `run_workflow` advances native control work and stops at an operation need;
  this pack does not turn it into an operation executor.
- Required-output actions keep their existing owning delivery surfaces. This
  source-completion pack does not invent projections for them.
- Source completion, workflow acceptance and physical cleanup retain their
  existing separate obligations, including external-source custody.

## Inspection And Debugging

`required()` exposes the exact native requirement. Observe ordinary governed
business reads and native history for actual effects. `NotSupplied` means this
pack lacks that binding; `AlreadyTransferred` means its input is held by the
returned operation. Binding and construction denials retain their typed native
causes. A foreign runtime refuses before transferring any entry.

`into_operation` returns the structural sum of the actual native owners for
deliberate lower-level integration. Its `Here`/`Other` position is a type-layout
detail, not workflow state or a permission decision.

## Anti-Patterns

- Matching stage or operation names to choose the next business operation.
- Treating registration or successful selection as capability authorization.
- Recapturing input or changing the key after an operation has transferred.
- Discarding completed/terminal/refused owners or treating Drop as settlement.
- Reporting two operations as one atomic or durably retained command.

## Current Limits

This is a finite in-memory input pack, not a serialized command, Store integration,
process-crash continuation, whole-command replay endpoint or managed multi-operation
runner. It does not automatically advance a graph, retry refusals, compensate a
first commit, or collect and close all operation owners.

One entry per binding applies to a pack, not to the workflow language. A loop or
later occurrence of the same binding needs a new logical submission with its
own original input. No scalar input is substituted from recorded digests.
Action delivery must be source-completion; output-delivery composition is not
generalized by this API.

## Related Docs

- [Workflows](workflows.md)
- [Ordinary application front door](ordinary-application-front-door.md)
- [Application aftermath and recovery](../execution/application-aftermath-and-recovery.md)
