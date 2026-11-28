# Branch-Native Effects

## What This Feature Is

`branchNative()` is the optimistic resource profile for requests that must stay
isolated while they are unresolved. Each admitted patch gets its own Signals
branch and effect identity. The runtime derives the visible value from confirmed
server data plus the still-open branches.

## Why You Use It

- show an edit immediately without maintaining a second UI cache;
- let overlapping requests succeed or fail independently;
- reconcile a server response at the declared item, field, region, JSON path,
  aspect, summary, insert, delete, or replacement locus;
- inspect branch, projection, dependency, and retirement proof.

## Stable Entry Points

- `await createSignals()`
- `signals.resource.effects.branchNative()`
- `line.patch(...)`
- `line.effects()`
- `line.history().rollbackEffect(effectId)`

Other profiles such as `pessimistic()`, `serverCanonical()`, and
`nonReversible()` deliberately make different optimism or recovery promises.

## Core Mental Model

The application branch is not the optimistic write container. Each effect owns
a child branch. Confirmed server data remains canonical; open effects form a
derived projection exposed through `line.value()`.

An effect identity is the handle for closeout. A branch ID proves native
ancestry. A dependency ID says which request must settle first. Keep those
roles separate.

## How It Executes

1. The patch helper validates the resource locus.
2. `line.patch(...)` admits an effect branch and returns its `effectId`.
3. The projection includes the effect immediately.
4. `confirm(effectId)` reconciles server truth and retires the branch.
5. `reject(effectId)` retires the branch without changing canonical value.
6. The projection rebuilds from the remaining open effects.

Unsupported loci, incompatible dependency generations, missing branch proof,
and invalid policies deny before visible state changes.

## Small Example

```ts
const signals = await createSignals();
const tasks = signals.api({
  effects: signals.resource.effects.branchNative(),
}).url("/tasks")
  .response(signals.resource.response.array({ itemId: (task) => task.id }))
  .list({ load: () => client.listTasks() });

const line = tasks.line({});
await line.awaitSettlement();
const admission = await line.patch(tasks.patch.item({
  itemId: "task:1",
  nextItem: { id: "task:1", title: "Draft" },
}));

if ("effectId" in admission) {
  await line.effects().confirm(admission.effectId);
}
```

## Real Example

```ts
const openBefore = line.effects().open();
const result = await line.patch(patch, {
  idempotencyKey: request.id,
  serverCorrelationId: request.correlationId,
});

if (!("effectId" in result)) throw new Error("branch-native admission required");

try {
  const response = await request.send();
  const closeout = await line.effects().confirm(result.effectId, {
    responseId: response.id,
    serverRevision: response.revision,
    serverPatch: response.patch,
  });
  audit(closeout.reconciliation, closeout.retirement);
} catch (failure) {
  const closeout = await line.effects().reject(result.effectId, {
    responseId: failure.responseId,
  });
  audit(closeout.retired, closeout.projection);
}
```

The application reports the outcome. It does not calculate the next visible
value or apply an inverse patch.

## How It Relates To Other Features

- [Concurrent Optimistic Effects](./concurrency-and-dependencies.md) covers
  siblings and dependency DAGs.
- [Rollback And Recovery](./rollback-and-recovery.md) covers targeted rejection.

## Inspection And Debugging

- Use `line.effects().open()` for all open effects.
- Use `line.effects().get(effectId)` for one effect.
- Use `line.effects().projection()` for canonical versus projected posture.
- Use `line.diagnostics().lastEffect` only as a recent diagnostic breadcrumb.
- Use settlement receipts for reconciliation and retirement evidence.

## Anti-Patterns

- Do not treat the current application branch as the effect branch.
- Do not choose closeout from `lastEffect` when several effects are open.
- Do not restore a shared snapshot or issue a compensating patch on rejection.
- Do not hand-build effect envelopes or dependency indexes.

## Current Limits

Admission requires a supported resource locus and native branch proof. Same-
locus conflict resolution is deterministic, but it does not invent domain
merging for two incompatible server responses.

## Related Docs

- [Concurrent Optimistic Effects](./concurrency-and-dependencies.md)
- [Effect Envelopes And Closeout](./effect-envelopes-and-closeout.md)
- [History And Restore](../debugging/restore-replay-and-recover.md)
