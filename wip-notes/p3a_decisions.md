# Phase 3a decisions (the orchestrator's rulings on p3a_design.md)

1. A condition whose body reads only one fact is legal in 3a. No diagnostic for it. No source kind is retired.
2. Use sites get two diagnostic codes, `UnknownExpressionUseSite` and `ExpressionUseSiteRoleMismatch`. Refused expressions are tracked in seal state, so a refused expression gives exactly one diagnostic.
3. Admission drift gets a new variant, `ExpressionResult`, which carries its axis. It maps onto the existing stops: `OperabilityDependencyChanged` for mutability and readiness, `PolicyChanged` for policy. No new stop variant. Confirmation validation uses the same mapping.
4. A condition slot never reaches operability as a coerced value. The owner already records a non-bool condition result as `Denied(RoleMismatch)`. The consumer reads through a typed read with no `Value` arm. No `unreachable!`, `panic!`, or `expect` stands in for the type split.
5. One `ConditionWithheld` cause carries the axis. It is ordered after StaleTarget and before PolicyDenied. Inspection projects the condition identity from the catalog.
6. Appearance class: a withheld Unavailable shows as Pending, Stale as Stale, and Denied as Denied.
7. Push is a `#[must_use]` settlement output returned from `invalidate_application`, `invalidate_published_frame`, and `follow`. It is not a stateful drain.
   - The consumer index lives in `declaration/intent/catalog/condition_consumers.rs`.
   - Re-observation refreshes only existing standing facts, through the interaction-free observation path (option A).
     - One composition is shared with `observe_activation_operability`.
     - Re-observation keeps the fact's own affinity.
     - It refreshes only when the route still resolves to the same route.
     - Confirmation routes are skipped.
     - It never creates a fact.
     - The revision bumps only when the decision differs.
   - `operability_reobservations` counts attempted refreshes.
   - Application-fact and projection sources stay pull-only in 3a.
8. `follow` reports only slots whose outcome changed. Re-stamped slots are not reported. The evidence-only successor refreshes facts; cutover clears them.
9. Semantic comparison matches conditions by identity. The execution-plan semantic no-op must treat an expression-catalog change as Different.
10. `.expect`s in touched files become typed where an honest local alternative exists. The rest are listed.
11. The expression catalog is prepared before the intent catalog in both freeze paths.
12. Downstream matches on the public inspection enum are updated honestly.
