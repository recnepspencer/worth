# Diagnostics: Make Derived State Show Its Work

Worth does not make you build a second observability system beside your state
system. The runtime that evaluates a derived value can also explain what it
read, why it ran, whether its answer changed, and what happened across the
rest of the committed flow.

Use this when a value should not merely be correct. It should be able to answer
the mildly awkward follow-up question: “Why did you do that?”

## Why You Use It

- explain the dependency path behind a derived decision;
- distinguish “recomputed and changed” from “recomputed but unchanged”;
- inspect a complete transaction instead of sprinkling logs through callbacks;
- retain or export runtime evidence deliberately when an application needs a
  longer record;
- debug missing reactivity without guessing which closure read was captured.

## Stable Entry Points

- `signals.diagnostics()`
- `diagnostics.why(id)`
- `diagnostics.latestFlow()`
- `diagnostics.latestObservation()`
- `diagnostics.latestFailure()`
- `diagnostics.performanceSummary()`
- `diagnostics.recentHistory()`
- `graph.inspectDiagnostics()`

`debugName` makes diagnostic output easier to read. It is metadata, not a
stable application identity.

## Core Mental Model

Keep three things separate:

1. **State** is the input value and the derived values computed from it.
2. **Runtime evidence** is the explanation Worth produces while evaluating and
   propagating that state.
3. **Application retention** is any list, export, database record, or audit
   presentation your application keeps afterward.

Worth owns the first two. Your application owns the third. A UI may retain ten
runtime-issued flow records for comparison, but the array itself is still UI
state. Calling it “runtime history” would make it sound more impressive and
less true.

The runtime produces the evidence. UI retention decides how long an
application keeps a projection of it. Neither one silently becomes durable
platform truth.

## How It Executes

1. An input changes through a transaction or writable handle.
2. The runtime invalidates dependents whose read contracts intersect the
   change.
3. Dirty nodes evaluate in graph order.
4. Output equivalence determines whether each result changed or remained the
   same.
5. The runtime commits structured flow evidence.
6. `why(id)` projects the current explanation for one node;
   `latestFlow()` returns the most recent complete flow.

A derived signal can run without changing its answer. That is not wasted
evidence. It proves the policy was reconsidered and stayed on the same side of
its boundary.

Published outputs are standing demand. A transaction that reaches one
recomputes it at commit, so the committed truth and the flow evidence include
it even when nothing read it during the transaction. Computed signals without
a watcher or published output stay on-demand: they appear in `why()` and
`latestFlow()` once something reads them.

## Small Example

```ts
import { createSignals } from "worth-signals-wasm";

const signals = await createSignals({
  deployment: "mainThreadCompatibility",
});

const amount = signals.input(8_000, {
  debugName: "transfer.amount",
});
const reviewLane = signals.computed(
  () => amount() >= 10_000 ? "Manual review" : "Automatic",
  { debugName: "transfer.reviewLane" },
);

reviewLane(); // establish the callback dependency now

await signals.transaction((tx) => {
  tx.set(amount, 12_000);
});

const explanation = await signals.diagnostics().why(reviewLane.id);
console.log(explanation);
```

This focused example selects `mainThreadCompatibility` because it performs
immediate synchronous inspection in the same browser call stack. Worker-first
remains the normal deployment; use its supported asynchronous boundaries when
the runtime should own execution off the UI thread.

## Real Example

One amount can drive several decisions without forcing the UI to maintain
copies of those decisions:

```ts
const amount = signals.input(8_000, {
  debugName: "transfer.requestedAmount",
});
const fee = signals.computed(
  () => Math.round(amount() * 0.004 * 100) / 100,
  { debugName: "transfer.processingFee" },
);
const reviewLane = signals.computed(
  () => amount() >= 10_000 ? "Manual review" : "Automatic",
  { debugName: "transfer.reviewLane" },
);

fee();
reviewLane();

async function commitAmount(nextAmount: number) {
  const summary = await signals.transaction((tx) => {
    tx.set(amount, nextAmount);
  });

  return {
    summary,
    values: {
      amount: amount(),
      fee: fee(),
      reviewLane: reviewLane(),
    },
    evidence: {
      reviewLane: await signals.diagnostics().why(reviewLane.id),
      flow: await signals.diagnostics().latestFlow(),
    },
  };
}

await commitAmount(9_800);
// reviewLane ran and remained "Automatic"

await commitAmount(14_500);
// reviewLane ran and changed to "Manual review"
```

The returned object is an application projection of runtime-owned facts. If a
screen appends it to a visible decision trail, that trail belongs to the
screen. If it is sent to a regulated audit system, that system owns its durable
retention. Worth supplies evidence; it does not quietly appoint browser memory
as your compliance archive.

## Reading The Evidence

Start with `why(signal.id)` when one value is surprising. The explanation can
include:

- the node’s current evaluation state;
- the callback reads the runtime observed;
- whether evaluation replaced, refreshed, or preserved the output;
- upstream version and invalidation evidence.

Use `latestFlow()` when the question concerns the whole transaction. It reports
the changed inputs, invalidation frontier, evaluation work, output outcomes,
and available performance accounting for the most recent committed flow.

Use `recentHistory()` or the history surface when you need retained runtime
inspection rather than only the latest flow. Retention is bounded runtime
support, not an external system of record.

## Graph-Shaped Diagnostics

Published graphs can project diagnostics onto their public input and output
names:

```ts
const inspection = checkout.inspectDiagnostics();

console.log(inspection.output("total").why);
console.log(inspection.dependenciesForOutput("total"));
```

Prefer this boundary when the application speaks in graph inputs and outputs.
Use raw runtime diagnostics when the question genuinely concerns runtime node
identity.

## How It Relates To Other Features

- [Inputs And Computed State](./inputs-and-computed.md) define the state and
  derivation being explained.
- [Transactions](./transactions.md) create the committed flow diagnostics
  describe.
- [Graphs And Controllers](./graphs-and-controllers.md) publish named
  application boundaries with graph-shaped inspection.
- [History](./history.md) retains and replays supported runtime evidence.
- [Aspects](./aspects.md) narrow invalidation using semantic dependency lanes.
- Resource, form, router, and Local Truth surfaces add diagnostics at their own
  semantic level. Prefer those explanations when that is the object the user
  is reasoning about.

## Inspection And Debugging

When a computed value looks wrong:

1. Read the computed value directly.
2. Call `diagnostics.why(signal.id)`.
3. Check the callback’s current reads.
4. Check output change to distinguish no run, unchanged output, and changed
   output.
5. Inspect `latestFlow()` if several nodes participated.
6. Inspect `latestFailure()` when evaluation did not complete.

If the read set is empty, make sure the callback has executed. A random closure
variable does not become reactive merely because a computed function looked at
it. Model changing values as signals or admitted host capabilities.

## Anti-Patterns

- keeping a second writable copy of a committed signal value in component
  state;
- calling a UI-owned array of snapshots runtime-owned history;
- using `latestFlow()` as durable audit storage;
- branching application behavior on human-readable diagnostic strings;
- using `debugName` as a database or protocol identity;
- assuming richer diagnostics are free or always retained.

## Current Limits

- Host-capability event replay reports explicit unavailability where the
  active deployment cannot honestly reconstruct it.
- Diagnostics explain Worth runtime behavior. They do not add external
  durability, user identity, signatures, approval authority, or compliance
  retention.
- `why()` explains a decision; it does not become the policy that made it.
- Rich evidence follows the active diagnostic and retention posture. Do not
  design application correctness around an optional forensic artifact.

## Related Docs

- [Inputs And Computed State](./inputs-and-computed.md)
- [Transactions And Coordinated Writes](./transactions.md)
- [Graphs And Controllers](./graphs-and-controllers.md)
- [History, Replay, And Runtime Branches](./history.md)
- [Support Status](../reference/support-status.md)
