# Optimistic Updates

Use optimistic effects when the UI should respond before the server does and
each request must still succeed, reject, or wait on dependencies independently.
The branch-native profile gives every admitted request its own effect identity
and native Signals branch.

## The Mental Model

A line keeps two truths distinct:

- **canonical value** — confirmed server data;
- **projected value** — canonical value plus every still-open optimistic
  effect, applied in deterministic order.

`line.value()` exposes the projection while work is open. The projection is
derived by the runtime. React, a form, or another store should not maintain a
parallel optimistic ledger.

An effect ID identifies one request's lifecycle. A branch ID proves its native
history ancestry. A dependency edge says another request must settle first.
Those are related, but they are not interchangeable.

## The Small Path

```ts
const tasks = signals.api({
  effects: signals.resource.effects.branchNative(),
}).url("/tasks")
  .response(signals.resource.response.array({ itemId: (task) => task.id }))
  .list({ load: () => client.listTasks() });

const line = tasks.line({});
await line.awaitSettlement();

const admission = await line.patch(tasks.patch.item({
  itemId: "task-42",
  nextItem: { id: "task-42", title: "Reviewed" },
}));

if (!("effectId" in admission)) {
  throw new Error("optimistic effect was not admitted");
}

try {
  const saved = await client.saveTask("task-42", "Reviewed");
  await line.effects().confirm(admission.effectId, {
    responseId: saved.requestId,
    serverPatch: tasks.patch.item({
      itemId: saved.task.id,
      nextItem: saved.task,
    }),
  });
} catch {
  await line.effects().reject(admission.effectId, {
    responseId: "task-42:rejected",
  });
}
```

The rejection path does not calculate an inverse patch. Retiring that effect's
branch removes only its contribution, then the runtime rebuilds the projection
from canonical value plus the remaining open effects.

## Siblings And Dependencies

Sibling effects have no dependency edge. Ten sibling requests can settle in
any order; five may confirm and five may reject without sharing one rollback
snapshot.

Use `resourcePatch.dependsOn(...)` only for a real parent/child relationship:

```ts
import { resourcePatch } from "worth-signals-wasm";

const editDraft = resourcePatch.dependsOn(
  tasks.patch.item({
    itemId: draft.id,
    nextItem: { ...draft, title: "Reviewed" },
  }),
  [createDraftEffectId],
);

const editAdmission = await line.patch(editDraft);
```

If a child's confirmation arrives first, the runtime records it and waits. If
a required parent rejects, the dependent effect is cancelled according to the
declared dependency policy and its branch retires.

## Inspect The Real Effect

```ts
const open = line.effects().open();
const one = line.effects().get(effectId);
const projection = line.effects().projection();
const counters = line.effects().counters();
```

Use these surfaces to choose and explain concurrent work. A recent
`diagnostics().lastEffect` entry is a breadcrumb, not authority for deciding
which request to confirm or reject.

## Recovery

- `reject(effectId)` closes one failed request without changing canonical
  server truth.
- `line.history().rollbackEffect(effectId)` uses the recovery supported by the
  selected profile and retained proof.
- Exact restore and replay can be unavailable when exact history is absent.

## Common Mistakes

- Restoring one shared snapshot when one of several requests fails.
- Applying a compensating patch after rejecting a branch-native effect.
- Treating the current application branch as the effect branch.
- Treating branch ancestry as request dependency.
- Freeing a line while effects are still open.
- Promoting projected value to durable server truth before confirmation.

## Go Deeper

- [Branch-Native Effects](./branch-native-effects.md)
- [Concurrent Optimistic Effects](./concurrency-and-dependencies.md)
- [Effect Envelopes And Settlement](./effect-envelopes-and-closeout.md)
- [Rollback And Recovery](./rollback-and-recovery.md)
