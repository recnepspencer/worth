# Response Topology Proof

## What This Feature Is

This page explains the proof-bearing side of `signals.resource.response.*(...)`
when response shape is richer than a plain direct array.

## Why You Use It

Use it when you need to know:

- where a narrow response effect landed
- whether the runtime proved `entityStore`, `mapCollection`, `groupedCollection`,
  `tree`, `detailResponse`, or another topology
- how that topology feeds effect envelopes and settlement

## Stable Entry Points

- `signals.resource.response.*(...)`
- `line.diagnostics().lastEffect.locusProof`
- `line.history().verificationPackage().lifecycle.lastEffect.locusProof`

## Core Mental Model

Response topology proof is what turns "some nested response changed" into an
honest, named runtime locus.

## How It Executes

Response declarations compile proof-bearing lenses. Later patches and
deliveries copy the relevant topology data into the effect locus proof.

## Small Example

```ts
const response = signals.resource.response.map()({
  itemId: (task) => task.id,
  entries: (value) => new Map(value.taskMapEntries),
  replaceEntries: (value, tasks) => ({ ...value, taskMapEntries: [...tasks] }),
  replaceEntry: (value, itemId, nextItem) => {
    const tasks = new Map(value.taskMapEntries);
    tasks.set(itemId, nextItem);
    return { ...value, taskMapEntries: [...tasks] };
  },
});
```

## Real Example

```ts
const line = tasks.line({});
line.patch(tasks.patch.item({
  itemId: "task:1",
  nextItem: { id: "task:1", title: "Mapped" },
}));

const effect = line.diagnostics().lastEffect;
console.log(effect.locus.kind);
console.log(effect.locusProof.topology);
console.log(effect.locusProof.cost.lookup);
```

## How It Relates To Other Features

- Use [Partial Updates And Derived Views](../partial-updates/README.md) for the
  user-facing explanation of narrow updates.

## Inspection And Debugging

Read `effect.locusProof` after a patch or delivery. The most useful fields are:

- `topology`
- `locus`
- `cost`

## Anti-Patterns

- Do not borrow proof between unrelated response declarations.
- Do not claim topology proof from raw collection shape alone.

## Current Limits

Topology proof explains runtime-observed patch placement, not arbitrary server
schema meaning.

## Related Docs

- [Response Topology Proof](./response-topology-proof.md)
- [Branch-Native Effects](../effects/branch-native-effects.md)
