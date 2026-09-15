# Aspects: Semantic Invalidation

Most reactive systems become precise by breaking state into smaller stores,
tracking object properties, or stacking selectors until the dependency graph
roughly resembles the domain.

Worth Signals takes a more direct route. An **aspect** is a named-in-your-code,
numeric-in-the-runtime meaning carried by both sides of the graph:

- a write declares *what kind of meaning changed*;
- a dependency edge declares *what kind of meaning it consumes*;
- diagnostics report the exact lanes that moved and the subscribers they
  reached.

That is the differentiator. Worth makes domain meaning part of the reactive
DAG itself. You keep one complete authoritative value without making every
consumer wake up whenever any part of that value changes.

## The Opinionated Version

Use aspects when your domain has a stable distinction that matters to
invalidation: financial terms versus an operator note, geometry versus visual
style, approval state versus display copy.

Do not create an aspect for every object property. That is property tracking
with extra paperwork. A good aspect survives a refactor because it names a
business or computational concern, not the current shape of an interface.

## The Three-Part Contract

| Boundary | What you declare | Stable entry point |
| --- | --- | --- |
| Producer | The meanings a signal can produce | `producesAspects` |
| Consumer | The meanings a derived node depends on | `reads: [{ id, aspect }]` |
| Write | The meanings changed by this transaction | `tx.setWithAspects(...)` |

All three must agree. Worth can enforce and explain the contract you declare;
it cannot read your product manager's mind. Yet.

## A Complete Example

Keep the numeric IDs behind domain names. The runtime wants compact lanes;
your application wants words.

```ts
import { createSignals } from "worth-signals-wasm";

const TransferAspect = {
  financialTerms: 0,
  operatorNote: 1,
} as const;

interface Transfer {
  amount: number;
  note: string;
}

const signals = await createSignals({
  deployment: "mainThreadCompatibility",
});

const transfer = signals.spec.input<Transfer>(
  "transfer",
  { amount: 8_000, note: "Standard vendor invoice" },
  {
    debugName: "transfer",
    producesAspects: [
      TransferAspect.financialTerms,
      TransferAspect.operatorNote,
    ],
  },
);

const reviewLane = signals.spec.computed<string>("reviewLane", {
  reads: [
    { id: transfer.id, aspect: TransferAspect.financialTerms },
  ],
  expr: {
    kind: "if",
    condition: {
      kind: "gte",
      left: {
        kind: "get",
        target: { kind: "read", id: transfer.id },
        field: "amount",
      },
      right: { kind: "value", value: 10_000 },
    },
    thenExpr: { kind: "value", value: "Manual review" },
    elseExpr: { kind: "value", value: "Automatic" },
  },
  identity: { kind: "exact" },
});

const notePreview = signals.spec.computed<string>("notePreview", {
  reads: [
    { id: transfer.id, aspect: TransferAspect.operatorNote },
  ],
  expr: {
    kind: "get",
    target: { kind: "read", id: transfer.id },
    field: "note",
  },
  identity: { kind: "exact" },
});

// Materialize both dependency edges before the first write.
reviewLane();
notePreview();

await signals.transaction((tx) => {
  tx.setWithAspects(
    transfer,
    { ...transfer(), note: "Urgent vendor invoice" },
    [TransferAspect.operatorNote],
  );
});
```

The input still owns one complete `Transfer`. The transaction did not submit a
note fragment, and the runtime did not diff an object after the fact. It
accepted the next authoritative value and the application's explicit claim
that only `operatorNote` changed.

The result is the point of the feature: `notePreview` is invalidated;
`reviewLane` is not.

## Why the Explicit Spec Lane Appears Here

Callback-computed signals discover the handles they read. That is excellent
for the ordinary happy path, but a callback read cannot say, "I depend on this
signal only when its financial meaning changes."

Aspect-filtered dependency edges therefore use `signals.spec.computed(...)`.
The explicit `reads` list is not ceremony for ceremony's sake. It is the
contract that makes semantic invalidation portable, inspectable, and honest
across the worker boundary.

## Runtime Placement

The focused example uses `mainThreadCompatibility` because it authors a fresh,
explicitly named aspect graph synchronously. That is an honest specialist lane,
not the platform deployment default.

Worker-first remains the production default. Empty-root worker authoring and
imported graph definitions both admit the three-part aspect contract:
`producesAspects`, aspect-filtered `reads: [{ id, aspect }]`, and
`setWithAspects` / related writes. Prefer a published graph definition when the
contract must travel as portable graph truth; see
[Graphs And Controllers](./graphs-and-controllers.md).

Do not delete the aspect contract merely to make a string-id `reads` list fit.
If a consumer depends on one meaning and not another, keep the filtered read
descriptor.

## Prove What Happened

Do not settle for "it looked fast." Ask the runtime what it invalidated.

```ts
const latest = signals.diagnostics().latestFlow();

console.log(latest?.flow.change.changed_aspects);
// [1]

console.log(latest?.flow.invalidation.invalidated_direct_subscribers);
// 1
```

`changed_aspects` is the semantic claim carried by the write.
`invalidated_direct_subscribers` is the immediate fan-out selected by the DAG.
Together they let a test prove that the right meaning moved and the wrong work
stayed asleep.

## Aspects Are Not Fields, Selectors, or Regions

**Fields describe storage shape.** An aspect describes meaning. Several fields
can belong to one aspect, and one conceptual aspect can survive a complete
storage redesign.

**Selectors compute a projection.** An aspect narrows the dependency edge
before recomputation. You may still use selectors or computed nodes to produce
values; aspects decide which semantic changes can reach them.

**Regions answer where.** Aspects answer what kind. A canvas edit might change
the `geometry` aspect in two spatial regions. Use
`tx.setWithRegionsAndAspects(...)` when both facts matter.

**Local Truth aspects can also be merge loci.** That layer uses semantic
boundaries to compare and resolve process-local branches. Core signal aspects
are the invalidation contract underneath; they do not make the Signals runtime
durable, collaborative, or authoritative over server truth.

## Correctness Rules

1. Pass the complete next value to `setWithAspects`. Aspects describe a change;
   they are not partial patches.
2. Include every semantic lane that actually changed. Omitting one can suppress
   required work.
3. Keep the ID-to-meaning mapping beside the domain model and reuse it at
   declaration, read, and write boundaries.
4. Prime or observe derived nodes before expecting them to appear as active
   subscribers in diagnostics. Published outputs are standing demand and
   recompute at commit when an aspect write reaches them; other computeds
   recompute on their next read.
5. Test negative space: prove that unrelated subscribers were *not*
   invalidated.

One especially important sharp edge: ordinary `tx.set(...)` records the
default aspect (`0`). Once an input has multiple aspect lanes, use
`tx.setWithAspects(...)` for semantic writes. Do not assume a plain set means
"invalidate every declared aspect."

## Current Limits

- The active native profile supports at most 32 numeric aspects.
- Aspect names are application-owned. The runtime stores IDs, not your domain
  vocabulary.
- Aspect-filtered reads live in explicit structural specs, not ordinary
  callback-computed authoring. Worker-first and mainThreadCompatibility both
  admit `reads: [{ id, aspect }]` on `signals.spec.computed` /
  `signals.spec.output`; portable graphs remain the preferred publication path
  when the contract must cross process boundaries as a definition artifact.
- The runtime trusts the change set supplied by the application. A false claim
  can make invalidation precisely wrong, which is much worse than vaguely
  inefficient.
- Aspects are not permissions, persistence, synchronization, or durable truth.

## When Not to Use Them

Stay with ordinary signals when every consumer genuinely cares about every
write, or when the proposed lanes are unstable implementation details. Split
authority into separate inputs when the values have different owners or
lifecycles. Aspects preserve one authority while refining its invalidation;
they should not glue unrelated authorities together.

## Related Docs

- [Transactions And Coordinated Writes](./transactions.md)
- [Diagnostics And Explanation](./diagnostics.md)
- [Local Truth](../local-truth/README.md)
- [Support Status](../reference/support-status.md)
