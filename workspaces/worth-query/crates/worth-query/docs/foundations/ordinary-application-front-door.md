# Ordinary Application Front Door

## What This Feature Is

The ordinary application front door is the supported way for an application
to declare Query meaning, install it in a host runtime, admit a real request,
execute it, and handle the typed outcome. Use it when application code needs
Query behavior without importing Query implementation crates or lower-runtime
plumbing.

## Why You Use It

- define typed queries, operations, policies, and schema members once;
- adapt authenticated identities without treating authentication as permission;
- apply current capability, purpose, disclosure, and conflict rules;
- execute reads and mutations through the installed provider session;
- recover honestly after response loss or an indeterminate commit;
- bind conditional providers and named clocks without owning Signal scheduling;
- request bounded produced outputs and consume their exact retained occurrence;
- perform source-bound edits and current, exact, or live reads through one request;
- keep transport code descriptive rather than authoritative.

## Stable Entry Points

Application declarations import `worth_query_decl::facade`:

<!-- doctest: bank-estate-certification -->
```rust
use worth_query_decl::facade::{
    application_capability,
    application_query,
    application_schema,
};
```

Application hosts import `worth_query_host::facade`:

<!-- doctest: bank-estate-certification -->
```rust
use worth_query_host::facade::{
    admission,
    domain,
    primary_graph,
    publication,
    runtime,
};
```

Certification code may separately import `worth_query_replay::facade`. Replay
is not an application or host entry point.

Most applications should expose domain-native methods over these facades. A
bank may offer `account_summary(...)` or `send_money(...)`; those names improve
the product API but do not become another authorization or execution owner.

## Core Mental Model

Declarations describe what an application query or operation means.
Installation binds that meaning to one runtime, schema generation, provider
contract, and lower-runtime correspondence. A request then supplies fresh
identity and controls. Query admits the request against current authority and
only then lets the installed provider execute.

The object passed to the next step carries the proof earned by the previous
step. A copied identifier, receipt, diagnostic, cursor, or wire token describes
work but cannot recreate that proof.

For required scope-field literal writes, an
[executable operation body](../authoring/scope-field-operation-bodies.md) owns
source guards and assigned values. Bind it into the program and install the
native body rather than supplying a replacement handler. Its current limits are
explicit; it is not general command composition or a signature implementation.

The main owners remain separate:

- the application domain owns business vocabulary and intent;
- Relational owns current graph facts and commits;
- Runtime Bridge owns installed correspondence;
- Signal owns policy decisions and temporal wake eligibility;
- Query owns application installation, admission, progression, and publication;
- the host supplies adapters and resources but does not reinterpret decisions.

### Keep the original branch across a host default switch

A fixed owner exposes its original issued selector through `branch()`. If the host
changes its selected default branch, create fresh ordinary context with
`application.request(&principal, &scope).on_branch(owner.branch())` before calling
`advance`. After settlement, select `settled.branch()` for `read_current` and
`close`. The selector carries identity, not permission; each call still admits
fresh authority. Query's installed root from `current_world()` remains immutable.

Context on another branch returns typed `FreshRequestMismatch` and preserves the
owner without source effects, producer work or delivery queries on that branch.
The [executable branch court](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/fixed_branch_affinity.rs)
proves refusal and fresh original-branch continuation. The
[two-instance court](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/fixed_action_instances.rs)
uses one binding at two canonical instances with distinct installed delivery
contracts; a caller cannot substitute one instance's delivery at the other.

## Complete action meaning

Begin one ordinary action with `.action::<Binding, Delivery>()` on its feature
parent. Compose `.locality_and_change::<Scope, Shape>()`,
`.correspondence::<Correspondence>()`, `.evaluated_requirement::<Rule>()` and
`.external_input::<Provider>()` as applicable. Each optional clause can be
attached only once; `.finish_action()` appends the complete declaration and
returns the feature parent. The macro's `action(Binding, Delivery) { ... };`
block delegates to those same typed transitions. Empty blocks are valid, and
clause order does not alter canonical meaning.

Delivery/locality/change belong to the exact occurrence. Correspondence lookup
is binding-scoped; installed rule/provider lookup is scoped to operation plus
rule/provider type across declared actions. These clauses are not a new
occurrence-isolated policy engine. The installed evaluator supplies guidance and
submission enforcement; it is not an unbypassable native authorization gate.
An external revision must still be resolved and admitted, and its accepted
values must enter the actual source action.

`ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery>`
implements the sealed `ApplicationAction` selection contract. `selector()`
contains occurrence/binding/delivery facts and cannot manufacture a complete
`ApplicationActionDeclaration`. Installation returns the complete actual
action, and fresh progression compares every retained contract against current
installed meaning. Duplicate TypeIds and portable occurrence/feature/binding
keys are validation refusals; separate legal instances remain distinct.

Deliberately specialized native work uses `.native_action::<Binding>()`, the
same applicable clauses and `.finish_action()`. It has neither delivery nor
an output-source toggle: native required-output membership comes from actual
roots. An ordinary mutation or retry needs genuine delivery even if it was
previously composed manually. See [Feature Capsule Authoring](../authoring/feature-capsule-authoring.md).

## Declared output delivery shapes

`ApplicationOutputProjection` is the closed vocabulary for an installed output
projection: `ApplicationNoProjection`, `ApplicationQueryProjection<Read>`, or
`ApplicationQueryBatchProjection<Read, N>`. The batch has one query binding and
an exact ordered selector count; an empty batch is rejected during authoring.
Fixed delivery keeps its existing canonical encoding.

A discovered declaration uses `ApplicationDiscoveredActionDelivery` and
`ApplicationDiscoveredDeliveryMode<Root, Projection>`, with a genuine
`ApplicationDiscoveredOutputRoot` from the program's actual output inventory.
Attach it using `.action::<Binding, Delivery>() ... .finish_action()` at the same composition
instance and source feature as that root. `ApplicationDiscoveredQueryDelivery`
and `ApplicationDiscoveredBatchDelivery` bind selectors that borrow the original
mutation input. Installation resolves the exact action occurrence and concrete
delivery implementation; reusing its string identity does not replace its
selector. Runtime type stamps are excluded from canonical meaning.

The [nested Planar declaration](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_program/discovered_delivery.rs)
shows a query, two-selector batch and explicit no-projection declaration against
one real discovered root. Declaration and installation establish meaning, not
permission. The owning entry below composes that meaning with native admission,
original-carrier recovery, traversal and complete current delivery.

## Owning fixed and source-only actions

Install a program and declare the exact typed action occurrence and its delivery.
`request.mutate(intent)` returns `WorthQueryOperationBuilder`; it exposes original
source preparation, preconditions and `idempotency_owned(key)`, followed by
`retain_operation::<Program, Action>(&runtime)`. Fixed, discovered and source-only callers
cannot execute, prepare or recover through that builder or its borrowed-key view.
The selected action carries composition instance, feature, binding and delivery;
a binding label alone cannot select its contract.

Capture the original meaning once. Supply a fresh authenticated ordinary request
for every `advance`, delivery and close. Native admission checks current authority
and the original runtime/branch/program affinity. The descriptor is not a grant.
Recovery promotes the original native carrier; registered replay rejoins its
original outputs without running the source handler. Recovered evidence does not
manufacture the original handler result.

Before disclosing a retained SourceOnly command's status, call
`owner.assess_current_authorization(&fresh_request)`. Active, completed and
terminal SourceOnly owners expose this borrowed check. Query selects the exact
retained runtime, branch and installed action, then uses the binding's declared
conventional or capability authorization with the original captured input,
preconditions and workflow actor context. A foreign runtime or current denial
returns the native typed refusal without consuming the owner. This check does
not stage a candidate, claim an idempotency key, advance recovery, spend a
workflow grant or repeat a completed mutation. Its successful result is only a
current authorization observation: it is neither execution admission nor a
promise that a later advance will succeed. Authentication or matching a cached
command-owner identity cannot substitute for this check.

At the Fixed source-preparation boundary, `prepared_source_receipt()` borrows the
actual original action receipt. Capture any needed descriptive runtime/scope,
installed binding, product incarnation and publication generation before the next
`advance` starts outputs; the borrow must end before moving that same owner.
`None` means the receipt is unavailable in the current phase, not that no
publication occurred. Descriptions grant no currentness, selection, retry or
publication authority. A settled output's `root_receipt()` belongs to its producer
and cannot identify the original source action. Observe semantic membership as a
multiset, preserving multiplicity and rejecting ambiguous projected identities;
no raw settlement slot is needed. Continue and close the original native owners.

A fixed settled owner borrows its complete settlement for `read_current` or a
finite `read_current_with_policy`; the currentness check and declared query share
one observation and one native phase. Read refusal retains the owner. Stale
settlement supports `read_historical` and `close`, never output advancement.
Current and historical reads return the same published result type. The selected
method and `result.receipt().inspect().basis()` distinguish their observation
basis; a historical result is not evidence of currentness.
No-projection delivery uses `deliver_current` without a dummy query. Source-only
completion is distinct from required-output settlement; retained delivery selects
the action's authentic original commit observation.

Completion and delivery retain independent cleanup. `close` returns the complete
owner with a typed denial when cleanup refuses. Drop is abandonment, not external
cleanup or crash durability. Only existing native checkpoint/recovery contracts
survive process loss.

The [compiled original descriptor capture](../../../../../../docs/build-an-application.md#capture-complete-original-meaning) is exercised by the real
public result-set court.

The real result-set court calls the compiled capture with nondefault preconditions before
an actual post-effect append failure. It reads the resulting source and outputs
independently after native recovery. No original intent clone is required per pump.
Its false-precondition twin also calls that same compiled capture. Omitting the
supplied preconditions would permit an effect and fail the twin's native refusal
and unchanged-product assertions. Advanced recovery courts separately refuse
omitted conditions and substituted row/result-set identity while preserving the
actual unpublished owner; they do not manufacture recovered handler results.

## Retained runtime and configuration custody

Fixed, discovered and source-only operations retain the authentic installed
runtime allocation and complete installed action. Fixed/discovered pending,
refused, settled and terminal types have no construction-runtime lifetime.
You can store the returned owner after the construction borrow ends and drop
the constructing handle. This shares the same runtime allocation; it does not
copy the graph, clone a native obligation or create a new admission grant.

Keep a separate genuine runtime handle to construct each fresh request. The
operation exposes neither a runtime accessor nor `Deref` as an admission door.
Use `request.on_branch(owner.branch())` for the original branch. Runtime affinity
compares the underlying allocation, so a clone of the original handle is valid;
a newly installed runtime or a different branch is refused with custody intact.
Every call still needs a fresh request. Advance and current delivery retain
branch-selected program-support and complete-action checks. Close follows its
native cleanup eligibility and fresh request/resource checks; stale current
delivery does not erase historical custody or lawful cleanup.

The [compiled fixed consumer](../../../../../../docs/build-an-application.md#compiled-owning-call-bodies)
retains the operation, drops its constructing runtime handle and continues on
an independently retained handle for the same allocation. Its declaration also
composes locality/change, evaluated requirement and actual external-input use;
metadata presence alone is not its source/output observation.

Install initial workflow vocabulary and other unique-runtime configuration
before cloning handles or retaining operations. If `retain_workflow_spec`
refuses, `WorthQueryWorkflowRuntimeBindingRefusal::denial()` describes why;
`into_parts()` restores the original runtime, installed spec, signing owner and
denial. `support_workflow_spec(&mut self, spec)` returns the original spec with
its denial on failure. Release the real keepalives before retrying a
`RuntimeKeepaliveRetained` refusal; do not substitute another runtime or signing
owner or discard refused configuration inputs.

Runtime close also refuses `RuntimeKeepaliveRetained` while an operation keeps
its allocation alive. Successful operation close releases that keepalive;
refused close returns the complete usable owner. Releasing independent cleanup
does not settle uncertain source/output work. Drop is abandonment, and retention
promises neither serialization, cross-thread progression nor crash-durable
continuation. Original native source/output obligations keep their distinct
recovery and cleanup rules.

## Owning discovered actions

Declare a discovered delivery at the actual source occurrence and call the same
`retain_operation::<Program, Action>` entry. It returns
`WorthQueryDiscoveredOperation`; construction refuses an undeclared occurrence,
foreign root, uninstalled projection or counterfeit concrete delivery before any
effect. An identical delivery identity string cannot substitute a different
selector implementation. Keep the original input, key, row/result-set expectation
and complete typed preconditions in that owner. An attempt takes only a fresh
ordinary request; it cannot replace those fields.

A source publication can perform, retain an unpublished native partial, or commit
but refuse output-source retention. All three remain genuine native owners inside
the operation. `advance` delegates one step in one admitted phase. Match
`Progress { operation, .. }` and `Refused { operation, denial }` and retain the
returned operation. A committed retention refusal retries its actual original
carrier; it does not rerun the handler or reconstruct discovery from a receipt.
The native failure owner's posture decides whether that carrier is retryable.

The small caller shape is:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let operation = request.mutate(intent)
    .expect_source(original_source)
    .preconditions(original_conditions)
    .idempotency_owned(original_key)?
    .retain_operation::<Program, DeclaredDiscoveredAction>(&application)?;
let next = operation.advance(&fresh_request);
```

The [original partial-publication consumer](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered_owning.rs)
uses a non-Clone intent, nondefault conditions and a real append failure. Its
original request ends before recovery. The
[committed-refusal consumer](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered_owning/retention.rs)
retains the actual failed registration, refuses a stopped fresh attempt, then
promotes the same carrier and settles native fanout. These are the concrete
consumer bodies for the source batch; their verification status belongs to the
associated test run, not this illustrative fragment.

`Settled(settled)` contains the original complete discovery evidence and all
native output settlements. `settled.read_current(&fresh_request)` resolves the
same action against the branch's current installed program, checks the complete
original discovery and every settlement, then executes the original declared
selector at that **same observation and phase**. It never rediscovers roots to
replace the original required set. Originally empty discovery retains its
negative facts. Retired roots retain membership evidence without invented output
settlements.

`read_current_with_policy` narrows finite native currentness/query ceilings.
A refusal borrows the settlement; retry with fresh authority on that same owner.
For a declared `ApplicationQueryBatchProjection<Read, N>`, use
`read_batch_current::<N>` or `read_batch_current_with_policy::<N>`: one native
aggregate allowance funds the ordered batch, and a late item refusal publishes
no partial collection. For `ApplicationNoProjection`, use `deliver_current`;
there is no dummy selector or query permission.

`read_historical` and `read_batch_historical::<N>` use the original declared
selectors at the authentic retained settlement basis under fresh query authority.
They do not claim Current, adopt a new selector, republish outputs, or advance a
completed handle. A consumed-fact change or removal/change of the current action
refuses current delivery while preserving this history and lawful close.

Fresh handler results and recovered evidence remain distinct. `handler_result()`
is present only when the actual native Fresh result survives. A partial recovery
or registered replay does not manufacture it. Reply loss can rejoin a genuine
registered output owner using the original descriptor; a receipt cannot recreate
missing registration or one-shot source delivery. `Terminal` has no advance
method. Keep its native evidence and cleanup even when its business obligation
cannot complete. `release_cleanup` discharges only the independent cleanup
collection; `close` refuses while unresolved native obligations remain and
returns the whole owner. None of these owners is a crash-durability promise.

Deliberate partial-carrier manipulation, start/promotion and raw registered
recovery remain current native controls under `request.advanced()`. They serve
native custody and hostile identity courts, not an alternate ordinary retry path.
Workflow and assessment owners retain their distinct native authority boundaries.

## Preparing a program action before publication

### Advanced native preparation

Use the [ordinary owning entry](#owning-fixed-and-source-only-actions) for fixed, discovered and source-only actions.
Select `request.advanced()` only for a present specialized native staging or
authority/phase responsibility. Ordinary create/edit/retry stays on the owning
entry; moving its manual preparation loop here is not a native specialization.


A keyed mutation request may call `prepare_in_program(&runtime, allocation)` when its
installed candidate must wait before publication. Match
`WorthQueryApplicationProgramMutationPreparation`: `Prepared` carries a sealed
`WorthQueryPreparedProgramMutation`; `Settled` carries an ordinary replay,
domain denial, cancellation, or deadline outcome without a new result.

Preparation performs the same admission, source binding, idempotency check,
tracked decision and candidate construction as `execute_in_program`. It retains
the native read set and exact branch-selected program owner, holds no commit
lane, applies no effects and registers no idempotency outcome. The request is
borrowed until the handle is consumed or dropped. Its one-shot source and
preconditions are consumed once: preparing that same request again returns
`PreparationSpent`; build a fresh request with the same key for a real retry.

Call `candidate.commit(allocation)` to compare and publish. The result is inaccessible
before an accepted new commit. Drop discards it; stale basis, revoked authority,
changed program, cancellation, deadline and duplicate replay never release it.
Any intervening same-branch commit conservatively rejects the prepared basis,
including an unrelated edit. This surface does not claim precise reuse across
unrelated changes, detached result authority, durable queued work, or effect
recovery. Those use their existing source, workflow, and publication owners.

## How It Executes

### Advanced native substrate

Use the [ordinary owning entry](#owning-fixed-and-source-only-actions) for fixed, discovered and source-only actions.
Native request rebuilding in this section requires explicit `advanced()` selection.


Native mutation execution, preparation and publication take an explicit
`ExecutionAllocationPolicy`. A host may choose `SystemAllocation`, or supply
`Execution(&lease)` from its caller-owned process authority. The policy governs
the completed decision and provider read-set arrays; it grants no graph authority
and does not charge nested metadata or temporary maps. Preparation retains its
payload charge with the candidate. Publication takes a fresh policy for its
provider comparison. An admitted idempotent replay returns before new allocation.
Request cancellation and deadlines constrain a child lease without changing the
caller's parent policy or shortening payload custody to the request's lifetime.

### Observing installed mutation work

#### Advanced native reports

Use the [ordinary owning entry](#owning-fixed-and-source-only-actions) for fixed, discovered and source-only actions.
The following reports belong to the explicitly advanced native builder.


Use `execute_in_program_report` when a caller needs the installed handler's
decision projection evidence alongside the ordinary outcome:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
use worth_query_host::facade::primary_graph::WorthQueryMutationHandlerWork;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

let report = request.advanced().mutate(mutation_intent)
    .without_source().idempotency(&command_id)
    .execute_in_program_report(&application, ExecutionAllocationPolicy::SystemAllocation);
if let WorthQueryMutationHandlerWork::Captured(capture) = report.decision_work() {
    let projection = capture.projection_work();
    println!("{} fields read", projection.field_reads());
}
let outcome = report.into_outcome()?;
```

`execute_report`, `prepare_in_program_report`, the prepared handle's
`commit_report`, and `execute_performed_report` preserve the same distinction.
Existing methods execute that same path and return only its ordinary outcome.
Reports have sealed construction; captured values describe actual Query work
and grant no execution, read, publication or result-reuse authority.

`NotStarted` means no decision projection reader ran in this invocation.
`Captured` can contain zero work and records whether the installed handler was
contacted. A denial after projection begins retains its observed work. A cached
idempotency replay before handler execution is `NotStarted`; a duplicate found
when committing an already prepared candidate retains that candidate's original
capture without releasing its unpublished result. Preparation and commit reports
repeat the same capture, so adding them would double-count one execution.

The eleven counters cover indexed candidates, adjacency/endpoint visits, field
reads, aggregate activity and output-lineage lookups during decision projection.
They exclude candidate construction, commit work, physical I/O, latency and
whole-process memory. Compare the complete work value when checking structural
amplification: `provider_work_units()` excludes aggregate cache hits and rebuild
input rows. Commit receipts continue to describe landed history rather than
the fresh work of a retry.

The ordinary request path is:

```text
typed declaration
    -> installed application meaning
    -> authenticated principal resolution
    -> capability, purpose, disclosure, and conflict admission
    -> graph-read or mutation-plan admission
    -> provider-session execution
    -> typed terminal outcome
    -> governed publication or legal recovery action
```

For a contribution-composed application, install through
`worth_query_host::facade::application_installation`:

- `in_memory_program` validates and installs an application program and returns
  `WorthQueryProgramApplicationRuntime<Schema, Program>`;
- `in_memory` installs contribution-owned schema meaning without an application
  program and returns `WorthQueryPrimaryGraphApplicationRuntime<Schema>`.

Execute through a borrowed typed request. A mutation whose action belongs to an
installed program commits through the branch-resolved program lane:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

let request = application.request(&external_principal, &request_scope);
let result = request.query(query_intent).execute()?;
let outcome = request
    .advanced().mutate(mutation_intent)
    .without_source()
    .idempotency(&command_id)
    .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)?;
```

`execute(allocation)` without a program serves only mutations that require no
application program; for a program-owned action it returns
`WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired`.

The root lists contributions once. Entries own declarations, producer and
conditional contracts, handler and provider configuration, and invariant
factories; installation checks their exact
membership before publishing the application. The [host guide](../../../worth-query-host/README.md#contribution-composed-applications)
and [public consumer](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/main.rs)
show this construction. A borrowed request selects no World and caches no
permission. Each execution performs selection, resolution, admission, and
publication through the existing owners.

Installed mutation handlers supply `decide` and `build_candidate`. Computing
handlers override `prepare_decision` with an installed owner's `decision_plan`.
Query completes decision dependencies and runs installed invariants against the
actual candidate plus affected untouched neighbors. Handlers do not forecast
candidate bytes or validator work. Invalid candidates cannot publish.

Handlers use tracked typed field and relation reads through `DecisionReader` and
build the attempt's effect program through `CandidateWriter`'s create,
initialize, write, link, unlink, delete, emit, and output-role operations.
Complete equality selections use `select_entities` with a finite candidate
limit; empty selections remain dependencies and overflow denies publication.
Query application checkpoint version 7 preserves these predicate facts with
bounded scalar values and sorted unique membership. Versions 3–6 remain
readable. These facts compare against the exact selected branch index after
restore; a missing or overflowing index result cannot authorize output reuse.
Invariant factories resolve typed field and relation bindings once and evaluate
the actual proposed overlay and committed before-image inside a declared prepared
scope and finite work budget.

Fixed output roles are marker types. Each role's
`WorthQueryApplicationOutputRole` impl declares its contract, entity marker,
action, cardinality and name, and the contract lists the derived `DESCRIPTOR`
in `ROLES`, so the handler writes and every reader reads through the
declaration itself. A role use that disagrees with its contract fails to
compile. An exactly-one role must be bound; leaving it unbound is
`MissingOutputRole`. Any second binding is `DuplicateOutputRole`, and an unbound
at-most-one role commits without that output. Reading an at-most-one role
through `entity::<Role>()`, `DecisionReader::prior_output` or generated-output
reconstruction returns `Option`, so absence is a value rather than a denial. An
exactly-one role keeps those reads total.

Regenerating handlers declare variable semantic roles as a
`WorthQueryApplicationOutputRoleFamily` marker listed in
`ApplicationMutationOutputContract::ROLE_FAMILIES`, bind members with
`create_member`, `preserve_member` and `retire_member`, and read a prior family with
`DecisionReader::prior_output_family`. Query returns live members in deterministic
role order from the selected branch occurrence and product generation. The
resulting typed identities are normal tracked decision reads. An undeclared
family fails to compile; stale correspondence and exhausted work fail through
`WorthQueryPriorOutputDenial`.
Handlers shared by initial publication and regeneration use
`DecisionReader::prior_output_family_if_present`. It returns `None` only when the
exact prior binding has no correspondence at the selected occurrence and
generation. Declaration, identity, visibility, consistency, and work failures
remain denials.

Committed receipts expose `outputs_of::<Contract>()` for preserve/create/retire
roles and `committed_changes()` for immutable structural and lineage
observations from the same commit. `outputs_of` checks once that the commit was
made under `Contract`, refusing another contract with
`WorthQueryApplicationOutputProjectionDenial::ForeignContract`. Projecting
`entity::<Role>()` on the returned view names the role's marker, so its name,
action and entity marker come from the declaration, and a role of another
contract fails to compile. These
observations carry no new execution authority.
The [public replacement proof](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/output_correspondence.rs)
checks projection, readback, rejected-candidate isolation, and idempotent recovery.

`application.discovery()` describes installed mutations, queries, request
bindings, and fields, including units, scope, effects, and typed failures.
Installed availability does not grant current permission.

### Produced output and read lifecycles

Contribution contracts declare producer/output-family and conditional inventory;
configuration supplies only the implementations owned by that contribution.
Installation rejects missing, duplicate, foreign, mismatched, uncovered, or
ambiguous bindings before it publishes the application.

`request.demand(demand).controls(controls).start()` selects the exact source and
one applicable installed producer under finite work and retained-byte limits.
`advance(&fresh_request)` returns `Progressed`, `Pending` or `Settled`. `Progressed` permits the same original owner to continue; `Pending` yields rather than spinning. Convenience `settle` follows progress under the same request until settlement, waiting, interruption or an explicit finite step boundary. A settlement carries the
committed receipt, observed source, exact retained observation, and actual
readiness delivery. The handle exposes owner notifications and `close()` releases
the interest. Source drift returns `Superseded`; output readiness follows the
derived publication rather than source success.

Query results expose bounded `observed_sources()`. A source-bound mutation calls
`.expect_source(...)`; fresh admission and publication compare the declared native
source footprint. Sibling edits outside that footprint are allowed; sibling
membership and selector-field changes may invalidate it. Missing, foreign,
retired, ABA-changed, or changed source evidence is denied explicitly.

When an input selects subjects within its scope (for example, a copied occurrence
and its destination parent), its `ApplicationMutationBinding` implements
`expected_source_parameters(input)` with `Ok(Some(typed_query_parameters))`.
Query compares those selectors against the observation's exact canonical parameter
basis before binding source facts or invoking the handler. Row sources, result-set
sources and framework producers use the same check; a mismatch returns
`SourceParametersMismatch`. The query's installed canonical-work budget still
applies. `Ok(None)` means the binding intentionally accepts any parameter selection
of its declared source query; it must not be used to bypass input-selected subjects.
Scope/branch affinity and native source currentness remain separate required checks.

`request.retain_read()` captures an exact application occurrence.
`request.at(&observation).query(intent).execute()` reads it after fresh identity and
scope admission. Current live reads use
`request.query(intent).subscribe(WorthQueryApplicationLiveLimits::bounded(...))`;
each `next(&fresh_request)` rechecks the application and branch, and `close()`
releases the lease. Retained requests cannot open live subscriptions.

Host integrations that explicitly own a selected product attempt can use the
selection and admission surface for reads:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let branch = application.current_world();
let selected = application.on_branch(branch).select()?;
let admitted = selected.admit_application_query(
    &query,
    &access,
    ApplicationQueryParameterSet::new(),
    controls,
)?;
let result = application.execute_application_query_one_shot(admitted)?;
```

`selected` pins the exact composite occurrence. Query carries its World,
Relational, Signal, and Bridge affinity through admission and execution; later
phases do not resolve latest product truth again.

### Branch-resolved mutation lanes

#### Advanced native branch lanes

Use the [ordinary owning entry](#owning-fixed-and-source-only-actions) for fixed, discovered and source-only actions.
The covered manual lanes below require `request.advanced()`. Guarded workflow
actions use the [ordinary workflow owner](workflows.md#operations), retaining the original issued requirement.


A mutation does not name the program that commits it. Each lane selects the
request's exact branch, resolves the commit owner from the program that branch
carries, and commits through that installed owner:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

let outcome = application
    .request(&principal, &scope)
    .on_branch(branch)
    .advanced().mutate(intent)
    .without_source()
    .idempotency(&command_id)
    .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)?;
```

| Lane | Use it for |
|---|---|
| `execute_in_program(&runtime, allocation)` | A deliberate advanced mutation committed through the branch's program owner. |
| `execute_capability_in_program(&runtime, allocation)` | The same lane for a binding that implements `ApplicationCapabilityMutationBinding`. |
| `execute_retained_in_program(&runtime, allocation)` | A mutation whose committed outcome also returns a retained read observation (`WorthQueryApplicationRetainedMutationOutcome`). |
| `execute_performed::<Program, Root>(&runtime, allocation)` | A source mutation that commits and starts the required outputs declared under `Root`. |
| `execute_performed_discovered::<Program, Root>(&runtime, allocation)` | The discovered-output counterpart of `execute_performed`. |

Each lane takes the `WorthQueryProgramApplicationRuntime` that the request was
built from. A different runtime returns `ApplicationProgramMismatch`, or
`WorthQueryPerformedMutationExecutionDenial::ForeignProgram` on the performed
lanes, which also refuse a `Root` the program does not declare
(`UndeclaredOutputRoot`) and require the source phase returned by
`expect_source` or `without_source`. A retried idempotency key replays its recorded outcome before
any commit, even after the branch adopts another program. If the branch's
program removed the action, the lane returns the inactive-program denial and
cannot commit the removed action.

A key records its native intent durably. Replay after restore or reopen requires
the full original intent to be re-expressible under fresh authorization. The tested
no-source, unpinned actions satisfy this requirement where their binding permits
that source posture and all other native intent material remains identical.
No-source alone is not a blanket restart guarantee.

A row or result-set source expectation commits to the original runtime authority.
A new observation after restore is different intent, not a recreation of the old
source. Do not change the key or source to force replay, retain a foreign old
observation, or use `without_source` where the binding requires a query source.
Process-local owning handles are not serialized restart authority; Store retains
its distinct durable restart responsibility.

When the original intent does match but its live receipt is no longer retained,
native replay answers `PreviouslyCommitted(observation)`, naming the original
`commit_id()` and committing nothing. It carries no live receipt or handler result.
A fresh current read is not proof that an ambiguous external effect never occurred.
Reconcile the authoritative outcome through the existing native, domain or Store
recovery owner as appropriate before choosing a new business operation; a new key
is not recovery for an uncertain old effect. Lower surfaces that require a live
receipt retain `CommittedReceiptNotRetained { commit }`.
A record from an earlier intent encoding that cannot be checked against the
request answers `RecordedIntentUnverifiable`, never drift.

A host integration that already owns an admitted change can use the advanced
product-transaction lane instead of a request lane. It is not the ordinary path:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let outcome = application
    .on_branch(branch)
    .transaction()
    .apply(WorthQueryAdmittedChange::new(program, idempotency))
    .commit_for_program(application.admit_program_operation::<Operation>()?)?;
```

`commit()` serves a change with no application program. Both return
`WorthQueryProductTransactionCommitError::{ApplicationMismatch, BranchMismatch}`
for a foreign application or branch, and otherwise a raw
`WorthQueryApplicationCommitOutcome` that the caller splits with `landed()`.
`admit_program_operation` refuses an operation guarded by workflow authority;
such an operation commits only through its typed workflow lane.

The complete executable
[ordinary product workflow](../../../worth-query-certification/examples/ordinary_product_workflow.rs)
installs a validated program with `in_memory_program`, retains a read, commits a
typed mutation through a captured SourceOnly owner, advances with fresh authority,
closes the completed owner, reads the successor and retained occurrence, and
closes runtime resources.

### Branch-local program selection and adoption

An ordinary request selects program meaning from its exact product branch. It
does not consult a process-wide "latest program":

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let programs = application
    .request(&principal, &scope)
    .on_branch(branch)
    .programs();

let selected = programs.inspect()?;
let requirements = programs.compare(&target_revision)?;
let prepared = programs
    .adopt(&requirements)
    .prepare(maximum_selection_work)?;

match prepared.publish() {
    WorthQueryBranchAdoptionPublicationOutcome::Performed(adoption) => {
        use_target_program(adoption.target());
    }
    WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => {
        handle_stale_or_equivalent(no_effect);
    }
    WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(pending) => {
        retain_exact_adoption_recovery(pending.into_recovery());
    }
}
```

`inspect` and `compare` are descriptive. Only the move-only prepared adoption
can attempt publication, and only World's `Performed` terminal changes the
branch program. Preparation returns typed support, source/target, migration,
target-rule, resource, and custody denials before publication. A retained read
continues interpreting its exact old occurrence, but it gains no current
mutation authority from that retention.

Broader adoption is deliberately non-atomic. The branch owner first issues
bounded coverage; the caller supplies an exact order; every target is
preflighted before the first publication. `advance()` records one
`Performed`, `NoEffect`, or `ProductUnpublished` disposition at a time.
Cancellation preserves the performed prefix, and recovery preserves both that
prefix and the untouched suffix. It never reports rollback of work that an
owner already performed.

### Commit terminals: landed or uncommitted

Every commit either landed or did not. The request lanes already report that
split: `WorthQueryApplicationMutationOutcome::Committed { receipt, result }` and
`AlreadyCommitted(receipt)` carry the canonical product receipt.
`PreviouslyCommitted(observation)` proves an earlier commit with the same
durable intent when its live receipt is unavailable; it grants no performed
mutation, publication, or workflow settlement authority. Every
commit terminal that did not land arrives as
`WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted)`.

Host code that receives a raw `WorthQueryApplicationCommitOutcome` splits it with
`landed()`:

<!-- doctest: bank-estate-certification -->
```rust,no_run
# use worth_query_host::facade::primary_graph::{
#     WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
#     WorthQueryApplicationUncommitted,
# };
# fn publish(_: WorthQueryApplicationCommitReceipt, _replayed: bool) {}
# fn handle_uncommitted(_: WorthQueryApplicationUncommitted) {}
# fn settle(commit_outcome: WorthQueryApplicationCommitOutcome) {
match commit_outcome.landed() {
    Ok((receipt, replayed)) => publish(receipt, replayed),
    Err(uncommitted) => handle_uncommitted(uncommitted),
}
# }
```

`Ok((receipt, replayed))` is a landed commit; `replayed` is `true` when the
landing replays an earlier commit of the same idempotency. `Err` carries a
`WorthQueryApplicationUncommitted`, which holds every commit terminal except a
landed commit. Match each of its variants:

| `WorthQueryApplicationUncommitted` variant | Meaning |
|---|---|
| `ProductUnpublished`, `Deferred`, `SettlementDeferred` | Owner-specific recovery custody is retained. |
| `Indeterminate` | The landing is unresolved, not failed; do not retry as though it failed. |
| `NoEffect`, `Stale`, `ProductStale`, `Cancelled`, `TimedOut`, `Denied`, `Aborted` | Distinct application decisions; nothing landed. |

`require_committed()` is the receipt-only convenience: its error is the original
typed `WorthQueryApplicationCommitOutcome` and must be handled rather than erased.

Every later governed transition rechecks the current evidence it depends on.
Continuation, live delivery, approval, recovery, and conditional-operation
re-entry therefore do not inherit stale permission from an earlier request.

## Small Example

A domain-native read keeps the caller-facing shape small while still using the
installed Query path:

<!-- doctest: bank-estate-certification -->
```rust,no_run
# use bank_domain::model::AccountId;
# use bank_server::{
#     queries, BankApplicationQueryDenial, BankAuthenticatedPrincipal, BankIdentityRuntime,
#     BankReadControlDenial, BankReadControls,
# };
# use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
# #[derive(Debug)]
# enum AppError {
#     UnexpectedCardinality,
#     Controls(BankReadControlDenial),
#     Query(BankApplicationQueryDenial),
# }
# impl From<BankReadControlDenial> for AppError {
#     fn from(denial: BankReadControlDenial) -> Self { Self::Controls(denial) }
# }
# impl From<BankApplicationQueryDenial> for AppError {
#     fn from(denial: BankApplicationQueryDenial) -> Self { Self::Query(denial) }
# }
# fn show<T>(_: T) {}
# fn account_summary(
#     bank: &BankIdentityRuntime,
#     principal: &BankAuthenticatedPrincipal,
#     account: AccountId,
#     request_scope: WorthQueryRequestScope,
# ) -> Result<(), AppError> {
let result = bank
    .query(queries::account_summary(account))
    .as_principal(&principal)
    .controls(BankReadControls::current(request_scope, 32)?)
    .execute()?;

let [summary] = result.rows() else {
    return Err(AppError::UnexpectedCardinality);
};
show(summary);
# Ok(())
# }
```

This Markdown is included directly in the Bank certification crate's API docs.
Its marked doctest blocks are checked against the real public types, so changes to the
constructor or method sequence break CI rather than leaving the guide stale.

The wrapper chooses product vocabulary. The fresh principal, controls, and
typed outcome remain part of the real Query progression.

## Real Example

Money movement must preserve commit uncertainty and idempotent retry rather
than translating every transport success or failure into a business result:

<!-- doctest: bank-estate-certification -->
```rust,no_run
# use bank_domain::{proposals::BankIdempotencyKey, schema::{BankSchema, SendMoney}};
# use bank_server::{mutations, BankApplication, BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationControls, BankOperationConstructionRefusal, SendMoneyAction};
# use worth_query_host::facade::{
#     admission::authenticated_principal::WorthQueryRequestScope,
#     application_entry::WorthQuerySourceOnlyOperation,
#     declaration::application_program::ApplicationRetainedCommit,
# };
# type Operation = WorthQuerySourceOnlyOperation<BankSchema, SendMoney, BankApplication, SendMoneyAction, ApplicationRetainedCommit>;
# fn send_money(
#     bank: &BankIdentityRuntime,
#     principal: &BankAuthenticatedPrincipal,
#     input: SendMoney,
#     request_scope: WorthQueryRequestScope,
#     idempotency_key: BankIdempotencyKey,
# ) -> Result<Operation, BankOperationConstructionRefusal<SendMoney>> {
bank.mutate(mutations::send_money(input))
    .as_principal(principal)
    .controls(BankMutationControls::new(request_scope, idempotency_key))
    .retain_operation()
# }
```

Construction captures the original input and key without executing a source.
Keep the returned operation and call `advance(&fresh_request, maximum_history_work)`
with an explicit nonzero history bound. Preserve the actual pending, completed or
terminal owner returned by that attempt. Completed evidence distinguishes the
fresh business result from a recovered native receipt; recovery does not invent a
second business result. A refused close returns its owner for a fresh lawful retry.

The idempotency binding is application meaning installed by Query. The
provider owns the commit. Deferred and indeterminate commit outcomes retain the
only legal follow-up posture for that exact attempt; any live recovery handle
derived from them remains server-side rather than becoming serialized authority.

Bank estate notifications and disbursements use their typed mutation builders'
`retain_operation()` entrance. Preserve the returned SourceOnly owner and supply
fresh authority to each `advance` and refused-close retry. After completion,
`close_with_evidence(&fresh_request)` releases native cleanup before transferring
the original completion and its receipt for the installed publication handoff.
A recovered completion supplies native receipt evidence rather than a handler
result. See the [Bank source and publication handoff](../../../../../worth-query-bank-world/docs/public-consumer-contract.md#own-estate-sources-funding-reversal-and-deployment-revocation)
for the concrete builders and fresh- versus recovered-evidence handling.

For time-driven operations, the host performs installation rather than calling
the operation directly:

```text
installed operation and conditional node
    -> admitted host predicate provider
    -> admitted named clock and reconstruction contract
    -> published application runtime
    -> runtime-bound clock observation port
```

The host submits observations. Signal decides whether a derived wake is
eligible. Query then freshly admits and invokes the installed application
operation. See the conditional-operation guide for the complete contract.

## Owning Fixed-Output Actions

A conventional unguarded fixed source action declares its root and read through
`ApplicationFixedActionDelivery` and
`.action::<Binding, Delivery>() ... .finish_action()`. An exact typed
`ApplicationActionReference` includes the composition instance and feature;
matching only a mutation binding does not select an occurrence.

Source completion uses ordinary native publication and returns receipt-only
success. Retained commit uses the native observation-retaining publication path.
Both borrow fresh authority and use the supplied phase's checked allocation
policy. The description shares immutable encoded input custody across initial,
recovery and keyed attempts without copying domain input on each attempt.

The prepared mutation's `idempotency_owned(key)` opens the initiating request's
native capture phase and returns a funded immutable description or the whole
`(original_builder, owned_key, WorthQueryOperationInputCaptureDenial)` refusal.
Retain those owners for a lawful fresh opening; refusal precedes any business
effect or workflow-slot transfer. The original Intent is moved, not cloned.
One Execution-admitted typed payload is shared across the description and native
capability consumers; its existing charge follows the final shared input owner.
The native quote excludes nested caller Clone heaps and allocator/Arc metadata;
it is not a total-heap measurement or a new authority grant. `retain_operation::<Program, Action>(&program)` resolves actual installed
delivery meaning and its concrete selector implementation, returning that description
on refusal. A different delivery type with copied metadata does not resolve. `advance(&fresh_request)`
opens one native phase and borrows the original intent, key, conditions and
source proof. The branch-selected installed owner checks the cached original
action meaning in the actual source execution and recovery paths.

Fixed replay resolves the original key with fresh binding-selected admission.
`SourceReplayObserved` retains the native committed receipt;
`RegisteredOutputsRecovered` rejoins its actual registered demand without
rerunning the source handler. No handler result is invented. A retryable refusal
returns the original owner. Missing, retired or superseded registration produces
terminal historical evidence, and terminal close delegates remaining native
cleanup rather than declaring output completion.

The same constructor resolves explicit no-output completion or retained-commit
meaning. Its native SourceOnly handoff borrows original request meaning for
binding-selected conventional or capability admission, unpublished recovery and
keyed resolution. The actual output-source inventory can refuse that lane even
when a caller supplies no-output metadata. No root or query stands in for source
completion. Its owning retained mode requires an explicit per-attempt history
bound and delivers the exact native original World observation after replay or
recovery. A bounded unavailable result preserves custody for retry; it does not
prove historical absence. See [retained delivery](source-only-retained-delivery.md).
A pinned SourceOnly row validates its consumed native facts before the decision
handler. If they changed or a consumed entity retired, `advance` returns terminal
custody; `native_admission_denial()` exposes the original `SourceChanged` or
`SourceRetired` preparation denial. No commit outcome or unpublished receipt is
invented. A result-set loan retains selection membership, so changing a projected
value outside those membership facts does not itself make that loan stale.
The [pinned source courts](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/source_only_pinned.rs)
exercise both meanings through `retain_operation`.

Terminal `close` still requires a fresh request on the original runtime and
branch. A refused close returns the same evidence and owned source loan. A
lawful close consumes the owner, releasing its observation references through
their native lifetimes. Native `ProductStale` and `NoEffect` outcomes likewise
retain their descriptive observations until close; they carry no unpublished
recovery receipt. Unresolved publication postures remain owned. The [commit-time pinned court](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/source_only_pinned/commit_stale.rs)
covers the exact native `ProductBasisStale` invariant refusal while retaining
the original loan. SourceOnly owning operations retain native stopped evidence
and cleanup; ordinary callers do not switch to manual execution to handle a stop.

An unpinned SourceOnly operation can return `Progress` with
`RetryableNoEffect` after native sealed-read staleness or an exact
`ProductBasisStale` invariant refusal. `native_retry_outcome()` borrows the
actual completed native refusal. The next explicit `advance` selects the
original branch again and admits the same input, key, declared action and typed
preconditions under fresh request authority. It never replaces a caller's
expected fact with current state, and it performs only one attempt per call.
The [unpinned stale courts](../../../worth-query-certification/tests/application_graph/source_only/stale_retry.rs)
exercise that path with a genuine competing ordinary operation.

Live `close` accepts only this proven retryable no-effect custody and delegates
the same fresh-request cleanup owner. Refusal returns that evidence and original
descriptor. It does not dispose an unpublished partial or resolve an uncertain
business effect. Generic `Denied`, `NoEffect` and `Unseen` do not authorize a
rerun; indeterminate outcomes keep the existing keyed recovery path.

World pre-effect product-head loss is a terminal `ProductStale`, distinct from
an invariant-stage `ProductBasisStale` refusal. A refusal during World
preparation retains its expected product; its optional observed product can be
absent. Read the competing current head independently rather than filling that
absence with a newly observed value. Generic `NoEffect` retains its actual World
cause. Neither posture grants an ordinary retry or reports required outputs
complete.

SourceOnly and fixed terminal owners retain these observations across close
refusal and delegate fresh close to the existing cleanup collection. That close
releases the terminal's native observation custody and retries its actual failed
managed-run cleanup. A failed lower release remains owned across refusal; a
World no-effect outcome alone never proves managed cleanup succeeded. Inspect
`managed_cleanup_failure()` for that separate native custody. `release_cleanup`
discharges only the cleanup collection while retaining the operation or terminal
and its business evidence. It cannot resolve an uncertain commit or complete
required outputs. Close remains subject to each lane's native terminal contract;
its receipt does not certify external cleanup or crash durability. Collection and
closed results retain `managed_cleanup_receipt()` when a failed managed run was
released. Inspect its actual disposition and provider recovery evidence: released
resource reservations can coexist with logical `RecoveryRequired` evidence.
The [native posture courts](../../../worth-query-certification/tests/application_graph/source_only/native_postures.rs)
exercise an actual pre-World native writer race and finite retained-observation
capacity, with unchanged valid counterparts.

Shared funded cleanup owns pending record release and native cleanup failures.
Its receipt proves that collection was released; it does not discharge a retained
source carrier, blocked attempt or active output handle. Each lane keeps those
obligations until its native close contract succeeds.

After native settlement, `read_current` validates all output lineages and queries
at one fresh retained observation. Delivery refusal keeps the settled owner;
retry uses fresh authority and does not publish the source again. The recovered
native progress and a fresh handler reply have separate accessors.

`close` retains an undisposed native recovery record and every refused owner
cleanup until a funded retry can discharge them. It returns the complete settled
owner on refusal. A real post-recovery caller capability can block release, so
observation custody and cleanup custody remain independent.

The executable
[`fixed partial-publication journey`](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/required_unpublished.rs)
uses the actual append fault, fresh recovery, all four dependent settlements,
World observation-capacity refusal and World caller-capability cleanup refusal.
Fixed, discovered, source-only and guarded workflow actions use their ordinary
owning entrances. Deliberate native adapters remain under `advanced()`.

## How It Relates To Other Features

- Use installed application queries for current, historical, continuation,
  preview, or live execution only when that query's support contract admits the
  requested lane.
- Use application authorization for product permission. Authentication only
  resolves the caller's external identity.
- Use graph-read access planning for filters, ordering, traversal, and nested
  expansion. Application loops are not a substitute for an admitted plan.
- Use application aftermath and recovery for committed, partial-effect, or
  indeterminate mutation outcomes.
- Use conditional installed operations when the host supplies a predicate or
  clock. Do not add an application scheduler.
- Use certification replay only to compare prior semantic execution. It cannot
  authorize ordinary work.

## Inspection And Debugging

Start with the typed outcome and its public inspection evidence. Useful
evidence includes:

- installed schema, query, operation, and generation identity;
- principal-resolution and authorization denial kinds;
- graph-read plan and provider-session work;
- disclosure omissions and publication posture;
- commit, idempotency, recovery, and external-effect posture;
- conditional intent, wake, Signal decision, attempt, and terminal provenance;
- ordinary and reconstructive work counters reported separately.
- temporal binding, runtime-installation, and fresh-admission canonical work in
  their named inspection phases; later execution and publication do not
  regenerate those identities.

Do not parse `Display` text or reconstruct authority from a digest. Inspection
explains a transition; it does not perform the transition.

## Capture And Reopen Cost Boundary

An installed application runtime can capture its opaque Query checkpoint and
encoder-owned section report together with
`capture_application_checkpoint_with_sections(phase, policy)`, where `phase`
is a `WorthQueryAdvancementPhase` lent by the same runtime (for example by
`with_host_advancement`). The report splits Query
framing, native Relational bytes and accepted-output identities; a locally
captured native checkpoint also reports envelope, branch-root, branch-cell,
partition-mirror, derived-index and framing bytes. These sizes describe the
same encoding pass. They cannot validate received bytes or authorize restore.

The caller supplies `WorthQueryCheckpointCapturePolicy::SystemAllocation` or
`Execution(&lease)` for capture, program transition and every repair attempt.
System allocation is fallible and uncharged. Execution mode checks the live
caller lease, reserves the checked exact final-frame payload before allocating,
and retains that one charge until its last Query or embedded native byte owner
drops. Neither mode charges native codec temporaries, decoded rows or allocator
metadata. There is no fallback from an exhausted or stopped lease to system
allocation.

`WorthQueryCheckpointCaptureDenial` preserves either the native durability
error or the typed physical allocation refusal and its available payload quote.
An acknowledged transition whose capture fails returns its unpublished repair
capsule, with `capture_denial()` identifying the latest capture failure. A later
native settlement failure clears that obsolete capture cause. Repair takes a
fresh explicit policy, retains its current phase on refusal and never reruns
authoring; a captured successor still requires ordinary target readmission.

The application owns the enclosing artifact, transport, compatibility check
and fresh installed-schema readmission. Relational verifies and rebuilds
committed truth; Query readmits accepted-output identities without rerunning
producers. Retained views and handles are reconstructed separately. Compare
work against the actual scene/dependency shape and report wall time with its
hardware; an absolute seconds threshold is not a portable Query contract.

## Anti-Patterns

Use the
[canonical consumer prohibitions](../../../../../../docs/capability-index.md#consumer-prohibitions).

## Observable workflows through ordinary requests

Use `request.retain_current_read()` to pin data, then `request.at(&basis)` for
freshly governed workflow discovery and history. The no-argument receiver owns
its native admission; examples need no internal control or fabricated phase.
Subject read and actor audit are independent. `found.target()` and
`observation.target()` return data locators, not approval needs or grants.
History marks never auto-refresh the retained basis: explicitly retain a fresh
read and reacquire a target. `subscription.close()` releases subscription custody;
original owning operation settlement and cleanup still use their own receivers.

The [neutral mixed workflow](../../../../../../docs/build-an-application.md#67-capture-approve-archive-and-observe-one-workflow)
shows proposal waiting, occurrence capture, conditional distinct approval,
real local archive ingress and the complete owning caller loop, with marker-bound
excerpts. The [workflow guide](workflows.md)
owns the detailed read policies and outcomes.

## Current Limits

- The synchronous M0 contribution, request, handler, candidate, invariant,
  producer, conditional, bounded output-demand, exact/live read, correspondence,
  discovery, and branch-local program-evolution foundation is certified.
  Deferred producer completion belongs to the later producer extension.
- Authored workflow definitions are available: an application builds a
  definition with `ApplicationWorkflowDefinitionBuilder`, publishes validated
  revisions, discovers the current one through
  `WorthQueryWorkflowDefinitionDiscovery`, and starts and progresses instances
  through typed proposal, approval, and progress outcomes. The
  [authored workflow example](../../../worth-query-certification/examples/authored_workflow/main.rs)
  runs that journey end to end, and the [workflows guide](workflows.md)
  documents every step, outcome, and denial. An installed inbound source may
  complete the exact owner of a preceding external-effect operation. The
  `await_inbound` node waits for that owner result; a fresh caller request
  advances the workflow. Callback bytes do not resume an instance.
- Historical, preview, continuation, and live lanes are available only for an
  installed query whose declared support and current admission allow that lane.
- Conditional providers and managed clocks are stable on the primary-graph
  application runtime. Query does not publish a separate general-purpose
  temporal workspace API.
- Recovery handles and temporal wake state are runtime-local. An application
  may capture Query's opaque native checkpoint with its accepted-output
  identities, then readmit it through its installed schema on restart; the
  application owns the enclosing artifact and Store transport. This is not a
  general Query Save/Open API. Temporal wakes reconstruct from surviving
  authoritative domain truth rather than persisted wake handles.
- Product branches, retained observations, and pending cleanup are
  memory-resident live capabilities. Process loss releases them; checkpoint
  readmission restores native committed truth and accepted output identities,
  not these handles or producer execution.
- Linear undo and redo remain provisional experiments. Milestone 9.18 owns any
  accepted public correction-history contract.
- Certification replay remains certification-only.

Completed touched-record evidence is sealed from the exact native prepared
candidate before effects using the caller's explicit `ExecutionAllocationPolicy`.
The shared immutable array retains its checked payload allocation through
receipt observers and unpublished recovery, including after the request lease
drops. There is no aggregate completed-evidence byte window or quota-driven
receipt eviction. Durable idempotency rows prevent re-execution when a fresh
process or retired product occurrence no longer retains its performed receipt.
Arc headers, native preparation and evidence indexes remain separate owners.

## Related Docs

- [Application Authorization And Emergency Elevation](../capabilities/application-authorization-and-emergency-elevation.md)
- [Graph Read Access Planning](../authoring/graph-read-access-planning.md)
- [Runtime-Installed Domains And Operations](../domain-capabilities/runtime-installed-domains.md)
- [Conditional Installed Operations](../domain-capabilities/conditional-installed-operations.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
- [Support Matrix And Admission](./support-matrix-and-admission.md)
- [Typed Stops And Remediation Guidance](../domain-capabilities/typed-stops-and-remediation-guidance.md)

Failed producer cleanup belongs to the admitted demand, not to cloned denial
diagnostics. Fixed owning progression transfers it into the existing operation
cleanup collection before returning terminal custody. A diagnostic copy grants
no cleanup authority. Conditional reinstallation and explicit runtime stop refuse while native release
is pending. Fresh clock progression can discharge only that release; reinstallation
then preserves the wake's original unresolved business evidence.

A failure in independently queued required work can return cleanup custody even
when the current caller's output is already current. The retained demand owns
that original queued producer terminal; cloned diagnostics cannot extract it.
Native cleanup release preserves its original business outcome and does not
re-execute either producer. Conditional lag reconstruction follows the same
rule as reinstallation: pending native release refuses replacement, and released
cleanup evidence remains retained without restoring business eligibility.
