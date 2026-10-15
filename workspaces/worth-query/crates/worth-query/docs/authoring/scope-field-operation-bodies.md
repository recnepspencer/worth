# Executable Scope-Bound Operation Bodies

## What This Feature Is

A scope-field body declares the source expectations and scalar writes of one
operation. It can also declare one typed entity creation in the admitted scope's
context, without writing an existing scope field. Required writes and created-field initializers can use authored literals or named
members of the operation's once encoded input. A created field may also copy a
tracked scope-field preimage from the same decision observation. Optional scalar
fields can be set from a present scalar input, explicitly cleared, or replaced
and initialized from one-level Option-valued input. Query installs and executes those
values; the host does not choose a handler that independently restates them.
An operation may declare presence requirements or Boolean Foundation expressions
over typed scope fields and related-item counts. The same installed requirements
drive admission and authorized read-only guidance.
Fixed typed output roles may preserve the admitted scope or identify the body's
created row through Query's existing commit correspondence. A preservation-only
body needs no dummy business-field write.
The creation may also link to/from that admitted scope through typed declared
relations, so domain membership does not depend on a later host effect.
This remains a bounded body form; it does not dispatch workflow commands.

## Why You Use It

Use it for an operation that changes scalar fields on its admitted scope,
such as assigning a fixed status or a retention period supplied by the request.
Use a creation-only body to create a business entity under an existing admitted
scope, such as a tenant, without bootstrapping business records or inventing a
scope counter write.
Changing an authored literal or input member binding
changes canonical schema meaning and the bound program revision, even when the
read/write footprint is unchanged. Use other supported native owners for queries,
assessments, approval authority and external effects; this body does not replace
them or supply missing joins between them.

## Stable Entry Points

Declare through `worth_query_decl::facade::application_schema`:

- `Operation::reference().body().expect(...).assign(...).assign_input(...).finish()`;
- `body.in_scope(Tenant::reference())` when creation does not touch an existing field;
- `body.set_optional_input(optional_field, "member")` or
  `body.clear_optional(optional_field)` for distinct optional-value intentions;
- `body.assign_optional_input(optional_field, "member")` for complete replacement
  from a serialized Option: Some sets and None clears;
- `body.require_present(optional_field)` for an operation-specific precondition;
- `Operation::reference().requirement_operands()` and
  `ApplicationOperationBodyRequirement::admit(...)`, attached with
  `body.requirement(requirement)` for conditional requirements;
- `body.create_in_scope(Entity::reference(), "candidate-slot")` followed by
  `.initialize(...)`, `.initialize_input(...)`, or
  `.initialize_from_scope(source_field, created_field)` and `.finish()`;
- `creation.initialize_optional_input(optional_field, "member")` to initialize
  the new field for Some and leave it absent for None;
- `creation.link_from_scope(relation, "link-slot")` and
  `creation.link_to_scope(relation, "link-slot")` for links to the new row;
- `body.preserve_scope_output::<Binding, Role>()` and
  `creation.create_output::<Binding, Role>()` for fixed typed output correspondence;
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
For a creation-only body, `in_scope` names a declared entity with
`OperationReads<Operation>`; registration derives its Entity read. Installation
still requires this scope to match the binding's actual scope field. It is not
a caller-selected context or a permission grant. An explicit scope matching
existing scope fields normalizes to their original meaning; a mismatch denies.

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
The structured binding's validator runs at the shared input-identity boundary
before input serialization, for both ordinary and preencoded operation inputs.
Its Validation denial remains typed; a host-side precheck is not a substitute.

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

`assign_optional_input` requires a present serialized Option member. Some(value)
sets the scalar through its installed codec; None invokes native Clear.
`initialize_optional_input` uses the same selector on an optional created field:
Some initializes it; None leaves it absent. Required destinations cannot use
either form. Each None consumes its own prepared input slot, so it cannot shift
later initializers. Decoder/profile admission remains mandatory at installation
even when this particular input is None.

Missing members, duplicate selected members (including a duplicate after None),
nested Options and wrong scalar families deny before effects. Do not use
`skip_serializing_if` to omit a selected None: omission is not Clear. Omitting
the field's body step remains Unchanged. Complete optional replacement does not
implement a three-way sparse-patch contract.

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
writes. Creation is attached before the complete body is validated: a bare scope,
expectations, or requirements without a creation, scope assignment or output use cannot
finish. `initialize_from_scope` requires a required scalar source field in the
admitted scope with `OperationReads<Operation>`, a target with the same Rust value,
unit, exact installed codec and scalar family, and an admitted tracked decision
read. Query captures the source at the decision snapshot before any candidate
write, reserves its retained value on the request, and holds that reservation
through candidate initialization. Missing source or changed source dependency
denies/stales through the ordinary owners; the host cannot supply a before-image.

An output use names a role of the exact binding's `Output` contract. The compiler
checks operation/input, schema, contract, entity and posture, including the role's
declared inventory. Fresh readmission checks the complete binding/role tuple and
target slot. Installation rebinds contract identity from the typed binding and
entity identity from installed marker provenance; portable labels grant nothing.
Every required fixed role must be supplied. Optional fixed roles may remain absent;
mandatory role families are refused because this body form cannot produce members.
The existing candidate owner binds and validates the role, and Relational's actual
commit resolves a creation slot to its durable entity identity. No host manufactures
an output receipt. A preservation-only body observes scope Entity kind/liveness
through the decision-read owner even though it has no scalar reads or writes.

Creation links name typed relations with `OperationLinks<Operation>`. The created
endpoint is fixed by the creation builder; finishing and fresh readmission bind
the other endpoint to the body's admitted scope. The key is candidate-local, not
a durable relation identifier. Multiple edges of one relation keep their distinct
keys and effects while deriving one Link-membership target. Installation checks
the link ceiling and native key validity. The native relation-effect owner still
checks target/program provenance, kind, membership, key and resources; Relational
enforces the relation's declared integrity and commits row, fields and links
together. A stale candidate publishes none of them. Appending a creation link
does not require pretending an existing edge was observed or absent.

Publication uses ordinary currentness, authorization, invariant and commit
machinery. A changed dependency can stale an already prepared candidate. A failed
expectation returns `DomainDenied(SourceMismatch { entity, aspect, field })` before
effects. Replay still returns the native original-commit outcome rather than
running a now-failing source guard again. Deferred or unresolved publication keeps
its native recovery outcome; a body introduces no retry or receipt-copy shortcut.
After checkpoint restoration, `PreviouslyCommitted` can identify the original
commit without recreating its old live receipt; read current state through a
fresh governed query.
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
name, not a getter callback. The input shape must be a flat struct with scalar or
one-level optional-scalar members. Required selectors reject an Option wrapper;
use the optional selector for that member. Nested shapes still deny.

For an optional note, author `set_optional_input(DraftNoteField::reference(),
"note")` in the Set operation and `clear_optional(DraftNoteField::reference())`
in the Clear operation. A governed read returns `None` for absence and
`Some("")` for present empty text. An unrelated operation with no note step
preserves either value.

For complete replacement carried as `description: Option<String>`, keep one
operation rather than dispatching Set or Clear from the host:

```rust
let body = SaveDraft::reference().body()
    .assign_optional_input(Description::reference(), "description")?
    .finish()?;
```

The input binding retains canonical input. A source-bound save keeps its existing
`ApplicationQueryMutationSource` contract; native source currentness and exact
replay still apply. The host forwards the caller's Option unchanged.

For an operation that may run only after that note is selected, add
`require_present(DraftNoteField::reference())` to its body. This does not
retroactively require the note for the draft Set or Clear operations.

For an audit that records the old retention while assigning a new one, add
`.create_in_scope(Audit::reference(), "audit")` and
`.initialize_from_scope(DocumentRetentionField::reference(),
AuditPriorRetentionField::reference())` before the creation builder's `finish()`.
The audit receives the observed old value even if the same body assigns a new
retention to the document.

To create the document itself under an existing tenant, declare
`OperationReads<CreateDocument>` on the tenant marker and the binding's scope
identity field, plus Create/initializer Write capabilities on the document.
The authored operation then needs no mutation of the tenant:

```rust
let body = CreateDocument::reference().body()
    .in_scope(Tenant::reference())?
    .create_in_scope(Document::reference(), "document")?
    .initialize_input(DocumentIdentityField::reference(), "identity")?
    .initialize_input(DocumentRetentionField::reference(), "retention_days")?
    .initialize(DocumentAuditSourceField::reference(), 7_u64)?
    .finish()?;
```

Attach and install it through the same definition/program/contribution funnel
above. The binding admits one creation and three initializer writes, uses the
existing tenant identity as its scope, opts into canonical input capture, and
returns unit. Query resolves that
scope, checks its current liveness, and initializes all required document fields.
Use the request's business identity in a governed query to read the committed
document; the candidate-local `"document"` key is not its durable identifier.
This example does not establish tenant-isolation policy or a uniqueness rule
for the business identity; declare those contracts separately.
If the schema has a `TenantDocuments` relation from Tenant to Document, declare
its Link capability and add `.link_from_scope(TenantDocuments::reference(),
"tenant-document")?` before `finish()`. A reverse relation from Document to Tenant
uses `link_to_scope`. A governed query's root path can then traverse the committed
membership. Do not infer ownership merely from the context partition or a copied
tenant column, and do not equate membership with authorization. A capability
still needs its declared grant witness and the ordinary policy/currentness checks.
For an optional description on that new document, add
`.initialize_optional_input(Description::reference(), "description")` before
`finish()`. No dummy empty value needs to be initialized first.

To expose its actual committed identity, give `CreateDocumentBinding::Output` a
fixed role whose Entity is Document and Action is `WorthQueryCreateOutput`, then
add `.create_output::<CreateDocumentBinding, CreatedDocument>()?` to that creation
builder. On the real commit receipt, read
`receipt.outputs_of::<CreateOutputs>()?.entity::<CreatedDocument>()?`.
ExactlyOne yields the typed entity directly; AtMostOne yields Option. Neither a
business identity nor the symbolic creation slot substitutes for this reference.

A conditional assessment publisher can keep its pure producer kernel and native
source/settlement contract while declaring the effect itself:

```rust
let body = PublishAssessment::reference().body()
    .set_optional_input(DescriptionPresent::reference(), "description_present")?
    .preserve_scope_output::<PublishBinding, FindingOutput>()?
    .finish()?;
```

Here `FindingOutput` belongs to `PublishBinding::Output`, names its scope entity,
and has Action `WorthQueryPreserveOutput`. The producer computes the input, not
the effects. The body publishes through Query; the existing demand settlement
owns currentness and reuse. A stored finding or a preserved entity reference
alone is not settled/current assessment evidence. For correspondence alone,
use `.in_scope(Document::reference())?.preserve_scope_output::<Binding, Role>()?`
without a scalar assignment; do not manufacture a counter increment.

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
Creation-only bodies use scope-bearing portable body record v8. Valid v1–v7
carriage keeps its previous bytes; an older record cannot silently acquire the
new creation-only semantics.
Optional-input assignment and initialization use portable body record v9.
Older versions reject those opcodes rather than reinterpret their intent.
Fixed output uses select portable body record v10; valid no-output v1–v9 records
retain their previous bytes. Their meaning enters both schema and program identity.
Creation links select portable body record v11; valid no-link v1–v10 encodings
retain their previous bytes. Direction, endpoints, relation and key enter both
schema and program meaning, and fresh reconstruction keeps that exact linkage.

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
Set-from-flat-input, explicit Clear, one-level Option-valued replacement and
initialization, operation-specific presence requirements,
and Foundation Boolean requirements over scope scalar/presence and distinct
incoming/outgoing related-item count operands, plus
one in-context created entity with literal, flat-input, or tracked scope-preimage
initializers, links between that creation and its admitted scope, plus fixed
Preserve-scope/Create-slot output uses. It requires
at least one real creation, scope assignment or output use and unit results.
Output roles are binding-affine; a body cannot silently reinterpret one binding's
roles under another binding with matching names. Arbitrary target selection,
Retire and role-family member uses are unsupported. Optional literal assignments,
general input/source assignments, filtered collections, governed-query operands,
custom expression kernels, multiple creations, arbitrary existing-target relation
edits, signature consequences and command
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
