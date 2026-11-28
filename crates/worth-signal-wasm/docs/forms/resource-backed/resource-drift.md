# Resource Drift

## What This Feature Is

This page covers what happens when the resource-backed source changes under a
local draft.

## Why You Use It

- inspect whether the local draft was preserved, rebased, blocked, or left in
  conflict
- understand whether the drift is stale or resolved
- see the blockers and messages attached to drift events

## Stable Entry Points

- `form.resourceDrift()`

## Core Mental Model

Resource drift is not just "the source changed." It is the controller's answer
to what that source change meant for the current draft.

## How It Executes

1. the resource-backed source changes
2. the runtime compares the new source to the current draft/effective value
3. a drift report is recorded with status, blockers, and messages

## Small Example

```ts
console.log(form.resourceDrift().summary);
```

## Real Example

```ts
console.log(form.resourceDrift().current);
console.log(form.resourceDrift().history);
```

## How It Relates To Other Features

- Read [Source Compatibility And Draft Migration](../validation/source-compatibility-and-draft-migration.md)
  when the source change is schema drift rather than resource drift.

## Inspection And Debugging

- `summary.status` and `summary.stale` are the fastest reads
- `current.blockers` and `current.messages` explain the user-facing impact
- `visibleSelectionKind` shows what source selection the drift is tied to

## Anti-Patterns

- treating resource drift like ordinary validation
- silently overwriting draft truth when the resource changes

## Current Limits

- route-coupled drift handling belongs in later docs

## Related Docs

- [Resource Settlement](./resource-settlement.md)
