# Executable Scope-Field Operation Bodies

## What This Feature Is

A scope-field body declares the source expectations and scalar writes of one
operation. It can also declare one typed entity creation in the admitted scope's
context. Writes and created-field initializers can use authored literals or named
members of the operation's once encoded input. Query installs and executes those
values; the host does not choose a handler that independently restates them.
This remains a bounded body form, not a general expression or workflow-command API.

## Why You Use It

Use it for an operation that changes required scalar fields on its admitted scope,
such as assigning a fixed status or a retention period supplied by the request.
Changing an authored literal or input member binding
changes canonical schema meaning and the bound program revision, even when the
read/write footprint is unchanged. Use other supported native owners for queries,
assessments, approval authority and external effects; this body does not replace
them or supply missing joins between them.

## Stable Entry Points

Declare through `worth_query_decl::facade::application_schema`:

- `Operation::reference().body().expect(...).assign(...).assign_input(...).finish()`;
- `body.create_in_scope(Entity::reference(), "candidate-slot")` followed by
  `.initialize(...)` or `.initialize_input(...)` and `.finish()`;
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
their fields' declared codecs and captures an input assignment's exact target
codec and scalar family. Registration derives the body's installed field
reads, writes and creation membership; do not repeat these with
`operation_read_field`, `operation_write` or `operation_create`. Normal type-level capability declarations and binding scope
resolution still apply. A scope identity lookup may need its own declared read.

`with_schema` captures this meaning in each matching program action before its
revision is minted. Pass that same completed schema to installation. An unbound
program cannot acquire a body merely because its binding label matches.
Installation and reconstruction retain exact schema/program affinity; unsupported
or missing support is a denial, never permission to resolve an ambient latest body.
Every declared body needs a native executable binding, even if no program action
references it. A body without that executor cannot install as metadata.

## How It Executes

For input assignments, the operation's structured input binding must opt in to
`RETAIN_CANONICAL_INPUT`. Its scalar destination binding must have a readable codec
and an explicit `NATIVE_INPUT_ADMISSION` decoder resource profile. The built-in
primitive scalar bindings supply profiles. Custom codecs default to unavailable;
an opt in declares conservative base and per-byte bounds for decoder and
validator work and peak memory, including the boxed result. Query calculates
these bounds from the native scalar's owned capacity with checked arithmetic,
before it calls supplier code. Installation refuses an input body without both
contracts. The supplier remains responsible for the accuracy of a custom
decoder's declared profile.

After normal request authorization and scope resolution, Query reads the body's
fields at the native decision observation. All expectations concern that source
state, not the result of earlier assignments. Write targets also retain preimage
dependencies. Query compares source expectations without copying stored carriers,
reserves candidate resources, and charges retained representation before cloning
literal values into its existing candidate owner. For input assignments, Query
captures the final canonical byte stream during the one ordinary input identity
encoding pass. The capture survives preencoded, cloned and owned identity
handoffs. A native selector reads a named scalar member from a flat serialized
struct; it scans the complete record and refuses a duplicate selected member,
missing member, nested value or
wrong-family members. This selector does not reinterpret JSON or serialize the
input again. It reserves selected values and the installed decoder's declared
peak allocation on the same request, charges declared work, validates through
the exact installed codec, then moves the prepared values into the candidate.

The optional creation recipe uses an entity marker with
`OperationCreates<Operation>` and field markers with `OperationWrites<Operation>`.
Initialization may set read-only fields because those fields are being created,
not edited later. The fixed key names a candidate-local symbolic creation slot;
it is neither a business identity nor an idempotency key. Query creates the row
in the existing admitted scope's context, checks that all required created
fields are initialized, and charges creation and initializers to the same
candidate. An operation's candidate ceiling must admit that creation and its
writes. Source before-images are not available to created fields yet.

Publication uses ordinary currentness, authorization, invariant and commit
machinery. A changed dependency can stale an already prepared candidate. A failed
expectation returns `DomainDenied(SourceMismatch { entity, aspect, field })` before
effects. Replay still returns the native original-commit outcome rather than
running a now-failing source guard again. Deferred or unresolved publication keeps
its native recovery outcome; a body introduces no retry or receipt-copy shortcut.
An invalid input member returns `DomainDenied(InputMismatch { member })` before
effects. Work or memory refusal remains an execution denial.

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

For a caller supplied scalar, use `assign_input(DocumentRetentionField::reference(),
"retention_days")` in place of `assign(...)`, and opt the operation's structured
input binding into canonical capture. This is a binding to the serialized member
name, not a getter callback. The input shape must be a flat struct with scalar
members; a nested document or an optional member currently denies at selection.

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
unsupported before-image, expression or signature consequence into a hidden host
callback while claiming the body owns it. Advanced handler-backed operations
remain a different supported surface, not an equivalent declarative body.

## Current Limits

This form supports literal equality expectations, literal assignments and flat
input member assignments on one admitted scope's required scalar fields, plus
one in-context created entity with literal or flat-input initializers. It requires
at least one scope assignment and unit results. Optional-field writes, clear
operations, input/source expressions, multiple creations, links, source-derived
audit fields, body-bound requirement guidance, signature consequences and command
continuation are not implemented here.
`ApplicationOperationBodyScopeField` describes only this field family; it is not
a universal effect instruction or authority carrier.

## Related Docs

- [Ordinary Application Front Door](../foundations/ordinary-application-front-door.md)
- [Programs and Adoption](../foundations/programs-and-adoption.md)
- [Workflows](../foundations/workflows.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
