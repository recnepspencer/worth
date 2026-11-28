# Projected Candidates

Projection answers one deliberately narrow question: **which declared route,
layouts, and outlets match this URL?** It does not decide whether the user may
enter that route.

```ts
const candidate = routes.project("/app/projects/p7?tab=files");

if (candidate) {
  console.log(candidate.route().routeId);
  console.log(candidate.layouts().map((layout) => layout.routeId));
  console.log(candidate.outlets().map((outlet) => outlet.outletId));
}
```

`project` returns `null` when no declaration matches. When it succeeds, the
candidate contains canonical route input and structural composition:

- `route()` exposes typed params, search, hash, and declared capabilities.
- `layouts()` exposes the matched layout stack.
- `outlet()` is the leaf route's placement contract.
- `outlets()` exposes the full matched outlet stack.
- `verification()` proves which canonical and structural digests were used.

## Preview Is Not Permission

It is safe to use a candidate to build an admission plan, prefetch native
resources or warm a route. It is not safe to treat
it as the visible, authorized route.

```ts
const candidate = routes.project(targetHref);

if (!candidate) {
  return { kind: "noDeclaredRoute" };
}

const plan = candidate.admission(admissionFacts);
const outcome = await plan.resolve();
```

The source of truth remains the canonical URL authority. Projection is derived
structure that can be rebuilt from that authority and the route schema.

## When To Use The Higher-Level Method

If you only need the final outcome, call `routes.admit(href, facts)`. Use the
two-step `project` then `candidate.admission(...)` flow when you need to inspect
the candidate, explain the plan, warm resources, or speculate before resolving.

Next: [Admit](../admission/admit.md), [Layout Placement](./layout_placement.md),
and [Projection Verification](./projection_verification.md).
