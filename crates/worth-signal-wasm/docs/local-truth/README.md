# Local Truth

Local Truth is the browser-local application-value authority for a standalone
editor that needs branches, aspect-aware commits, and snapshots. It is a good fit for a local design tool, an
offline experiment, or a single-process workflow whose state is allowed to end
when that browser process ends.

It is not the smaller version of Worth's durable platform. If the application
needs shared users, restart recovery, authenticated audit, server invariants,
or regulated records, start with **Worth Query over Worth Relational**. Query is
the ordinary application-facing runtime; Relational owns authoritative durable
truth; the Runtime Bridge carries committed cause into Signal; Signal rebuilds
derived computation.

For the process-local case, the common path is deliberately direct:

```ts
import { createSignals, localTruthSchema } from "worth-signals-wasm";

interface Gear {
  teeth: number;
  label: string;
}

const schema = localTruthSchema<Gear>({
  id: "gear",
  aspects: [
    {
      id: "teeth",
      field: "teeth",
      valueType: "number",
      equivalence: { kind: "exact" },
      costClass: "constant",
    },
    {
      id: "label",
      field: "label",
      valueType: "string",
      equivalence: { kind: "exact" },
      costClass: "constant",
    },
  ],
});

const signals = await createSignals();
const initialGear = { teeth: 18, label: "Drive" };
const gearInput = signals.input(initialGear, { producesAspects: [0, 1] });
const truth = signals.localTruth({
  authorityId: "gear-editor",
  schema,
  initialEntities: { gear: initialGear },
  bindings: [{
    entityId: "gear",
    input: gearInput,
    aspectMap: { teeth: 0, label: 1 },
  }],
});

await truth.ready?.();
```

The authority owns the values and commits. The input is the disposable Signal
projection. Destroying or rebuilding that projection does not change Local
Truth.

## How The Simple Path Grows

You do not graduate to a different local API when the editor becomes more
capable. Keep the same authority and add only the operation the workflow has
earned:

| Need | Public surface | Read next |
| --- | --- | --- |
| Declare fields and publish one change | `localTruthSchema(...)`, `signals.localTruth(...)`, `commit(...)` | [Your First Local Truth Store](./getting-started.md) |
| Isolate independent work | `forkBranch(...)`, `branch(...)` | [Branches And Snapshots](./branches-and-snapshots.md) |
| Inspect retained time | `history(...)`, `historicalSnapshot(...)`, `checkpoint(...)` | [History, Compaction, And Rebuild](./history-and-rebuild.md) |
| Recover disposable computation | `derivation(...)`, `destroyDerivation(...)`, `rebuildDerivation(...)` | [History, Compaction, And Rebuild](./history-and-rebuild.md) |
| Choose browser-local or platform authority | Query / Relational / Bridge / Signal ownership | [Authority Boundaries](./authority-boundaries.md) |
| Look up exact outcomes and methods | `LocalTruthAuthority<T>` | [Local Truth API Reference](./api-reference.md) |

## What Local Truth Owns

- declared top-level entity aspects and their validation/equivalence rules;
- immutable process-local commits, snapshots, branch heads, and current bases;
- retained history segments, checkpoints, and authority-local compaction;
- exact delivery of committed values into declared Signal aspects.

## What It Does Not Own

- disk or server persistence;
- cross-tab or cross-process serialization;
- multi-user presence, locks, leases, or authenticated actor identity;
- relational invariants, MVCC, replication, or durable recovery;
- derived Signal scheduling or Signal execution branches.

`inspect().supportPosture` says `"inMemoryProcessLocal"` on purpose. Do not call
process-local history durable, restart-stable, or collaboration-safe. A rich
commit graph in memory is still memory.

## Current Limits

- Entity values are plain objects and V1 aspects map to declared top-level
  fields.
- Nested paths, collections, deletion topology, and identity migration are
  unsupported until they have dedicated materializers.
- Ordinary branches have no public delete operation. Checkpoint compaction
  waits until every active branch is checkpointed at its current head.
- A checkpoint is retained process state, not an export or restore format.
- The active native Signal profile supports at most 32 numeric aspects, and
  every projected truth aspect needs an admitted numeric binding.

## Reading Order

1. [Your First Local Truth Store](./getting-started.md)
2. [Branches And Snapshots](./branches-and-snapshots.md)
3. [History, Compaction, And Rebuild](./history-and-rebuild.md)
4. [Authority Boundaries](./authority-boundaries.md)
5. [Local Truth API Reference](./api-reference.md)
