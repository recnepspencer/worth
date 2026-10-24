# worth-query-host

`worth-query-host` is the entry-band audience facade for applications and host
runtimes that install and execute Query behavior.

Use:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
use worth_query_host::facade::{admission, domain, primary_graph, publication, runtime};
```

The host facade exposes the production Query authority graph without exposing
Query implementation modules, certification-only replay, or raw lower-runtime
internals.

**Writing an application?** Start with
[Build an Application](../../../../docs/build-an-application.md). It shows the real calls, in order: install
the application graph (`application_installation::in_memory_program`), run
program-owned actions (`retain_operation`, then fresh-request `advance`), install a workflow vocabulary
and publish, start, and advance workflows, and adopt a new program on a
branch. This README covers the host's deeper contracts.

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
advances a declared source-only owning action, reads the
successor, and closes conditional resources. Its source is the executable host
entry example.

The [authored workflow](../worth-query-certification/examples/authored_workflow/main.rs)
authors a definition around one reusable review component, publishes and discovers
it, rejects a proposal so the retry asks for a revision, approves and applies the
revised effect, then publishes a second revision; a start from the superseded
revision is refused naming the current one.

For an ordinary installed feature, start with the Host facade's
[complete feature definition](../../../../docs/build-an-application.md#complete-feature-definitions):
`worth_query_application_contribution! { pub feature Definition<S: Bound> for Feature => Runtime { ... } }`.
Its typed `action` and `required_pair` members project schema access, feature
specs, output roots, contracts and runtime slots through the existing owners.
Supply actual handler/provider/readiness values to the generated runtime;
`WorthQueryFeatureDefinition::contracts` and `configure` install that same
definition. The runnable
[required-feature example](../worth-query-certification/fixtures/consumer_entry/topology_entry/examples/required_feature.rs)
uses both an ordinary action and a required production pair.

The lower-level `WorthQueryApplicationContributionContracts` and
`WorthQueryApplicationContributionSetup` remain available for specialized
composition and resolve installed bindings internally. A required Initial and
Preserve member uses one
`WorthQueryRequiredFeaturePair<Schema, Feature, Initial, InitialReadiness,
Preserve, PreserveReadiness>` alias. The complete definition projects that alias
through the existing feature-spec, contract and setup owners. When using those
owners directly, use the same alias in `.required_pair::<Roles>()`,
`contracts.feature::<Roles>()?` and `setup.feature::<Roles>(runtime)?`.
Construct the two role values with
`WorthQueryFeatureRoleRuntime::new(handler, provider, readiness_configuration)`
and combine them with `WorthQueryFeatureRuntime::new(initial, preserve)`.
The producers' associated types supply operation, provider, family and output
role; the carrier derives each readiness owner's producer association. Keep its
predicate and native output-readiness binding at the existing readiness owner.
Do not maintain separate paired producer/conditional registration lists or copy
that association into `REQUIRED_PRODUCERS`.

The [required-role guide](../../../../docs/build-an-application.md#required-initial-and-preserve-output-roles)
and runnable [required-feature example](../worth-query-certification/fixtures/consumer_entry/topology_entry/examples/required_feature.rs)
show the whole program, configuration and installation path. A standalone
conditional without a required production pair still uses
`contracts.conditional` and `setup.conditional`; independent handlers and
invariants keep their existing setup methods. For operation access, use one
typed `worth_query_operation!(... access { ... })` declaration and install it
with `schema.declared_operation::<Operation>(definition)`; see
[operation access](../../../../docs/build-an-application.md#21-declare-operation-access-once).
The sealed tuple traversal checks the complete installed contribution inventory
before invoking configuration. Missing handlers fail before `initial_state`;
producer applicability and required invariant closure, conditional dependencies,
and invariant factories are validated and installed before it runs. The initializer
borrows the unpublished typed graph and its installed schema, seeds initial
state, and returns a typed graph installation result. Successful program
construction returns `WorthQueryProgramApplicationRuntime<Schema, Program>`.

`WorthQueryInMemoryApplicationLimits::new` accepts World resources, an application
query profile and a conditional evaluation budget. Installed handlers implement
`decide` and `build_candidate`; computing handlers override `prepare_decision`
and return the installed computation's `decision_plan`. Decision reads retain
facts, and `CandidateWriter` stages the actual changes for invariant validation
and atomic publication. Handlers supply no candidate-size or validator-work quote.
A request does not retain admission or a selected World between executions.

## Owning fixed and source-only actions

Install a program and declare the exact typed action occurrence and its delivery.
`request.mutate(intent)` returns `WorthQueryOperationBuilder`; it exposes original
source preparation, preconditions and fallible `idempotency_owned(key)`, followed by
`retain_operation::<Program, Action>(&runtime)`. Fixed and source-only callers
cannot execute, prepare or recover through that builder or its borrowed-key view.
The selected action carries composition instance, feature, binding and delivery;
a binding label alone cannot select its contract.

Declare each ordinary action with `.action::<Binding, Delivery>()`, its applicable
metadata clauses, and `.finish_action()`. Metadata cannot be overwritten and
must not be split into another action to attach delivery. The sealed
`ApplicationActionReference` selects the exact occurrence; Installation resolves
the full actual action, including metadata, for later comparison.

Fixed, discovered and SourceOnly owners retain the authentic installed runtime
allocation. Fixed/discovered returned owners have no construction-runtime
lifetime and can be stored after the constructing borrow ends or handle is
dropped. Keep a separate genuine runtime handle to create fresh requests;
operations expose no runtime accessor or `Deref` as an admission entrance.
Native source/output/cleanup obligations remain move-only.

Bind workflow vocabulary and other unique-runtime configuration before cloning
handles or retaining operations. Every `retain_workflow_spec` refusal preserves
the original runtime, installed spec and signing owner in
`WorthQueryWorkflowRuntimeBindingRefusal`; inspect `denial()` and consume
`into_parts()` to recover them plus the denial. `support_workflow_spec` returns
`Err((original_spec, denial))` while its borrowed runtime remains owned by the
caller. Runtime close refuses `RuntimeKeepaliveRetained` while another genuine
handle or operation retains the allocation. Release those keepalives before
retrying configuration or runtime close; successful operation close releases its
own keepalive, while refused close retains the whole owner.

Capture the original meaning once. Supply a fresh authenticated ordinary request
for every `advance`, delivery and close. Native admission checks current authority
and the original runtime/branch/program affinity. The descriptor is not a grant.
Recovery promotes the original native carrier; registered replay rejoins its
original outputs without running the source handler. Recovered evidence does not
manufacture the original handler result.

A fixed settled owner borrows its complete settlement for `read_current` or a
finite `read_current_with_policy`; the currentness check and declared query share
one observation and one native phase. Read refusal retains the owner. Stale
settlement supports `read_historical` and `close`, never output advancement.
No-projection delivery uses `deliver_current` without a dummy query. Source-only
completion is distinct from required-output settlement; retained delivery selects
the action's authentic original commit observation.

Completion and delivery retain independent cleanup. `close` returns the complete
owner with a typed denial when cleanup refuses. Drop is abandonment, not external
cleanup or crash durability. Only existing native checkpoint/recovery contracts
survive process loss.

The [compiled original descriptor capture](../../../../docs/build-an-application.md#capture-complete-original-meaning) is exercised by the real
public result-set court.

The real result-set court calls the compiled capture with nondefault preconditions before
an actual post-effect append failure. It reads the resulting source and outputs
independently after native recovery. No original intent clone is required per pump.

### Owning discovered delivery and deliberate native control

Ordinary discovered actions use the same `retain_operation::<Program, Action>`
entry with a declared discovered delivery. Keep the returned move-only operation
on progress and refusal and supply fresh authority on each `advance`. Native
partial recovery, committed-retention retry, registration and fanout stay inside
that owner. A settled owner checks all original discovery facts and every output
settlement at the same retained observation and phase as the declared scalar or
batch read. Explicit no-projection delivery has no query selector.

Current delivery refusal preserves the settlement, original descriptor and
cleanup. Historical reads use its original declared selector and retained basis
with fresh authorization; they never claim Current. Fresh handler results remain
distinct from recovered native evidence, and registered replay never reconstructs
a result. Terminal custody has no advance entrance and cannot abandon unresolved
native obligations through a successful close.

For independent native custody tests, select `request.advanced().mutate(intent)`
before `execute_performed_discovered`, unpublished recovery or promotion.
Registered native recovery also starts at `request.advanced()`. These controls
preserve actual partial/source owners and return them on refusal; they are not
ordinary business retry helpers. See [owning discovered actions](../worth-query/docs/foundations/ordinary-application-front-door.md#owning-discovered-actions).

### Advanced fixed publication and recovery

Use the [ordinary owning entry](#owning-fixed-and-source-only-actions) for fixed actions.
The current native builder lives under `application_entry::advanced` and is reached
with `request.advanced().mutate(intent)`. Manual output start and registered fixed
recovery require `request.advanced()` as well.

Fixed required roots use `execute_performed`, which now owns its original
preparation independently of the starting request and runtime borrow. Start and
advance it with an explicit program runtime and fresh request. A genuine
`ProductUnpublished` fixed source retains its typed demand and native partial;
`recover_unpublished_required_in_program` and `promote_recovered_required_outputs`
perform the same checked native transition and original-carrier handoff as the
discovered lane. The promoted starter is result-free: it does not recreate the
original handler result. All independent read, publication and cleanup results
remain owned by the returned recovery result. A committed retention refusal
keeps its original result, preparation and carrier for a fresh exact-key
`retry_required_output_retention`; it is not an unpublished outcome.
The `RequiredOutputDenied` variant carries an opaque
`WorthQueryRequiredOutputRetentionFailure`. Its borrowed receipt, result and
denial accessors leave the original demand and native custody paired; pass the
whole failure to the retry entrance, which returns complete custody on refusal.

Both output-source lanes retain unsupported `Indeterminate` outcomes with
their original preparation in non-actionable `Blocked` custody. They are not no-effect outcomes or evidence that recovery
finished. Workflow recovery remains outside these entrances.

> **Ordinary owning actions.** Fixed, discovered, source-only and guarded workflow actions use
> `retain_operation` and fresh-request advancement, delivery and close. Deliberate
> native custody controls require `advanced()`. Guarded workflow actions retain
> their original issued requirement in an ordinary owning operation.

A native candidate-preparation refusal before product publication is a typed
`Denied` outcome, with its original native error available through
`native_preparation_error()` on the application denial. That error retains its
context and commit log; diagnostic `Debug` does not dump the log. Native
cancellation and deadline refusal remain `Cancelled` and `TimedOut`. Deferred,
durability-unresolved and uncertain publication outcomes retain their existing
recovery obligations; preparation classification does not turn them into denials.

Ordinary graph demands use `WorthQueryOutputDemandControls::default()` and inherit
the installed host policy. Configure that policy once with
`WorthQueryInMemoryApplicationLimits::with_output_demand_resources(...)` and
`WorthQueryOutputDemandResourceProfile`; currentness work, producer work, producer
retained bytes, and settlement attempts are separate dimensions. Explicit caller
controls and child artifact limits only narrow the relevant dimensions. A handle's
`settle(&request)` owns bounded progression and reports `Pending` when necessary;
applications do not implement retry counts in query-work units.

Before delivering stored output values, select one fresh retained observation.
Its `require_current_output_demand` checks a direct root/dependent settlement;
`require_current_program_output` checks the whole program settlement. Both require
native source lineage at that same observation, including receipt-free checkpoint
reuse. Read the descriptive fields through that retained request after admission.

Checkpoint capture retains the selected native output's producer/source locator
even when its cached Ready demand has been reclaimed. A locator without accepted
source facts is prior-output custody only: after reopen the installed Preserve
producer must read and revalidate its inputs before current delivery. Capturing a
stored output does not establish that it is current. Family publication order
selects the active output, so an older Initial row cannot displace its Preserve
successor or a distinct output family.
Capture keeps accepted payloads only for that selected native family head, across
both newly accepted and recovered records. Canonical checkpoint row order is not
publication order. After fresh installation, an ordinary `current_output` read
can verify the retained head before any output demand. Its source facts and
native output witness must still agree with the selected observation; changed
tracked source facts or native output evidence deny the stale publication rather
than reviving an older Initial row.
Earlier checkpoints that retained competing bindings without their publication
order do not acquire ordering proof from a new reader. This capture rule does not
retrospectively reconstruct lost lineage or guarantee repair of those archives.
Capture admits the required native prior locators before best-effort reuse facts.
If those optional facts exhaust their remaining allowance, the checkpoint retains
prior custody and the output starts fresh after reopening.

Capture and transition installation require an explicit
`application_installation::WorthQueryCheckpointCapturePolicy`. Choose
`SystemAllocation` for fallible uncharged storage or `Execution(&lease)` for
admitted final-frame payload backing. The same immutable bytes and their one
charge survive Query clones and embedded native-region ownership. The last
byte owner frees the backing before releasing its charge; imports and native
codec temporaries remain uncharged. No exhausted or stopped lease falls back
to system allocation. Typed capture denials retain native durability errors or
the lower physical refusal and available checked payload quote.

Every consuming `repair_to_checkpoint(policy)` attempt selects its policy anew.
An early stopped policy preserves the unpublished native settlement phase.
Capture refusal after acknowledgment retains the exact successor in its repair
capsule, with the current cause available through `capture_denial()`. Repair
never reruns authoring and returns no World; ordinary target admission still
authenticates a successfully captured successor.

Query frames a captured native checkpoint directly in its final byte buffer,
preserving the format-8 wire layout and checksum. Capture reports
`CheckpointSizeOverflow` when encoded lengths cannot fit this host or wire
format, and `CheckpointAllocationUnavailable` when reserving that buffer fails.
Both are write failures and do not suggest repairing the saved store. This
removes the intermediate full Query body copy; native capture, accepted fact
payloads, boxed-slice conversion and host compression have separate allocations.
An internal difference between reserved and emitted frame size reports
`CheckpointFrameSizeMismatch` before any checkpoint is returned.

A family read rejects an older candidate when its own recorded facts or native
output witness prove it changed, even if its upstream is pending. Unchanged own
evidence still returns `PendingUpstream`; it never establishes upstream currentness.
This lets a value-changing Preserve settlement remain selectable without an older,
conclusively stale Initial candidate hiding it.

A required refresh executes under the mode issued by its accepted predecessor.
After its exact Current join, an existing caller can follow that completed
refresh across selected-program and program-output modes. The caller retains
its own advance authority; unfinished execution still requires the successor
mode and installed producer edition to match.

Ordinary mutation bindings carry scope, source and output meaning; they do not
ask authors for candidate-size estimates. `CandidateWriter` checks each actual
change against the declared operation, and publication validates the candidate
and affected untouched neighbors. Native request controls, allocation failure,
currentness and invariant checks still apply.

`application.discovery()` exposes declaration-derived `mutations()`, `queries()`,
`query_requests()`, and `fields()`. These describe portable input/result and typed
denial identities, scope/effects, units/frames, and installed request-binding
availability. Availability describes installation; every execution still performs
fresh authorization and currentness checks.

Committed mutation receipts expose `outputs_of::<Contract>()` and
`committed_changes()`.

An output role is a marker type, and its `WorthQueryApplicationOutputRole` impl
is the declaration. The impl names the schema, the output contract, the entity
marker, the action (`WorthQueryPreserveOutput`, `WorthQueryCreateOutput` or
`WorthQueryRetireOutput`), the cardinality (`WorthQueryExactlyOneOutput` or
`WorthQueryAtMostOneOutput`) and the role name. The sealed
`WorthQueryApplicationDeclaredOutputRole` derives the role's `DESCRIPTOR`, which
the contract lists in `ROLES`. A role belongs to its contract, so one marker
serves every binding whose `Output` is that contract:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
pub struct CreatedAccountOutput;

impl WorthQueryApplicationOutputRole for CreatedAccountOutput {
    type Schema = BankSchema;
    type Contract = CreateAccountOutputs;
    type Entity = Account;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "account";
}

impl ApplicationMutationOutputContract<BankSchema> for CreateAccountOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<CreatedAccountOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}

candidate.create_output::<CreatedAccountOutput>(&created)?;
let account = receipt
    .outputs_of::<CreateAccountOutputs>()?
    .entity::<CreatedAccountOutput>()?;
```

A generic schema declares a generic marker,
`struct AnchorOutput<Schema>(PhantomData<fn() -> Schema>)`, with one impl that
covers every schema.

Every use names the marker, and a use that disagrees with the contract fails to
compile. `create_output`, `preserve_output` and `retire_output` bound the role's
contract to the handler's binding and its action to the writer.
`DecisionReader::prior_output::<PriorBinding, Role>` bounds the contract to
`PriorBinding`. A producer's `type OutputRole` must be an exactly-one role of its
operation's contract, for the entity its output family names. Each use also
evaluates the derived `DECLARED` constant, which fails unless the contract's
`ROLES` lists the role with the same name, entity, posture and cardinality.
`DECLARED` is evaluated when the use is compiled to code, so that error appears
in `cargo build` and `cargo test` but not in `cargo check`, and not in a generic
function nothing instantiates.

A commit receipt and an output-demand settlement store their outputs erased, so
they can be cloned, archived, recovered and readmitted without a type
parameter. `outputs_of::<Contract>()` is the one typed read: it checks once, at
run time, that the commit was made under `Contract`, and refuses any other
contract with `ForeignContract`. It returns
`WorthQueryApplicationTypedOutputCorrespondence<'_, Contract>`, whose `entity`,
`member` and `family_entries` bound each marker's contract to `Contract`, so a
read of another contract's role or family fails `cargo check`.

The cardinality decides the shape of every read. An exactly-one role must be
bound: a completed candidate that leaves it unbound is refused with
`MissingOutputRole`, and its reads return the output itself. The reads are
`outputs_of::<Contract>()?.entity::<Role>()`, `DecisionReader::prior_output` and
generated-output reconstruction's `entity::<Role>()`. An at-most-one role may be
left unbound, and the same reads return an `Option`, `None` when the commit left
the role unbound. Absence is a value, never a denial. Binding any fixed role a
second time is refused with `DuplicateOutputRole`. The cardinality is part of
the schema's canonical identity and of its portable and archived descriptions,
so changing it changes the installed schema. An optional single output is always
an at-most-one role, never a family with a minimum of zero. A producer names an
exactly-one role because a commit may omit an at-most-one role.

Generated-output reconstruction can contain newly generated children and preserved
roots. `output::<Role>()` and `output_member::<Family>(suffix)` return
`WorthQueryReconstructedOutputEntity::Generated` or `Retained` from the installed
correspondence. Generated handles claim the exact suspension manifest and admit
field reconstruction. Retained handles carry only the exact live identity and
kind outside that custody and admit relation endpoints. Reconstruction entry
validates preserved identities against the owner-admitted suspension basis;
restoration still requires the complete generated manifest and fresh publication
admission. Retired roles are refused. The narrower `entity` and `member` methods
remain available when a consumer requires created payload. An all-retained current
output has nothing to suspend: `NoGeneratedPayload` is a typed no-effect
qualification, and its current read surface remains available.

Bindings whose result cardinality follows the authored topology declare
families. A family is a marker type whose
`WorthQueryApplicationOutputRoleFamily` impl names the schema, the contract, the
entity marker, the member-name `PREFIX`, the allowed `POSTURES` and the
`MINIMUM` member count. The contract lists its derived `DESCRIPTOR` in
`ROLE_FAMILIES`. A handler binds each source-derived member by its suffix, as in
`candidate.create_member::<CreatedVertices>(&key, &created)?`; `preserve_member`
and `retire_member` work the same way. Readers name the family and the action,
as in `outputs.member::<CreatedVertices, WorthQueryCreateOutput>(&key)` on the
view `outputs_of` returns, or read the whole family with
`outputs.family_entries::<CreatedVertices>()`. Members are
exactly-one. A family the contract does not declare exactly as used, or an
action outside the family's postures, fails to compile. The suffix is run-time
data, so an empty, ambiguous or oversized member name is refused when it is
named.

Query keeps run-time checks only where the types cannot carry the contract.
A stored receipt carries no contract type, so `outputs_of::<Contract>()` refuses
a commit made under another contract with `ForeignContract`; reads on the view
it returns are checked at compile time. A family member's committed action
is data, so reading it with another allowed action is `ActionMismatch`. Portable
and readmitted forms are validated when admitted. Typed uses are checked again at
run time as defense in depth. Role names describe correspondence; the platform
resolves persistent identity.

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
The synchronous application foundation, branch-local program evolution, and
authored workflow definitions (see the authored workflow example above and the
[workflows guide](../worth-query/docs/foundations/workflows.md)) are
available through this facade. An installed inbound source can complete the
exact external-effect owner; `await_inbound` reads that result on fresh
workflow advance. No callback or resume message advances an instance itself.

## Branch-Local Program Evolution

One host may roster multiple validated application programs over the same
installed native contracts. A request enters the public path with
`application.request(&principal, &scope).on_branch(branch).programs()`:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let programs = application
    .request(&principal, &scope)
    .on_branch(branch)
    .programs();
let inspection = programs.inspect()?;
let requirements = programs.compare(&target_revision)?;
let prepared = programs
    .adopt(&requirements)
    .prepare(maximum_selection_work)?;
let outcome = prepared.publish();
```

Migrations, workflow dispositions, and every outcome are shown with real code
in [Build an Application §7](../../../../docs/build-an-application.md#7-adopt-a-new-program-on-a-branch).

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
When a handler reads a current output, Query merges its retained source facts
with the admitted query facts. Adjacency reads at the same native structural
revision combine their endpoint coverage and comparison limits; different
revisions still deny the attempt. A producer's own relation writes are rebased
at the committed snapshot before becoming reusable output evidence.
Bindings with input-selected subjects implement `expected_source_parameters` so
the same Query boundary rejects mismatched selectors for rows, result sets and
framework producers. A source query's type alone does not bind its parameters to
the mutation input.

## Dependent Managed Pair Reads

Cold pair reconstruction has two read topologies. When both scopes are known
before reading, prepare `WorthQueryDerivedPairReadPlans::new(first, second)`
and call `reconstruct_managed_derived_collection_pair`. For plans admitted on
a shared native read loan, use `WorthQueryDerivedPairReadPlans::in_batch`;
Query checks each issued item against that exact loan before dispatch.

When the second scope comes from the first observed row, use
`reconstruct_managed_derived_collection_pair_dependent_in_batch`. Its two
callbacks receive a `WorthQueryDependentPairReadContext`. Each callback resolves
its local scope, admits one ordinary plan using the issued advancement phase,
and calls `read.execute(plan)` before those local borrows end. Query keeps the
result, executes both branches on the calling thread, and holds their native
loan claims through link/source validation and projection:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let execution_request = phase.checked_execution_request_for(runtime.product_runtime())?;
let keys = runtime.reconstruct_managed_derived_collection_pair_dependent_in_batch(
    execution_request,
    &view,
    &product,
    &membership,
    members,
    &batch,
    |member, read| {
        let identity = resolve_member(member)?;
        let plan = admit_first(&phase, &identity, &batch)?;
        read.execute(plan)?;
        Ok(())
    },
    |first_row, read| {
        let identity = resolve_linked_scope(first_row)?;
        let plan = admit_second(&phase, &identity, &batch)?;
        read.execute(plan)?;
        Ok(())
    },
    first_link,
    second_link,
    project,
)?;
```

Inside host advancement, carry that admitted request or narrow its active lease
with a genuine child. A separately created request has unrelated custody and is
refused before reconstruction.

The two plans must name the registered public queries at the selected product's
exact native basis and use the same authenticated request scope. The first
query's scope and source root must equal the membership root; the second may
read a distinct linked root. Domain link projections must agree. Query validates
both row and result-set provenance, visits roots in EntityId order, and installs
one image only after all pairs and the final request/currentness check succeed.
Wrong plans, scopes, links or loans, cancellation, expiry and resource exhaustion
preserve the existing image; observing a newer product may first make it cold.

Read work is charged to both the shared read allowance and the caller's execution
request as it occurs. Owner staging and final image custody retain their existing
Query and Execution owners. This cold serial journey does not establish parallel
worker equivalence or replace warm reconciliation and entry refresh.

## Typed Handler And Invariant Access

For literal guards and assignments on required scope fields, prefer an
[executable scope-field body](../worth-query/docs/authoring/scope-field-operation-bodies.md).
Attach its completed typed body to the operation definition, bind that schema
into program authoring, and call `setup.operation_body::<Binding>()`. Query owns
the installed values and native execution; ordinary handler registration cannot
replace them. This bounded form does not yet cover creations, general expressions,
audit construction, requirements or command composition. Advanced handlers remain
honest advanced operations, not a way to claim those missing declarative joins.

Installed mutation handlers use `DecisionReader` for tracked typed field and
relation reads and `CandidateWriter` for the one reserved effect program. The
writer exposes create, initialize, write, link, unlink, delete, emit, and output
role operations. Existing entities without a domain identity field cross the
phase boundary through `DecisionReader::mutation_target` and
`CandidateWriter::projected_entity`; Query checks the installed projection
authority and completed attempt read set before returning a program-affine target.

For a variable prior carrier, `DecisionReader::field_with_predecode_admission`
admits owner work/storage on its original borrowed scalar before the declared
decoder runs. Its callback also borrows the same request checkpoint. See
[Tracked scalar predecode admission](docs/scalar-predecode-admission.md) for the
typed outcomes, allocation responsibility and staging complexity contracts.

For repeated equality predicates on one declared target, call
`DecisionReader::prepare_entity_selection(field)` once, then
`select_entities_prepared(&prepared, value, candidate_limit)` or
`resolve_optional_entity_prepared(&prepared, value)`. The opaque token retains
the selected native root and installed index generation. It belongs to that
exact runtime, snapshot, schema binding and operation admission; it grants no
latest-head authority. Every value still performs its exact native comparison
and retains a complete membership or absence predicate for commit and recovery.
Each call uses the current reader's cancellation, deadline, allocation policy
and finite selection/work contract. Preparation changes no installed declaration
or serialized source-fact meaning. Native temporary comparison-key and result
buffers retain their existing allocation ownership; the token does not claim
physical admission of those buffers.

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

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
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

Ordinary query bindings declare result cardinality, not engine cost arithmetic.
The installed `WorthQueryApplicationQueryResourceProfile` controls simultaneous
graph work through `with_maximum_concurrent_graph_work(NonZeroUsize)`.
It supplies no per-query work or byte estimate. Native execution still observes
the request's cancellation and deadline.

Callers can narrow result cardinality before execution, using a `NonZeroUsize`:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let published = request
    .query(AccountActivityRequest::new(account_id))
    .limit_results(maximum_results)
    .execute()?;
```

Exceeding the installed binding's result ceiling returns a typed `Limit` denial
before World selection or provider work. Other denial kinds preserve the failed boundary:
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
- install an inbound verifier for the exact declared operation, receive through
  `facade::application_entry::WorthQueryApplicationInboundOccurrencesExt`, and
  continue accepted custody within its installed work limits;
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

Use the [request and retained-operation guide](../../../../docs/capability-index.md#use-requests-and-retained-operations)
for submission, advancement, recovery and cleanup.


## Application Readiness For Editors

An editor or transport host that owns an installed primary-graph application
may inspect its current descriptive basis before presenting or submitting
work:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
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

- [Build an Application](../../../../docs/build-an-application.md): the centerpiece guide
- [Programs And Adoption](../worth-query/docs/foundations/programs-and-adoption.md)
- [Workflows](../worth-query/docs/foundations/workflows.md)
- [Ordinary Application Front Door](../worth-query/docs/foundations/ordinary-application-front-door.md)
- [Capability Index](../../../../docs/capability-index.md): find the facade call for a task
- [Query Engine Architecture Map](../worth-query/docs/AI_README.md): engine-internal, for Query maintainers
- [Application Authorization And Emergency Elevation](../worth-query/docs/capabilities/application-authorization-and-emergency-elevation.md)
- [Runtime-Installed Domains And Operations](../worth-query/docs/domain-capabilities/runtime-installed-domains.md)
- [Canonical Graph Obligation Progression](../worth-query/docs/domain-capabilities/canonical-graph-obligation-progression.md)
- [Conditional Installed Operations](../worth-query/docs/domain-capabilities/conditional-installed-operations.md)
- [Installed Operation Re-Execution And Replay](../worth-query/docs/domain-capabilities/installed-operation-reexecution-and-replay.md)
- [Typed Stops And Remediation Guidance](../worth-query/docs/domain-capabilities/typed-stops-and-remediation-guidance.md)
- [Installed Operation Lineage And Promotion](../worth-query/docs/domain-capabilities/installed-operation-lineage-and-promotion.md)
- [Application Aftermath, External Effects, And Recovery](../worth-query/docs/execution/application-aftermath-and-recovery.md)

Input capture uses the initiating request's native resources before moving any
original fields. Its refusal returns `(original_builder, owned_key,
WorthQueryOperationInputCaptureDenial)`; keep all three for retry. Success admits
one typed payload and shares its backing/charge through native input consumers.
Nested caller Clone heaps and allocator/Arc metadata retain their existing
contracts; this payload quote does not describe total heap or grant permission.

### Owning native capability commands

`application_entry` exposes `WorthQueryCapabilityCommand` from
`request.retain_capability_delegation_in_program(..)` and
`retain_capability_revocation_in_program(..)`. Move the original input and key
once; keep capture-refusal custody. Supply fresh requests to `advance` and
`close`, retain same-owner refusals, and use completed `close_with_evidence` to
transfer native Fresh/Recovered receipt evidence after cleanup. Commands are
not domain mutations and have no public execute/recover or clone escape.
See [owning capability commands](../worth-query/docs/foundations/workflows.md#owning-capability-commands)
for native replay and authority distinctions.

## Mixed workflow consumer

The [observable workflow journey](../../../../docs/build-an-application.md#67-capture-approve-archive-and-observe-one-workflow)
uses this facade for ordinary owning operations, current retained reads,
original-start reacquisition, independent history audit and native live marks.
`retain_current_read()` and subscription `close()` take no phase/request argument.
The first retains data; the second releases subscription custody. Neither grants
execution permission or settles operation cleanup.
