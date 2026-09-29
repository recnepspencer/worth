# Query Engine Architecture Map

> **Internal engine surface.** This page maps the internal Query engine for platform maintainers: the owner crates behind `worth-query-decl` and `worth-query-host`, and the Workspace engine surface (`WorthQueryWorkspace`, `worth_query::facade`) that `worth-ui-query-binding`, `worth-server`, and `worth-query-replay` use. Application code uses `worth-query-decl` and `worth-query-host`; start with the [API Map](../../../../../docs/api.md).

## Who This Is For

| If you are... | Read this instead |
|---|---|
| Writing application code: a schema, operations, handlers, queries, or a host that installs and runs them | The [API Map](../../../../../docs/api.md), then the [`worth-query-decl` README](../../worth-query-decl/README.md) and the [`worth-query-host` README](../../worth-query-host/README.md) |
| Learning what happens to a request and what each outcome means | [How WORTH Works §9 to §14](../../../../../docs/how-it-works.md#9-query-the-life-of-one-request) |
| Writing certification or replay evidence | [API Map §4](../../../../../docs/api.md#4-certification-api) and the [`worth-query-replay` README](../../worth-query-replay/README.md) |
| Looking up a term such as *basis*, *performed*, or *settled* | The [Glossary](../../../../../docs/glossary.md) |

This page covers the remaining case: you are **changing the Query engine
itself**, its audience facades, or the boundaries between them. It records
which crate owns which authority, where each stage of a request lives in the
source, and which laws a change must keep.

Two rules for reading it:

- **The application API is `worth_query_decl::facade` and
  `worth_query_host::facade`.** A name on this page that neither facade
  exposes is engine-internal. Do not add it to an application's imports. If an
  application needs it, the missing facade export is the defect to report.
- **`worth_query::facade` is not an application facade.** It is the Workspace
  engine surface. `worth-query-host` does not depend on `worth-query` and does
  not re-export it.

Every claim here describes the engine that exists. If a capability is not
described here or exposed by an audience facade, do not infer it from a type
name, an internal module, a digest, a report, or a neighboring runtime.

## Query In One Sentence

WORTH Query turns typed application intent and proof from the runtimes that own
truth into an admitted, bounded operation whose execution, aftermath, recovery,
and publication retain the exact authority, basis, and causality that made it
lawful.

Query is not a database, policy engine, identity provider, or storage system.
It composes those owners into governed application work. How WORTH Works
[§2](../../../../../docs/how-it-works.md#2-the-shape-of-the-platform) places it
in the platform.

## Query Crate Map

The Query workspace (`workspaces/worth-query/crates/`) splits authority across
owner crates. The audience facades re-export exact owner namespaces and add no
behavior.

### Owner crates

| Crate | Owns | Does not own | Applications reach it through |
|---|---|---|---|
| `worth-query-declaration` | Authored intent, canonicalization, schema-visible validation, binding grammar, result shapes, collection declarations, and the declaration macros | Installation, execution, publication, replay | `worth_query_decl::facade`; `worth_query_host::facade::declaration` |
| `worth-query-installation` | Callback-free domain package meaning, installation admission, generation affinity, conflict semantics, installed indexes, and installed operation contracts such as `WorthQueryOperationTouchContract` | Execution providers and workspaces | `worth_query_host::facade::domain` |
| `worth-query-admission` | Basis, policy, support, resource, and descriptive graph-read planning decisions, and the proof-bearing handoffs that execution accepts | Executable plans, allocation, provider contact, execution, publication | `worth_query_host::facade::admission` |
| `worth-query-execution` | Consuming admission proof, attempt-local provider sessions, and execution evidence. It also holds the primary-graph application runtime: installation, contributions, handlers, request admission, candidate touch admission, compare-and-commit, commit receipts and outcomes, settlement deferral, product branches, program adoption, discovery, workflows, and provisional aftermath | Derived publication and disclosure copies | `worth_query_host::facade::{application_installation, application_contribution, application_discovery, application_invariants, installed, primary_graph, product, runtime, convergence_epoch, provisional_aftermath}` |
| `worth-query-publication` | Derived publication: policy-materialized decision attachments, canonical disclosure copies, published application results, application aftermath publication, and the borrowed request API (`WorthQueryApplicationRequestExt`, mutation outcomes, output demand, live reads) | Execution | `worth_query_host::facade::{publication, application_entry}` |
| `worth-query-package-archive` | The store-neutral, authority-free archive protocol for portable packages | Trust, activation, or runtime authority | Release tooling only |
| `worth-query` | The Workspace engine surface (`WorthQueryWorkspace`), typed query authoring, and the engine modules for live views, subscriptions, continuations, policy narrowing, and the consumer kit | Anything an application imports | Nothing. See [Engine consumers](#engine-consumers). |

### Audience facades

| Crate | Audience | Re-exports |
|---|---|---|
| `worth-query-decl` | Application declarations, from `entry`-band and `cert`-band crates | Declaration modules and macros from `worth-query-declaration`, its only dependency |
| `worth-query-host` | Application hosts, from `entry`-band and `cert`-band crates | Namespaces from `worth-query-admission`, `worth-query-declaration`, `worth-query-execution`, `worth-query-installation`, and `worth-query-publication` |
| `worth-query-replay` | Certification only, from `cert`-band crates | Replay and certification-cost types from `worth-query`, its only dependency |
| `worth-query-certification` | Certification only | Provider comparison and hostile scenarios, with no construction authority |

`tools/boundary-check/snapshots/facades.toml` snapshots the exports of
`worth-query-decl`, `worth-query-host`, `worth-query-host::primary_graph`,
`worth-query-host::provisional_aftermath`, `worth-query-replay`, and
`worth-query-certification`. Adding, removing, or renaming one of those exports
is a deliberate snapshot update, not a side effect.

### Engine consumers

Inside this repository, the crates that depend on the `worth-query` engine are
`worth-ui-query-binding`, `worth-ui-certification`, `worth-server`,
`worth-query-replay`, and `worth-query-certification`. The boundary
constitution (`tools/boundary-check/config/road1.toml`) names
`worth-ui-query-binding` as the only UI production consumer. A new engine
consumer is a boundary change, not a convenience import.

The Workspace surface has its own guides, each marked with an "Internal engine
surface" banner. Start with the
[Workspace Overview](./foundations/workspace-overview.md).

## Runtime Authority Owners

How WORTH Works
[§2.3](../../../../../docs/how-it-works.md#23-who-owns-what) summarizes the
owners. This is the maintainer's version, with what each owner must not be
asked to do:

| Owner | Owns | Does not own |
|---|---|---|
| Application domain | Business vocabulary, schema meaning, operations, invariants, capability intent, and disclosure classifications | Runtime proof or lower-runtime truth |
| Proof substrate (`worth-proof`) | Generic proof-bearing progression, freshness, readmission, composition, and capability carriers | Query permission, live runtime state, or owner-specific authority |
| Foundational | Exact canonical values, keys, paths, portable bases, provenance, receipts, and shared boundary vocabulary | Proof progression, application permission, or relational truth |
| Relational | Entities, relations, aspects, immutable branch roots, mutable branch-reference cells, exact branch observations, detached transactions, opaque prepared candidates, branch-local linearization, commit history, and durable publication settlement | Product authorization, application operation meaning, Query index publication, or external completion |
| Runtime Bridge | Installed correspondence and lawful lowering between Query and lower runtimes | Relational facts, Signal decisions, or application policy |
| Signal | Policy evaluation, producer-local scoped invalidation, readiness and scheduling, performed execution receipts, component branch graph and bases, per-branch execution cells, weak owner services, local evaluation slots, and condition outcomes | Application capability admission, Query maintenance authority, composite product currentness, or relational mutation |
| Runtime World | Memory-resident product branch references, immutable single-parent composite history, exact component-basis composition, coordinated publication, and bounded retained owner effects | Application authorization, component truth or settlement authority, durable restart, or Query public completion |
| Query | Installed application meaning, authority composition, admission, typed progression, execution products, idempotency and outbox meaning, Query index publication, typed settlement recovery, runtime-local aftermath recovery, and consumer publication | Authentication truth, graph truth, policy truth, external completion, or Relational durability authority |
| Store | Durable persistence, journals, restart checkpoints, and reconstructive state | Ordinary Query admission, live recovery authority, or external completion |
| External effect owner | Whether an escaping consequence was accepted or completed | Query commit, application authorization, or recovery authority |

The distinction between **truth** and **authority** is fundamental. A lower
runtime can truthfully report that it can perform an action without proving
that a particular application principal may request that action for a
particular purpose and scope.

## Where Each Request Stage Lives

How WORTH Works explains each stage from the application's side. This table
maps each stage to the crate that owns it and to the host facade module that
exposes it. Read the linked section for the behavior; change the owner crate.

| Stage | Explained in | Owner crate | Host facade module |
|---|---|---|---|
| Declare | [§9.1](../../../../../docs/how-it-works.md#91-declare) | `worth-query-declaration` | `worth_query_decl::facade`; `declaration` |
| Install | [§9.2](../../../../../docs/how-it-works.md#92-install) | `worth-query-installation` for package meaning and installed contracts; `worth-query-execution` for `in_memory_program` and the application runtime | `domain`, `application_installation`, `application_contribution` |
| Request entry | [§9.3](../../../../../docs/how-it-works.md#93-request-entry) | `worth-query-publication` | `application_entry` |
| Admission | [§9.4](../../../../../docs/how-it-works.md#94-admission) | `worth-query-admission` for basis, policy, support, resource, and planning decisions; `worth-query-execution` for principal, scope, and operation authorization | `admission`, `primary_graph` |
| Execution | [§9.5](../../../../../docs/how-it-works.md#95-execution) | `worth-query-execution` (`DecisionReader`, `CandidateWriter`) | `primary_graph` |
| Candidate check and touched graph | [§9.6](../../../../../docs/how-it-works.md#96-candidate-check), [§10](../../../../../docs/how-it-works.md#10-the-touched-graph) | `worth-query-installation` for the declared ceiling; `worth-query-execution` for candidate touch admission and commit-sealed records | `domain`, `primary_graph` |
| Commit | [§9.7](../../../../../docs/how-it-works.md#97-commit), [§7](../../../../../docs/how-it-works.md#7-one-change-through-the-whole-stack) | `worth-query-execution` over Relational and Runtime World | `primary_graph` |
| Publication, live reads, output demand | [§9.8](../../../../../docs/how-it-works.md#98-publication-live-reads-and-output-demand) | `worth-query-publication` | `publication`, `application_entry` |
| Resources and budgets | [§9.9](../../../../../docs/how-it-works.md#99-resources-and-budgets) | Each owner enforces its own bounds | All |
| Outcomes | [§11](../../../../../docs/how-it-works.md#11-outcomes-every-way-a-request-can-end) | `worth-query-publication` (`WorthQueryApplicationMutationOutcome`); `worth-query-execution` (`WorthQueryApplicationCommitOutcome`) | `application_entry`, `primary_graph` |
| Branches, programs, adoption | [§12](../../../../../docs/how-it-works.md#12-branches-programs-and-adoption); calls in [Build an Application §7](../../../../../docs/build-an-application.md#7-adopt-a-new-program-on-a-branch); full guide [foundations/programs-and-adoption.md](foundations/programs-and-adoption.md) | `worth-query-execution` for rosters, activation, and adoption publication; `worth-query-publication` for `programs()` and branch-set requests | `application_entry`, `application_installation`, `product`, `primary_graph` |
| Workflows | [§13](../../../../../docs/how-it-works.md#13-workflows); calls in [Build an Application §6](../../../../../docs/build-an-application.md#6-author-publish-and-run-workflows); full guide [foundations/workflows.md](foundations/workflows.md) | `worth-query-declaration` for specs, builders, and node kinds; `worth-query-installation` for vocabulary installation and resource ceilings; `worth-query-execution` for the workflow kernel, runtime, outcomes, discovery, and adoption inventory; `worth-query-publication` for workflow requests and their preparation denials | `application_program`, `domain`, `application_installation`, `application_discovery`, `application_entry`, `primary_graph` |
| Aftermath and recovery | [§14](../../../../../docs/how-it-works.md#14-aftermath-and-recovery) | `worth-query-execution` for recovery and provisional aftermath; `worth-query-publication` for published aftermath | `primary_graph`, `publication`, `provisional_aftermath` |

## Engine Laws

The platform [principles](../../../../../docs/philosophy.md#principles) bind
the engine, and How WORTH Works
[§8](../../../../../docs/how-it-works.md#8-state-versus-truth-as-the-code-enforces-it)
lists where the code enforces them. Their engine consequences:

- **Meaning is declared; authority is admitted.** Declaration, installation,
  admission, and execution are separate owners. Skipping a step, or letting one
  crate mint another crate's product, creates a parallel authority lane.
- **Proof is carried, not rediscovered.** An admitted object carries what its
  legal successor needs. `worth-proof` supplies generic progression law, but a
  generic proof or a caller-defined `AuthorityMarker` cannot open a Query
  operation. Authority-bearing Query methods accept the exact Query-owned types
  that their owning workflows return.
- **Narrowing cannot widen.** Purpose, tenant, relationship proof, capability,
  disclosure, branch, basis, and lifecycle constraints may narrow a request. No
  projection, helper, adapter, or lower-runtime result may widen it again.
- **Reporting is not authority.** Digests, counters, inspection reports,
  explanations, support rows, serialized documents, and public projections
  authorize nothing unless a typed contract says they do.
- **Progression is explicit.** Requested, admitted, prepared, executing,
  performed, settled, completed, stopped, published, and released are different
  states. `performed` means the branch reference moved; `settled` means the
  owning runtime also acknowledged durability and any required Query
  publication. Methods appear only on states that may perform them. Do not
  simulate progression with booleans or status strings.
- **Currentness is part of authority.** Authentication, principal mapping,
  graph observations, Signal decisions, grants, lifecycle, branch, snapshot,
  and version can change. Query binds them to the request and revalidates the
  relevant ones before governed work or commit.
- **Commit is not external completion.** A committed mutation or outbox row
  proves local state only. Acknowledgement, silence, timeout, disconnect, and
  lost response keep their exact typed posture.
- **Support is explicit.** An exported type may be accepted, provisional,
  deferred, or vocabulary-only. `provisional_aftermath` is a compiled undo and
  redo experiment, not an accepted product contract.

## Audience Facade Rules

Application-facing imports are documented in
[API Map §3](../../../../../docs/api.md#3-application-api). These are the rules
a maintainer keeps when changing the facades.

- **`worth-query-decl` adds nothing.** It re-exports declaration types and
  macros without another type identity or behavior layer. Pure schema crates
  stay Query-agnostic; declaration integration belongs in the entry band.
- **`worth-query-host` exposes the production authority graph, not the
  engine.** It does not expose raw primary-graph handles that would let a
  consumer bypass Query. Program-output publication custody is deliberately
  absent; the boundary checker reserves its issuer to the publication owner.
- **Stable aftermath enters through `primary_graph` and
  `publication::application_aftermath`.** Do not teach
  `facade::provisional_aftermath` as stable undo or redo support.
- **Settlement recovery stays opaque.** The host reaches it through
  `WorthQueryApplicationSettlementDeferred` and
  `WorthQueryPrimaryGraphApplicationRuntime::recover_deferred_application_settlement`.
  The host never receives Relational's raw settlement capability. Recovery
  finishes an already-performed commit and refreshes Query-owned publication;
  it does not rerun the operation.
- **Product workflow enters through `primary_graph` and `application_entry`.**
  On the application runtime, `current_world()` returns the managed occurrence
  and `branches()` covers fork and reuse creation, bounded history, recovery
  inspection, and cleanup. On the borrowed request, `on_branch(branch)` covers
  reads, transactions, conditional delivery, and close. These accept
  Query-issued branch occurrences, never raw World or component identities.
- **Installed meaning is inspectable without an owner import.** Through
  `facade::domain`, `installed_schema.native_contracts()` gives the sealed
  native aspect catalog, and an installed operation's `contracts().graph_reads()`,
  `contracts().touches()`, `contracts().emissions()`,
  `contracts().external_effect()`, and `contracts().aftermath()` give its exact
  graph scopes, emissions, escaping-effect meaning, and aftermath meaning. These
  are borrowed inspection values, not operational authority.
- **The facade route is part of the contract.** Boundary enforcement verifies
  that `worth-query-host` re-exports the exact installed owner namespace and
  snapshots it recursively. Retargeting an alias to a broader implementation
  namespace is a contract change even when existing imports still compile. Do
  not preserve an obsolete path by re-exporting the same types from a second
  authority lane.
- **Replay stays in certification.** `worth-query-replay` reconstructs and
  compares prior semantic execution. Certification cost evidence also enters
  there, through `WorthQueryCertificationCostRuntimeExt` and a bounded
  `WorthQueryCertificationCostScope`. Neither may enter application or host
  code.
- **Facade rule.** If an application example needs `worth_query`,
  `worth_query_installation`, `worth_query_admission`, `worth_query_execution`,
  `worth_query_publication`, Relational, Runtime Bridge, Signal, or Runtime
  World directly, it crosses an authority boundary. The audience facade must
  expose the lawful product instead.

Before finishing a change that touches a manifest or a facade, run
`cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` and
`cargo run --manifest-path tools/agent-context/Cargo.toml -- check`.

## Engine Contracts By Subsystem

Each subsection lists the invariants the engine must keep. The
application-facing usage of the same subsystem lives in the guide named at the
top of the subsection; do not copy it here.

### Installed query bindings and result projection

Application usage:
[host README, Ordinary Typed Query Entry](../../worth-query-host/README.md#ordinary-typed-query-entry).

- `ApplicationQueryBinding<Schema>` is one installed contract: the input and
  its structured-value binding, the installed query, parameter and result
  bindings, exact principal mapping, scope resolution, stable binding identity,
  a finite result ceiling and an optional deliberate work cap. Ordinary bindings
  use `ApplicationQueryBindingLimits::results(n)`; the installed host supplies
  the finite operational work safeguard. `ApplicationQueryIntent<Schema>` carries
  only the request values needed for parameters and scope.
- Installation compiles that association once.
  `installed_schema.installed_query_binding::<Binding>()` exposes the installed
  query, principal binding, scope, identity, and limits together.
- A borrowed request reads no World state. Each `execute()` selects the current
  World once and carries that occurrence through principal and scope
  resolution, admission, execution, and publication. `.limits(results, work)`
  may only narrow the resolved host/binding ceiling; widening is denied before
  provider or basis work. All modes, including retained reads and continuation,
  use the same resolution policy.
- Result projectors receive only disclosure-admitted rows.
  `WorthQueryApplicationProjectionRow::entity_id()` distinguishes repeated
  traversal of one entity from distinct entities; topology membership and
  deduplication use it rather than projected field values.
- `ApplicationQueryResultShapeBuilder::relation_where_equal(...)` binds a typed
  equality-queryable field on a nested entity to a typed query parameter. Query
  filters actual relation targets before cardinality and child projection,
  while source evidence retains the complete examined sibling set and predicate
  aspect revisions. Filtered result relations are not continuation targets.

### Programs, contributions, and installation

Application usage:
[decl README, Application Program Meaning](../../worth-query-decl/README.md#application-program-meaning),
[host README, Contribution-Composed Applications](../../worth-query-host/README.md#contribution-composed-applications),
[host README, Branch-Local Program Evolution](../../worth-query-host/README.md#branch-local-program-evolution),
and How WORTH Works
[§12](../../../../../docs/how-it-works.md#12-branches-programs-and-adoption).

**Program validation.** `ApplicationProgramDefinition<Schema>` is the canonical
static root. It owns one program identity and the typed inventories of
contributions, feature instances, declared outputs, actions, output-graph
connections, and scoped rules. `ApplicationProgramAuthoring::<Schema, Program>::begin()`
followed by `.validated_program()` validates that meaning before installation.
Invalid or duplicate identities, dangling features, missing required inputs,
undeclared outputs, duplicate bindings, cycles, and unexported cross-instance
connections are typed denials, never deferred to the first request.

**Features and the output graph.** Features declare semantic ownership and
typed ports. Composition-instance identity distinguishes installations of one
reusable feature; it does not identify a domain entity. Connections bind
compatible ports at explicit instances. Each output-graph child edge must leave
its parent feature, and `ApplicationOutputLeaf` terminates a branch.
Declaration order, registration ordinal, strings, and traversal never create
these relationships. `worth_query_feature_spec!` removes builder plumbing but
still produces the canonical `ApplicationFeatureSpec` and creates no registry;
see [Feature Capsule Authoring](./authoring/feature-capsule-authoring.md).

**Action attachments.** An authored action may attach one repeated-row
correspondence, one evaluated requirement, one external-input provider, and a
required-output-source posture. These are parts of the installed action
contract, not parallel registries. `repeated_optional_member` preserves
`Unchanged`, `Set`, and `Clear` and carries the required source observation.
`evaluated_requirement` returns the same rule evaluation used for input guidance
and submission enforcement, so a consumer never keeps a second callability
predicate. `external_input_provider` resolves a typed selection into values,
revision, and provenance, and requires revision validation before the captured
input is admitted. Capture never grants mutation authority.

**Derived artifacts.** `ApplicationProgramDefinition::DERIVED_ARTIFACT_GOVERNANCE`
selects the posture. `Compatible` admits a program with no artifact meaning.
`Required` makes it complete: every connected output target declares at least
one derived artifact, or program validation denies it. An artifact declaration
binds producer family, succession, locality, retention, resource ceiling,
stopped outcome, and exact dependencies as one contract. Query rejects a demand
whose family, occurrence, change, source evidence, resource use, or parent
settlement does not match it. Recompute, replacement, and reconstruction remain
distinct meanings, and a caller cannot invent a no-work or reuse outcome by
reproducing identifiers. Once a program selects `Required`, do not switch back
to `Compatible` to admit a new path.

**Managed computations and derived collections.** A contribution declares each
computation's input, output artifact, partition, ordering, reuse, stopped
outcome, and ceilings, then installs one owner for that declaration.
Installation rejects missing, foreign, duplicate, or mismatched owners.
Execution evidence is minted only by the active `DecisionReader`, which borrows
the real request scope and checks deadline, cancellation, retained bytes, and
work. There is no unscoped or test-only execution path. A derived collection
declaration grants no mutable collection authority: product runtimes own
incremental state, mark a row complete only from current contributor evidence
observed through one retained request observation, and must reproduce the same
rows on reconstruction.

**Speculative work.** A preview starts from a Query-issued retained read
observation, keeps that exact Bridge source basis live, carries no mutation or
publication authority, and must be readmitted against the current program
runtime before the real mutation. Source or runtime drift denies readmission.
Discard, replacement, close, and drop all terminate the speculative Bridge
work.

**Normalized manifests.** `ValidatedApplicationProgram::normalized_manifest()`
derives a deterministic, sorted diagnostic description from validated meaning.
It holds owned strings only, no type identity, handle, or authority, and it
never participates in validation or installation.

**Installation lanes.** `application_installation::in_memory_program` returns
`WorthQueryProgramApplicationRuntime<Schema, Program>`. Program-owned actions
execute through `.execute_in_program(&application)`; capability-owned actions
through `.execute_capability_in_program(&application)`. A program-owned action
on the weaker `.execute()` path returns `ApplicationProgramRequired`, and a
runtime from another host returns `ApplicationProgramMismatch`. The lane commits
under the program that the request's branch runs.

**Rosters and adoption.** One host may roster several validated revisions. The
revision and its codec describe meaning; they do not select current meaning.
Each World product occurrence carries one performed branch-program activation.
Semantic comparison is admission input, never write authority. A prepared
adoption binds exact source and target programs, branch head, affected state,
target-rule validation, migration, dispositions, resources, and custody; only
World's final performed publication activates the target. Retained reads and
unpublished adoption recoveries lease exact program support, and
`retire_program_support` cannot evict a program still needed for correctness.
Broader adoption is owner-covered, ordered, and explicitly non-atomic.

**Contributions.** `worth_query_application!` lists independently compiled
contributions once and supplies `ApplicationSchemaComposition::Contributions`.
Each declares members through `ApplicationSchemaContribution<Schema>` and
implements `WorthQueryApplicationContribution<Schema>`. Contributions are
implementation and provider slots under the program, not a second semantic
root. Installation validates the exact contribution inventory before any
callback, restricts each setup to its members, and denies missing, duplicate,
foreign, mismatched, uncovered, or ambiguous members before the runtime or
initial state becomes visible. Handler completeness, producer and conditional
binding, and invariant installation precede the initial-state callback. The
[public consumer](../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/main.rs)
holds the executable definitions.

### Handlers, candidates, and invariants

Application usage:
[host README, Typed Handler And Invariant Access](../../worth-query-host/README.md#typed-handler-and-invariant-access)
and How WORTH Works
[§9.5](../../../../../docs/how-it-works.md#95-execution).

- The three handler roles (`decide`, `candidate_requirements`,
  `build_candidate`) are bounded. Query retains and completes the decision's
  actual read dependencies, reserves the candidate's demand against the
  installed ceiling before allocation, and lets `CandidateWriter` bind effects
  without selecting a new runtime basis.
- An existing target with no scalar identity field crosses the phase boundary
  through `DecisionReader::mutation_target` on an entity observed through typed
  reads, then `CandidateWriter::projected_entity`. The installed projection
  authority mints the target with its runtime, schema binding, and exact
  operation admission. A target from another runtime, binding, or admission, or
  absent from this attempt's completed read set, is rejected.
- Candidate cardinality, retained representation bytes, and validator work are
  separate finite bounds. `representation_bytes(n)` leaves validator allowance
  to the installed invariant closure; optional binding/handler caps only restrict
  that allowance, while host capacity and exact closure checks remain enforced.
  Relation integrity and installed invariants inspect
  the actual candidate and its affected untouched neighbors before atomic
  publication. Domain prechecks or handler success cannot substitute for
  invariant receipts. Denied, cancelled, or invalid candidates do not publish.
- `DecisionReader::prior_output_family` returns the live members of a declared
  output-role family in semantic role order, resolved at the selected branch
  occurrence and product generation, under the ordinary decision work budget.
  Undeclared families, mismatched markers, exhausted budget, or unresolvable
  correspondence return `WorthQueryPriorOutputDenial`.
  `prior_output_family_if_present` returns `None` only when the binding has no
  correspondence at that occurrence and generation; integrity and resource
  failures stay denials.
- Installed invariant factories resolve typed field and relation bindings once.
  Their proposed and committed views enforce binding, view, declared-access,
  prepared-scope, entity-kind, value, and finite-work rules. Application code
  never decodes native aspect payloads or constructs lower-runtime effect
  programs.

**Optional fields.** An operation must declare the field as both a decision
read and a write. The projection observes the exact value or lawful absence,
and `write_optional_field` authors presence or absence:

```rust
let note = reader.decision_field(projected, DraftNote::reference())?;
let quantity = reader.decision_field(projected, DraftQuantity::reference())?;

// After projected dependencies are completed and the effect target is bound:
set_effects.write_optional_field(&draft, DraftNote::reference(), Some(String::new()))?;
set_effects.write_optional_field(&draft, DraftQuantity::reference(), Some(0_u64))?;

// A later admitted operation clears the optional note without a sentinel:
clear_effects.write_optional_field(&draft, DraftNote::reference(), None)?;
```

Empty text, zero, and absence are distinct authoritative states, and absence is
a retained decision fact: a competing absent-to-present change makes the older
attempt stale. Required fields cannot call `write_optional_field`; the field
marker enforces that at compile time. Never encode absence as an empty string,
zero, `AspectValue::Null`, or a sentinel, and never bypass the projection with a
direct Relational patch. Query lowers the admitted effect to Relational's
native, contract-validated field patch.

### Output correspondence and program outputs

Application usage:
[host README, Contribution-Composed Applications](../../worth-query-host/README.md#contribution-composed-applications)
(output roles and committed changes) and
[host README, Output Demand, Exact Observation, And Live Reads](../../worth-query-host/README.md#output-demand-exact-observation-and-live-reads).

- `WorthQueryApplicationOutputRole<Binding, Entity, Action>` names one declared
  semantic output. The role name supplies no persistent identity; Relational
  resolves created identities and co-commits their structural changes and
  lineage. `output_correspondence().entity(role)` checks binding, role name,
  action, and entity marker, so a caller cannot relabel a committed identity by
  changing a generic argument. The projected identity still needs fresh
  admission for later use.
- Each fixed role in `Binding::Output::ROLES` carries an
  `ApplicationMutationOutputRoleCardinality`. `for_entity` declares
  `ExactlyOne`: leaving it unbound fails the candidate with `MissingOutputRole`.
  `optional_for_entity` declares `AtMostOne`: the commit may omit it. A second
  binding of either is `DuplicateOutputRole`. The role token matches the
  declaration: `WorthQueryApplicationOutputRole` for exactly-one roles, whose
  `entity(role)`, `prior_output` and reconstruction `entity` reads are total,
  and `WorthQueryApplicationOptionalOutputRole` for at-most-one roles, whose
  same reads return `Option`, `None` meaning the commit left the role unbound.
  A token of the other cardinality is refused on write
  (`OutputRoleCardinalityMismatch`) and on read (`CardinalityMismatch`). The cardinality is part of the canonical schema
  identity and the portable and archived descriptions. Declare an optional
  single output this way, never as a family with minimum zero.
- `WorthQueryApplicationOutputRoleFamily<Binding, Entity>` names a family
  already declared by `Binding::Output::ROLE_FAMILIES`. It creates no second
  lineage store.
- `committed_changes()` returns `WorthQueryApplicationCommittedChanges`, an
  immutable view with no field payloads or mutation authority. Its constructor
  and canonical artifact stay private. Event order and numeric identity do not
  establish an entity-to-lineage association. Receipt clones and
  `AlreadyCommitted` recovery keep the observations, but the receipt's
  performed product-change capability is single-use and never recreated.

**Required outputs.** For a program-owned action, required output means the
complete authored output graph. `performed.start_required_outputs(&request, controls)`
and `request.start_program_outputs(&application, demand, controls)` return
`WorthQueryApplicationProgramOutputHandle`.

```text
typed action intent + exact installed program
    -> program-affine mutation admission and publication
    -> performed source result
    -> retained exact root demand basis
    -> authored output-graph traversal
    -> per-edge occurrence discovery at the carried parent basis
    -> producer admission, readiness, and exact settlement
    -> complete program-output settlement
```

- `settle(&fresh_request)` performs at most the resolved installed-host
  settlement allowance, optionally narrowed by caller controls, and returns
  `Pending` when that bound is exhausted. `Default::default()` controls use the
  installed host policy without copied numeric caps. `advance(&fresh_request)` serves hosts that wait on owner
  notifications between advances. Ordinary consumers never hard-code an
  advance count or rediscover the dependent graph.
- Settlement of the root and every discovered edge returns one
  `WorthQueryApplicationProgramOutputSettlement`, with typed
  `outputs_for::<Schema, Connection>()` and instance-qualified outputs.
- Dependent discovery runs at the exact carried traversal basis. A later
  unrelated observation, equal visible ordinal, foreign installation, or
  foreign parent settlement cannot authorize an edge. Completed outputs retain
  their owner settlement, so reconstruction and retry never depend on a
  consumer keeping a view alive.
- `settled_root_observation()` reports that the source output completed when a
  dependent edge later denies. It does not prove graph completion.
  `notifications()` exposes owner progress.
- `request.demand(demand).controls(controls).start()` settles one producer
  family. It is not a substitute for the program-output handle when the program
  declares transitive required outputs. Source drift returns `Superseded`.

### Source expectations, exact reads, and live reads

Application usage:
[host README, Output Demand, Exact Observation, And Live Reads](../../worth-query-host/README.md#output-demand-exact-observation-and-live-reads)
and How WORTH Works
[§9.4](../../../../../docs/how-it-works.md#94-admission) and
[§9.8](../../../../../docs/how-it-works.md#98-publication-live-reads-and-output-demand).

- Source-bound edits compare the declared source footprint during fresh
  admission and publication. Sibling edits outside the footprint stay legal;
  membership and selector-field changes may invalidate it. Missing, foreign,
  retired, ABA-changed, or changed evidence returns a typed source-expectation
  denial.
- Input-selected subjects need the binding's `expected_source_parameters`, so
  Query rejects a row or result-set proof for different selectors before the
  handler runs. Matching the query type alone binds nothing.
- Exact reads (`retain_read`, `at`) and live reads (`subscribe`) recheck
  application, branch, principal, and scope on every use. A retained request
  cannot open a live subscription. Observations identify state but grant no
  mutation, retention, or publication authority.
- `application.discovery()` describes installed declarations. It grants no
  execution authority or promise of current authorization.

The [public application proof](../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof.rs)
exercises program-owned actions, source-bound edits, candidate construction,
invariant access, transitive output demand, readiness delivery, exact and live
reads, cleanup, output correspondence, and idempotent recovery.

### Authentication and principal resolution

Authentication answers who an external caller claims to be. It does not answer
what that caller may do.

```text
external identity proof
    -> authenticated external principal
    -> installed principal-binding resolution
    -> Query principal bound to request scope
```

The request scope carries cancellation and deadline state. The resolved
principal stays bound to the authentication and mapping evidence that
constructed it; a role string, subject string, or copied identifier cannot
replace that proof. Cancellation or mapping drift can deny a later transition
even after an earlier admission succeeded.

### Capability authorization

Keep four concepts separate:

1. A **lower-runtime ability** says that infrastructure can perform or observe
   something.
2. An **application capability** says that a principal may request a declared
   application operation under exact constraints.
3. A **lifecycle command** says which state transition the principal may
   perform now.
4. A **governed upper bound** states the maximum resource, operation, purpose,
   field, and provenance authority that progression may activate.

Capability admission evaluates installed subject-relation-object paths against
current Relational truth through the installed Bridge lowering and Signal
decision boundary. The path answers **who may perform the requested command**.
Query also verifies the installed operation, input scope, purpose, exact grant,
prohibitions, and composition rules.

- **Exact-grant binding.** When several grants could satisfy a capability,
  Query retains the exact grant witness that authorized the request, and later
  revalidation uses it. An equivalent-looking grant does not silently replace
  it.
- **Composition.** An operation may require all named capabilities, one lawful
  alternative, distinct principals for distinct duties, conflict prohibitions,
  exact relationship or tenant scope, or purpose and field constraints. Query
  evaluates the installed composition law; callers cannot combine independent
  booleans.
- **Delegation.** Delegation derives a narrower capability with lineage to its
  source and enforces depth, scope, purpose, resource, operation, field, and
  validity bounds. A selected delegation-activation program may be a proper,
  duplicate-free subset of the operation's installed program union. Delegation
  cannot widen its source, discard provenance, outlive its source, cross a
  foreign runtime, branch, or installation generation, or turn a report into a
  grant.
- **Revocation** is a separately authorized command over the delegated grant,
  not proof that the revoker may perform the governed operation.

See [Application Authorization And Emergency Elevation](./capabilities/application-authorization-and-emergency-elevation.md)
and [Policy, Tenant, And Relationship-Proof Narrowing](./foundations/policy-tenant-and-relationship-proof-narrowing.md).

### Emergency elevation

Emergency elevation is a governed state machine over a request for a bounded
application capability, not a superuser switch. The request retains requester,
governed resource, operation, purpose, exact field or disclosure bound, grant
and provenance constraints, validity window, and installed lifecycle identity.

```text
request -> approve -> active use -> close -> required review -> reviewed
                    \-> expire
```

Each lifecycle command (approve, use, close, revoke, review) targets the
lifecycle object and needs its own installed command authorization. The carried
upper bound remains the maximum authority the elevation may activate, so
approving is not using. Consequences:

- requesters cannot approve their own elevation when separation of duty forbids
  it, and a conflicting approver relationship blocks approval;
- ordinary operation admission cannot publish lifecycle-transition authority;
- lifecycle drift before commit is a stale outcome, not a false success;
- expiry is evaluated from trusted runtime time, and revocation or expiry stops
  publication;
- completion does not erase the required review.

### Purpose and disclosure

Permission to use a protected fact inside governed computation is distinct from
permission to disclose it. An admitted operation may use a protected field for
membership, ordering, conflict, or invariant outcome when its capability and
purpose allow. Publication then evaluates the result shape and field-level
disclosure for the same request authority, and can omit values that lawfully
influenced computation.

Omission must cover indirect channels: result membership, ordering or rank,
counts and aggregates, cursors, summaries, explanations, patches or
invalidation metadata, and live-delivery timing or shape. Masking after
materialization is not sufficient; Query shapes the published result from
governed disclosure authority.

### Graph obligations and access planning

Application meaning describes graph work through sealed obligation rows: graph
reads, authorization observations, mutation touches, effect application, and
invariant execution. Each row names its owner, selection basis, resource
posture, and required terminal evidence. A support row or obligation kind is
not execution proof.

Graph-read access planning answers a different question: how the declared read
executes without hidden N+1 traversal, unbounded expansion, or consumer-local
materialization. The plan binds required adjacency, predicate, ordering,
traversal, deduplication, proof, and buffering support; cost and capacity
bounds; the selected strategy; plan consumption; and receipt counters. Do not
collapse obligation meaning (what must be proved) into access strategy (how the
read obtains it).

See [Canonical Graph Obligation Progression](./domain-capabilities/canonical-graph-obligation-progression.md),
[Graph Touch Obligation Authority](./authoring/graph-touch-obligation-authority.md),
and [Graph Read Access Planning](./authoring/graph-read-access-planning.md).

### Provider sessions, execution, and commit

How WORTH Works
[§9.5 to §9.7](../../../../../docs/how-it-works.md#95-execution) and
[§7](../../../../../docs/how-it-works.md#7-one-change-through-the-whole-stack)
describe the read and mutation paths step by step. The engine invariants
behind them:

- Execution occurs inside a managed provider session bound to the admitted
  application, branch, basis, request, and installed graph obligations. The
  session coordinates lower-runtime observations without taking their
  ownership. It retains session and installation identity, the branch-qualified
  snapshot and version basis, principal and authorization dependencies,
  graph-read products, Bridge correspondence, Signal decision facts, proposed
  state, invariant receipts, and commit serialization and terminal evidence.
- A read result retains query identity, basis, ordering, cursor, disclosure,
  and execution evidence, not only values.
- A proposed state is not committed truth. A selected invariant is not an
  executed invariant. A successful local effect program is not a commit
  receipt.
- The prepared candidate is opaque, runtime-affine, branch-bound, and
  single-use. It has no method that publishes itself; only the Relational owner
  consumes it, through compare-and-publish or explicit discard.
- Commit revalidates the dependencies whose drift could make the operation
  unlawful. Commit authority stays bound to its admission and serialization
  proof and cannot be paired with another admitted operation.
- With a declared external effect, the local mutation and dispatch intent share
  one Relational commit, and Query dispatches only from that committed fact.
  Even an operation with no domain mutation must commit its outbox and
  idempotency fact before a consequence escapes. The external owner decides
  completion; Query records what it observed.
- `SettlementDeferred` means the branch moved but durability acknowledgement or
  Query's derived publication did not finish. The typed deferred carrier is the
  only legal retry input. Recovery repairs the performed publication, refreshes
  Query indexes, proves the original commit is still in current ancestry, binds
  the current Bridge head, and readmits idempotency when required, under the
  application commit serialization boundary. It never reruns the mutation or
  obtains a raw Relational settlement token. The outcome table is in
  [§11](../../../../../docs/how-it-works.md#11-outcomes-every-way-a-request-can-end).

See [Provider Sessions And Decision Read-Sets](./domain-capabilities/provider-sessions-and-decision-read-sets.md),
[Provisional State And Invariant Execution](./domain-capabilities/provisional-state-and-invariant-execution.md),
and [Authoritative Mutation Evidence](./capabilities/authoritative-mutation-evidence.md).

### Aftermath and recovery

How WORTH Works
[§14](../../../../../docs/how-it-works.md#14-aftermath-and-recovery) describes
declared aftermath, undo, settlement recovery, external effects, and product
publication recovery. Engine notes:

- Relational savepoints and rollback discard provisional transaction work. They
  create no application authority, do not alter committed history, and are not
  a recorded inverse, compensation, reconciliation, or recovery.
- Publication-settlement recovery comes before ordinary aftermath:
  `WorthQueryApplicationSettlementDeferred` directs the host to
  `WorthQueryApplicationSettlementNextAction::RecoverDeferredApplicationSettlement`.
- Runtime-local recovery opens from the exact sealed commit receipt and stays
  bound to the originating runtime, operation, principal, action, scope,
  idempotency record, outbox observation, and currentness evidence. A wire
  identity or published recovery report is not the live handle.
- The accepted recovery surface supports inspection, resolution, safe retry,
  disposal, and expiry. Reconciliation and compensation currently stop at
  owner-bound admission products; Query does not yet execute those corrective
  effects.

See [Application Aftermath, External Effects, And Recovery](./execution/application-aftermath-and-recovery.md).

### Basis, branch, and currentness

A **basis** identifies the exact truth context against which work was admitted
or executed: runtime and installation generation, branch, snapshot and version,
schema and query identity, policy, tenant, relationship, and purpose context,
principal mapping, and continuation, cursor, or live-delivery identity. Equal
version ordinals on different branches are not equal bases. Matching digests
from different owners are not interchangeable. A cursor means something only
with the query, ordering, branch, and basis that produced it.

- **Relational.** Branch truth has an immutable root selected by an exact basis
  and a mutable reference cell. An owner-issued reference observation records
  the target and branch-local version seen together. A serialized descriptor
  proves only its shape until the owner readmits it. An admitted basis pins its
  root, so reads through it are repeatable; a branch-bound transaction keeps
  that basis, and publication compares the prepared candidate's expected
  observation with the one current branch cell.
- **Signal.** One canonical component branch graph. Its owner-issued
  `SignalOwnerServicePorts` expose weak basis, mutation, and lifecycle ports;
  same-branch work serializes in one execution cell. These are lower-owner
  contracts, not Query composite branches.
- **Runtime World.** `ProductBranchObservation` binds the exact product
  reference and admitted Relational, Signal, and Bridge bases. Only its final
  compare-and-publish installs a performed composite publication. Component
  movement without it stays `ProductUnpublished`, with settlement or cleanup
  obligations; it is neither rollback nor permission to run a missing sibling.
  See the [Runtime World contract](../../../../../crates/worth-runtime-world/README.md).
- **Query.** The host facade selects World-owned product branches, carries the
  exact composite observation through reads and changes, and returns World's
  terminal unchanged. Outbox eligibility is bound to the original performed
  occurrence. A caller cannot substitute a branch token, component basis, or
  fresh latest observation after admission.
- **Programs.** Selected program meaning follows the occurrence. Sibling
  branches can run different revisions at once; no path consults global
  latest-program state.
- **History.** `application.branches().history(branch, maximum)` retains one
  bounded ancestry segment and selects an entry only by asking World for an
  exact historical observation. Commit identities and history entries stay
  descriptive. A `RuntimeWorldRecoveryCursor` is a descriptive position in the
  bounded recovery catalog; it retains no owner effects and grants no cleanup
  authority. Drop retained reads and history pages before expecting branch
  close to complete.

Currentness checks compare retained dependencies with the owning runtime; they
never rebuild authority from a fresh report. See
[Basis Capability Lifecycle](./capabilities/basis-capability-lifecycle.md),
[Branches And Previews](./foundations/branches-and-previews.md), and
[Historical Diff And Basis](./capabilities/historical-diff-and-basis.md).

### Query authoring and result shapes

Query authoring is typed intent, not a string query language. The declaration
surface supports field and aspect selection, predicates, graph traversal and
composition, ordering and stable cursors, collections, grouping, aggregation,
named scopes and templates, saved queries and view shapes, and detail, table,
inspector, and grouped result families. Canonicalization resolves equivalent
forms into one portable artifact; validation rejects ill-typed fields,
incompatible predicates, unsupported graph shapes, invalid result bindings, and
ambiguous ordering before runtime work. A caller may not add an undeclared field
to a published row or reinterpret one result family as another.

See [Query Expressions And Result Shapes](./authoring/query-expressions-and-result-shapes.md),
[Collections, Cursors, Ordering, And Aggregations](./authoring/collections-cursors-ordering-and-aggregations.md),
and [Scopes, Templates, Saved Queries, And View Shapes](./authoring/scopes-templates-saved-queries-and-view-shapes.md).

### Installed domain computation

Domains contribute portable operation meaning; Query owns installation,
admission, execution state, and typed outcomes. Domain hooks provide semantics
at the installed seam and gain no raw authority to mutate Relational state or
mint Query receipts. Managed artifacts stay owned by the runtime that produced
them; copying their fields into a domain struct transfers neither ownership nor
proof strength.

Installation validates the complete package, binds related declarations so
execution cannot mix pieces from different schemas, generations, operations,
policies, or layouts, and is the only point where domain meaning becomes
executable. Hosts may supply adapters and resources but cannot alter semantics
afterward. Operation builders are typestates: `finish()` requires an explicit
external-effect choice and an explicit aftermath choice. Do not recover
installed meaning by parsing rendered scope strings or rebuilding aspect
contracts during execution.

After validation, `export_typed_records()` (through `facade::domain`) yields a
versioned, bounded, authority-free record set. Every decoded payload, signed
envelope, repository load, and reconstructed candidate stays untrusted until
the consuming host selects the expected identity, applies its trust policy, and
obtains fresh validation. See [Portable Query Packages](./portable-packages.md).

See also [Runtime-Installed Domains And Operations](./domain-capabilities/runtime-installed-domains.md),
[Installed Computation Artifact Contracts](./domain-capabilities/installed-computation-artifact-contracts.md),
and [Managed Artifact Ownership And Native Access](./domain-capabilities/managed-artifact-ownership-and-native-access.md).

### Conditional operations, temporal wakes, and Signal

Query installs conditional meaning; Bridge lowers the exact correspondence;
Signal evaluates the condition and mints decision evidence. Evaluated true can
make an effect eligible, false can skip it, unfinished evaluation keeps a typed
continuation posture, and denial performs nothing. An effect still needs its own
admitted execution. A Signal boolean, slot value, or explanation never
authorizes an application effect.

- **Ownership.** Durable temporal intent is authoritative Relational and domain
  truth; Signal's wake table is volatile derived state. The host supplies a
  typed predicate, a named clock, a bounded reconstruction projection, and an
  ordinary operation invoker through `worth-query-host`. A clock reading is
  time evidence only. Signal decides eligibility; Query then performs fresh
  principal, capability, purpose, invariant, idempotency, and compare-and-publish
  progression, and the effect and the intent's completed posture commit
  atomically.
- **Decision postures.** Bridge evidence enters Query as eligible,
  dependency-unchanged, reverted-clean, suppressed, or deferred. Only eligible
  evidence reaches admission. Query classifies the real Bridge evidence rather
  than re-running the predicate.
- **Identity.** The portable binding identity covers the node authority, clock,
  source, timeline, reconstruction query and projector, principal source, and
  invoker; publication derives a runtime-qualified identity from it. Both use
  Foundational canonical-basis preparation and typed digests. A due wake derives
  its idempotency key and intent identity once, during fresh admission; no later
  phase of that attempt regenerates it.
- **Reinstallation.** Commit publication refreshes the derived temporal-intent
  index before returning, and clock observation never performs reconstruction.
  Same-installation reinstallation discards Bridge and Signal state and
  reconstructs active work from current authoritative intent; completed or
  cancelled work does not return, and committed effects are not repeated. A
  successor installation must be rebound or fails closed.
- **Evidence.** Clock receipts expose descriptive `execution_provenance()` and
  report ordinary and reconstructive work separately. The
  `conditional_runtime_lifecycle_probe()` observes the real owners weakly;
  `live_inventory()` after `Drop` reports whether they were released. Neither
  carries execution authority.

See [Conditional Installed Operations](./domain-capabilities/conditional-installed-operations.md)
and [Signal Orchestration](./domain-capabilities/signal-compatibility-orchestration.md).

### Continuations and managed runs

Application workflows are in How WORTH Works
[§13](../../../../../docs/how-it-works.md#13-workflows) and the full
[workflows guide](foundations/workflows.md). Underneath, a
continuation retains unfinished work together with the basis, workspace,
runtime, query, request, and execution posture needed to resume it. Resumption
is a new checked transition, not a callback that inherits ambient authority. The
current stage product carries the only lawful next-stage authority; naming a
stage or reconstructing a prior receipt cannot jump to it. Suggested next
actions from a stop are descriptive guidance only.

See [Continuation Pipeline](./domain-capabilities/continuation-pipeline.md),
[Execution Resource Admission And Managed Runs](./domain-capabilities/execution-resource-admission-and-managed-runs.md),
and [Typed Stops And Remediation Guidance](./domain-capabilities/typed-stops-and-remediation-guidance.md).

### Live views and invalidation

Live execution promotes an admitted result into a managed subscription bound to
the same query, basis, branch, disclosure, and support contracts.

```text
published query result
    -> live promotion
    -> lower-runtime change evidence
    -> Query invalidation and patch admission
    -> authorization and disclosure revalidation
    -> governed patch or typed termination
```

A lower-runtime notification is evidence that something changed, not a lawful
patch. Live delivery preserves subscription selection, current authorization
and disclosure, ordering and cursor meaning, region or collection scope,
mixed-cause classification, backpressure, and terminal release. Permission,
purpose, relationship, tenant, or elevation drift can narrow or terminate
delivery before protected data is projected.

Granular invalidation runs from committed Relational truth, through installed
Bridge correspondence and optional performed Signal work, to Query impact
admission, Query-owned maintenance, and consumer publication. Direct truth and
performed Signal evidence stay separate. Bind a live owner through
`bind_primary_runtime_granular_invalidations`, then consume the runtime-owned
observation or batch through the matching `maintain_*` entry point. Never
reconstruct this authority from raw change data, copied aspect or scope fields,
or a prior installation identity.

See [Live Views](./runtime-surfaces/live-views.md),
[Granular Live Invalidation](./runtime-surfaces/granular-live-invalidation.md),
[Region-Scoped Live Invalidation And Stream Contracts](./runtime-surfaces/region-scoped-live-invalidation-and-stream-contracts.md),
[Subscription Selection And Diagnostics](./capabilities/subscription-selection-and-diagnostics.md),
and [Async Resources And Result State](./capabilities/async-resources-and-result-state.md).

### Publication and downstream consumption

Publication is an authority boundary, not serialization convenience. It takes
a completed or recovered Query-owned terminal and derives the consumer product
that its disclosure, purpose, basis, and publication contract allow, keeping
omission evidence and enough identity for verification. Published mutation
values (commit, aftermath posture, dispatch posture, recovery support) are
weaker than execution authority: they cannot mint a recovery handle,
redispatch, compensate, or resolve an indeterminate result.

A performed-but-unsettled mutation returns opaque typed recovery authority.
Generic effect paths use `EffectExecutionSettlementDeferred` or
`EffectBatchSettlementDeferred` and repair with fresh owning
`EffectExecutionAuthority`. Application and branch-merge paths wrap the same
fact in their own carriers, so callers never reach the raw
`DeferredPublicationSettlement`.

Downstream runtimes consume bound projections or publication receipts and never
reach behind the facade. Transport adapts a published product to HTTP,
messaging, UI, or another process; headers, routes, and user-node state do not
become policy or Query authority.

See [Projection Consumption](./capabilities/projection-consumption.md),
[Downstream Runtime Integration](./foundations/downstream-runtime-integration.md),
and [Bound Projection Sharing And Invalidation](./domain-capabilities/bound-projection-sharing-and-invalidation.md).

### Outcomes and managed resources

The application-facing outcome families are in How WORTH Works
[§11](../../../../../docs/how-it-works.md#11-outcomes-every-way-a-request-can-end),
and resource bounds in
[§9.9](../../../../../docs/how-it-works.md#99-resources-and-budgets). Engine
code also keeps these distinctions typed: skipped versus suppressed, pending
versus terminal, published versus internally completed, partial effect versus
indeterminate, and live recovery authority versus published recovery support.
Never flatten a distinction into `bool`, `Option`, or an error string when it
changes legal next actions, effects, inspection, or release.

Every terminal path releases or transfers its managed resources (sessions,
runs, subscriptions, continuations, leases, checkpoints, recovery handles, and
admitted capacity) explicitly. Dropping a report or serializing an opaque
recovery identity proves neither.

See [Ordinary Outcomes](./domain-capabilities/ordinary-outcomes.md),
[State](./foundations/state.md), and [Inspection](./capabilities/inspection.md).

### Support and admission

Public vocabulary and executable support are different facts. The Query support
matrix is the runtime-owned source of support posture; admission is the
executable check. A report, matching digest, or provider presence never becomes
support. Installed operations carry consumer-support requirements, and their
admission returns either a pair-bound witness or a typed denial.

See [Support Matrix And Admission](./foundations/support-matrix-and-admission.md)
and [Consumer Kit](./foundations/consumer-kit.md).

### Inspection, explanation, and certification

Inspection explains retained runtime state without creating operational
authority. Explanation keeps typed causes distinct across boundaries: a scope
mismatch, authorization denial, stale basis, unsupported access strategy, or
invariant failure never collapses into a generic failure. Certification uses
independent evidence and hostile cases; replay stays in that audience because
reconstruction must not become an execution shortcut.

See [Cross-Runtime Causal Inspection](./capabilities/cross-runtime-causal-inspection.md),
[Operational Identity Authority](./foundations/operational-identity-authority.md),
and [Certification Surface And Closeout Bundle](./domain-capabilities/certification/certification-surface-and-closeout-bundle.md).

## Lower-Runtime Routing

Use this table when deciding where a change belongs.

| Question | Owner |
|---|---|
| What entities, relations, fields, or versions exist? | Relational |
| What transaction committed and at which version? | Relational |
| Which immutable root and exact observation does a branch reference select? | Relational |
| Which branch cell linearizes a prepared candidate? | Relational, independently per branch |
| Which exact Signal branch basis and execution cell govern component work? | Signal owner services, independently per branch |
| Did canonical branch movement occur even though durability acknowledgement failed? | Relational performed-publication evidence |
| May an already-performed application or merge publication be repaired through this facade? | Query typed settlement recovery over the owning Relational runtime |
| How does installed Query meaning correspond to lower-runtime structures? | Runtime Bridge |
| Which installed semantic dependencies match one committed change? | Runtime Bridge candidate selection followed by Query admission |
| Which scoped recomputation did the lower runtime actually perform? | Signal performed execution receipt |
| Which projection, membership, ordering, group, or window consequence is required? | Query impact admission and maintenance |
| What did an installed policy condition evaluate to? | Signal |
| Which Relational and Signal bases form the current product? | Runtime World composition authority, selected and carried through Query's host facade |
| What generic proof progression or readmission law applies? | `worth-proof` |
| What exact canonical value, provenance, receipt, or portable basis represents this meaning? | Foundational |
| What application operation or query was declared? | Application domain |
| Is this principal authorized for this operation, purpose, and scope? | Query composition over owner evidence |
| May this field be disclosed to this consumer? | Query publication over installed disclosure meaning |
| Which lifecycle transition is legal now? | Query typed progression over current owner evidence |
| What idempotency and dispatch-outbox meaning belongs to this operation? | Query over committed Relational facts |
| Did an escaping consequence complete? | External effect owner |
| Is a receipt-bound recovery action legal now? | Query runtime |
| Can a live Query aftermath recovery handle survive restart or cross a process boundary? | Store-backed capability; currently deferred |
| How is committed graph state persisted and versioned? | Relational |
| How are durable reconstructive artifacts retained? | Store |
| How is a published product encoded for another process? | Transport adapter |

Put pure meaning in the domain schema. Put lower-runtime truth and mechanics in
their owning runtime. Put cross-owner application authority in Query. Put
presentation outside all of them.

## Prohibited Shortcuts

Do not:

- treat authentication as authorization;
- treat a role as an unconstrained capability;
- treat entity visibility as field disclosure;
- treat a lower-runtime ability as application permission;
- treat a lifecycle command as the governed application operation;
- use the governed upper bound as the authority target of every lifecycle
  command;
- reconstruct proof from a digest, ID, report, serialized document, or copied
  fields;
- use a generic proof marker where a Query operation requires its concrete
  owner-issued proof, authority, or capability type;
- combine independently valid proofs when no installed composition contract
  authorizes the combination;
- expose internal Query authority packages, or the `worth_query::facade`
  engine surface, to application consumers;
- import Query into pure schema crates;
- import replay into ordinary code;
- read Relational directly to bypass graph obligations or access planning;
- accept a Signal decision as effect authority;
- treat direct Bridge truth as proof that Signal executed, or require a Signal
  receipt for a direct Query consequence that performs no Signal work;
- copy a producer-local aspect or semantic scope through transitive Signal
  descendants instead of deriving each immediate dependency cause;
- treat a reverse-index lookup key such as `ProducerAspectKey`, a semantic
  scope path, or a shard, region, or worker identifier as authority;
- reuse a granular invalidation binding, delivery batch, source-read basis, or
  consumer lease after runtime restore, reinstallation, or rebind;
- construct Query patches directly from raw change data or copied Bridge or
  Signal fields;
- dispatch an external effect without its co-committed local outbox and
  idempotency fact;
- treat acknowledgement, silence, timeout, disconnect, or lost response as
  external completion;
- serialize a recovery handle or reuse its opaque wire identity as live
  authority;
- treat a recovery cursor as retained owner effects, cleanup authority, or a
  product-history continuation;
- use `provisional_aftermath` as accepted undo or redo support;
- treat proposed state as committed truth;
- treat selected invariants as executed invariants;
- publish protected fields and mask them afterward;
- use a cursor outside its query, ordering, branch, and basis;
- reuse consumed lifecycle or commit authority;
- infer support from method presence;
- execute a program-owned action through the weaker ordinary mutation entry;
- settle only the root producer when the installed program declares dependent
  required outputs;
- hard-code an ordinary output-advance loop, rediscover output dependencies, or
  replace retained recovery custody with a fresh demand;
- use `settled_root_observation()` as evidence that the complete output graph
  settled;
- rediscover a dependent output from a current or consumer-retained view when
  Query carries the exact parent traversal basis and owner settlement;
- add production glob imports or glob re-exports to an authority-governed
  surface; keep those bindings explicit and named;
- hide typed denial, stale, cancellation, or resource state inside a generic
  success or failure flag;
- treat Relational rollback as an application-level authority transition;
- retry an application mutation or merge after a settlement-deferred outcome;
- expose, serialize, or repair from a raw `DeferredPublicationSettlement`
  instead of the audience facade's opaque typed carrier;
- let an opaque prepared Relational candidate publish itself or be reused after
  publication or discard.

## Documentation Map

Platform documentation, for everyone:

- [API Map](../../../../../docs/api.md): which crate to import
- [How WORTH Works](../../../../../docs/how-it-works.md): the request lifecycle, outcomes, and guarantees
- [Philosophy](../../../../../docs/philosophy.md) and [Glossary](../../../../../docs/glossary.md)

Application API guides:

- [`worth-query-decl` README](../../worth-query-decl/README.md) and [`worth-query-host` README](../../worth-query-host/README.md)
- [Ordinary Application Front Door](./foundations/ordinary-application-front-door.md)
- [Branches And Previews](./foundations/branches-and-previews.md)
- [Application Authorization And Emergency Elevation](./capabilities/application-authorization-and-emergency-elevation.md)
- [Application Aftermath, External Effects, And Recovery](./execution/application-aftermath-and-recovery.md)
- [Ordinary Product Workflow](../../worth-query-certification/examples/ordinary_product_workflow.rs)
- [Advanced Product Branching](../../worth-query-certification/examples/advanced_product_branching.rs)

Engine internals, for maintainers:

- [Query Docs Index](./README.md): every page, grouped by audience
- [Workspace Overview](./foundations/workspace-overview.md) and [Query Operating Modes](./foundations/query-operating-modes.md)
- [Declarative Query Experience](./capabilities/declarative-query-experience.md)
- [Runtime-Installed Domains And Operations](./domain-capabilities/runtime-installed-domains.md)
- [Canonical Graph Obligation Progression](./domain-capabilities/canonical-graph-obligation-progression.md)
- [Graph Touch Obligation Authority](./authoring/graph-touch-obligation-authority.md)
- [Graph Read Access Planning](./authoring/graph-read-access-planning.md)
- [Provider Sessions And Decision Read-Sets](./domain-capabilities/provider-sessions-and-decision-read-sets.md)
- [Provisional State And Invariant Execution](./domain-capabilities/provisional-state-and-invariant-execution.md)
- [Authority-Scoped Effect Execution](./execution/authority-scoped-effect-execution.md)
- [Lower-Runtime Capability Routing](./domain-capabilities/lower-runtime-capability-routing.md)
- [Projection Consumption](./capabilities/projection-consumption.md)
- [Granular Live Invalidation](./runtime-surfaces/granular-live-invalidation.md)
- [Inspection](./capabilities/inspection.md)
- [Hard Prohibitions](./foundations/hard-prohibitions.md)
- [Operational Identity Authority](./foundations/operational-identity-authority.md)
- [worth-proof Authority And Workflow Contracts](../../../../../crates/worth-proof/docs/features/authority-and-workflow-contracts.md)

Generated `AGENT_CONTEXT.md` files describe each crate's local dependencies and
enforcement; never hand-edit them. Plans and engineering ledgers describe
intent and are not a substitute for this map.

## AI Checklist Before Editing

Before changing the Query engine, answer these questions:

1. Which runtime owns the underlying truth?
2. Which application declaration owns the meaning?
3. Which Query crate owns the authority being changed?
4. What exact typed authority enters this path?
5. What does the next product prove that the input did not?
6. Is the proof carried forward or being reconstructed from representation?
7. Which basis and currentness dependencies remain bound?
8. Does the change preserve capability, purpose, disclosure, branch, and
   lifecycle bounds?
9. Does application code still enter only through `worth_query_decl::facade`
   and `worth_query_host::facade`, from the correct repository band?
10. Are denial, stale, cancellation, and resource-release outcomes still typed?
11. Could a forged, copied, foreign, stale, or equivalent-looking product open
    the path?
12. Does a lower-runtime observation remain evidence rather than application
    authority?
13. If an external effect exists, what local fact was co-committed before it
    escaped, and which owner decides completion?
14. Does an uncertain result remain acknowledged, unresolved, partial, or
    indeterminate instead of being guessed into success or failure?
15. Is the API accepted, provisional, deferred, or vocabulary-only?
16. Is the authority an exact owner-issued type rather than a generic proof
    substrate value?
17. Do the focused tests fail if the disputed authority check is bypassed?

If any answer is unclear, stop and identify the semantic owner before editing.
