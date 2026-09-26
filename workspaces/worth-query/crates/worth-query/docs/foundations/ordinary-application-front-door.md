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

```rust
use worth_query_decl::facade::{
    application_capability,
    application_query,
    application_schema,
};
```

Application hosts import `worth_query_host::facade`:

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

The main owners remain separate:

- the application domain owns business vocabulary and intent;
- Relational owns current graph facts and commits;
- Runtime Bridge owns installed correspondence;
- Signal owns policy decisions and temporal wake eligibility;
- Query owns application installation, admission, progression, and publication;
- the host supplies adapters and resources but does not reinterpret decisions.

## How It Executes

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

For a contribution-composed application, install with
`worth_query_host::facade::application_installation::in_memory` and execute
through a borrowed typed request:

```rust,ignore
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;

let request = application.request(&external_principal, &request_scope);
let result = request.query(query_intent).execute()?;
let outcome = request.mutate(mutation_intent).idempotency(command_id).execute();
```

The root lists contributions once. Entries own declarations, producer and
conditional contracts, handler and provider configuration, and invariant
factories; installation checks their exact
membership before publishing the application. The [host guide](../../../worth-query-host/README.md#contribution-composed-applications)
and [public consumer](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/main.rs)
show this construction. A borrowed request selects no World and caches no
permission. Each execution performs selection, resolution, admission, and
publication through the existing owners.

Installed mutation handlers supply `decide`, `candidate_requirements`, and
`build_candidate`. Query completes declared decision dependencies, reserves the
candidate's finite cardinality and representation before allocation, and runs
installed invariants against the actual candidate plus affected untouched
neighbors under a separate work bound. Invalid candidates cannot publish.

Handlers use tracked typed field and relation reads through `DecisionReader` and
build the one reserved effect program through `CandidateWriter`'s create,
initialize, write, link, unlink, delete, emit, and output-role operations.
Invariant factories resolve typed field and relation bindings once and evaluate
the actual proposed overlay and committed before-image inside a declared prepared
scope and finite work budget.

Regenerating handlers declare variable semantic roles through
`ApplicationMutationOutputContract::ROLE_FAMILIES` and read a prior family with
`DecisionReader::prior_output_family`. Query returns live members in deterministic
role order from the selected branch occurrence and product generation. The
resulting typed identities are normal tracked decision reads; undeclared families,
wrong entity markers, stale correspondence and exhausted work fail through
`WorthQueryPriorOutputDenial`.
Handlers shared by initial publication and regeneration use
`DecisionReader::prior_output_family_if_present`. It returns `None` only when the
exact prior binding has no correspondence at the selected occurrence and
generation. Declaration, identity, visibility, consistency, and work failures
remain denials.

Committed receipts expose `output_correspondence()` for preserve/create/retire
roles and `committed_changes()` for immutable structural and lineage
observations from the same commit. Projecting `entity(role)` requires the exact
binding, role name, action, and entity marker; substituting the entity marker
returns `WorthQueryApplicationOutputProjectionDenial::EntityMismatch` even when
the other three match. These observations carry no new execution authority.
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
`advance(&fresh_request)` returns `Pending` or `Settled`. A settlement carries the
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
selection and admission surface:

```rust,ignore
let branch = application.current_world();
let selected = application.on_branch(branch).select()?;
let admitted = selected.admit_application_query(
    &query,
    &access,
    ApplicationQueryParameterSet::new(),
    controls,
)?;
let result = application.execute_application_query_one_shot(admitted)?;

let outcome = application
    .on_branch(branch)
    .transaction()
    .apply(admitted_change)
    .commit_for_program(application.admit_program_operation::<Operation>()?)?;
```

`selected` pins the exact composite occurrence. Query carries its World,
Relational, Signal, and Bridge affinity through admission and execution; later
phases do not resolve latest product truth again. The complete executable
[ordinary product workflow](../../../worth-query-certification/examples/ordinary_product_workflow.rs)
constructs and installs a validated program, reads the selected branch,
performs a World publication, delivers the patch, executes its conditional,
checks the successor and a retained read, and closes runtime resources.

### Branch-local program selection and adoption

An ordinary request selects program meaning from its exact product branch. It
does not consult a process-wide "latest program":

```rust,ignore
let programs = application
    .request(&principal, &scope)
    .on_branch(branch)
    .programs();

let selected = programs.inspect()?;
let requirements = programs.compare(target_revision)?;
let prepared = programs
    .adopt(target_revision)
    .requirements(&requirements)
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

Match every commit terminal. `Committed` and `AlreadyCommitted` carry the
canonical product receipt. `ProductUnpublished`, `Deferred`,
`SettlementDeferred`, and `Indeterminate` retain owner-specific recovery
custody. `NoEffect`, `Stale`, `ProductStale`, `Cancelled`, `TimedOut`, `Denied`,
and `Aborted` are distinct application decisions. `require_committed()` is a
convenience: its error is the original typed terminal and must be handled rather
than erased.

Every later governed transition rechecks the current evidence it depends on.
Continuation, live delivery, approval, recovery, and conditional-operation
re-entry therefore do not inherit stale permission from an earlier request.

## Small Example

A domain-native read keeps the caller-facing shape small while still using the
installed Query path:

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
    .controls(BankReadControls::current(request_scope, 32, 20_000)?)
    .execute()?;

let [summary] = result.rows() else {
    return Err(AppError::UnexpectedCardinality);
};
show(summary);
# Ok(())
# }
```

This Markdown is included directly in the Bank certification crate's API docs.
Its Rust blocks are doctested against the real public types, so changes to the
constructor or method sequence break CI rather than leaving the guide stale.

The wrapper chooses product vocabulary. The fresh principal, controls, and
typed outcome remain part of the real Query progression.

## Real Example

Money movement must preserve commit uncertainty and idempotent retry rather
than translating every transport success or failure into a business result:

```rust,no_run
# use bank_domain::{proposals::{BankIdempotencyKey, BankProposalDenial}, schema::SendMoney};
# use bank_server::{
#     mutations, BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationControls,
# };
# use worth_query_host::facade::{
#     admission::authenticated_principal::WorthQueryRequestScope,
#     application_entry::{
#         WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
#     },
#     primary_graph::{WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt},
# };
# fn publish(_: WorthQueryApplicationCommitReceipt) {}
# fn inspect_commit_outcome(_: WorthQueryApplicationCommitOutcome) {}
# fn explain_domain(_: BankProposalDenial) {}
# fn explain_request(_: WorthQueryApplicationRequestMutationDenial) {}
# fn handle_terminal_stop() {}
# fn send_money(
#     bank: &BankIdentityRuntime,
#     principal: &BankAuthenticatedPrincipal,
#     input: SendMoney,
#     request_scope: WorthQueryRequestScope,
#     idempotency_key: BankIdempotencyKey,
# ) {
let outcome = bank
    .mutate(mutations::send_money(input))
    .as_principal(&principal)
    .controls(BankMutationControls::new(
        request_scope,
        idempotency_key,
    ))
    .execute();

match outcome {
    Ok(WorthQueryApplicationMutationOutcome::Committed { receipt, .. })
    | Ok(WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt)) => publish(receipt),
    Ok(WorthQueryApplicationMutationOutcome::Commit(commit)) => inspect_commit_outcome(commit),
    Ok(WorthQueryApplicationMutationOutcome::DomainDenied(reason)) => explain_domain(reason),
    Err(reason) => explain_request(reason),
    Ok(WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)
    | Ok(WorthQueryApplicationMutationOutcome::Cancelled)
    | Ok(WorthQueryApplicationMutationOutcome::DeadlineExceeded) => handle_terminal_stop(),
}
# }
```

The idempotency binding is application meaning installed by Query. The
provider owns the commit. Deferred and indeterminate commit outcomes retain the
only legal follow-up posture for that exact attempt; any live recovery handle
derived from them remains server-side rather than becoming serialized authority.

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
`capture_application_checkpoint_with_sections()`. The report splits Query
framing, native Relational bytes and accepted-output identities; a locally
captured native checkpoint also reports envelope, branch-root, branch-cell,
partition-mirror, derived-index and framing bytes. These sizes describe the
same encoding pass. They cannot validate received bytes or authorize restore.

The application owns the enclosing artifact, transport, compatibility check
and fresh installed-schema readmission. Relational verifies and rebuilds
committed truth; Query readmits accepted-output identities without rerunning
producers. Retained views and handles are reconstructed separately. Compare
work against the actual scene/dependency shape and report wall time with its
hardware; an absolute seconds threshold is not a portable Query contract.

## Anti-Patterns

- importing internal Query authority crates from application code;
- treating a role, relationship, token claim, or Signal decision as permission;
- executing a provider or operation callback directly;
- reading protected fields and redacting them after projection;
- rebuilding cursors, continuation, or recovery authority from wire values;
- maintaining a local temporal scheduler or conditional wake registry;
- retrying an indeterminate mutation as though it definitely failed;
- teaching provisional undo or redo as an accepted application contract.

## Current Limits

- The synchronous M0 contribution, request, handler, candidate, invariant,
  producer, conditional, bounded output-demand, exact/live read, correspondence,
  discovery, and branch-local program-evolution foundation is certified.
  Dynamic workflow-definition instances and their adoption dispositions belong
  to the separately governed successor milestone. Deferred producer completion
  belongs to the later producer extension.
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

## Related Docs

- [Application Authorization And Emergency Elevation](../capabilities/application-authorization-and-emergency-elevation.md)
- [Graph Read Access Planning](../authoring/graph-read-access-planning.md)
- [Runtime-Installed Domains And Operations](../domain-capabilities/runtime-installed-domains.md)
- [Conditional Installed Operations](../domain-capabilities/conditional-installed-operations.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
- [Support Matrix And Admission](./support-matrix-and-admission.md)
- [Typed Stops And Remediation Guidance](../domain-capabilities/typed-stops-and-remediation-guidance.md)
