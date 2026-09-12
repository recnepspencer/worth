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

The root lists contributions once. Entries own declarations, handler
configuration, and invariant factories; installation checks their exact
membership before publishing the application. The [host guide](../../../worth-query-host/README.md#contribution-composed-applications)
and [public consumer](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/main.rs)
show this construction. A borrowed request selects no World and caches no
permission. Each execution performs selection, resolution, admission, and
publication through the existing owners.

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
    .commit()?;
```

`selected` pins the exact composite occurrence. Query carries its World,
Relational, Signal, and Bridge affinity through admission and execution; later
phases do not resolve latest product truth again. The complete executable
[ordinary product workflow](../../../worth-query-certification/examples/ordinary_product_workflow.rs)
constructs the real declaration and host runtime, reads the selected branch,
performs a World publication, delivers the patch, executes its conditional,
checks a retained read, and closes runtime resources.

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
# use bank_domain::{proposals::BankIdempotencyKey, schema::SendMoney};
# use bank_server::{
#     mutations, BankAuthenticatedPrincipal, BankCommitReceipt, BankIdentityRuntime,
#     BankMutationControls, BankMutationDenial, BankMutationStatus, BankUnresolvedCommitEvidence,
# };
# use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
# fn publish(_: BankCommitReceipt) {}
# fn retry_from_fresh_state(_: usize) {}
# fn inspect_before_retry(_: BankUnresolvedCommitEvidence) {}
# fn explain(_: BankMutationDenial) {}
# fn handle_terminal_stop(_: BankMutationStatus) {}
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

match outcome.into_status() {
    BankMutationStatus::Committed(receipt)
    | BankMutationStatus::AlreadyCommitted(receipt) => publish(receipt),
    BankMutationStatus::Stale { stale_fact_count } => {
        retry_from_fresh_state(stale_fact_count)
    }
    BankMutationStatus::PartialEffect(evidence)
    | BankMutationStatus::Indeterminate(evidence) => inspect_before_retry(evidence),
    BankMutationStatus::Denied(reason) => explain(reason),
    stop => handle_terminal_stop(stop),
}
# }
```

The idempotency binding is application meaning installed by Query. The
provider owns the commit. Partial-effect and indeterminate evidence retain the
only legal follow-up posture for that exact attempt; any live recovery handle
derived from it remains server-side rather than becoming serialized authority.

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

- Historical, preview, continuation, and live lanes are available only for an
  installed query whose declared support and current admission allow that lane.
- Conditional providers and managed clocks are stable on the primary-graph
  application runtime. Query does not publish a separate general-purpose
  temporal workspace API.
- Recovery handles and temporal wake state are runtime-local. Durable restore
  belongs to the Store handoff; temporal wakes reconstruct from surviving
  authoritative domain truth rather than persisted wake handles.
- Product branches, exact composite history, retained observations, and pending
  cleanup are memory-resident. Process loss releases those live capabilities;
  restart durability requires Store-owned descriptive state followed by fresh
  owner readmission.
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
