# Visible Route Truth

## What This Feature Is

Visible route truth is the transition-facing answer to "which route is the user
actually seeing right now?"

## Why You Use It

- distinguish target admission from visible route state
- explain redirect, direct, prefetch, or continuity-driven route changes
- keep route UI tied to one explicit story

## Stable Entry Points

- `transition.target()`
- `transition.diagnostics().visibleChangeSource`
- `transition.diagnostics().visiblePolicy`

## Core Mental Model

During transitions, visible truth is a product of:

- current route truth
- target route truth
- continuity policy
- resource continuity state

The router exposes that combination directly instead of making the app infer
it from loading booleans.

## How It Executes

1. decide the target route
2. classify the requested source
3. apply visibility policy
4. report the source of the visible change

## Small Example

```ts
console.log(transition.diagnostics().visibleChangeSource);
console.log(transition.diagnostics().visiblePolicy);
```

## Real Example

```ts
const direct = await routes.transition(home, "/about");
const redirect = await routes.transition(home, "/private", {
  facts: { auth: "anonymous" },
});

console.log(direct.diagnostics().visibleChangeSource);
console.log(redirect.diagnostics().visibleChangeSource);
```

## How It Relates To Other Features

- browser-history explanation happens later in [Navigation Auditability](../history/navigation_auditability.md)

## Inspection And Debugging

- `transition.diagnostics().requestedSource`
- `transition.diagnostics().visibleChangeSource`
- `transition.diagnostics().visiblePolicy`

## Anti-Patterns

- equating target route outcome with visible route truth during every transition

## Current Limits

- visible route truth is reported through transition diagnostics, not as a
  standalone route store

## Related Docs

- [Transition Artifacts](./transition_artifacts.md)
- [Pending Visibility](./pending_visibility.md)
