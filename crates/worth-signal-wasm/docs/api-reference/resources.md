# Resource API Reference

This is the compact map of the shipped resource surface. Start with the
[Resources Overview](../resources/index.md) if you are still choosing a model;
use this page when you know the model and need the public entry point.

## Author A Family

Recommended route-first lane:

```ts
const api = signals.api({ baseUrl: "/api" });
const route = api.url("/projects/:projectId");

const detail = route.detail({ load });
const list = api.url("/projects").items((item) => item.id).list({ load });
const pages = api.url("/events").items((item) => item.id).paged({
  load,
  accumulatePage,
});
```

Write finalizers are `.create(...)`, `.update(...)`, `.remove(...)`, and
`.command(...)`. Use `api.scope(...)` for shared request defaults whose values
depend on params.

Raw family lane:

- `signals.resource.detail(...)`
- `signals.resource.collection(...)`
- `signals.resource.paged(...)`
- `resourceParams<T>()`
- `resourceParamIdentity(params, canonicalKey)`

See [Resource Family Authoring](./resource-family-authoring.md) for declaration
shape, identity rules, and family capabilities.

## Materialize And Read

```ts
const line = family.line(params);
const optional = family.optionalLine(enabled ? params : null);
const execution = family.execute(params, { freeOnSettle: true });
```

Core line reads:

- value: `value()`, `signal()`, `view(project)`;
- grouped state: `summary()`, `summarySignal()`;
- identity and request: `descriptor()`, `request()`;
- lifecycle: `status()`, `freshness()`, `awaitSettlement()`, `execute()`;
- operations: `invalidate()`, `refresh()`, `revalidate()`, `free()`;
- adjacent state: `processing()`, `upload()`, `download()`;
- evidence: `diagnostics()`, `diagnosticsSummary()`, `history()`;
- write result: `mutationResponse()`.

See [Resource Line](./resource-line.md) for result shapes and lifecycle rules.

Namespace-wide operations:

- `signals.resource.invalidateAll()` marks every materialized line of every
  family stale (`manualRuntimeInvalidateAll`, scope `runtimeAll`) and returns
  the count;
- `signals.resource.refreshAll()` starts a refresh on every materialized line
  and returns the count.

See [Invalidation And Refresh](../resources/caching/invalidation-and-refresh.md).

## Patch, Deliver, And Reconcile

Patch-capable detail, collection, and paged lines expose:

- `line.patch(family.patch.*(...))`
- `line.effects()`
- `line.deliver(family.delivery.*(...))`
- `line.reconciliation()`

The available helpers depend on declared shape. Broad replacement does not
prove item, field, region, path, aspect, or summary identity.

## Inspect Effects And History

Branch-native effect lines expose `open()`, `get(effectId)`, `projection()`,
`counters()`, `confirm(effectId, options)`, and `reject(effectId, options)`
through `line.effects()`.

History exposes availability, lifecycle, basis, exact replay/restore attempts,
targeted effect rollback, and `verificationPackage()`. Exact operations can
return typed unavailable results.

## Related Reference

- [Resource Request And Policy](./resource-request-and-policy.md)
- [Resource Transfers](./resource-transfers.md)
- [Resource Binary And Download](./resource-binary-and-download.md)
- [Route Authoring](./route-authoring.md)
- [Mutation Response Reconciliation](../resources/responses/README.md)
- [Optimistic Updates](../resources/effects/README.md)
