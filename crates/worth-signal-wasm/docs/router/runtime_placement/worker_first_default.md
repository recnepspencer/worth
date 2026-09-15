# Worker-First Default

Worth's default deployment posture is worker-first, and the public
`signals.router` namespace is available in both deployment modes.

```ts
const signals = await createSignals({ deployment: "workerFirst" });
const routes = signals.router.define({
  home: signals.router.route("/"),
});
```

That is the useful starting point. It does **not** mean browser APIs moved into
the worker, or that every JavaScript callback declared on a route is proven to
execute there.

## Tip Notify Vs Authored Settlement

> **1.5 breaking change:** default `awaitSettlement()` no longer drains authored
> work. See [migration-1.5](../../package/migration-1.5.md).

Host tip is the UI paint authority. Every mutation ingress
(`set` / `setWithAspects` / graph / import / resource binding apply) advances
host tip and notifies React (`createReactSignalsStore` / `useSignalValue` /
`useResourceLine`) in the same turn or next microtask. Do **not** put
dialog/popover open in React `useState`, and do not use
`mainThreadCompatibility` as a paint fallback.

Settlement is tip-honest handoff only:

| API | Resolves when | Use for |
|---|---|---|
| `line.awaitSettlement({ timeoutMs? })` | This line's **tip status** leaves pending | Wait for load/refresh tip |
| `line.awaitSettlement({ drainAuthoredWork: true })` | Tip status settled, then global authored drain | Tip-honest handoff after load |
| `signals.settleAuthoredWork()` | Pending pubs + mutations drained | Submit / write / worker proof |

`timeoutMs` is a failure deadline only — never the paint path.

```ts
// Paint: follow tip notify (useSignalValue / useResourceLine).
dialogOpen.set(false);

// Tip-honest handoff only (submit, worker proof) — not required to close UI:
await signals.settleAuthoredWork();
```

Form field mutations may return thenables under worker-first — await those (or
call `settleAuthoredWork`) before submit paths that assume published
summary/input truth on the worker.

The same host tip methods exist on `mainThreadCompatibility`, where the
runtime applies the tip writes in one synchronous transaction; application
code does not need a deployment branch to reach them.

## The Honest Boundary

The browser host observes `location`, `popstate`, clicks, and external
navigation. It packages those observations with the public router namespace:

```ts
const ingress = signals.router.browserHistory.pop(window.location.href);
const report = await routes.admitBrowserHistoryIngress(ingress);
story.record(report);
```

Worth admits that typed ingress against the route tree and returns a report.
The host remains responsible for installing listeners, rendering the result,
and performing `pushState`, `replaceState`, or external navigation.

## Compatibility Is Explicit

Use `mainThreadCompatibility` when the application intentionally needs that
deployment:

```ts
const signals = await createSignals({
  deployment: "mainThreadCompatibility",
});
```

Construction should fail explicitly when the requested worker deployment is
unavailable. A silent fallback would make performance and placement impossible
to reason about.

Public router docs intentionally teach `signals.router` and resolved route-tree
methods, not raw worker bridge methods. The bridge is runtime infrastructure,
not the application authoring surface.

Next: [Host And Worker Boundary](./host_worker_boundary.md) and
[Worker Navigation Auditability](./worker_navigation_auditability.md).
