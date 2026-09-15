# Resource Line Reference

A resource line is the live browser-runtime handle for one canonical member of
a resource family. Pass the line when consumers need value plus lifecycle;
peeling off `value()` loses the state that explains whether that value is
pending, stale, projected, or confirmed.

## Obtain A Line

```ts
const line = family.line(params);
```

Families also expose:

```ts
const optional = family.optionalLine(enabled ? params : null);
const execution = family.execute(params, { freeOnSettle: true });
const result = await execution.settled();
```

`optionalLine(...)` returns `null` for `null`, `undefined`, or
`{ enabled: false }`. `execute(...)` groups a line with settlement and cleanup
for one-shot workflows.

## Value And Subscription

- `value()` — current projected value; can be `null` before first settlement.
  Line values are JSON values: a loaded result is committed as the JSON it
  serializes to (`toJSON` honored once per value, so a `Date` is stored as its
  ISO string; `undefined` members omitted; non-finite numbers stored as
  `null`). A result JSON cannot represent faithfully (`Map`, `Set`, typed
  arrays, boxed primitives, functions, cyclic data) is not committed: the line
  settles `rejected` with a message naming the value class and its path, on
  the initial load and on every reload;
- `signal()` — computed signal handle for the value;
- `view(project)` — a derived signal view of the line value;
- `summary()` — grouped current resource posture;
- `summarySignal()` — live signal handle for that grouped posture.

`view(...)` does not create a second line or a new source of server truth.

## Identity And Request

- `descriptor()` — family identity, canonical params, runtime line ID, scope,
  and compatibility posture;
- `request()` — canonical params, target URL, method, body, auth, context,
  continuation, processing, upload, and effect posture.

The request descriptor records what the family admitted. The `load` function
uses it to perform I/O; the descriptor does not send a request itself.

## Lifecycle

- `status()` — pending, fulfilled, rejected, or timed out;
- `freshness()` — fresh or stale;
- `awaitSettlement({ timeoutMs?, drainAuthoredWork? })` — wait for the next
  settled line tip-status truth; when `timeoutMs` elapses first the promise
  resolves with `resultKind: "timedOut"` (the line stays pending and settles
  later; it never rejects for time); set `drainAuthoredWork: true` to also
  drain authored publications/mutations after tip status settles
  ([1.5 migration](../package/migration-1.5.md));
- `invalidate()` — retain visible value and mark stale;
- `refresh()` — start a new load;
- `revalidate()` — run the family's revalidation behavior;
- `free()` / `[Symbol.dispose]()` — release the consumer's line handle.

Settlement returns a discriminated `resultKind`:

```ts
const result = await line.awaitSettlement({ timeoutMs: 5_000 });

switch (result.resultKind) {
  case "fulfilled":
  case "partial":
    console.log(result.value, result.freshness);
    break;
  case "rejected":
  case "timedOut":
    console.error(result.status, result.diagnosticsSummary);
    break;
}
```

`partial` carries a value but records that the runtime does not have complete
canonical truth. It should not be flattened into full fulfillment when later
behavior depends on completeness.

## Processing, Uploads, And Downloads

- `processing()` — ready, accepted, or in-progress remote processing posture;
- `upload()` — ready, prepared, or uploaded transfer posture;
- `download()` — ready, unavailable, or incompatible download descriptors.

These are adjacent lifecycle reads on the same line. Remote jobs, browser file
APIs, network transport, and storage remain external boundaries.

## Diagnostics And History

- `diagnosticsSummary()` — compact current and recent explanation;
- `diagnostics()` — detailed request, lifecycle, reconciliation, delivery, and
  effect evidence;
- `history()` — availability, lifecycle entries, basis, exact recovery actions,
  targeted effect rollback, and verification package.

```ts
console.log(line.summary());
console.log(line.diagnosticsSummary().latest);
console.log(line.history().availability);
```

History availability is explicit. `replayExact()` and `restoreExact()` return
typed unavailable variants when the line lacks the required executable history
or exact retained state. Diagnostic history alone does not imply replay.

## Mutation Responses

`mutationResponse()` returns the declared reconciliation plan for the latest
write response, or `null` when the family has no such plan. Use it to understand
which detail, item, summary, delete, tombstone, or identity-migration targets
the response was allowed to update.

## Patch-Capable Lines

Detail, collection, and paged lines expose these methods through their typed
family surface:

- `patch(family.patch.*(...), options?)`;
- `effects()`;
- `deliver(family.delivery.*(...))`;
- `reconciliation()`.

The actual patch and delivery helpers are constrained by declaration. A family
that declares only broad replacement does not acquire field, item, path,
aspect, or summary authority at runtime.

## Concurrent Effects

For a line using `signals.resource.effects.branchNative()`:

```ts
const effects = line.effects();

console.log(effects.open());
console.log(effects.get(effectId));
console.log(effects.projection());
console.log(effects.counters());

await effects.confirm(effectId, { responseId, serverPatch });
// or
await effects.reject(effectId, { responseId });
```

Always settle the runtime-issued effect identity for the request that finished.
`diagnostics().lastEffect` is not a concurrent settlement selector.

## Resource Line Summary

`summary()` is the ordinary debugging and presentation read. It groups current
status/freshness, request posture, processing, upload, download, diagnostics,
and history availability. It deliberately does not inline the full lifecycle,
replay artifacts, or exact restore results.

## Cleanup Rules

- Keep a line alive while UI or workflow code consumes it.
- Use `execute(..., { freeOnSettle: true })` for one-shot work.
- Settle or reject open effects before freeing a branch-native line.
- Do not recreate a family or copy `value()` into another cache as a cleanup
  strategy.

## Related Docs

- [Resource API Reference](./resources.md)
- [Resource Family Authoring](./resource-family-authoring.md)
- [Reading And Caching](../resources/caching/README.md)
- [Debugging And Recovery](../resources/debugging/README.md)
- [Optimistic Updates](../resources/effects/README.md)
