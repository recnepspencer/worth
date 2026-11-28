# Pending Visibility

## What This Feature Is

Pending visibility is the transition policy that decides whether visible route
truth switches immediately, preserves the current route, or shows target
continuity while work is pending.

## Why You Use It

- make route transition freshness explicit
- keep pending-route UI honest
- avoid guessing visibility from loading state alone

## Stable Entry Points

- `transition.diagnostics().visiblePolicy`
- `transition.diagnostics().visibleChangeSource`
- `continuity` in `routes.transition(...)`

## Core Mental Model

Transition visibility is separate from route admission. The target route may be
admitted while the visible route still follows a continuity policy.

## How It Executes

1. admit or reuse the target route truth
2. inspect continuity policy
3. check whether target resources already have visible value
4. emit one visible policy and visible change source

## Small Example

```ts
const transition = await routes.transition(home, "/about", {
  continuity: "preserve-visible-while-pending",
});
```

## Real Example

```ts
if (prefetched) {
  const transition = await routes.transition(home, prefetched, {
    continuity: "preserve-visible-while-pending",
  });

  console.log(transition.diagnostics().visiblePolicy);
  console.log(transition.diagnostics().pendingResourceNames);
}
```

## How It Relates To Other Features

- resource continuity is covered in [Continuity Preservation](./continuity_preservation.md)

## Inspection And Debugging

- `transition.diagnostics().visiblePolicy`
- `transition.diagnostics().visibleChangeSource`
- `transition.diagnostics().pendingResourceNames`

## Anti-Patterns

- assuming pending always means "keep the current route"
- mixing speculation visibility rules into ordinary transition code

## Current Limits

- pending visibility is a route transition policy, not a general UI loading API

## Related Docs

- [Transition Artifacts](./transition_artifacts.md)
- [Continuity Preservation](./continuity_preservation.md)
