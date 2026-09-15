# Action History

## What This Feature Is

This page covers the retained action histories available through the form
controller and diagnostics surface.

## Why You Use It

- inspect denied or accepted attempts separately from execution lifecycle
- debug retry, cancel, timeout, fulfill, and reject flows
- connect action history counts to verification and diagnostics

## Stable Entry Points

- `form.actionHistory()`
- `form.actionExecutionHistory()`
- `form.debugAction(actionId)`
- `form.diagnostics().actionHistory`
- `form.diagnostics().actionExecutionHistory`

## Core Mental Model

There are two histories:

- action attempt history
- action execution lifecycle history

Planning and execution are intentionally separate. `debugAction(...)` is the
task-shaped common path that pulls the nearby pieces together for one action.

## How It Executes

1. `attemptAction(...)` records a result artifact
2. `executeAction(...)` starts execution lifecycle tracking
3. fulfillment, rejection, timeout, cancel, or retry extend execution history

## Small Example

```ts
const result = form.attemptAction("submit");

console.log(result.resultKind);
console.log(form.actionHistory().length);
```

## Real Example

```ts
const execution = await form.executeAction("submit");

if (execution.resultKind === "pending") {
  form.fulfillAction(execution.operationId, {
    reason: "server accepted canonical value",
  });
}

console.log(form.actionHistory());
console.log(form.actionExecutionHistory());
console.log(form.debugAction("submit"));
```

## How It Relates To Other Features

- Read [Action Execution](../actions/action-execution.md) for the execution
  surface itself.
- Read [Action Verification](../verification/action-verification.md) for the
  proof package view of the same history families.

## Inspection And Debugging

- action attempt history tells you whether a plan was accepted or denied
- action execution history tells you what happened after execution started
- `debugAction(actionId)` gives you the current plan, nearby blockers, and the
  filtered attempt/execution history for one action; its `verification` and
  `digest` fields are computed on first access, so reading `pending` or
  `latestExecution` on every render never rebuilds the verification package
- `form.diagnosticsSummary().histories.actionAttempts` and
  `actionExecutions` show retained counts

## Anti-Patterns

- treating an accepted action plan as if execution has already started
- using execution history alone when you need denied-attempt evidence

## Current Limits

- action history explains form-owned action lifecycle, not external job
  processing after the form hands off control

## Related Docs

- [Action Execution](../actions/action-execution.md)
- [Action Verification](../verification/action-verification.md)
