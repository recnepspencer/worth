# Concurrent Optimistic Effects

## What This Feature Is

Concurrent effects let several optimistic requests remain visible without
sharing one rollback snapshot. Each request gets its own runtime-issued effect
identity and branch. You report that request's server outcome by identity; the
runtime updates confirmed data and rebuilds the visible projection.

Use this for autosave, queues, bulk edits, dependent create-and-edit flows, or
any screen where responses can arrive in a different order from requests.

## Why You Use It

- One rejected request cannot erase a successful sibling.
- A child request can wait for its parent without turning every request into a
  chain.
- Ten mixed responses can settle in any order and still end at server truth.
- Logs and UI can show the real effect, dependency, projection, and closeout
  receipts instead of maintaining a second optimistic ledger.

## Stable Entry Points

- `await createSignals()` — worker-first by default
- `signals.resource.effects.branchNative()`
- `line.patch(patch)` — returns the admitted `effectId` in branch-native mode
- `resourcePatch.dependsOn(patch, effectIds)`
- `line.effects().get(effectId)`
- `line.effects().open()`
- `line.effects().projection()`
- `line.effects().confirm(effectId, options)`
- `line.effects().reject(effectId, options)`
- `line.history().rollbackEffect(effectId)`

`line.diagnostics().lastEffect` remains useful for recent diagnostics. It is
not the authority for choosing among multiple open effects.

## Core Mental Model

The line has two kinds of value:

- **canonical value** is confirmed server truth;
- **projected value** is canonical value plus every still-open optimistic
  effect, applied in deterministic order.

`line.value()` exposes the projected value while work is open. The projection
is derived by the runtime; React, a form, or a router does not compose it.

Every admitted optimistic patch owns a native Signals branch. A dependency is
a separate semantic edge between effect identities. Native branch ancestry
proves where the branch came from; the dependency edge explains which request
must settle first. Neither substitutes for the other.

Sibling effects have no dependency edge. They settle independently. A child
effect can name one or several parents. If its confirmation arrives early, the
runtime records the response and waits. If a required parent is rejected, the
child is cancelled and its branch is retired.

## How It Executes

1. `line.patch(...)` validates the resource locus and admits one effect-owned
   branch.
2. The runtime returns an `effectId` and rebuilds the derived projection.
3. Your request runs outside the runtime.
4. On success, call `confirm(effectId, { responseId, serverPatch })`.
5. On failure, call `reject(effectId, { responseId })`.
6. Confirmation reconciles exactly the declared resource locus into canonical
   truth. Rejection changes no canonical value.
7. The runtime retires the settled branch, applies dependency closeout, and
   rebuilds the projection from the remaining open effects.
8. When no effects remain, `projection().kind` is `"canonical"`.

Retry an interrupted closeout with the same `responseId`. If native closeout
already completed, the runtime resumes projection finalization without closing
the effect twice. Once closeout is terminal, the same response identity returns
a duplicate-settlement receipt.

## Small Example

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

if (!("effectId" in admission)) throw new Error("effect was not admitted");

try {
  const saved = await client.saveTask(draft);
  await line.effects().confirm(admission.effectId, {
    responseId: saved.requestId,
    serverPatch: tasks.patch.insert({
      itemId: saved.task.id,
      placement: "append",
      nextItem: saved.task,
    }),
  });
} catch {
  await line.effects().reject(admission.effectId, {
    responseId: `task:${draft.id}:failed`,
  });
}
```

The catch path does not issue an inverse patch. Retiring the effect branch is
what removes the optimistic row.

## Real Example

This example admits two siblings and an edit that depends on a create:

```ts
import { resourcePatch } from "worth-signals-wasm";

const create = await line.patch(tasks.patch.insert({
  itemId: draft.id,
  placement: "append",
  nextItem: draft,
}));
const independent = await line.patch(tasks.patch.insert({
  itemId: note.id,
  placement: "append",
  nextItem: note,
}));

if (!("effectId" in create) || !("effectId" in independent)) {
  throw new Error("branch-native admission required");
}

const edit = await line.patch(resourcePatch.dependsOn(
  tasks.patch.item({
    itemId: draft.id,
    nextItem: { ...draft, title: "Reviewed" },
  }),
  [create.effectId],
));
if (!("effectId" in edit)) throw new Error("dependent admission failed");

// This response may arrive before the create response.
const earlyEdit = await line.effects().confirm(edit.effectId, {
  responseId: "response:edit",
});
// earlyEdit.kind === "responseRecorded"

await line.effects().confirm(independent.effectId, {
  responseId: "response:note",
});

const createCloseout = await line.effects().confirm(create.effectId, {
  responseId: "response:create",
});
// createCloseout.automaticallySettled includes edit.effectId
```

For a child that requires several parents, pass all required effect IDs:

```ts
const publish = resourcePatch.dependsOn(publishPatch, [
  uploadEffectId,
  approvalEffectId,
]);
```

The child becomes ready only after every required parent confirms.

## How It Relates To Other Features

- [Branch-Native Effects](./branch-native-effects.md) explains profile choice
  and effect-owned branches.
- [Rollback And Recovery](./rollback-and-recovery.md) separates targeted effect
  rejection from historical restore.
- Resource-backed forms consume the same line effects; they do not own a second
  rollback system.

## Inspection And Debugging

Use `line.effects()` for concurrent authority:

```ts
const open = line.effects().open();
const one = line.effects().get(effectId);
const projection = line.effects().projection();
const counters = line.effects().counters();
```

Open summaries include branch IDs, dependency IDs, dependency basis, locus,
admission order, lifecycle, and terminal detail. Settlement receipts include
canonical value, projection, reconciliation, retirement, closeout proof, and
automatically settled dependents when applicable.

## Anti-Patterns

- Do not restore a shared snapshot when one request fails.
- Do not issue a compensating patch to undo a rejected branch-native patch.
- Do not use `lastEffect` to choose among concurrent effects.
- Do not treat branch IDs as dependency IDs.
- Do not compose canonical plus optimistic values in React state.
- Do not free a line with open effects; settle or reject them first.

## Current Limits

- Dependency closeout currently uses `cancelOnDependencyRejection` for patches
  created with `resourcePatch.dependsOn(...)`.
- Same-locus confirmations use server revision when supplied, then stable
  admission order. A superseded effect still retires cleanly.
- A patch whose resource locus or topology cannot be proved is denied before
  branch creation.
- Explicit historical replay and restore remain separate capabilities; they do
  not settle open request effects.

## Related Docs

- [Effects And Recovery](./README.md)
- [Branch-Native Effects](./branch-native-effects.md)
- [Rollback And Recovery](./rollback-and-recovery.md)
- [History And Restore](../debugging/restore-replay-and-recover.md)
