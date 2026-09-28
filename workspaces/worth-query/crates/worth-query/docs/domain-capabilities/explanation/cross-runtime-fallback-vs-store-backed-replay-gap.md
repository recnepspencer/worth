# Cross-Runtime Fallback Vs Store-Backed Replay Gap

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Feature Is

This doc explains the difference between cross-runtime fallback explanation and
store-backed replay-gap explanation so downstream domains do not teach one as a
substitute for the other.

## Why You Use It

- you need to choose the right explanation posture for geometry workflows
- replay ambiguity and runtime fallback are different truths
- the wrong explanation posture produces misleading debugging and certification
  stories

## Stable Entry Points

- `WorthQueryLowerRuntimeExplanationRequest::explains_cross_runtime_fallback(...)`
- `WorthQueryLowerRuntimeExplanationRequest::explains_store_backed_replay_gap(...)`
- matching ordinary domain contribution verbs on the lower-runtime surface

## Core Mental Model

Cross-runtime fallback means the runtime could still explain the operation, but
it had to fall back across a runtime boundary to do it.

Store-backed replay gap means replay or restore cannot reconstruct the same
authoritative explanation detail that the live runtime once had.

Those are not the same failure mode.

## How It Executes

1. decide whether you are explaining a fallback or a replay gap
2. build the matching lower-runtime explanation request
3. materialize through review or final artifact lanes

## Small Example

```rust
let request = WorthQueryLowerRuntimeExplanationRequest::explains_store_backed_replay_gap(
    reference_set,
    target,
    evidence_families,
    redaction_policy,
    materialization_policy,
);
```

## Real Example

Use cross-runtime fallback when a live geometry explanation can still be
materialized by walking bridge-backed evidence across runtimes.

Use store-backed replay gap when a restored or replayed geometry artifact can
only say "this is where explanation detail was lost" rather than re-minting the
live authoritative explanation.

## How It Relates To Other Features

- Lower-Runtime Explanation Contributions
  covers the authoring and materialization lanes
- Continuity Vs Correspondence
  is a similar distinction on the identity side: authoritative continuation is
  stronger than weaker match evidence

## Inspection And Debugging

- do not treat replay-gap artifacts as if they were full live explanations
- if the live runtime could not reconstruct exact detail and fell back, use the
  fallback posture instead

## Anti-Patterns

- using replay-gap wording for ordinary runtime fallback
- erasing the difference between missing explanation detail and cross-runtime
  fallback traversal
- promising exact live explanation parity on restored artifacts when the runtime
  no longer has that authority

## Current Limits

- both paths still rely on real lower-runtime causal evidence vocabulary
- the ordinary Query lane wraps these requests, but does not replace the
  underlying explanation semantics

## Related Docs

- Lower-Runtime Explanation Contributions
- [Inspection](../../capabilities/inspection.md)
- Continuity Vs Correspondence
