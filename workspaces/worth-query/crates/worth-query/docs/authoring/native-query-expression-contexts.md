# Native Query Expression Contexts

## What This Feature Is

An opt-in one-shot read can retain its native projected row as an immutable
Foundation expression record beside the row's observed source. The record is
derived before the application's Rust result projector runs. It is descriptive
data, not read permission, approval or mutation authority.

## Why You Use It

Use this when a nested query's native values must remain available even though
the public application result is a smaller summary. Correlated child rows stay
records; independent scalar lists do not need to be zipped back together.

## Stable Entry Points

- `request.query(intent).execute_with_expression_context()` for an ordinary
  exactly-one one-shot query.
- `admitted_plan.retain_expression_context()` before native one-shot execution
  when the caller already owns a genuine admitted plan, including governed plans.
- `observed_source.expression_context()` returns borrowed `(schema, value)`.
- `Operation::reference().requirement_operands().query_context::<Binding, _, _, _, _>(name, query_definition)`
  consumes an exact query declaration for a native operation requirement.
- Foundation's `ExpressionType::from_scalar_family` and
  `ExpressionValue::from_aspect_value` provide the lossless scalar correspondence.

Ordinary `execute()` does not retain a context. There is no caller-supplied
replacement-context setter.

## Core Mental Model

The query owns selection, correlation, disclosure and recursive dependencies.
The retained context interprets that already-materialized disclosed tree; it
does not read the graph again or call the domain projector. Its expression
schema has one operand, `source`, whose record fields are the installed query's
output names. Optional values remain `Option`; relations preserve their
declared exactly-one, optional-one or many cardinality.

The observed source retains the context and its native dependency footprint
together. Clones share immutable custody. Dropping the result does not destroy
a source clone's context. A context is the earlier observation, not a live view.

## How It Executes

Installation derives the expression schema once from the exact installed query
projection and schema. The existing query admission and native read run first.
For the opt-in read, Query requires one complete root and converts the disclosed
native tree against that retained schema. Only then does
the application projector construct its public result. The source retains the
context's backing until the last source/context owner drops it.

Omitted fields/relations, incomplete sources, truncation, unsupported scalar
meaning and declared units refuse retention. They never become an empty
collection or a legitimate absent optional value. Cancellation, deadline and
retention-resource failures retain their existing typed outcomes.

## Small Example

```rust,ignore
let result = runtime.request(&principal, &scope)
    .query(ReadDraft { identity: "draft-1".into() })
    .execute_with_expression_context()?;
let source = result.observed_sources()[0].clone();
let (schema, value) = source.expression_context().expect("requested context");
// Bind `value.clone()` as operand `source` against this exact `schema`.
// Optional description is Option<String>, not an empty string substitute.
```

## Real Example

For an installed tenant query containing `drafts`, and each draft's `evidence`,
the same Foundation language can inspect:

```text
source.drafts.all(d,
    is_some(d.description) &&
    d.evidence.all(e, e.description == unwrap(d.description)))
```

The public projector may return only the tenant number. The native context
still contains each draft's own evidence rows; it is not rebuilt from that DTO.
The runnable deviation-request supplier example demonstrates this path through
the public declaration/host entry points.

To make correlated information readiness an operation requirement, author it
once against the same query definition registered in the schema:

```rust,ignore
let operands = Check::reference().requirement_operands()
    .query_context::<CheckBinding, _, _, _, _>("context", readiness_query())?;
let requirement = ApplicationOperationBodyRequirement::admit(
    "draft-information", operands, None,
    expressions().reading_within(ApplicationOperationBodyRequirementExpression::PROFILE)
        .parse("length(context.drafts) >= 2 && context.drafts.all(d, is_some(d.description) && length(unwrap(d.description)) > 0)")?,
    "context",
)?;
// Attach with body.requirement(requirement), then install the ordinary body.
```

`CheckBinding::SourceExpectation` must name that query. It must be parameter-free
and exactly-one at the root; the initial requirement profile does not silently
accept a Rust callback that chooses different source parameters. Declare the
query once through a definition factory used by schema registration and operand
authoring. Do not independently map its fields into a second expression record.

At the host front door, pass the retained source to both operations:

```rust,ignore
let source = runtime.request(&principal, &scope).query(ReadReadiness(tenant))
    .execute_with_expression_context()?.observed_sources()[0].clone();
let findings = runtime.request(&principal, &scope).advanced().mutate(Check(tenant))
    .expect_source(source.clone())
    .assess_current_capability_body_requirements(program, projection)?;
let outcome = runtime.request(&principal, &scope).advanced().mutate(Check(tenant))
    .expect_source(source).idempotency(&logical_key)
    .execute_capability_in_program(program, allocation)?;
```

In the supplier's actual readiness journey, `drafts` additionally contains each
draft's optional `current_round` record and that round's optional description.
The authored expression requires information on the selected round when present;
it does not require that round to be signed. Completing unsigned B permits the
operation even though A signed earlier. A signature-only change does not stale
this query, which deliberately excludes signature rows.

## How It Relates To Other Features

`expect_source(source)` still checks the source's exact installed query, scope,
parameters when required by the binding, and native dependencies. Field or
membership changes can stale the later mutation. The retained value itself
remains immutable. A new read is new source intent, not an automatic retry of
an ambiguous old mutation.

For an authored context requirement, cold admission checks the complete embedded
query against its schema declaration and the mutation binding. Installation
checks the actual binding's source-query correspondence. Native source admission
checks exact installed query, branch, scope and an empty parameter set, transfers
values and dependencies together, and compares those dependencies at the
operation's retained observation. The same body evaluator serves guidance and
execution. Nested consumed-field edits and relation-membership changes remain
publication obligations, even after successful requirement evaluation.

Exact committed replay resolves before fresh source currentness or requirement
evaluation, so later edits cannot erase an already-recorded outcome. Keep the
original source and logical key for that replay; replacing the source changes
intent. A fresh attempt still needs fresh capability authorization. Findings
are descriptive, not a mutation permit or approval grant. Do not add a host
Boolean or validation loop as an alternate admission path.

## Inspection And Debugging

Use `expression_context()` to inspect the exact schema and value. `None` means
the read did not request retention. `ExpressionContextUnavailable` means the
installed shape cannot provide a supported context. `ExpressionContextIncomplete`
means this read was incomplete or disclosure withheld data; it is not cached as
a defect of the shared source row, and recovery requires fresh admission.
Neither outcome permits using partial values. Typed resource/interruption
outcomes remain distinct. Result-buffer observation
includes the retained context backing, not just its logical scalar payload.
Using ordinary `execute()` and submitting its source to a context-dependent body
denies with native `MissingAuthoritativeFact`, not an invented Unmet finding.
An ordinary stale context is denied as `SourceChanged` before execution; a change
after preparation stales publication. Guidance targets the query rather than an
individually failed child field.

## Anti-Patterns

Do not reconstruct a context from application DTOs, infer correlation by zipping
lists, coerce omission into `None`, erase units, or treat the record as authority.
Do not interpret a result-set membership token as whole-result value custody.

## Current Limits

Only exactly-one root one-shot results support context retention. Nested
collections are supported, but all projected scalar families must have the
lossless Foundation correspondence and no declared unit. Output names must be
valid expression record field names. Live reads, continuations and multi-root
value contexts are not provided by this surface. Contexts are process-local
source custody, not durable serialized mutation authority.
Types follow the actual native codec, not the Rust DTO: an application structure
encoded as a native String remains a String; this surface does not decode JSON
or invent a nested object schema for it.

Authored requirement operands currently support only parameter-free queries and
one context operand per requirement, with any additional ordinary scalar or
principal operands. All context operands on an operation must match its actual
source query and binding. Parameterized contexts need a future authored native
correspondence; do not supply a new host selector callback. Stable producer-cutoff
execution refuses a context-dependent body rather than discarding its required
context. Per-failing-row navigation and durable context recovery are not provided.
Portable operation-body v25 carries the full query and binding; v1–v24 keep their
meaning and refuse the new source tag when encountered under an older profile.

## Related Docs

- [Ordinary application front door](../foundations/ordinary-application-front-door.md)
- [Scope-field operation bodies](scope-field-operation-bodies.md)
