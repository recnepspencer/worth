# Authored Conditional External Effects

## What This Feature Is

An operation body can emit its exact typed input or its bound native query
context through its installed external effect slot, optionally guarded by a native Foundation expression. Query evaluates
the guard and publishes the business changes and typed outbox together. The host
does not reconstruct the condition or assemble an effect after the commit.

These are bounded native payload sources, not a host graph reader or an external
delivery service. A query payload binding formats already retained data; it
cannot perform another query or replace the admitted source with a public DTO.

## Why You Use It

Use it when a business operation may request an external action, such as a review
notification whose payload is the original subject reference. False or
inapplicable conditions suppress only the external effect, not the business
operation. An unmet business requirement still denies the operation.

## Stable Entry Points

Declare through `worth_query_decl::facade::application_schema`:

- `body.emit_external_input(effect, condition)?`;
- `body.emit_external_query_context::<Binding, _, _, _, _, _>(effect, query, condition)?`;
- `worth_query_decl::facade::application_query::ApplicationQueryExpressionSnapshot`
  for pure schema/value inspection inside a query-backed payload codec;
- `ApplicationOperationBodyRequirement::admit(...)` for the optional condition;
- `operation.definition().external_effect(effect, correlation_family)`;
- `ApplicationExternalEffectBinding` and `ApplicationRetainedEffectBinding` for
  the payload's pure protocol encoding, validation and retained size.

Attach the finished body with `with_body`, bind the program with `with_schema`,
and install through the existing `setup.operation_body::<Binding>()?` funnel.
No extra host codec catalog or effect dispatcher is installed.

## Core Mental Model

The declaration owns the effect, input/payload correspondence, protocol, byte
bound and optional condition. For the input profile, the payload binding may
have a different portable identity but must have the operation input's actual
Rust value type. For the query profile, its Value is
`ApplicationQueryExpressionSnapshot`; the command input keeps its own type.
The effect requires `OperationEmits<Operation>`.

A query payload names the exact parameter-free, ExactlyOne query declaration
and its mutation binding. That binding must declare the same source query.
Request the query with `execute_with_expression_context()` and pass its actual
observed source through `expect_source`. A normal read without the context is
insufficient. The native source retains recursive disclosed values before the
public result projection can discard them; its facts remain commit obligations.
Snapshot construction grants no query, source or mutation authority.

Typed authoring retains a pure codec recipe in compiler-local schema provenance.
Portable metadata cannot manufacture that recipe or emission authority. The
native evaluator produces the emission disposition from the current operation's
admitted reads, and the normal effect/outbox owners consume it.

A declared external slot remains mandatory unless the matching native body
earns an explicit suppressed disposition. Missing payloads on ordinary
handler-backed slots do not become optional.

## How It Executes

Query checks the normal admission, authorization, source guards and mandatory
requirements, then evaluates the optional effect condition at that same decision
observation. Met emits; Unmet or NotApplicable suppresses. Observation,
expression and resource failures remain denials, never false conditions.

For an emission, Query validates and funds the declared retained payload and
maximum encoded bytes before cloning/encoding the selected native payload.
Encoding happens once in the native decision. The candidate consumes that
prepared projection without rereading the source or encoding again. Query
retains the original context backing and funding through emission clones.
The native candidate carries the selected effect and all source/guard dependencies into ordinary
publication. Changed dependencies stale both emitting and suppressed candidates.
Business changes and the outbox share the existing atomic commit path.

Exact settled logical-request replay returns the original outcome before
reevaluating today's condition. Keep the original input, key and preconditions;
do not create a new request key to recover a lost response.

## Small Example

Given a typed effect whose external binding's Value is this operation's Input:

```rust,ignore
let body = RequestReview::reference().body()
    .in_scope(Document::reference())?
    .emit_external_input(ReviewNotification::reference(), None)?
    .finish()?;

let definition = RequestReview::reference().definition()
    .external_effect(
        ReviewNotification::reference(),
        WorthQueryExternalEffectCorrelationFamily::new("review-notification")?,
    )
    .no_aftermath()
    .finish()
    .with_body(body)?;
```

The binding admits one Emit and sufficient retained candidate resources. An
emission-only body still names its actual admitted scope. Correlation families
are atomic names, not dotted protocol identities.

## Real Example

Author a notification condition separately from mandatory review readiness:

```rust,ignore
let operation = EnterReview::reference();
let operands = operation.requirement_operands()
    .field("requested", NotificationRequested::reference())?;
let expressions = worth_foundational::expression_api::expressions()
    .reading_within(ApplicationOperationBodyRequirementExpression::PROFILE);
let notify = ApplicationOperationBodyRequirement::admit(
    "notify-review",
    operands,
    Some(expressions.parse("is_some(requested)")?),
    expressions.parse("unwrap(requested)")?,
    "requested",
)?;
let body = operation.body()
    .expect(CurrentStage::reference(), Stage::Draft)?
    .require_present(Description::reference())?
    .assign(CurrentStage::reference(), Stage::Review)?
    .emit_external_input(ReviewNotification::reference(), Some(notify))?
    .finish()?;
```

Attach the effect slot as above. A workflow-backed binding still requires its
actual issued occurrence and capability admission. A false or unanswered
notification flag permits the stage transition but yields no outbox. A missing
Description denies both. Adding an ordinary unsigned audit creation to the same
body keeps its source stage, actor attribution and notification atomic.

For a notification that needs graph data absent from the command, use the same
body and outbox funnel with a query-backed payload instead:

```rust,ignore
let body = EnterReview::reference().body()
    .expect(CurrentStage::reference(), Stage::Draft)?
    .assign(CurrentStage::reference(), Stage::Review)?
    .emit_external_query_context::<EnterReviewBinding, _, _, _, _, _>(
        ReviewNotification::reference(),
        notification_query_definition(),
        None,
    )?
    .finish()?;
```

Here the effect's payload binding has `type Value =
ApplicationQueryExpressionSnapshot`. Its pure `validate` and
`external_effect_bytes` methods can read
`snapshot.field("description").and_then(|v| v.as_option()).flatten()` and format
the disclosed value. `field` is a root-record lookup in the retained schema/value,
not a graph read. `schema()` and `value()` expose the full recursive expression
data for more complex pure formatting. Declare the actual wire size bound and
retained representation size on the binding as usual; the native emission
separately keeps the already funded original query context alive.

The source query can return a public DTO containing only the subject identity.
Neither that DTO nor the command needs to duplicate the description. The
operation still needs fresh capability and workflow occurrence authority. A
description edit between preparation and commit stales the entire candidate,
including its audit and notification. The `deviation-request` supplier's
`--query-notification` journey exercises this exact input/DTO split against an
ordinary Query installation; it stops at the committed outbox, not delivery.

## How It Relates To Other Features

The guard uses the existing body-requirement operand vocabulary, including
tracked related data and supported sealed query contexts. It is not part of the
mandatory readiness findings. Native authorization, invariants, workflow
occurrence custody, branch observation and replay retain their own owners.
External dispatch, acknowledgement and recovery use the existing aftermath
machinery; emitting a request does not prove delivery or receipt.
An ordinary workflow operation with an emitted external request retains its
native external-work custody until that owner settles it. A suppressed request
has no external custody and settles locally with the business commit.

## Inspection And Debugging

Inspect the installed body and external contract for exact source binding,
protocol and guard. Inspect the successful receipt's `dispatch_outbox()` for the
typed external record. Suppression has no outbox; it is not a failed dispatch.
Use native denials and fresh governed reads for authorization, stale publication
and business requirements, rather than assuming absence means success.

## Anti-Patterns

Do not evaluate the guard in React, emit after a separate business commit,
manufacture a payload from DTO fields, globally accept missing declared payloads,
or treat a copied body/codec/receipt as execution authority. Do not turn native
read failures into an Unmet condition.

## Current Limits

One external effect slot, either the exact whole input or the bound native
query expression snapshot, unit-result native bodies, and optional native
Boolean conditions are supported. Query sources remain parameter-free and
ExactlyOne, with the existing expression-context scalar/unit/disclosure limits.
Arbitrary host graph traversal and multiple external emissions are not supplied here.
Combining this profile with workflow control bodies, succession, signing
creation or workflow-enrolled creation is refused at installation rather than
partially executing. Ordinary workflow business operations may use it.

Input-only bodies keep canonical v16 and portable body v26 unchanged. Query
payloads use canonical v17 and portable body v27, including the full query
declaration. Older profiles cannot silently reinterpret them. Transported records remain descriptive until freshly admitted against
native typed schema provenance. This API does not itself establish Store restart
durability or external exactly-once delivery.

## Related Docs

- [Executable Scope-Bound Operation Bodies](./scope-field-operation-bodies.md)
- [Native Query Expression Contexts](./native-query-expression-contexts.md)
- [Application Aftermath And Recovery](../execution/application-aftermath-and-recovery.md)
- [Workflows](../foundations/workflows.md)
