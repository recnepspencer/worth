# Invalidation And Refresh

Use this page when you need to choose between invalidating a line, refreshing
it immediately, or revalidating while preserving visible truth.

## Stable Entry Points

- `line.invalidate()`
- `line.refresh()`
- `line.revalidate()`
- `line.freshness()`
- `line.diagnostics()`
- `family.invalidate(params)` / `family.invalidateAll()`
- `signals.resource.invalidateAll()` / `signals.resource.refreshAll()`

## What Each One Means

- `invalidate()`
  Mark the line stale without pretending new data arrived.
- `refresh()`
  Start a fresh request for the line.
- `revalidate()`
  Re-check the line while preserving the visible value continuity lane.

## Example

```ts
const line = userDetail.line({ userId: "u1" });

line.invalidate();
console.log(line.freshness());

line.refresh();
console.log(line.status());

line.revalidate();
console.log(line.freshness());
```

## Runtime-Wide Invalidation And Refresh

Sign-out, tenant switch, and "the server told us everything changed" are not
per-family events. The namespace sweeps every materialized line of every
family and returns how many it touched:

```ts
const marked = signals.resource.invalidateAll();
// every line: freshness { kind: "stale", reason: "manualRuntimeInvalidateAll" }
// diagnostics().lastInvalidationScope === "runtimeAll"

const refreshed = signals.resource.refreshAll();
// every line: a new load, with line.refresh() semantics (pending reloads are
// superseded)
```

Both are events over the lines that exist at the call: a line materialized
afterwards starts fresh. Released lines are skipped and not counted. Visible
values stay in place through `invalidateAll()`; `refreshAll()` replaces them
as each load settles.

## Why Revalidate Exists

`revalidate()` is the nicer lane when you want to keep showing the current
visible value while checking whether it is still current.

## Related Docs

- [Stale, Pending, And Settled State](./stale-pending-and-settled-state.md)
- [External Delivery And Compatibility](../external-delivery-and-compatibility.md)
