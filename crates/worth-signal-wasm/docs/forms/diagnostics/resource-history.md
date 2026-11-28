# Resource History

## What This Feature Is

This page covers the retained history families around resource-backed forms:

- replay and restore
- reset
- resource drift

## Why You Use It

- inspect how a resource-backed form reconciled local and server truth
- debug exact replay or restore operations

## Stable Entry Points

- `form.replayRestoreHistory()`
- `form.resetHistory()`
- `form.diagnostics().replayRestoreHistory`
- `form.diagnostics().resetHistory`
- `form.resourceDrift()`

## Core Mental Model

Resource-backed history is not one timeline. It is a family of adjacent
histories around resource truth.

## How It Executes

1. resource-backed operations occur
2. the form retains the corresponding history artifacts
3. diagnostics summary and verification expose retained counts and digests

## Small Example

```ts
console.log(form.replayRestoreHistory());
console.log(form.resetHistory());
```

## Real Example

```ts
await form.restoreExactResourceSource({ reason: "return to server truth" });

console.log(form.resourceDrift().history);
console.log(form.replayRestoreHistory());
console.log(form.resetHistory());
```

## How It Relates To Other Features

- Read [Replay And Restore](../resource-backed/replay-and-restore.md) for the
  resource-backed operations themselves.
- Read [Verification Packages](../verification/verification-packages.md) for the
  retained history counts inside the proof package.

## Inspection And Debugging

- use replay/restore history for exact replay and restore events
- use reset history for local reset and rollback behavior
- use resource drift reports for the current drift and its retained history

## Anti-Patterns

- expecting all resource-backed history to live in one generic list
- using ordinary state history when you need resource-source restore proof

## Current Limits

- this page covers form-owned resource history only, not the deeper resource
  line history families outside the form facade

## Related Docs

- [Replay And Restore](../resource-backed/replay-and-restore.md)
- [Verification Packages](../verification/verification-packages.md)
