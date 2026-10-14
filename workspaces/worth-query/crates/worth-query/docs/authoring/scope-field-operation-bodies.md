# Executable Scope-Field Operation Bodies

## What This Feature Is

A scope-field body declares the source expectations and scalar writes of one
operation. It can also declare one typed entity creation in the admitted scope's
context. Required writes and created-field initializers can use authored literals or named
members of the operation's once encoded input. A created field may also copy a
tracked scope-field preimage from the same decision observation. Optional scalar
fields can be set from a present input member or explicitly cleared. Query installs and executes those
values; the host does not choose a handler that independently restates them.
An operation may declare presence requirements or Boolean Foundation expressions
over typed scope fields and related-item counts. The same installed requirements
drive admission and authorized read-only guidance.
This remains a bounded body form; it does not dispatch workflow commands.

## Why You Use It

Use it for an operation that changes scalar fields on its admitted scope,
such as assigning a fixed status or a retention period supplied by the request.
Changing an authored literal or input member binding
changes canonical schema meaning and the bound program revision, even when the
read/write footprint is unchanged. Use other supported native owners for queries,
assessments, approval authority and external effects; this body does not replace
them or supply missing joins between them.

## Stable Entry Points

Declare through `worth_query_decl::facade::application_schema`:

- `Operation::reference().body().expect(...).assign(...).assign_input(...).finish()`;
- `body.set_optional_input(optional_field, "member")` or
  `body.clear_optional(optional_field)` for distinct optional-value intentions;
- `body.require_present(optional_field)` for an operation-specific precondition;
- `Operation::reference().requirement_operands()` and
  `ApplicationOperationBodyRequirement::admit(...)`, attached with
  `body.requirement(requirement)` for conditional requirements;
- `body.create_in_scope(Entity::reference(), "candidate-slot")` followed by
  `.initialize(...)`, `.initialize_input(...)`, or
  `.initialize_from_scope(source_field, created_field)` and `.finish()`;
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

A requirement owns its stable name, typed operands, optional applicability draft,
Boolean satisfaction draft, and guidance target. The target names a declared
operand, so guidance refers to the same native field or collection used by the
rule. Foundation owns expression typing, compilation and semantic identity;
Query binds that identity and the source selectors into schema/program meaning.
Callers do not supply an independently evaluated context or a `supplied` Boolean.

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

An optional Set requires a present flat scalar input member. `""` and zero remain
present values; they are not Clear. An authored `clear_optional` is a distinct
body step. Omitting that field from a different operation means Unchanged. Both
steps use Query's existing optional-field candidate and lowering owner; field
value/absence and writable aspect presence are tracked at the decision snapshot.
An absent aspect can be initialized without rewriting sibling fields. A direct
request still receives ordinary authorization, currentness, and invariant checks.

`require_present` is an operation precondition, not a global field requirement:
draft-save operations without it may persist an incomplete item. Its native
observation tests presence, so `Some("")`, `Some(0)`, and `Some(false)` are met;
absence is unmet. A missing field yields
`DomainDenied(RequirementUnmet { entity, aspect, field })` before any effect.
The authorized read-only
`request.mutate(intent).assess_current_body_requirements(program, projection)`
uses the same installed body and reports Met or Unmet for each declared presence
step. Native projection or authorization failure remains a denial, not Unmet.
The assessment owns its finding bytes until it is dropped and carries no mutation
permit or retained snapshot. A later
submission observes afresh, and a Clear after preparation stales the candidate.

For an expression requirement, Query evaluates applicability first. False reports
NotApplicable without reading satisfaction-only operands. Otherwise the Boolean
satisfaction result reports Met or Unmet. An unmet named rule denies submission
with `DeclaredRequirementUnmet { identity }`. Expression, authorization, read,
resource and request-stop failures remain typed denials; they are never invented
findings. The assessment entry above handles both requirement forms and returns
only descriptive findings.

`field(name, field)` exposes the canonical scalar with its declared optionality;
`presence(name, field)` exposes only presence. For example, a present false value
is present, but its value remains false. `outgoing_item_count` and
`incoming_item_count` count distinct live related endpoint entities. Parallel
edges do not inflate a minimum. A dangling endpoint makes the observation
unavailable. Query seals adjacency plus anchor/endpoint liveness, so later
membership or lifecycle changes cannot silently reuse an earlier successful
requirement. Fields, relations and entity reads are derived from these bindings.

Native expression compilation occurs during authoring/readmission, not at each
invocation. Evaluation borrows the phase's request owner, admits memory before
materializing values, narrows expression work to the request's remaining budget,
and steps with cancellation checkpoints and native cost accounting.

The optional creation recipe uses an entity marker with
`OperationCreates<Operation>` and field markers with `OperationWrites<Operation>`.
Initialization may set read-only fields because those fields are being created,
not edited later. The fixed key names a candidate-local symbolic creation slot;
it is neither a business identity nor an idempotency key. Query creates the row
in the existing admitted scope's context, checks that all required created
fields are initialized, and charges creation and initializers to the same
candidate. An operation's candidate ceiling must admit that creation and its
writes. `initialize_from_scope` requires a required scalar source field in the
admitted scope with `OperationReads<Operation>`, a target with the same Rust value,
unit, exact installed codec and scalar family, and an admitted tracked decision
read. Query captures the source at the decision snapshot before any candidate
write, reserves its retained value on the request, and holds that reservation
through candidate initialization. Missing source or changed source dependency
denies/stales through the ordinary owners; the host cannot supply a before-image.

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

For an optional note, author `set_optional_input(DraftNoteField::reference(),
"note")` in the Set operation and `clear_optional(DraftNoteField::reference())`
in the Clear operation. A governed read returns `None` for absence and
`Some("")` for present empty text. An unrelated operation with no note step
preserves either value.

For an operation that may run only after that note is selected, add
`require_present(DraftNoteField::reference())` to its body. This does not
retroactively require the note for the draft Set or Clear operations.

For an audit that records the old retention while assigning a new one, add
`.create_in_scope(Audit::reference(), "audit")` and
`.initialize_from_scope(DocumentRetentionField::reference(),
AuditPriorRetentionField::reference())` before the creation builder's `finish()`.
The audit receives the observed old value even if the same body assigns a new
retention to the document.

For a conditional collection minimum, assume an optional Boolean
`ReviewEnabledField` and a `Validators` relation from the admitted document.
Declare their normal `OperationReads` capabilities, then author:

```rust
use worth_foundational::expression_api::expressions;

let parser = expressions()
    .reading_within(ApplicationOperationBodyRequirementExpression::PROFILE);
let operands = SetRetention::reference().requirement_operands()
    .field("review", ReviewEnabledField::reference())?
    .outgoing_item_count("validators", Validators::reference())?;
let requirement = ApplicationOperationBodyRequirement::admit(
    "validators-minimum",
    operands,
    Some(parser.parse("review == some(true)")?),
    parser.parse("validators >= exact_cast<UInt64>(1)")?,
    "validators",
)?;
let body = SetRetention::reference().body()
    .requirement(requirement)?
    .assign(DocumentRetentionField::reference(), 30_u64)?
    .finish()?;
```

Propagate these errors as `ApplicationOperationBodyAuthoringDenial`. Counts are
UInt64; the explicit cast uses Foundation's numeric rules. An absent or false
review flag makes this authored rule NotApplicable. When it is true, the same
body supplies guidance and refuses the write until one distinct validator exists.
The host does not recheck the count or choose a default rule result.

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
input member assignments on one admitted scope's required scalar fields, optional
Set-from-flat-input, explicit Clear, operation-specific presence requirements,
and Foundation Boolean requirements over scope scalar/presence and distinct
incoming/outgoing related-item count operands, plus
one in-context created entity with literal, flat-input, or tracked scope-preimage
initializers. It requires
at least one scope assignment and unit results. Optional literal assignments,
general input/source assignments, filtered collections, governed-query operands,
custom expression kernels, multiple creations, links, signature consequences and command
continuation are not implemented here.
Value operands support unitless Boolean, fixed-width integer, finite float,
decimal and string families. Unsupported scalar interpretations and unit-bearing
value operands are refused; presence alone does not interpret a value's family
or physical unit. Pending output work is not synthesized from an absent field.
`ApplicationOperationBodyScopeField` describes only this field family; it is not
a universal effect instruction or authority carrier.

## Related Docs

- [Ordinary Application Front Door](../foundations/ordinary-application-front-door.md)
- [Programs and Adoption](../foundations/programs-and-adoption.md)
- [Workflows](../foundations/workflows.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
