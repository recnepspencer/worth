# Nearest Valid Truth

## What This Feature Is

Nearest valid truth is the admitted route the router reaches after applying a
declared recovery fallback.

## Why You Use It

- explain the route the user actually landed on
- keep restored or retained UI aligned with a valid route
- distinguish "attempted truth" from "visible truth"

## Stable Entry Points

- `outcome.routeId`
- `outcome.href`
- `outcome.recovery()`
- `outcome.provenance()`

## Core Mental Model

Recovery gives you two route stories at once:

- the attempted route truth
- the nearest valid admitted truth

The nearest valid route is the one the app should render and reason about after
recovery. The attempted route still exists for diagnostics, analytics, and
debugging.

## How It Executes

1. the attempted route reaches a terminal artifact
2. a recovery fallback projects a new route candidate
3. the fallback candidate admits successfully
4. the final outcome exposes the admitted route as current truth
5. provenance links current truth back to the stale attempt

## Small Example

```ts
const outcome = await routes.admit("/projects/project-1", {
  projectState: "deleted",
});

console.log(outcome.routeId); // app.projects.index
console.log(outcome.href);    // /projects
```

## Real Example

```ts
const outcome = await routes.admit("/projects/project-1", {
  workspaceState: "active",
  projectState: "deleted",
});

if (outcome.kind === "admitted") {
  console.log(outcome.route().routeId);
  console.log(outcome.layouts().map((layout) => layout.routeId));
  console.log(outcome.outlet().descriptor().occupantRouteId);
  console.log(outcome.provenance().attemptedRouteId);
  console.log(outcome.provenance().resolvedRouteId);
}
```

The admitted route capability, layout stack, and outlet contract all describe
the nearest valid route, not the stale attempted one.

## How It Relates To Other Features

- nearest valid truth comes from [Stale Deep Link Recovery](./stale_deep_link_recovery.md)
- it still flows through ordinary [History](../history/README.md) if admitted
  from browser ingress

## Inspection And Debugging

- `outcome.routeId`
- `outcome.href`
- `outcome.route()`
- `outcome.layouts()`
- `outcome.outlet()`
- `outcome.provenance().attemptedRouteId`
- `outcome.provenance().resolvedRouteId`

## Anti-Patterns

- reading `attemptedRouteId` and rendering UI from it after recovery
- treating recovery as if it preserved the stale route with a soft warning
- comparing only `href` without checking whether recovery happened

## Current Limits

- nearest valid truth only exists when recovery produces an admitted route
- if recovery does not project or admit, the router fails closed
- recovery does not silently normalize stale routes into equivalent routes; it
  remains an explicit fallback lane

## Related Docs

- [Stale Deep Link Recovery](./stale_deep_link_recovery.md)
- [Recovery Provenance](./recovery_provenance.md)
- [Route Outcomes](../admission/route_outcomes.md)
