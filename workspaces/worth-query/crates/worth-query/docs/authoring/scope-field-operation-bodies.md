# Executable Scope-Field Operation Bodies

## What This Feature Is

A scope-field body declares the actual source expectations and literal writes of
one operation. Query installs and executes those values; the host does not choose
a handler that independently restates them. This is currently a deliberately
bounded body form, not a general expression, creation or workflow-command API.

## Why You Use It

Use it for an operation that changes required scalar fields on its admitted scope,
such as assigning a fixed status or retention period. Changing an authored literal
changes canonical schema meaning and the bound program revision, even when the
read/write footprint is unchanged. Use other supported native owners for queries,
assessments, approval authority and external effects; this body does not replace
them or supply missing joins between them.

## Stable Entry Points

Declare through `worth_query_decl::facade::application_schema`:

- `Operation::reference().body().expect(...).assign(...).finish()`;
- a completed operation definition's `with_body(body)`;
- `ApplicationProgramAuthoring::begin().with_schema(&schema).validated_program()`.

Install through `WorthQueryApplicationContributionSetup::operation_body::<Binding>()`
from `worth_query_host::facade::application_contribution`. The binding uses
`WorthQueryOperationBodyDecision<Schema, Scope>`, `WorthQueryOperationBodyDenial`
and result `()`, with its normal input, result/denial codecs, principal, scope,
idempotency and resource contracts. These decision/denial types are available
through `worth_query_host::facade::primary_graph`.

## Core Mental Model

The completed schema owns the body. Body construction validates literals through
their fields' declared codecs. Registration derives the body's installed field
reads and writes; do not repeat these with `operation_read_field` or
`operation_write`. Normal type-level capability declarations and binding scope
resolution still apply. A scope identity lookup may need its own declared read.

`with_schema` captures this meaning in each matching program action before its
revision is minted. Pass that same completed schema to installation. An unbound
program cannot acquire a body merely because its binding label matches.
Installation and reconstruction retain exact schema/program affinity; unsupported
or missing support is a denial, never permission to resolve an ambient latest body.
Every declared body needs a native executable binding, even if no program action
references it. A body without that executor cannot install as metadata.

## How It Executes

After normal request authorization and scope resolution, Query reads the body's
fields at the native decision observation. All expectations concern that source
state, not the result of earlier assignments. Write targets also retain preimage
dependencies. Query compares source expectations without copying stored carriers,
reserves candidate resources, and charges retained representation before cloning
literal values into its existing candidate owner.

Publication uses ordinary currentness, authorization, invariant and commit
machinery. A changed dependency can stale an already prepared candidate. A failed
expectation returns `DomainDenied(SourceMismatch { entity, aspect, field })` before
effects. Replay still returns the native original-commit outcome rather than
running a now-failing source guard again. Deferred or unresolved publication keeps
its native recovery outcome; a body introduces no retry or receipt-copy shortcut.

## Small Example

With the application's normal typed field and operation declarations in scope:

```rust
let body = SetRetention::reference()
    .body()
    .assign(DocumentRetentionField::reference(), 30_u64)?
    .finish()?;

let operation = SetRetention::reference()
    .definition()
    .no_external_effect()
    .no_aftermath()
    .finish()
    .with_body(body)?;
```

Register `operation` in the schema alongside the binding and its normal scope,
principal and resource declarations. `finish` rejects an empty body, multiple
scope entities and duplicate assignments. Attaching a second body is refused.

## Real Example

To change retention only from the reviewed source value, author:

```rust
let body = SetRetention::reference().body()
    .expect(DocumentRetentionField::reference(), 7_u64)?
    .assign(DocumentRetentionField::reference(), 30_u64)?
    .finish()?;
```

Attach it as above, then bind the completed declaration once:

```rust
let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
    .with_schema(&schema)
    .validated_program()?;

// In the contribution's configure method: no handler or destination argument.
setup.operation_body::<SetRetentionBinding>()?;
```

The ordinary caller sends its intent and stable logical key through
`execute_in_program`. The installed body supplies `30`, not a caller-provided
destination. Normal policy and invariants can still deny that write. A successful
unit result is not a row snapshot: read authoritative data through a fresh
governed query. `prepare_in_program` can retain the actual candidate before
publication; it grants neither an unpublished result nor a fresh retry permit.

## How It Relates To Other Features

Program manifests and semantic descriptions include body values. Portable schema
and whole-operation records retain them, with versioned body carriage; decoded
records remain authority-free until fresh native readmission. Old body-free
record encodings are unchanged. Product branches and pinned programs keep their
existing native owners. Checkpoint restoration is not a claim of Store-backed
process-crash durability.

Workflow occurrence authority and approval evidence remain separate. A source
expectation is not an approval, signature or general evaluated requirement, and
this API does not dispatch a second workflow operation after the first commit.

## Inspection And Debugging

Inspect the program's normalized manifest and semantic description to see the
authored body. Use native installation denials for unsupported bindings, ordinary
mutation outcomes for guard/currentness/replay results, and governed queries for
committed values. Missing execution fails installation; attempts to install a
replacement handler fail with `OperationBodyReplacement`, including through
bootstrap registration. Neither diagnostic records nor a copied receipt authorize
execution.

An advanced caller-authored effect program for a body-backed operation is refused
at the shared commit boundary with `OperationBodyExecutionRequired`, even if its
types, scope, authorization and write footprint are valid. Use the installed
native body entry; ordinary admission is not permission to replace its values.

## Anti-Patterns

Do not install a host handler beside a body, restate its destinations in React,
use handler identity strings as executable meaning, or omit `with_schema` and
expect installation to fill in missing program semantics. Do not turn an
unsupported audit, input expression or signature consequence into a hidden host
callback while claiming the body owns it. Advanced handler-backed operations
remain a different supported surface, not an equivalent declarative body.

## Current Limits

This form supports literal equality expectations and literal assignments on one
admitted scope's required scalar fields. It requires at least one assignment and
unit results. Optional-field writes, clear operations, input/source expressions,
entity creation, links, audit construction, body-bound requirement guidance,
signature consequences and command continuation are not implemented here.
`ApplicationOperationBodyScopeField` describes only this field family; it is not
a universal effect instruction or authority carrier.

## Related Docs

- [Ordinary Application Front Door](../foundations/ordinary-application-front-door.md)
- [Programs and Adoption](../foundations/programs-and-adoption.md)
- [Workflows](../foundations/workflows.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
