# Executable Scope-Bound Operation Bodies

## What This Feature Is

A scope-field body declares the source expectations and scalar writes of one
operation. It can also declare one typed entity creation in the admitted scope's
context, without writing an existing scope field. Required writes and created-field initializers can use authored literals or named
members of the operation's once encoded input. A created field may also copy a
tracked scope-field preimage from the same decision observation. An unsigned
creation may instead copy required or optional fields of one selected related
subject, without passing those values through caller input. An unsigned
audit field can also copy the exact admitted principal identity, with its source
derived from the operation's mutation binding rather than input. Optional scalar
fields can be set from a present scalar input, explicitly cleared, or replaced
and initialized from one-level Option-valued input. Query installs and executes those
values; the host does not choose a handler that independently restates them.
An operation may declare presence requirements or Boolean Foundation expressions
over typed scope fields, related-item counts and outgoing child-field lists. The same installed requirements
drive admission and authorized read-only guidance.
For correlated multi-field or nested-row requirements, use an exact sealed
[native query expression context](./native-query-expression-contexts.md) as a
typed operand, not a host-built validation record.
Fixed typed output roles may preserve the admitted scope, preserve its single
selected outgoing subject, or identify the body's created row through Query's
existing commit correspondence. A preservation-only
body needs no dummy business-field write.
The creation may also link to/from that admitted scope through typed declared
relations, so domain membership does not depend on a later host effect.
This remains a bounded body form; it does not dispatch workflow commands.
For source changes that must remove obsolete selections beneath persistent rows,
use [owned-selection retirement](./owned-selection-retirement.md), not a host cascade.
For a conditional typed external request using the exact operation input, use
[Authored Conditional External Effects](./conditional-body-effects.md). Its guard
suppresses the effect, not the business operation, and uses the ordinary outbox.
For one unsigned creation that must publish with its fresh native instance, use
the explicit [workflow-enrollment attachment](./workflow-enrolled-creations.md).
It does not replace a body with host-created workflow effects.
For an immutable authenticated snapshot rather than unsigned attribution, use
[Signed Operation Creations](./signed-operation-creations.md). Its signing handoff
does not turn ordinary information gathering into mandatory workflow approval.

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
- `body.expect_input(field, "member")` to require a present field equal to one
  captured flat scalar, without declaring a write;
- `body.expect_input_from_path(path, field, "member")` for a related field,
  preserving the path-start scope; see [path-input guards](./path-input-guards.md);
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
- `creation.initialize_from_related(relation, source_field, created_field)` and
  `creation.initialize_optional_from_related(relation, source_field, created_field)`
  to copy the single live outgoing subject's required or optional preimage;
- `creation.initialize_from_principal_identity::<Binding, _, _, _, _, _>(actor_field)`
  to attribute a created row to the operation's admitted principal;
- `creation.link_from_scope(relation, "link-slot")` and
  `creation.link_to_scope(relation, "link-slot")` for links to the new row;
- `creation.replace_from_scope(relation, "link-slot")` to atomically replace an
  at-most-one outgoing selection with that exact new row;
- `body.preserve_scope_output::<Binding, Role>()`,
  `body.preserve_related_output::<Binding, Role>(relation)` and
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

For input assignments and input expectations, the operation's structured input binding must opt in to
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

### Compare a field with captured input

Use `expect_input` when a mutation must compare committed graph truth with a
value already in its original command. It takes a typed field with
`OperationReads<Operation>`; it does not need or grant write permission. For
example, a source operation can guard the document's identity and then assign
a different input member:

```rust,ignore
// Illustrative fragment; the surrounding operation declares both reads and its write.
let body = SetRetention::reference().body()
    .expect_input(DocumentIdentityField::reference(), "identity")?
    .assign_input(DocumentRetentionField::reference(), "retention_days")?
    .finish()?;
```

Both required and optional stored fields are supported. The input member must
be a present scalar, not an Option. An absent stored field never matches;
present-empty, false and zero remain ordinary values. Missing, duplicate, nested,
wrong-family or codec-invalid input denies with `InputMismatch { member }`.
A different or absent graph value denies with `SourceMismatch { entity, aspect,
field }`. Neither publishes an assignment or creation. The source fact remains
a publication obligation, so another session changing only the compared field
can stale a prepared candidate.

A realistic use is selecting a provider lot whose captured company must equal
the NCR's committed company. Resolve and revision-admit the provider selection
through the installed external-input port, construct the original input with
that accepted company, then declare:

```rust,ignore
// Illustrative domain declaration, not a provider transaction or graph grant.
let body = SelectLot::reference().body()
    .expect_input(Company::reference(), "company")?
    .set_optional_input(SelectedLot::reference(), "lot")?
    .finish()?;
```

Retain the complete original input and logical key through the normal owning
action. Exact committed replay recovers its original result; it does not rerun
the guard against today's graph or fetch today's provider values. Native
authorization still applies. Provider freshness at admission is not atomic
with graph publication. Do not replace the guard with a host comparison, a
copied readiness Boolean or a filtered query created just to enforce equality.

This is a body execution guard, not a new expression operand or a requirement
assessment finding. Read-only requirement assessment does not report this
input comparison as Met/Unmet. Use declared requirements for guided readiness.
Canonical schema body v20 and portable body v30 encode this new meaning only
when present; older body encodings retain their existing meaning. Decoding or
inspecting a body never grants execution authority.

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

For a capability-owned binding, use the capability counterpart:

```rust,ignore
let requirements = request.mutate(intent)
    .assess_current_capability_body_requirements(program, projection)?;
```

It uses the same installed body evaluator after fresh encoded-input capability
admission for the principal, purpose, resource and current grant. The selected
branch/program/action and admission share one native observation. Conventional
assessment cannot bypass a required capability; it returns `CapabilityRequired`.
Revoked, mismatched or unavailable authorization returns its typed denial, not
an invented Unmet finding. Neither assessment method needs an idempotency key or
consumes workflow occurrence authority. A guarded operation may be inspected
before its frontier, but executing it still needs the exact native requirement.
Met describes only the authored body requirements—not stage eligibility, a signed
approval, or a mutation permit. Never cache it as authorization or auto-advance
from it; execution observes and admits afresh.

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

`initialize_from_related` uses the outgoing relation's typed target as the source
entity. Source and destination must share value type, unit, exact codec and scalar
family. The required form requires both fields to be Required; the optional form
requires both to be Optional. `None`, present-empty and present text remain
distinct. Zero selected subjects denies with native `MissingAuthoritativeFact`
naming the relation; multiple distinct subjects deny with `AmbiguousRelation`.
A missing subject is never an absent optional field. Relation endpoints, field
metadata and complete Entity/Relation/Field reads are rechecked during cold admission.

Query selects each distinct related source once per body decision, observes its
complete adjacency and endpoint liveness, and reserves captured source values on
the request. If `replace_from_scope` uses that relation too, copying and retiring
the predecessor share the exact retained native selection. Candidate construction
uses only retained values/absence, never a new read or host-supplied ID. Source or
selection changes after preparation stale the whole publication, including the
creation, history link and replacement. Exact logical-request replay recovers the
original outcome without copying or replacing again. This copies unsigned
business data, not signatures or approval authority. Combining related-source
initializers with `signed_by` is explicitly refused until related certified
footprints have their own native support.

`initialize_from_principal_identity` binds to the exact mutation operation/input,
principal binding, identity source field and codec. Its target is unitless and
uses that same identity value and codec. Registration derives the principal-field
read; cold readmission checks the declared contracts and native installation
checks their correspondence with the actual typed binding. Execution reads from
the principal entity retained by admission at the same decision observation—not
from a scope ID, input member, display name, or fresh host lookup. The existing
tracked scalar read and reservation retain the value through candidate creation.
This is an unsigned actor attribution. It does not retain a signing event,
establish freshness of authentication, or grant approval authority.

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

`preserve_related_output` requires a Preserve/ExactlyOne role whose Entity is the
typed relation's target. The source must be the binding's actual scope; both
entities and the relation need normal `OperationReads<Operation>` permission.
Registration derives those three installed reads. Cold admission checks them,
the endpoints and the exact binding/role contract independently. No live target
refuses with `MissingAuthoritativeFact`; multiple distinct live targets refuse
with `AmbiguousRelation`, both naming the relation. Neither produces a receipt.
The body seals the same tracked selection used by related-source initializers
and selection replacement. Candidate output binding uses that retained target,
not a fresh relation lookup. A racing replacement stales the whole candidate.
Exact committed replay returns the original correspondence even if the pointer
later changes. This reference identifies a committed output; it is not by itself
current workflow, signature or assessment authority.

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

For a current-subject pointer, use `replace_from_scope` rather than a host unlink
followed by a create/link request. Its relation requires declared
`OperationReads`, `OperationLinks` and `OperationUnlinks`, plus installed
`integrity.cardinality.source_max == Some(1)`. This bounds outgoing **edges**,
including duplicates—not merely distinct targets. An unbounded relation or a
zero limit cannot be used. Direction remains FromScope; replacement is a separate
`ReplaceSingle` posture, not an inverse link or a flag on the caller's payload.

Decision reads seal the complete outgoing adjacency, including empty selection,
and root/predecessor liveness. The decision retains the native predecessor
target. Candidate construction retires exactly the carried old edge IDs and
links the actual created reference through the existing relation-effect owner;
it does not reread, choose a first/latest row, or accept an old-subject ID.
No predecessor means initial selection. Ambiguous authoritative selection fails
closed. Installation requires room for one unlink per replacement instruction,
even when this particular invocation starts empty. Conflicting instructions for
the same selection relation are refused.

Keep ownership/history as a separate append link. Replacement does not delete
the previous subject, clear its fields or supersede its signatures. Any later
source edit still follows its own declared native supersession law. Creation,
history link, predecessor retirement and new selection share one guarded
publication: concurrent replacement stales a loser without publishing its new
subject or any link. Exact replay recovers the original outcome without replacing
again. A prepared consumer over the old selection also becomes stale.

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

### Unique business values

Declare uniqueness once with the schema builder's `unique_field(entity, field)`;
the field must be equality-queryable. This is independent of a creation's
symbolic slot, the logical request key and authorization. For example, declaring
`DocumentIdentityField` unique prevents two live Documents from holding the same
business identity, including creates under different admitted tenant scopes.
Use a different authored identity shape if business uniqueness is tenant-local.

The shared native handler tail observes each distinct assigned unique value
through its installed equality index at the candidate's retained snapshot.
Creation initializers, required assignments and present optional replacements
use that same owner; optional clears consume no value. These intrinsic write
dependencies are retained before signing/sealing, without requiring a supplier
lookup or granting access to the other holders. Previously observed predicates
are reused, and the ordinary native lowering law still refuses taken values or
two writes of the same unique value within one candidate. A concurrent claimant
cannot turn an observed absence into permission: publication rechecks the
retained predicate and uniqueness. No losing creation or membership link commits.
Unavailable indexes fail closed; uniqueness is never downgraded to a host precheck.

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

The ordinary caller declares the action's delivery and captures its intent and
stable logical key with `idempotency_owned`, then `retain_operation` for the exact
action. Advance with fresh request authority and retain every returned owner;
see the [owning front door](../foundations/ordinary-application-front-door.md#owning-fixed-and-source-only-actions).
The installed body supplies `30`, not a caller-provided destination. Normal policy
and invariants can still deny that write. A successful unit result is not a row
snapshot: read authoritative data through a fresh governed query. Deliberate
native preparation uses the explicitly advanced lane's `prepare_in_program`;
it is not the ordinary mutation/retry API and grants neither an unpublished
result nor a fresh retry permit.

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

For an actor-bearing transition audit, initialize the actor in that same body:

```rust
let body = EnterReview::reference().body()
    .expect(CurrentStage::reference(), Stage::Draft)?
    .assign(CurrentStage::reference(), Stage::Review)?
    .create_in_scope(TransitionAudit::reference(), "transition")?
    .initialize_from_scope(CurrentStage::reference(), AuditFrom::reference())?
    .initialize(AuditTo::reference(), Stage::Review)?
    .initialize_from_principal_identity::<EnterReviewBinding, _, _, _, _, _>(
        AuditActor::reference(),
    )?
    .link_from_scope(DraftAudits::reference(), "audit-owner")?
    .finish()?;
```

Here `AuditActor` has `OperationWrites<EnterReview>` and exactly the identity
value/codec of `EnterReviewBinding`. Admit the creation, link and four writes in
the binding's normal candidate ceiling. Query commits the old stage, new stage,
actor, and membership atomically. A governed audit query reads the persisted
actor independently of the unit mutation result. The intent contains no actor
selector. This is not a signed attestation; a workflow occurrence or signature
requirement still needs its own native authority.

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

For a new revision with preserved historical membership, combine the two link
intentions in the same creation:

```rust,ignore
let body = CreateRevision::reference().body()
    .in_scope(Document::reference())?
    .create_in_scope(Revision::reference(), "revision")?
    .initialize_input(RevisionDescription::reference(), "description")?
    .link_from_scope(AllRevisions::reference(), "history")?
    .replace_from_scope(CurrentRevision::reference(), "current")?
    .create_output::<CreateRevisionBinding, CreatedRevision>()?
    .finish()?;
```

Here CurrentRevision is Document→Revision with outgoing edge maximum one;
AllRevisions is the separate history relation. Initialize every required revision
field and admit one creation, two links, one unlink and the initializer writes
in the ordinary binding. The actual supplier example uses the same arrangement
for Tenant→Deviation selection, preserves both drafts in TenantDrafts, and signs
each genuinely through Query. The old signature cannot satisfy a related-evidence
requirement on the new selected subject.
This is not a complete revision/approval-round lifecycle: declare its own edit,
signing and business-open policy. Reopen and existing-subject workflow enrollment
need their own native joins; a current pointer is not that authority.

To seed that revision from the actual current one rather than values supplied by
the caller, replace its input description initializer with:

```rust,ignore
.initialize_optional_from_related(
    CurrentRevision::reference(),
    RevisionDescription::reference(),
    RevisionDescription::reference(),
)?
```

Here RevisionDescription is Optional String on Revision. A fuller round-copy
body can copy required assignment in the same selection:

```rust,ignore
let body = CopyRound::reference().body()
    .expect(CurrentStatus::reference(), Status::Active)?
    .create_in_scope(ApprovalRound::reference(), "round")?
    .initialize_input(RoundIdentity::reference(), "identity")?
    .initialize_optional_from_related(
        CurrentRound::reference(), RoundDescription::reference(), RoundDescription::reference(),
    )?
    .initialize_from_related(
        CurrentRound::reference(), RoundAssignee::reference(), RoundAssignee::reference(),
    )?
    .initialize_from_scope(PolicyRevision::reference(), RoundPolicyRevision::reference())?
    .link_from_scope(RoundHistory::reference(), "history")?
    .replace_from_scope(CurrentRound::reference(), "current")?
    .create_output::<CopyRoundBinding, CreatedRound>()?
    .finish()?;
```

Declare the corresponding typed Read/Create/Write/Link/Unlink capabilities and
ordinary candidate ceiling. The intent contains parent and fresh round identity
only. Query derives installed reads and publishes the new fields, history and
current pointer together. Historical signature/provenance remains unchanged;
the new subject has no evidence until it signs independently. The real runnable
deviation supplier exercises this path, source/selection races and retry after a
later current selection. The Active guard is intentional: this is not a Reopen
operation or fresh enrollment of an existing item.

To return only the selected round without changing it, declare `CheckedRound` as
a Preserve/ExactlyOne role in `CheckRoundBinding::Output`, with Entity
`ApprovalRound`, and author:

```rust
let body = CheckRound::reference().body()
    .in_scope(Deviation::reference())?
    .preserve_related_output::<CheckRoundBinding, CheckedRound>(CurrentRound::reference())?
    .finish()?;
```

To expose both the copied preimage and the new round, the copy operation's own
output contract declares `SourceRound` as Preserve/ExactlyOne and `CreatedRound`
as Create/ExactlyOne. Attach the first before creating, and the second to the
creation builder:

```rust
let body = CopyRound::reference().body()
    .in_scope(Deviation::reference())?
    .preserve_related_output::<CopyRoundBinding, SourceRound>(CurrentRound::reference())?
    .create_in_scope(ApprovalRound::reference(), "copied-round")?
    .initialize_input(RoundIdentity::reference(), "identity")?
    .initialize_optional_from_related(
        CurrentRound::reference(), RoundDescription::reference(), RoundDescription::reference(),
    )?
    .initialize_from_related(
        CurrentRound::reference(), RoundAssignee::reference(), RoundAssignee::reference(),
    )?
    .link_from_scope(RoundHistory::reference(), "history")?
    .replace_from_scope(CurrentRound::reference(), "current")?
    .create_output::<CopyRoundBinding, CreatedRound>()?
    .finish()?;
```

Keep any real domain guards and required initializers from the complete operation.
On commit, `receipt.outputs_of::<CopyRoundOutputs>()?.entity::<SourceRound>()?`
names the original round; `entity::<CreatedRound>()?` names the new one. Both
ExactlyOne roles return typed entities directly. Copying and returning the source
share one decision observation even though the candidate replaces CurrentRound.
Do not resolve the source again after commit: it now points at the created round.

To expose its actual committed identity, give `CreateDocumentBinding::Output` a
fixed role whose Entity is Document and Action is `WorthQueryCreateOutput`, then
add `.create_output::<CreateDocumentBinding, CreatedDocument>()?` to that creation
builder. On the real commit receipt, read
`receipt.outputs_of::<CreateOutputs>()?.entity::<CreatedDocument>()?`.
ExactlyOne yields the typed entity directly; AtMostOne yields Option. Neither a
business identity nor the symbolic creation slot substitutes for this reference.

When such a body performs a guarded workflow operation, Query also retains its
actual output correspondence on that native operation transition. This happens
in the same publication as the body and transition, not through another host
write. There is no extra supplier registration or caller-supplied output list.
For example, a closing body may Preserve its item and Create its audit row;
the retained entries name those actual entities, not today's selected round.
See [retained operation correspondence](../foundations/workflows.md#retained-operation-correspondence)
for the historical identity boundary and the remaining chain-affinity limit.

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

### Require values on every related item

Use `outgoing_field_values(name, relation, field)` on the same operand builder
when a count alone is insufficient. The field must belong to the relation's
typed target and both must declare `OperationReads<Operation>`. Fresh schema
admission also checks the exact field binding, scalar width, optionality,
unit-free contract and complete relation/entity/field read closure.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled. The runnable supplier uses this same call shape.
let operands = Submit::reference().requirement_operands()
    .outgoing_field_values("notes", Items::reference(), ItemNote::reference())?;
let requirement = ApplicationOperationBodyRequirement::admit(
    "complete-item-notes", operands, None,
    parser.parse("length(notes) >= 2 && notes.all(n, is_some(n) && length(unwrap(n)) > 0)")?,
    "notes",
)?;
// Attach to the ordinary body with .requirement(requirement), then install it.
```

Here `ItemNote` is optional text. Required fields produce plain list elements;
optional fields produce Option elements, including `None`. Membership is a
distinct-live-child set: duplicate edges do not add elements, dangling endpoints
are unavailable, and missing required or malformed values are typed denials,
not omitted elements or fabricated false. Empty membership produces an empty
list. Foundation's `all` over that list does not establish a collection minimum;
author the length requirement explicitly when the business requires one.

Query reads adjacency, child liveness and child fields on the same retained
snapshot. The existing tracked observations drive both guidance and actual
publication currentness. A later member or consumed-field change can make a
prepared attempt stale. A ready finding is descriptive, not permission to
bypass fresh admission or automatically retry a stale mutation.
These are admission dependencies, not automatically certified signature subjects.
Only the signed body's declared captures define its permanent subject footprint;
a field-list requirement does not add collection captures to an attestation.

The runnable deviation supplier checks its real tenant-to-draft collection
through two immutable installed actions with minimums one and two. Its common
entry/driver stays unchanged. One ready early-signed draft does not hide another
incomplete member; completing that member permits the check without requiring
its signature. A preserved scope output describes the actual checked tenant,
not an ongoing collection-validity grant.

Enumeration follows native identity order, not user/business ordering. This
single outgoing hop has no filters, nested paths, row records, or correlated
multi-field predicate contract. Findings target the collection and child field,
not individual failing rows. Do not zip independent field lists into a custom
host rule engine. Resource limits refuse rather than truncate the collection.
Body record v24 carries this new source; v1–v23 keep their existing meaning and
refuse its source tag.

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
Related Preserve outputs select portable body record v22, schema body meaning v14
and program body meaning v17. Earlier Scope/Create encodings remain unchanged.
Profiles older than v22 reject related-output tag 3 rather than reinterpreting it.
Creation links select portable body record v11; valid no-link v1–v10 encodings
retain their previous bytes. Direction, endpoints, relation and key enter both
schema and program meaning, and fresh reconstruction keeps that exact linkage.
Replacement posture selects body v19 and distinct canonical meaning. Earlier
Append-only records keep their bytes and decode as Append; an old profile cannot
silently acquire replacement semantics.
Principal-identity initializers select portable body record v12; valid bodies
without them retain their previous v1–v11 encoding. Their exact binding/source
tuple enters schema and program meaning. An older record cannot consume the new
opcode. Transported actor metadata remains descriptive, not principal authority.

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
Do not implement replacement with a second host commit, predict a created ID,
use a mutable generation field to erase old attestations, or reuse an unbounded
history relation as a current-subject pointer.
Do not read the selected round in a host and feed copied fields into an input
initializer: that discards the native preimage/selection join. Do not substitute
a signature row, policy revision or copied fields for approval or release identity.

## Current Limits

This form supports literal equality expectations, literal assignments and flat
input member assignments on one admitted scope's required scalar fields, optional
Set-from-flat-input, explicit Clear, one-level Option-valued replacement and
initialization, operation-specific presence requirements,
and Foundation Boolean requirements over scope scalar/presence, distinct
incoming/outgoing related-item count operands and outgoing child-field lists, plus
one in-context created entity with literal, flat-input, tracked scope-preimage or
single outgoing related-subject required/optional preimage
initializers or an exact unitless admitted-principal identity initializer,
links between that creation and its admitted scope, including at-most-one outgoing
selection replacement, plus fixed
Preserve-scope, Preserve-single-outgoing-subject and Create-slot output uses. It requires
at least one real creation, scope assignment, output use or
[whole-input or native-query-context external emission](./conditional-body-effects.md) and unit results.
Output roles are binding-affine; a body cannot silently reinterpret one binding's
roles under another binding with matching names. Arbitrary target selection,
Retire and role-family member uses are unsupported. Optional literal assignments,
general input/source assignments, filtered collections, governed-query operands,
custom expression kernels, multiple creations, arbitrary existing-target relation
edits, signature consequences and command
continuation are not implemented here.
Related outputs have no Optional, reverse or filtered selection form. They return
the preimage, not a post-write query result. Guarded operation settlement retains
that correspondence on the performed operation transition.
`require_predecessor_output::<Binding, Selected, Producer, Produced>()` on an
authored succession joins a captured Preserve/ExactlyOne selection to the latest
performed occurrence of the named producer's ExactlyOne output. See
[succession output affinity](../foundations/workflow-succession-output-affinity.md).
The supplier still owns the graph and business-current selection meaning.
Related copying has no filtered, multi-subject or reverse-selection form and
cannot initialize a required target from an optional source. Related certified
captures are refused, not reinterpreted as root-scope captures. Copying alone does
not start/succeed a native workflow instance or admit a retained release pin.
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
