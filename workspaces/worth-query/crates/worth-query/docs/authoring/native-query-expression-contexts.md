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

## How It Relates To Other Features

`expect_source(source)` still checks the source's exact installed query, scope,
parameters when required by the binding, and native dependencies. Field or
membership changes can stale the later mutation. The retained value itself
remains immutable. A new read is new source intent, not an automatic retry of
an ambiguous old mutation.

**Current boundary:** retaining a context does not make an expression a native
operation requirement. Authored context operands and transfer of their values
into requirement admission are not yet implemented. Do not add a host Boolean
or validation loop as a substitute. Mutation authority remains independently
required, and retained data is not a fresh query authorization grant.

## Inspection And Debugging

Use `expression_context()` to inspect the exact schema and value. `None` means
the read did not request retention. `ExpressionContextUnavailable` means the
installed shape cannot provide a supported context. `ExpressionContextIncomplete`
means this read was incomplete or disclosure withheld data; it is not cached as
a defect of the shared source row, and recovery requires fresh admission.
Neither outcome permits using partial values. Typed resource/interruption
outcomes remain distinct. Result-buffer observation
includes the retained context backing, not just its logical scalar payload.

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

## Related Docs

- [Ordinary application front door](../foundations/ordinary-application-front-door.md)
- [Scope-field operation bodies](scope-field-operation-bodies.md)
