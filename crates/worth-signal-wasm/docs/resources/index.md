# Resources

Most endpoints are not a thesis. Declare a route, choose whether it returns one
record, a list, or pages, and materialize a line. You get stable identity,
reactive value, lifecycle state, and useful inspection without assembling a
second client-side state machine.

If the endpoint stays ordinary, stop there. When it becomes less ordinary, the
same family and line can grow into scoped auth and policy, narrow item or aspect
updates, server-response reconciliation, concurrent optimistic effects,
transfers, history, recovery, forms, routes, and verification. You add the proof
the endpoint needs; you do not replace the simple API with a different
framework.

Worth describes the request and owns the browser-local resource lifecycle. Your
`load(params, request)` function performs the actual I/O, and your server remains
the authority for durable server data. Worth reports unavailability when the
declared shape or retained evidence cannot support a narrow update or exact
recovery.

## The Common Path

```ts
import { createSignals } from "worth-signals-wasm";

const signals = await createSignals();
const api = signals.api({ baseUrl: "/api" });

const invoiceDetail = api.url("/invoices/:invoiceId").detail({
  async load({ invoiceId }, request) {
    if (!request.target.url) throw new Error("invoice URL was not admitted");

    const response = await fetch(request.target.url);
    if (!response.ok) {
      throw new Error(`invoice ${invoiceId} failed: ${response.status}`);
    }
    return response.json();
  },
});

const invoice = invoiceDetail.line({ invoiceId: "inv-42" });
const settled = await invoice.awaitSettlement();

if (settled.resultKind === "fulfilled" || settled.resultKind === "partial") {
  console.log(invoice.value());
} else {
  console.error(invoice.summary());
}
```

This is the whole starting model:

1. `signals.api(...)` declares shared API defaults.
2. `api.url(...)` turns route params into a canonical request identity.
3. `.detail(...)`, `.list(...)`, or `.paged(...)` declares the value shape.
4. `family.line(params)` gives you one live resource member.
5. `load(params, request)` performs the network call.
6. The line keeps value, pending state, freshness, diagnostics, and history
   aligned around that identity.

A newly materialized asynchronous line can have `null` visible value while its
first load is pending. Await settlement when later work requires a fulfilled
value; render the line's explicit status when the UI should remain responsive.

## The Two Objects To Remember

The **family** is the reusable recipe. It owns param normalization, canonical
identity, request posture, value shape, and any declared update structure.

The **line** is one live member of that family. It exposes the current projected
value and the lifecycle evidence for one canonical identity:

```ts
invoice.value();
invoice.signal();
invoice.summary();
invoice.status();
invoice.freshness();
invoice.request();
invoice.diagnostics();
invoice.history();
```

The line is browser-local resource state, not your durable database. Confirmed
server observations are canonical for the line. Open optimistic effects may
temporarily contribute to its projected value, but they remain separately
identified intent until confirmed.

## When The Endpoint Grows

Keep the family and line. Add only the capability the problem now requires.

| Need | Add | Read next |
| --- | --- | --- |
| Auth, headers, retry, or stale policy | API scope and request posture | [Reading And Caching](./caching/README.md) |
| Create, update, remove, or reconcile a response | Write finalizers and declared response targets | [Writing And Server Reconciliation](./updating/README.md) |
| Immediate UI with independent request outcomes | A declared effect profile | [Optimistic Updates](./effects/README.md) |
| Update one item, field, region, or semantic view | Declared identity and narrow update structure | [Collections And Partial Updates](./partial-updates/README.md) |
| Explain stale data, delivery, rollback, or replay | Line summaries, diagnostics, and history | [Debugging And Recovery](./debugging/README.md) |
| Transfers, downloads, external delivery, forms, or routes | The adjacent capability on the same line | [Advanced Resource Capabilities](./advanced/README.md) |

## Stable Entry Points

Use the route-first lane for ordinary application code:

- `signals.api(...)`
- `api.scope(...)`
- `api.url(...)`
- `.detail(...)`, `.list(...)`, `.paged(...)`
- `.create(...)`, `.update(...)`, `.remove(...)`, `.command(...)`
- `family.line(...)`, `family.optionalLine(...)`, `family.execute(...)`

Use `signals.resource.detail(...)`, `signals.resource.collection(...)`, and
`signals.resource.paged(...)` when you intentionally need manual canonical
identity or compatibility-oriented declarations. That raw lane uses the same
resource runtime; it simply gives you more authoring responsibility.

## Boundaries That Matter

- `api.url(...)` constructs request metadata. It does not send HTTP.
- A collection needs stable item identity. A paged family also needs an honest
  rule for accumulating pages.
- Narrow patches and response mapping exist only for fields, regions, paths,
  items, aspects, and summaries the family can identify.
- Mutation responses update declared targets. Partial or stale responses remain
  visibly partial, stale, delivery-awaited, or refetch-required.
- Optimistic rollback and exact recovery depend on the selected effect profile
  and retained runtime support.
- Replay and restore report typed unavailability when required history or
  snapshots are missing.
- Named `apiScope` defaults must be stable. Put parameter-dependent defaults in
  `api.scope({...})`.

## Choose Your Next Page

- [Your First Resource](./start-here/your-first-resource.md) — build one real
  fetched detail line.
- [Reading And Caching](./caching/README.md) — identity, pending state,
  freshness, invalidation, and refresh.
- [Writing And Server Reconciliation](./updating/README.md) — writes, local
  patches, and server-owned results.
- [Optimistic Updates](./effects/README.md) — isolated effects, concurrency,
  dependencies, rollback, and recovery.
- [Collections And Partial Updates](./partial-updates/README.md) — stable item
  identity and narrow changes.
- [Debugging And Recovery](./debugging/README.md) — summaries, diagnostics,
  history, restore, and replay.
- [Advanced Resource Capabilities](./advanced/README.md) — transfers,
  downloads, external delivery, forms, routes, and the raw lane.
- [Resource API Reference](../api-reference/resources.md) — exact family and
  line surface maps.
- [Resource Glossary](./glossary.md) — the short version of every resource term
  used across these guides.

If you are still choosing the shape, start with [Choose A Resource Shape](./start-here/choose-a-resource-shape.md).
For task-first examples, use [Common Resource Recipes](./start-here/common-resource-recipes.md)
or the broader [Resource Recipes](./recipes.md) index.
