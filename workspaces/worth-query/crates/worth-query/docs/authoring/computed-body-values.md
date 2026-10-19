# Shared Computed Body Values

## What This Feature Is

A body can declare a named, typed Foundation expression once and reuse its
prepared result in scope assignments, created-row initializers and an external
payload. Query evaluates it at the native decision observation, validates the
result against the installed destination codecs and publishes all sinks together.

## Why You Use It

Use this when several effects must agree on a value derived from current facts:
for example, the next business revision written to a document, its transition
audit and its notification. Do not put the arithmetic in an input serializer,
audit builder or payload codec.

## Stable Entry Points

Through `worth_query_decl::facade::application_schema`:

- `ApplicationOperationBodyValue::<Schema, Operation, Binding>::admit(name, operands, draft)`;
- `ApplicationOperationBodyExpression::PROFILE` for the shared expression profile;
- `body.assign_value(field, &value)?`;
- `body.assign_value_at_path(path, field, &value)?` for an exact related destination;
- `creation.initialize_value(field, &value)?`;
- `body.emit_external_evaluation::<Binding, _, _, _, _, _>(effect, query, condition)?`;
- `ApplicationOperationBodyEvaluationSnapshot` for the payload binding's Value.

Use `operation.requirement_operands()` for tracked sources. A computed value
currently has a unitless UInt64 result and at least one scope, exact path or created-field
sink. This is not a general feature-output reference or a host callback API.

## Core Mental Model

The typed value is authored meaning, not a supplied answer or execution permit.
Its name identifies one definition within the body. Reusing its reference retains
that definition once; conflicting meanings under one name are refused. Field
authoring requires the value's exact binding. Native installation also requires
that binding's readable, bounded input-admission codec.

The expression reads the admitted preimage. It does not observe earlier writes
in the same candidate. A prepared evaluation snapshot keeps `query()` (original
disclosed data) distinct from `value(name)` (computed decision data). Neither
public snapshot constructor grants source, decision or mutation authority.

## How It Executes

After ordinary admission and requirements, Query evaluates each definition once
with the same Foundation compiler and budgeted evaluator used by Boolean body
requirements. Requirements still require Boolean results. Computed values
require UInt64; conversion is exact, never saturating or wrapping.

Query validates the prepared result against every installed destination codec
before constructing candidate effects. Scope writes and audit initializers borrow
that prepared result. An evaluation payload formats the same retained values
alongside its exact admitted query context. Encoding happens once in native
preparation, not during candidate construction.

All tracked expression sources remain commit obligations. A concurrent revision
change stales the whole candidate, including its audit and outbox. Exact settled
request replay returns the original outcome without calculating today's value.
Keep the original input, request key and preconditions when recovering a result.

## Small Example

Given a required UInt64 revision field using `U64ApplicationValueBinding`:

```rust,ignore
let operation = EnterReview::reference();
let operands = operation.requirement_operands()
    .field("revision", CurrentRevision::reference())?;
let expressions = worth_foundational::expression_api::expressions()
    .reading_within(ApplicationOperationBodyExpression::PROFILE);
let next = ApplicationOperationBodyValue::<
    MySchema, EnterReview, U64ApplicationValueBinding,
>::admit("next-revision", operands, expressions.parse("revision + uint64(1)")?)?;
let body = operation.body()
    .assign_value(CurrentRevision::reference(), &next)?
    .finish()?;
```

`uint64(...)` is Foundation's typed literal constructor. A naked numeric literal
is not made UInt64 retroactively by its destination binding.

## Real Example

Reuse `next` from the example rather than authoring another revision expression:

```rust,ignore
let body = operation.body()
    .expect(CurrentStage::reference(), Stage::Draft)?
    .assign(CurrentStage::reference(), Stage::Review)?
    .assign_value(CurrentRevision::reference(), &next)?
    .emit_external_evaluation::<EnterReviewBinding, _, _, _, _, _>(
        ReviewNotification::reference(), notification_query_definition(), None,
    )?
    .create_in_scope(TransitionAudit::reference(), "audit")?
    .initialize_value(AuditRevision::reference(), &next)?
    // Initialize the audit's other required fields and relation as usual.
    .finish()?;
```

Declare the normal external effect slot, attach the body and install with
`setup.operation_body::<EnterReviewBinding>()?`. The effect's payload binding has
`type Value = ApplicationOperationBodyEvaluationSnapshot`; its pure codec reads
`snapshot.value("next-revision")` and `snapshot.query().field("description")`.
It validates/formats those values, never adds one to an observed revision.

The query must be the binding's exact parameter-free, ExactlyOne source query.
Execute it with `execute_with_expression_context()` and pass its real observed
source to `expect_source`. The supplier's `deviation-request --body-values`
journey uses an identity-only command and public query DTO, then observes revision
2 in the row, its audit and the committed notification bytes. It stops at outbox
publication, not external delivery.

## How It Relates To Other Features

[Computed graph-target writes](./computed-path-writes.md) use the same prepared
value at an existing parent or related row. Their native target selection adds
publication dependencies; it does not grant additional permission.

Boolean requirements, computation and effect conditions share source vocabulary
and Foundation semantics, not independent evaluators. Requirements deny unmet
business readiness; an effect condition can suppress just the external effect.
Feature-derived artifacts retain their own producer/settlement lifecycle and are
not automatically computed-body references. Signature dependency declarations,
workflow occurrences, authorization and external delivery keep their existing
owners; this API does not replace any of them.

A signed creation may include computed scope bookkeeping. For example, assign
`next` to CurrentRevision on the body, then use
`capture_from_scope(CurrentRevision, VerifiedAuditRevision, AuditContext)` on the
signed creation. The audit capture authenticates the observed old revision;
the root assignment publishes the prepared next revision in the same candidate.
Neither is calculated again by the signing adapter. The existing current-evidence
lifecycle rejects writing a `Subject` capture's source in that same operation.
An `initialize_value` inside a signed artifact is still refused: it is not an
explicitly classified source capture and cannot become certified by omission.

## Inspection And Debugging

Inspect `body.values()` for the single definition and the scope/created sink
names. Fresh portable admission checks the reference, binding and read closure.
Expression failure reports `OperationBodyValueExpressionDenied` with its
Foundation cause. An inadmissible destination reports `InvalidEffectValue`;
resource failures remain resource denials. Use fresh governed reads and the
receipt's `dispatch_outbox()` to observe publication; copied snapshots are data.

## Anti-Patterns

Do not calculate next revision in Rust codecs, send it as a new command field,
re-read graph values after commit, fabricate query context, or copy an expression
into every sink. Do not treat representable UInt64 as sufficient for a constrained
domain codec. A failed computation or codec is not a false optional condition.

## Current Limits

Computed results are unitless, frameless UInt64 only. Operands use the existing
supported body-source vocabulary. A value must feed a scope assignment or created
field; payload-only definitions are not currently supported. Computed field
initializers inside signed artifacts are refused rather than attaching a signature
to a guessed postimage. Computed scope writes outside the certified subject may
accompany signing through the normal native candidate/authentication owners.
The evaluation payload inherits the one-slot, unit-result native-body and query
context limits described in [conditional effects](./conditional-body-effects.md).

Named values/evaluation payloads use canonical body v18 and portable body v28.
Existing input-only v16/v26 and query-only v17/v27 meanings are unchanged.
Transported descriptions require fresh native readmission; they do not carry
the typed codec or mutation authority. No Store restart durability or external
exactly-once-delivery guarantee follows merely from a committed outbox.

## Related Docs

- [Scope-Bound Operation Bodies](./scope-field-operation-bodies.md)
- [Conditional External Effects](./conditional-body-effects.md)
- [Native Query Expression Contexts](./native-query-expression-contexts.md)
