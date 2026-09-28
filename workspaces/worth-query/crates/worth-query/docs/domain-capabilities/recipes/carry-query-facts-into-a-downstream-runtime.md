# Carry Query Facts Into A Downstream Runtime

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Recipe Covers

Use this recipe when a downstream runtime must act on facts produced by a
completed ordinary Query read. The handoff is one sealed
`WorthQueryConsumedProjectionAuthority`, not a tuple of rows, receipts,
digests, and basis labels.

## When To Use It

Use it for allocation, invalidation, rendering, or another downstream decision
whose correctness depends on the exact Query basis, source lineage, and
admitted projection facts.

## Stable Entry Points

- `WorthQueryReadCompletion::consume_projection(...)`
- `read::project_facts()`
- fact selectors such as `entity_identities()`, `memberships()`,
  `target_identity()`, and `source_references()`
- `WorthQueryProjectionOutcome::into_admitted()`

The completion owns the result-shape, authorized-projection, basis, source,
and settlement binding. Ordinary callers choose required facts; they do not
reassemble those safety inputs.

## The Smallest Useful Path

```rust
use worth_query::facade::read::{
    project_facts, WorthQueryProjectionOutcome, WorthQueryReadCompletion,
};

fn query_authority_outcome(
    completion: &WorthQueryReadCompletion,
) -> WorthQueryProjectionOutcome {
    completion.consume_projection(
        project_facts()
            .target_identity()
            .source_references(),
    )
}

let outcome = query_authority_outcome(&completion);
let (authority, warnings) = outcome
    .into_admitted()
    .expect("required projection facts should admit");

downstream_runtime.apply_query_facts(authority, warnings)?;
```

Declare only the facts the consumer needs. A violation, deferral, or
unavailable outcome is terminal for that attempt; do not retry by extracting
raw IDs or comparing basis digests locally.

## Core Mental Model

```text
completed ordinary read
-> declare required facts
-> Query verifies the sealed result/basis/source relationship
-> move one admitted authority object into the downstream operation
```

`into_admitted()` accepts both completed and advisory outcomes. Advisory
warnings move with the non-cloneable authority. Every other posture returns
the original typed outcome so the caller can match it without losing evidence.

## What To Retain

- Retain `WorthQueryConsumedProjectionAuthority` while downstream work depends
  on that exact relationship.
- Use `source_identity()`, `receipt()`, and `evidence()` for inspection or
  indexing only.
- Use `facts()` for the admitted typed facts.
- Retain warnings when the downstream policy distinguishes advisory admission.

Do not serialize the authority as a reusable token. It proves one admitted
consumption. Durable declaration replay and store-backed authority reload are
separate, support-gated capabilities; a JSON document or digest is not a
replacement for the live authority object.

## Advanced Substrate Sources

Retained derived artifacts, live artifact bindings, write receipts, and query
context execution artifacts have source-specific projection-consumption
operations in the foundation/runtime surface. Use those only when the source
really is that advanced artifact. Do not downcast an ordinary completion into
that lower-level path or manually supply result-shape and authorization
artifacts that the completion already owns.

## Anti-Patterns

- passing `(basis_digest, receipt_digest, facts)` as authority
- calling a lower-level projection method for an ordinary read completion
- reconstructing authorized projection or result-shape binding in consumer code
- treating advisory warnings as either success with no warning or total denial
- persisting the admitted authority as a durable bearer token

## Related Docs

- [Projection Consumption And Downstream Authority](../../capabilities/projection-consumption.md)
- [Declarative Query Experience](../../capabilities/declarative-query-experience.md)
- [Downstream Runtime Integration](../../foundations/downstream-runtime-integration.md)
- [Support Matrix And Admission](../../foundations/support-matrix-and-admission.md)
