# Branch-Native Resource Effects

Branch-native effects isolate each optimistic request on its own Signals branch.
Confirmed server data remains canonical. The line derives its visible value by
projecting every still-open effect over canonical truth.

## Quick Decision

- Use `branchNative()` when several optimistic requests may overlap.
- Keep the `effectId` returned by `line.patch(...)`.
- Call `line.effects().confirm(effectId, ...)` for one successful response.
- Call `line.effects().reject(effectId, ...)` for one failed response.
- Declare parent/child relationships with `resourcePatch.dependsOn(...)`.
- Use `restoreExact()` only for deliberate historical restore, not request
  failure.

## Minimal Flow

```ts
import { createSignals } from "worth-signals-wasm";

const signals = await createSignals();
const tasks = signals.api({
  effects: signals.resource.effects.branchNative(),
}).url("/tasks")
  .response(signals.resource.response.array({ itemId: (task) => task.id }))
  .list({ load: () => client.listTasks() });

const line = tasks.line({});
await line.awaitSettlement();

const admission = await line.patch(tasks.patch.insert({
  itemId: draft.id,
  placement: "append",
  nextItem: draft,
}));

if (!("effectId" in admission)) throw new Error("optimism unavailable");

try {
  const response = await client.saveTask(draft);
  await line.effects().confirm(admission.effectId, {
    responseId: response.id,
    serverPatch: tasks.patch.insert({
      itemId: response.task.id,
      placement: "append",
      nextItem: response.task,
    }),
  });
} catch (failure) {
  await line.effects().reject(admission.effectId, {
    responseId: failure.responseId,
  });
}
```

Rejection retires the effect branch. It does not apply a compensating patch or
restore a shared snapshot.

## Concurrent Authority

Use `line.effects()` when more than one effect can be open:

- `open()` lists immutable summaries in dependency-safe order;
- `get(effectId)` returns one summary and its sealed effect envelope;
- `projection()` reports canonical versus derived projection posture;
- `confirm(...)` and `reject(...)` settle one effect by identity;
- `counters()` reports retained and live DAG state.

`line.diagnostics().lastEffect` is a recent diagnostic breadcrumb. It cannot
identify the correct target among ten open effects.

## Dependency Behavior

A child confirmation can arrive before its parent. The runtime records the
response and closes it automatically after all parents confirm. Rejection of a
required parent cancels descendants whose policy is
`cancelOnDependencyRejection`.

Sibling effects do not depend on one another and settle independently. Same-
locus conflicts use server revision when provided, then stable admission order.

## Resource Locus And Native Proof

Resource-locus materialization applies the winning item, aspect, field,
region, JSON path, summary, insert, delete, or replacement to canonical value.

## Anti-Patterns

- Do not reuse the current application branch for optimistic requests.
- Do not compose optimistic and canonical values in React state.
- Do not use branch IDs as semantic dependency IDs.
- Do not free a line while effects are open.
- Do not hand-build effect envelopes or closeout receipts.

## Read Next

- [Concurrent Optimistic Effects](./effects/concurrency-and-dependencies.md)
- [Branch-Native Effects](./effects/branch-native-effects.md)
- [Rollback And Recovery](./effects/rollback-and-recovery.md)
- [History And Restore](./debugging/restore-replay-and-recover.md)
