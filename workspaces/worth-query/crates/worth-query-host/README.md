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
members and `WorthQueryApplicationContribution<Schema>` to declare required
producer and conditional contracts and configure their handlers, invariant
factories, providers, and conditionals. Each entry owns its `Configuration` type.
The root lists its contributions once; the macro carries that same list into
`ApplicationSchemaComposition::Contributions` and the host configuration tuple.

The [public consumer fixture](../worth-query-certification/fixtures/consumer_entry/consumer_root/src/main.rs)
checks separately compiled contributions and static program validation. The
[ordinary product workflow](../worth-query-certification/examples/ordinary_product_workflow.rs)
installs a validated program with `application_installation::in_memory_program`,
admits its exact operation, publishes through `commit_for_program`, reads the
successor, and closes conditional resources. Its source is the executable host
entry example.

`WorthQueryApplicationContributionContracts` and
`WorthQueryApplicationContributionSetup` resolve installed bindings internally.
An entry declares `contracts.producer::<Binding>()` and
`contracts.conditional::<Binding>()`, then configures its owned members with
`setup.handler::<Binding, _>(handler)`,
`setup.invariant(reference, execution_point, factory)`,
`setup.producer::<Binding>(provider)`, and
`setup.conditional::<Binding>(configuration)`.
The sealed tuple traversal checks the complete installed contribution inventory
before invoking configuration. Missing handlers fail before `initial_state`;
producer applicability and required invariant closure, conditional dependencies,
and invariant factories are validated and installed before it runs. The initializer
borrows the unpublished typed graph and its installed schema, seeds initial
state, and returns a typed graph installation result. Successful program
construction returns `WorthQueryProgramApplicationRuntime<Schema, Program>`.

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
`committed_changes()`. Output roles carry the binding, entity marker and
preserve/create/retire action. `output_correspondence().entity(role)` checks
those exact types and the role name against the committed association. It
returns `WorthQueryApplicationOutputProjectionDenial::EntityMismatch` for a
different entity marker even when the binding, name and action match; foreign
bindings, missing roles and action mismatches have their own typed denials.
Role names describe correspondence; the platform resolves persistent identity.

Bindings with a finite result shape declare exact roles in `ROLES`. Bindings
whose result cardinality follows the authored topology declare typed namespaces
in `ROLE_FAMILIES`, including the entity marker, allowed action postures and
minimum member count. A handler constructs each source-derived member with
`WorthQueryApplicationOutputRole::try_new(format!(...))` and passes that token
directly to `create_output`, `preserve_output` or `retire_output`. Query rejects
empty, ambiguous and oversized runtime names, validates each member against the
installed family, and seals the resolved identity in the same correspondence.
`from_static` remains the constructor for exact compile-time role names.

`committed_changes()` provides the exact `commit_reference()`, an
`entity_changes()` iterator of `(EntityId, RecordStructuralChange)`, and native
`lineage_events()`. Its constructor and canonical artifact are private; the view
exposes no field payloads or mutation authority. The
[replacement journey](../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/output_correspondence.rs)
demonstrates preserve/create/retire roles, exact entity-affinity denial,
same-commit lineage, readback, rejection, and receipt recovery. Receipt clones
and idempotent recovery retain the observations without recreating the single-use
performed product-change capability.
The [cycle publication journey](../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/publication.rs)
demonstrates variable source-derived create roles through the installed public
contract and typed post-commit projection.

The executable configuration and resource setup live in
[consumer installation](../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/installation.rs),
with contribution inventory, ownership, and handler-completeness denials in
[contribution denials](../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/contribution_denials.rs).
The synchronous application foundation and branch-local program-evolution path
are complete. The separately governed dynamic workflow-definition product
remains outside this surface.

## Branch-Local Program Evolution

One host may roster multiple validated application programs over the same
installed native contracts. A request enters the public path with
`application.request(&principal, &scope).on_branch(branch).programs()`:

```rust,ignore
let programs = application
    .request(&principal, &scope)
    .on_branch(branch)
    .programs();
let inspection = programs.inspect()?;
let requirements = programs.compare(&target_revision)?;
let prepared = programs
    .adopt(&target_revision)
    .requirements(&requirements)
    .prepare(maximum_selection_work)?;
let outcome = prepared.publish();
```

Inspection and semantic comparison are descriptive. Preparation consumes exact
branch, source-program, target-support, target-rule, migration, resource, and
custody evidence. Only World's performed publication changes the branch's
selected program. Unpublished outcomes retain exact recovery authority; raw
revision text, a World handle, or a receipt cannot substitute for it.

`application.branches().program_adoption_coverage(...)` issues bounded live
coverage for explicit non-atomic broader adoption. The caller may order that
exact set, but cannot add branches or claim rollback of a performed prefix.
Support removal is separate:
`WorthQueryProgramApplicationRuntime::retire_program_support` succeeds only
after current branches, retained interpretations, and mandatory custody reach
zero, and its inventory reports stable retained program bytes.

## Output Demand, Exact Observation, And Live Reads

An installed producer binding associates one declared output family with an
operation, output role, applicability table, required invariants, and resource and
reuse policies. Its provider supplies typed operation input, idempotency, and finite
work and retained-byte demand. Conditional contribution bindings can attach actual
output-readiness delivery to the producer's performed publication, so source
success alone does not imply output readiness.

Application code starts bounded synchronous work with
`request.demand(demand).controls(controls).start()`. Calling
`advance(&fresh_request)` yields `Pending` or `Settled`; settlement exposes its
receipt, observed source, exact read observation, and readiness delivery. The
handle exposes owner notifications and must be closed when its interest ends.
Source drift returns `Superseded`, and installed applicability must select exactly
one producer.

Current reads use `request.query(intent).execute()`. Exact reads use
`request.retain_read()` followed by
`request.at(&observation).query(intent).execute()`, with fresh principal and scope
admission against the retained occurrence. A live-capable query opens with
`request.query(intent).subscribe(WorthQueryApplicationLiveLimits::bounded(...))`;
each `next(&fresh_request)` rechecks the application and branch before delivery,
and `close()` releases the lease. Retained requests cannot open live subscriptions.

Published rows expose `observed_sources()`. Mutations whose bindings require exact
source evidence call `.expect_source(observed_source)` before idempotent execution.
The source-local comparison rejects missing, foreign, retired, ABA-changed, or
changed sources while allowing sibling edits outside the recorded footprint.
Sibling membership and selector-field changes can still invalidate that footprint.
Bindings with input-selected subjects implement `expected_source_parameters` so
the same Query boundary rejects mismatched selectors for rows, result sets and
framework producers. A source query's type alone does not bind its parameters to
the mutation input.

## Typed Handler And Invariant Access

Installed mutation handlers use `DecisionReader` for tracked typed field and
relation reads and `CandidateWriter` for the one reserved effect program. The
writer exposes create, initialize, write, link, unlink, delete, emit, and output
role operations. Existing entities without a domain identity field cross the
phase boundary through `DecisionReader::mutation_target` and
`CandidateWriter::projected_entity`; Query checks the installed projection
authority and completed attempt read set before returning a program-affine target.

Invariant factories resolve installed typed field and relation bindings. Their
proposed and committed views expose decoded fields and complete bounded relation
traversals while enforcing binding, prepared scope, entity kind, declared access,
and work budgets. Invalid values, missing required fields, and truncated traversal
are typed denials rather than empty successful observations.

Certification cost observations are intentionally exposed by
`worth-query-replay`, not this host facade. Ordinary host code cannot turn those
diagnostics into execution authority.

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
- inspect and adopt a rostered application program on an exact product branch,
  progress owner-issued branch coverage, recover unpublished adoption, and
  retire unused program support through typed custody inventories.

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
