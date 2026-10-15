# Feature Doc Template

Use this as the default skeleton for a product-facing feature doc.

```md
# <Feature Name>

## What This Feature Is

One short paragraph that defines the feature in plain language.

## Why You Use It

- concrete use case
- concrete use case
- concrete use case

## Stable Entry Points

- `api.surface(...)`
- `api.surface_variant(...)`

Call out any vocabulary-only or deferred neighbors here.

## Core Mental Model

Explain:
- where truth lives
- what this feature derives or orchestrates
- what handle/object the developer is holding
- what the runtime keeps track of automatically

## How It Executes

Give the lifecycle or phase shape in the order the runtime sees it.

## Small Example

```rust
// Minimal correct example
```

Explain why this is the smallest honest example.

## Real Example

```rust
// Realistic example that touches adjacent features
```

Walk through:
- what is authoritative
- what is derived
- what gets retained
- what gets inspected or observed

## How It Relates To Other Features

- When to pair it with X
- When Y is a better fit
- How it behaves in preview/branch/history/policy contexts, if relevant

## Inspection And Debugging

- which inspection or state surfaces help explain behavior
- what evidence the reader should expect to see

## Anti-Patterns

- misuse
- misuse
- unsupported shortcut

## Current Limits

- stable limit
- deferred future boundary
- explicit non-goal

## Related Docs

- [Adjacent Feature](./adjacent-feature.md)
- [Another Feature](./another-feature.md)
```
