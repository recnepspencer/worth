# Router Overview

## What This Feature Is

Worth's router takes a browser location through one explicit path: normalize
the URL, match declared structure, decide whether that match may become route
truth, and retain the boundary event that made it visible.

Three terms recur in the API:

- **Projection** means structural matching: what route, layouts, outlets, and
  declared capabilities fit this URL?
- **Admission** means permission now: may that structural match become route
  truth, or must it redirect, fail, or recover?
- **Provenance** means the retained explanation: which check, fallback, browser
  event, or restore boundary produced the result?

That separation is the point. A match is useful, but it is not permission.

## Why You Use It

- build typed links instead of assembling paths and query strings by hand
- keep route structure, access decisions, resources, breadcrumbs, and form
  continuity on one declared route tree
- preserve the current screen honestly while a target is pending
- explain why a route is visible without reverse-engineering UI state

## Stable Entry Points

- `signals.router.route(...)`, `layout(...)`, and `define(...)`
- `route.href(...)`, `to(...)`, `match(...)`, `intent(...)`, and `plan(...)`
- `routes.project(...)` and `routes.admit(...)`
- `routes.transition(...)`
- `signals.router.browserHistory.*(...)`
- `signals.router.browserHistory.story(...)`

## Core Mental Model

The browser owns the raw address bar location. The router owns the typed,
normalized interpretation of that location and whether it is admitted. A UI
renders admitted route truth; it should not maintain a second `currentRoute`
store beside the router.

The ordinary objects have deliberately different powers:

1. A `RouteReference` builds and matches typed locations.
2. A `ProjectedRouteCandidate` describes a structural match and may prefetch,
   speculate, or compile an admission plan.
3. A `RouteOutcome` says whether the candidate was admitted or why it was not.
4. A browser-history report says what happened at the host boundary.
5. A history story retains reports and derives current, back, breadcrumb, and
   audit views.

The browser-history APIs create envelopes and reports. They do not call
`window.history`, open external URLs, or broadcast between tabs. Your host
adapter performs those effects and carries the evidence back.

## How It Executes

For app-driven navigation:

1. build a typed location from a route reference;
2. optionally inspect its navigation plan;
3. project it when you need a preview;
4. admit it when it may become route truth;
5. transition from the current admitted outcome;
6. send the corresponding writeback through the browser host.

For browser-driven navigation:

1. the host creates a `load`, `pop`, `manual`, or `external` ingress envelope;
2. `routes.admitBrowserHistoryIngress(...)` resolves it through the route tree;
3. the history story records the report;
4. the UI reads the story's current admitted entry.

## Small Example

```ts
// app/routes.ts
import type { CallableSignals } from "worth-signals-wasm";

export function defineAppRoutes(signals: CallableSignals) {
  return signals.router.define({
    home: signals.router.route("/"),
    projectDetail: signals.router.route("/projects/:projectId", {
      search: {
        tab: signals.router.search.optional.string(),
      },
    }),
  });
}

export type AppRoutes = ReturnType<typeof defineAppRoutes>;
```

```ts
// app/project-link.ts
import type { AppRoutes } from "./routes.js";

export function projectHref(routes: AppRoutes, projectId: string) {
  return routes.projectDetail.href({
    params: { projectId },
    search: { tab: "files" },
  });
}
```

The route reference owns encoding, search validation, and canonical ordering.
The link function supplies business values; it does not know URL grammar.

## Real Example

Keep route declarations, admission policy, and host orchestration in separate
modules. They change for different reasons.

```ts
// app/route-access.ts
import type { CallableSignals } from "worth-signals-wasm";

export const signedIn = (signals: CallableSignals) => {
  const sessionReady = signals.router.host.boolean("sessionReady");

  return {
    sessionReady,
    prerequisite: signals.router.prerequisite("signedIn", {
      consumes: [sessionReady] as const,
      evaluate: (context) => context.consume(sessionReady)
        ? context.allow({ reason: "session admitted" })
        : context.redirect({ href: "/sign-in", reason: "sign in required" }),
    }),
  };
};
```

```ts
// app/routes.ts
import type { CallableSignals } from "worth-signals-wasm";
import { signedIn } from "./route-access.js";

export function defineAppRoutes(signals: CallableSignals) {
  const access = signedIn(signals);
  const appRoute = signals.router.route("/app", {
    breadcrumb: signals.router.breadcrumb({ id: "app", label: "App" }),
  });

  return {
    access,
    routes: signals.router.define({
      home: signals.router.route("/"),
      signIn: signals.router.route("/sign-in"),
      app: signals.router.layout(appRoute, { outlet: "main" }, {
        projectDetail: signals.router.route("/app/projects/:projectId", {
          admission: [access.prerequisite],
          breadcrumb: signals.router.breadcrumb({
            id: "project",
            label: ({ params }) => `Project ${params.projectId}`,
          }),
        }),
      }),
    }),
  };
}

export type AppRouting = ReturnType<typeof defineAppRoutes>;
```

```ts
// app/navigation-session.ts
import type { CallableSignals } from "worth-signals-wasm";
import type { AppRouting } from "./routes.js";

export async function createNavigationSession(
  signals: CallableSignals,
  routing: AppRouting,
  href: string,
  sessionReady: boolean,
) {
  const ingress = signals.router.browserHistory.load(href);
  const report = await routing.routes.admitBrowserHistoryIngress(
    ingress,
    { sessionReady },
  );
  const story = signals.router.browserHistory.story(report);

  return {
    report,
    story,
    current: story.current(),
    explanation: story.auditability().summary(),
  };
}
```

The prerequisite declares the fact it consumes. The host supplies that fact at
the admission boundary. The router never reaches into an ambient session store
and never mistakes a redirect for an admitted route.

`createNavigationSession` records the report in one retained story and returns
both ordinary current truth and the richer explanation surface. The browser
adapter can keep that story alive and record later `pop` and writeback reports.

## How It Relates To Other Features

- [Route declaration and matching](./projection/README.md) establish structure.
- [Admission](./admission/README.md) consumes explicit facts and returns typed
  outcomes.
- [Transitions](./transitions/README.md) describe visible continuity between
  admitted outcomes.
- [History](./history/README.md) handles browser ingress, writeback, and the
  retained navigation story.
- [Resources](./resources/README.md) bind native resource lines without a
  second router cache.
- [Forms](./forms/README.md) consume admitted route authority for draft
  continuity.

## Inspection And Debugging

- `route.verification()` checks declaration and schema identity.
- `candidate.verification()` checks structural projection.
- `outcome.diagnostics()` explains one admission result.
- `outcome.provenance()` shows prerequisite and recovery decisions.
- `story.inspection()` inventories retained navigation evidence.
- `story.auditability()` answers why the current route is visible now.

Use the smallest surface that answers the question. Digests are useful proof;
they are miserable application control flow.

## Anti-Patterns

- passing raw route strings throughout business code
- rendering a projected candidate before it is admitted
- calling browser APIs from admission callbacks
- copying the story's current entry into component state
- treating cross-tab coherence metadata as a cross-tab transport

## Current Limits

- the host still owns actual browser history, external navigation, and
  cross-tab delivery
- exact restore requires a real runtime snapshot boundary
- worker-first is the default runtime direction, but the public router facade
  does not imply that browser APIs or every JavaScript callback run in a worker

## Related Docs

- [Route Schema Authoring](./projection/route_schema_authoring.md)
- [Admit](./admission/admit.md)
- [Browser History Story](./history/browser_history_story.md)
- [Diagnostics Surfaces](./diagnostics/diagnostics_surfaces.md)
- [Router Glossary](./glossary.md)
- [Raw Location Authority](./authority/raw_location_authority.md)
- [Path, Search, And Hash State](./authority/path_search_hash_state.md)
- [Canonical URL Authority](./authority/canonical_url_authority.md)
- [Route Identity And Equivalence](./authority/route_identity_and_equivalence.md)
- [Hydration Handoff](./boundaries/hydration_handoff.md)
- [Coherence And Boundary Artifacts](./diagnostics/coherence_and_boundary_artifacts.md)
