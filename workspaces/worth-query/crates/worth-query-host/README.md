# worth-query-host

`worth-query-host` is the entry-band audience facade for applications and host
runtimes that install and execute Query behavior.

Use:

```rust
use worth_query_host::facade::{admission, domain, primary_graph, publication, runtime};
```

The host facade exposes the production Query authority graph without exposing
Query implementation modules, certification-only replay, or raw lower-runtime
internals.

## Contribution-Composed Applications

Pure domain value crates own their values and validation without importing Query.
Entry-local bindings associate those values with stable portable identities,
codecs, and declared units or frames; established wire identities stay stable.

Entry packages implement `ApplicationSchemaContribution<Schema>` to declare
members and `WorthQueryApplicationContribution<Schema>` to configure their
handlers and invariant factories. Each entry owns its `Configuration` type.
The root lists its contributions once; the macro carries that same list into
`ApplicationSchemaComposition::Contributions` and the host configuration tuple.

This example uses the two separately compiled entries in the
[public consumer fixture](../worth-query-certification/fixtures/consumer_entry/consumer_root/src/main.rs).
The configuration values, resource limits, authenticated principal, request
scope, and initializer are supplied by the host:

```rust,ignore
use worth_query_decl::facade::worth_query_application;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    application_installation as installation,
};
use worth_query_parameter_entry::{ParameterContribution, ParameterSchemaBinding};
use worth_query_topology_entry::{TopologyContribution, TopologySchemaBinding};

worth_query_application! {
    pub ConsumerSchema {
        owner: "worth.query.certification.consumer",
        version: (1, 0),
        contributions: [TopologyContribution, ParameterContribution],
    }
}
impl TopologySchemaBinding for ConsumerSchema {}
impl ParameterSchemaBinding for ConsumerSchema {}

let application = installation::in_memory(
    ConsumerSchema::declaration()?,
    (topology_configuration, parameter_configuration),
    limits,
    initial_state,
)?;
let request = application.request(&external_principal, &request_scope);
let outcome = request.mutate(planar_mutation).idempotency(command_id).execute();
let published = request.query(planar_read).execute()?;
```

`WorthQueryApplicationContributionSetup` resolves installed bindings internally.
An entry calls `setup.handler::<Binding, _>(handler)` and
`setup.invariant(reference, execution_point, factory)` for members it owns.
The sealed tuple traversal checks the complete installed contribution inventory
before invoking configuration. Missing handlers fail before `initial_state`;
invariant factories are validated and installed before it runs. The initializer
borrows the unpublished typed graph and its installed schema, seeds initial
state, and returns a typed graph installation result. Successful construction
returns `WorthQueryPrimaryGraphApplicationRuntime<Schema>`.

`WorthQueryInMemoryApplicationLimits::new` accepts World resources, an application
candidate profile, an application query profile, and a conditional evaluation
budget. Candidate cardinality, retained representation bytes, and validator work
remain distinct bounds. Installed handlers implement `decide`,
`candidate_requirements`, and `build_candidate`: decision reads retain facts,
reservation precedes candidate allocation, and the candidate is checked with its
affected untouched neighbors before atomic publication. A request does not retain
admission or a selected World between executions.

`application.discovery()` exposes declaration-derived `mutations()`, `queries()`,
`query_requests()`, and `fields()`. These describe portable input/result and typed
denial identities, scope/effects, units/frames, and installed request-binding
availability. Availability describes installation; every execution still performs
fresh authorization and currentness checks.

Committed mutation receipts expose `output_correspondence()` and
`committed_changes()`. The latter provides the exact `commit_reference()`, an
`entity_changes()` iterator of `(EntityId, RecordStructuralChange)`, and native
`lineage_events()`. Its constructor and canonical artifact are private; the view
exposes no field payloads or mutation authority. The
[replacement journey](../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/output_correspondence.rs)
demonstrates preserve/create/retire roles, same-commit lineage, readback, rejection,
and receipt recovery.

The executable configuration and resource setup live in
[consumer installation](../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/installation.rs),
with contribution inventory, ownership, and handler-completeness denials in
[contribution denials](../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/contribution_denials.rs).
The broader application API hardening milestone, 9.17.4, remains open.

## Ordinary Typed Query Entry

Declare each application read as an `ApplicationQueryBinding<Schema>`. The
binding owns its input binding, installed query, parameter and result bindings,
exact principal mapping, scope rule, stable identity, and finite result and
work ceilings. Register it with
`application_query_binding::<Binding>()` during schema authoring. Installation
then exposes the complete contract through
`installed_query_binding::<Binding>()`.

Application callers use the binding indirectly through its typed intent:

```rust
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;

let request = application.request(&external_principal, &request_scope);

let first = request
    .query(AccountActivityRequest::new(account_id))
    .execute()?;

let second = request
    .query(AccountActivityRequest::new(account_id))
    .execute()?;
```

The request borrows authentication and request-scope evidence but selects no
World state. Every `execute()` starts a fresh attempt: it selects the current
World product, resolves the authenticated principal through the binding's
installed mapping, resolves the declared scope, admits and executes the exact
installed query, and returns a `WorthQueryPublishedApplicationResult`.
Reusing the request therefore observes later lawful publications rather than
retaining an earlier selection.

Callers can request narrower finite ceilings before execution:

```rust
let published = request
    .query(AccountActivityRequest::new(account_id))
    .limits(maximum_results, maximum_work)
    .execute()?;
```

Widening either installed ceiling returns a typed `Limit` denial before World
selection or provider work. Other denial kinds preserve the failed boundary:
binding installation, product selection, principal resolution, scope
resolution, admission, or execution.

## Installed Domain Use

Host code may:

- install portable domain packages, application schemas, typed queries, and
  typed operations;
- register the exact providers and lower-runtime adapters required by installed
  meaning;
- obtain the installed application and domain handles;
- resolve authenticated principals through installed principal bindings;
- admit application queries and operations through current capability,
  purpose, disclosure, conflict, and graph-obligation evidence;
- execute managed provider sessions and consume typed terminal outcomes;
- consume fresh, recovered, partial-effect, and indeterminate commit outcomes;
- publish governed results and closed application-aftermath posture;
- bind an exact host predicate provider, named clock, and authoritative
  temporal-intent reconstruction contract before application-runtime
  publication;
- submit typed observations through a runtime-bound clock port while Signal
  owns wake eligibility and Query freshly admits the installed operation;
- reinstall derived temporal wakes from current authoritative domain truth and
  inspect non-authoritative lifecycle, work, and provenance evidence;
- inspect base-binding, complete runtime-installation, and fresh-admission
  canonical work through the carried clock, runtime-inspection, and provenance
  surfaces;
- dispatch declared external effects only from co-committed outbox facts;
- inspect, resolve, safely retry, dispose, or expire an exact receipt-bound
  runtime recovery handle;
- admit reconciliation or compensation against exact owner authority without
  claiming that Query executes the corrective effect;
- run ordinary installed workflow re-execution;
- inspect trace-bound lineage and request sparse promotion from an exact
  carrying publication.

Host code must not:

- create a second operating-world or application-authority root;
- call operation executors directly;
- expose raw Relational, Bridge, or Signal handles to application code;
- reconstruct Query authority from receipts, reports, projections, or digests;
- import certification replay through the host facade;
- construct lineage outcomes or promoted graph identities from raw identity
  material.
- treat acknowledgement, timeout, disconnect, or lost response as external
  completion;
- serialize a recovery handle or treat its opaque wire identity as live
  authority;
- teach `facade::provisional_aftermath` as accepted undo/redo support.
- schedule temporal work locally, return raw Signal decisions, or invoke a
  conditional operation directly.
- invent a host-local temporal binding or idempotency hash, or derive either
  again during commit.

## Application Readiness For Editors

An editor or transport host that owns an installed primary-graph application
may inspect its current descriptive basis before presenting or submitting
work:

```rust
let readiness = application.inspect_application_readiness()?;
let optimistic_basis = readiness.basis_token();
```

The snapshot identifies the installed schema binding and current Query basis.
Query releases the inspection lease before returning it. The basis token is an
optimistic transport precondition only: it carries no query, mutation,
installation, or basis authority. Execution must still enter through the typed
Query application adapter, which performs fresh authorization, projection,
admission, and currentness checks.

## Related Docs

- [Ordinary Application Front Door](../worth-query/docs/foundations/ordinary-application-front-door.md)
- [WORTH Query Orientation](../worth-query/docs/AI_README.md)
- [Application Authorization And Emergency Elevation](../worth-query/docs/capabilities/application-authorization-and-emergency-elevation.md)
- [Runtime-Installed Domains And Operations](../worth-query/docs/domain-capabilities/runtime-installed-domains.md)
- [Canonical Graph Obligation Progression](../worth-query/docs/domain-capabilities/canonical-graph-obligation-progression.md)
- [Conditional Installed Operations](../worth-query/docs/domain-capabilities/conditional-installed-operations.md)
- [Installed Operation Re-Execution And Replay](../worth-query/docs/domain-capabilities/installed-operation-reexecution-and-replay.md)
- [Typed Stops And Remediation Guidance](../worth-query/docs/domain-capabilities/typed-stops-and-remediation-guidance.md)
- [Installed Operation Lineage And Promotion](../worth-query/docs/domain-capabilities/installed-operation-lineage-and-promotion.md)
- [Application Aftermath, External Effects, And Recovery](../worth-query/docs/execution/application-aftermath-and-recovery.md)
