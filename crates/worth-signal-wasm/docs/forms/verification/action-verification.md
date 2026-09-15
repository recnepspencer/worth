# Action Verification

## What This Feature Is

This page covers the action-related proof surfaces inside `form.verification()`.

## Why You Use It

- verify that action planning and execution history stay distinct
- inspect attempt and execution counts in one package
- confirm action catalog and readiness digests
- compare the verification package with `form.debugAction(actionId)` when you
  need a friendlier runtime read beside the proof surface

## Stable Entry Points

- `form.verification().digests.actionCatalogDigest`
- `form.verification().digests.actionReadinessAdmissionDigest`
- `form.verification().actionHistory`
- `form.verification().actionExecutionHistory`

## Core Mental Model

The verification package preserves the same separation as the runtime:

- planning
- attempts
- execution lifecycle

## How It Executes

1. action declarations and plans are derived
2. attempts and execution operations are retained
3. verification exposes the corresponding digest and count lanes

## Small Example

```ts
const verification = form.verification();

console.log(verification.actionHistory.attempts);
console.log(verification.actionExecutionHistory.operations);
```

## Real Example

```ts
const before = form.verification();
form.attemptAction("submit");
const after = form.verification();

console.log(before.digests.actionCatalogDigest === after.digests.actionCatalogDigest);
console.log(after.actionHistory.attempts);
```

## How It Relates To Other Features

- Read [Action History](../diagnostics/action-history.md) for the retained
  runtime histories.
- Read [Action Execution](../actions/action-execution.md) for the public action
  execution surface.

## Inspection And Debugging

- action catalog digests stay stable when only lifecycle history changes
- action lifecycle digests change when attempts or execution history changes
- `form.debugAction(actionId).verification` gives the matching action-facing
  digest view beside the broader verification package; it is built on first
  access of that field (once per report), not on every `debugAction` call

## Anti-Patterns

- assuming a changed action lifecycle digest means the action schema changed
- treating action planning proof as if it were execution proof

## Current Limits

- the verification package reports counts and digests, not every host-side
  effect after handoff

## Related Docs

- [Action History](../diagnostics/action-history.md)
- [Action Execution](../actions/action-execution.md)
