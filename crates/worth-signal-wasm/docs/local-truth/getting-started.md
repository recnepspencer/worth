# Your First Local Truth Store

## What This Feature Is

This is the smallest honest Local Truth setup: one declared entity shape, one
process-local authority, and one aspect-aware Signal projection.

Use it only when losing the authority at browser-process termination is an
acceptable product decision. For durable or shared application state, start at
[Authority Boundaries](./authority-boundaries.md) and use the full Query and
Relational platform.

## Why You Use It

- keep browser-local application values in one explicit authority;
- commit a semantic field without replacing an arbitrary object;
- give Signal exact aspects to invalidate after the commit;
- gain branches, snapshots, and inspection without adding another
  state store later.

## Stable Entry Points

- `localTruthSchema(...)`
- `signals.localTruth(...)`
- `truth.ready?.()`
- `truth.branch(...)`
- `truth.commit(...)`
- `truth.inspect()`
- `truth.terminate()`

## Core Mental Model

```text
LocalTruthAuthority values and commits -> Signal input -> computed/output state
```

Local Truth is authoritative on the left side of that arrow. Signal state is a
rebuildable projection on the right. The initial entity and its bound Signal
input must start equal so the first projection has no ambiguous basis.

An **aspect** is a declared semantic update lane. The schema maps `teeth` to the
top-level `teeth` field. The Signal binding maps the same semantic aspect to a
numeric Signal aspect. Local Truth never guesses either mapping from an object
diff.

## How It Executes

1. `localTruthSchema(...)` validates and seals the schema.
2. `signals.localTruth(...)` creates the main branch and genesis snapshot.
3. The authority initializes the bound Signal projection.
4. `branch()` returns the runtime-issued current basis.
5. `commit(...)` validates the basis and every aspect operation.
6. The authority reconstructs and publishes one immutable snapshot and commit.
7. Signal receives the complete entity value with the exact changed aspects.

If step 7 fails, the commit remains true and the derivation reports
`RebuildRequired`. Derived failure does not roll back application truth.

## Small Example

```ts
import { createSignals, localTruthSchema } from "worth-signals-wasm";

interface CounterState {
  count: number;
}

const counterSchema = localTruthSchema<CounterState>({
  id: "counter",
  aspects: [{
    id: "count",
    field: "count",
    valueType: "number",
    equivalence: { kind: "exact" },
    costClass: "constant",
  }],
});

const signals = await createSignals();
const initial = { count: 0 };
const counterInput = signals.input(initial, { producesAspects: [0] });
const truth = signals.localTruth({
  authorityId: "counter-editor",
  schema: counterSchema,
  initialEntities: { counter: initial },
  bindings: [{
    entityId: "counter",
    input: counterInput,
    aspectMap: { count: 0 },
  }],
});

await truth.ready?.();
const main = await truth.branch();
if (main.posture !== "success") throw new Error(main.message);

const committed = await truth.commit({
  requestId: crypto.randomUUID(),
  branchId: main.value.id,
  expectedBasis: main.value.basis,
  operations: [{ entityId: "counter", aspectId: "count", value: 1 }],
});
if (committed.posture !== "success") throw new Error(committed.message);
```

This is small, but it is not a toy authority path. The commit uses a current
runtime-issued basis and changes one schema-declared locus atomically.

## Real Example

Keep domain shape, authority construction, and operations in separate modules.
That is enough structure to make ownership obvious without inventing an
application framework inside the application.

```ts
// gear/gear.model.ts
export interface Gear {
  teeth: number;
  thickness: number;
  label: string;
}

export const initialGear: Gear = {
  teeth: 18,
  thickness: 0.58,
  label: "Drive",
};
```

```ts
// gear/gear.local-truth.ts
import { localTruthSchema, type CallableSignals } from "worth-signals-wasm";
import { initialGear, type Gear } from "./gear.model.js";

export const gearSchema = localTruthSchema<Gear>({
  id: "gear",
  version: 1,
  aspects: [
    { id: "teeth", field: "teeth", valueType: "number",
      equivalence: { kind: "exact" }, costClass: "constant" },
    { id: "thickness", field: "thickness", valueType: "number",
      equivalence: { kind: "numberEpsilon", epsilon: 0.001 },
      costClass: "constant" },
    { id: "label", field: "label", valueType: "string",
      equivalence: { kind: "exact" }, costClass: "constant" },
  ],
});

export function createGearLocalTruth(signals: CallableSignals) {
  const input = signals.input(initialGear, { producesAspects: [0, 1, 2] });
  const authority = signals.localTruth({
    authorityId: "gear-editor",
    schema: gearSchema,
    initialEntities: { gear: initialGear },
    bindings: [{
      entityId: "gear",
      input,
      aspectMap: { teeth: 0, thickness: 1, label: 2 },
    }],
  });
  return { authority, input };
}
```

```ts
// gear/gear.commands.ts
import type { LocalTruthAuthority } from "worth-signals-wasm";
import type { Gear } from "./gear.model.js";

export async function changeGearTeeth(
  authority: LocalTruthAuthority<Gear>,
  branchId: string,
  teeth: number,
) {
  const branch = await authority.branch(branchId);
  if (branch.posture !== "success") return branch;

  return authority.commit({
    requestId: crypto.randomUUID(),
    branchId,
    expectedBasis: branch.value.basis,
    operations: [{ entityId: "gear", aspectId: "teeth", value: teeth }],
  });
}
```

The command fetches the basis immediately before the commit. Reusing the basis
from initial setup after another commit would correctly return
`staleLocalTruthBasis`.

## How It Relates To Other Features

- Add [branches](./branches-and-snapshots.md) when work must be isolated.
- Use [history and rebuild](./history-and-rebuild.md) for retained inspection
  and disposable Signal recovery.
- Use Query and Relational instead when the authority must survive this
  browser process or coordinate with another one.

## Inspection And Debugging

```ts
const inspection = await truth.inspect();

inspection.supportPosture; // "inMemoryProcessLocal"
inspection.branches;
inspection.heads;
inspection.values;
inspection.counters;
inspection.decisionLog;
```

Read values from `inspection.values[branchId]`. Do not read the bound Signal
input and promote that projection back into an authority basis.

## Anti-Patterns

- Do not mirror Local Truth values into React state and write both copies.
- Do not mutate `initialEntities` or a value returned from inspection.
- Do not submit a whole replacement object when one declared aspect changed.
- Do not cache a basis and keep retrying it after the branch advances.
- Do not put fetches, random conflict policy, or business decisions inside the
  schema declaration.

## Current Limits

- `bindings` are required; Local Truth always has an explicit Signal
  projection in the public factory.
- Every initial entity must be a non-empty-id plain object satisfying every
  declared aspect.
- `valueType: "any"` permits broader values, but the canonical clone/digest
  boundary still rejects unsupported JavaScript data.
- `metadata` on a commit is a host assertion. It is not authenticated actor or
  audit identity.

## Related Docs

- [Local Truth](./README.md)
- [Branches And Snapshots](./branches-and-snapshots.md)
- [Authority Boundaries](./authority-boundaries.md)
- [Local Truth API Reference](./api-reference.md)
