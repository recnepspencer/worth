# Collections And Delivery

Use this page when an API route returns a collection-like response and you want
the family to expose item, aspect, or summary patch helpers. The usual path is a
response contract: you describe how to find and replace the items once, and
Worth derives the patch and delivery helpers from that declaration.

## What This Feature Is

Collection reconciliation lets a resource family update one item or one item
field without replacing the whole visible value. A response contract is the
route-first version of that idea: the response type owns item extraction,
replacement, stable item identity, and ordinary field aspects.

## Why You Use It

- Use `resource.response.array(...)` when the response value is the item array.
- Use `resource.response.objectItems<T>()(...)` when the response is an object
  with one array-valued item field.
- Use `resource.response.collection<T>()(...)` when the response shape is
  custom and array-like.
- Use `resource.response.connection<T>()(...)`, `entityStore<T>()(...)`,
  `map<T>()(...)`, `grouped<T>()(...)`, `named<T>()(...)`, `multiple<T>()(...)`,
  `sparse<T>()(...)`, `discriminated<T>()(...)`, or `tree<T>()(...)` when the
  server shape has a known topology and narrow patches should preserve
  topology-specific proof and cost evidence.
- Use `resource.response.detail<T>()` or `resource.response.summary<T>()` when
  a response-owned detail or summary line should admit broad replacement
  through the same effect proof lane.
- Use `.items(...).reconcile(...)` directly when you need custom item/aspect or
  summary behavior that should stay explicit in the route chain.

## Stable Entry Points

- `signals.resource.response.array(...)`
- `signals.resource.response.objectItems<T>()(...)`
- `signals.resource.response.collection<T>()(...)`
- `signals.resource.response.connection<T>()(...)`
- `signals.resource.response.entityStore<T>()(...)`
- `signals.resource.response.map<T>()(...)`
- `signals.resource.response.grouped<T>()(...)`
- `signals.resource.response.named<T>()(...)`
- `signals.resource.response.multiple<T>()(...)`
- `signals.resource.response.sparse<T>()(...)`
- `signals.resource.response.discriminated<T>()(...)`
- `signals.resource.response.tree<T>()(...)`
- `signals.resource.response.detail<T>()`
- `signals.resource.response.summary<T>()`
- `signals.resource.response.objectAspects<T>()(...)`
- `signals.resource.response.jsonPathAspects<T>()(...)`
- `api.url(...).response(responseContract).list(...)`
- `api.url(...).response(responseContract).paged(...)`
- `api.url(...).response(responseContract).detail(...)`
- `api.url(...).items(...).aspect(...)`
- `api.url(...).items(...).reconcile(...)`
- `api.url(...).items(...).summary(...)`
- `api.url(...).items(...).pageWindowSummary(...)`
- family-owned `patch`
- family-owned `delivery`

## Core Mental Model

The loaded response value stays authoritative. A response contract tells Worth
how to read the relevant item, detail, or summary shape out of that value and
how to write admitted changes back into the same response topology.

Once the contract proves item identity and aspects, Worth exposes narrow patch
helpers such as `family.patch.item(...)` and `family.patch.itemAspect(...)`.
The caller does not pass per-patch extraction or write functions.

## How It Executes

1. Build a response contract with item identity, item extraction, item
   replacement, and optional object-field aspects.
2. Attach it with `.response(responseContract)`.
3. Finish the route with `.list(...)` or `.paged(...)`.
4. Call `family.line(params)` to get the resource line.
5. Apply item or aspect patches through `line.patch(...)`, or server push
   updates through `line.deliver(...)`.

If a response contract cannot extract an array during patching, or if
replacement produces a response that no longer contains an item array, the
patch is rejected instead of silently broadening or corrupting the value.

## Small Example

Use `array(...)` for the smallest case: the response is already the item array.

```ts
type Task = {
  id: string;
  title: string;
  status: "open" | "done";
  assigneeId: string | null;
};

const taskResponse = signals.resource.response.array({
  itemId: (task: Task) => task.id,
  aspects: signals.resource.response.objectAspects<Task>()({
    title: "title",
    status: "status",
    assigneeId: "assigneeId",
  }),
});

const tasks = api.url("/workspaces/:workspaceId/tasks")
  .response(taskResponse)
  .list({
    load: ({ workspaceId }) => [
      {
        id: `${workspaceId}:1`,
        title: "First",
        status: "open",
        assigneeId: null,
      },
    ],
  });

tasks.line({ workspaceId: "demo" }).patch(
  tasks.patch.itemAspect({
    itemId: "demo:1",
    aspect: "title",
    value: "Updated",
  }),
);
```

## Real Examples

Use `objectItems<T>()(...)` when a typed response wraps items in an object
field and the rest of the response should be preserved.

```ts
type TaskEnvelope = {
  tasks: readonly Task[];
  nextCursor: string | null;
};

const taskEnvelopeResponse =
  signals.resource.response.objectItems<TaskEnvelope>()({
    field: "tasks",
    itemId: (task) => task.id,
    aspects: signals.resource.response.objectAspects<Task>()({
      title: "title",
    }),
  });

const taskPage = api.url("/workspaces/:workspaceId/task-page")
  .response(taskEnvelopeResponse)
  .list({
    load: ({ workspaceId }) => ({
      tasks: [
        {
          id: `${workspaceId}:1`,
          title: "First",
          status: "open",
          assigneeId: null,
        },
      ],
      nextCursor: "cursor-2",
    }),
  });

taskPage.line({ workspaceId: "demo" }).patch(
  taskPage.patch.itemAspect({
    itemId: "demo:1",
    aspect: "title",
    value: "Updated",
  }),
);
```

Use `collection<T>()(...)` when the item array is derived from a custom
array-like response shape.

```ts
type TaskConnection = {
  edges: readonly { node: Task }[];
  pageInfo: { hasNextPage: boolean };
};

const taskConnectionResponse =
  signals.resource.response.collection<TaskConnection>()({
    itemId: (task) => task.id,
    items: (value) => value.edges.map((edge) => edge.node),
    replaceItems: (value, nextItems) => ({
      ...value,
      edges: nextItems.map((node) => ({ node })),
    }),
    aspects: signals.resource.response.objectAspects<Task>()({
      status: "status",
    }),
  });

const taskConnection = api.url("/workspaces/:workspaceId/task-connection")
  .response(taskConnectionResponse)
  .list({
    load: () => ({
      edges: [
        {
          node: {
            id: "task:1",
            title: "First",
            status: "open",
            assigneeId: null,
          },
        },
      ],
      pageInfo: { hasNextPage: false },
    }),
  });
```

## How It Relates To Other Features

Use `.items(...).aspect(...)` directly when aspect reads or writes are not plain
object-field replacement.

```ts
const tasks = api.url("/workspaces/:workspaceId/tasks")
  .items((item: { id: string; title: string }) => item.id)
  .aspect(
    "title",
    (item) => item.title,
    (item, title: string) => ({ ...item, title }),
  )
  .list({
    load: ({ workspaceId }) => [
      { id: `${workspaceId}:1`, title: "First" },
    ],
  });
```

Use `.reconcile(...)` when you want the route chain itself to own custom item
extraction and replacement, especially if you also need summaries.

```ts
const catalog = api.url("/workspaces/:workspaceId/catalog")
  .items((item: { id: string; title: string }) => item.id)
  .reconcile(
    (value: { items: Array<{ id: string; title: string }>; total: number }) =>
      value.items,
    (value, nextItems) => ({ ...value, items: [...nextItems] }),
  )
  .summary(
    "total",
    (value) => value.total,
    (value, total: number) => ({ ...value, total }),
  )
  .list({
    load: ({ workspaceId }) => ({
      items: [{ id: `${workspaceId}:1`, title: "First" }],
      total: 1,
    }),
  });
```

If the family owns reconciliation truth, it also owns delivery helpers for
server push updates.

```ts
const catalogLine = catalog.line({ workspaceId: "demo" });

catalogLine.deliver(
  catalog.delivery.summary({
    packetId: "pkt-1",
    basisId: null,
    nextBasisId: "basis-1",
    summary: "total",
    value: 2,
  }),
);
```

## Advanced Topologies

When the response shape is not just "some array of items", prefer the topology
that describes the server contract directly. These response contracts preserve
topology-specific effect proof, diagnostics and history evidence, and
traversal and reconstruction cost counters.

- `connection<T>()(...)` for GraphQL-style edges and nodes
- `entityStore<T>()(...)` for normalized entity bags
- `map<T>()(...)` for `ReadonlyMap`-shaped collections
- `grouped<T>()(...)` for grouped collections
- `named<T>()(...)` and `multiple<T>()(...)` for multiple named collections
- `sparse<T>()(...)` for loaded page chunks
- `discriminated<T>()(...)` for active tuple/envelope variants
- `tree<T>()(...)` for recursive trees
- `detail<T>()` and `summary<T>()` for response-owned broad replacement lines

For branch-native optimistic effects, JSON path aspects, and lifecycle reads,
see [Branch-Native Resource Effects](./branch-native-effects.md).

## Inspection And Debugging

Use `line.reconciliation()` to see which item aspects and summaries the family
admits. Use `line.summary()` or `line.diagnosticsSummary()` to inspect the last
patch scope and current line status.

When response-contract extraction fails, the error names the response helper
that owns the bad shape, such as `resource.response.array(...)` or
`resource.response.collection(...)`.

## Anti-Patterns

- Do not use `.response(...)` and then repeat `itemIdentity` or `reconcile` in
  `.list(...)`; the response contract owns those.
- Do not use the generic `collection<T>()(...)` helper for known server
  topologies when a dedicated response topology exists. Use the topology helper
  so diagnostics, history, and cost counters can name the real
  locus.
- Do not hide custom item replacement inside `objectAspects<T>()(...)`.
  `objectAspects` is for replacing fields on item objects, not for changing the
  outer response shape.
- Do not return a non-array from `collection<T>()(...).items(...)` or from the
  response produced by `replaceItems(...)`.

## Related Docs

- route-first CRUD and advanced request shaping:
  [Fetch And Write Resources](./fetch-and-write.md)
- branch-native effects, advanced response topology proof, and JSON aspects:
  [Branch-Native Resource Effects](./branch-native-effects.md)
- line reads and diagnostics:
  [Line Inspection](./line-inspection.md)
- lower-level reconciliation reference:
  [Reconciliation Contract](./responses/README.md)
