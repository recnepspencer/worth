# Build an Application

> **The application story, end to end:** declare the program and its
> features, install the application graph, run it, author and run workflows,
> and adopt a new program on a branch. A code block that follows a
> `<!-- compiled: ... -->` marker is checked byte-for-byte by CI against a
> compiled fixture. A `<!-- doctest: crate -->` marker binds a block to the
> crate's rustdoc and CI doctest lane. Illustrative blocks visibly say
> `// Illustrative: not compiled.`; remaining unclassified Rust fences are
> tracked by a per-document debt that may only shrink.

This is the centerpiece guide for building on WORTH. Other public documents
explain *why* the platform works the way it does
([Philosophy](philosophy.md)) and *what* each runtime guarantees
([How WORTH Works](how-it-works.md)). This one shows *which names you call, in
which order*.

An application imports exactly two crates, `worth-query-decl` and
`worth-query-host`. Their facades are made of modules, so import from the
module that owns each name:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Declare meaning: features, program, workflow authoring.
use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::{application_operation, application_query, application_schema};

// Install and run.
use worth_query_host::facade::{
    admission::authenticated_principal,   // principals and request scopes
    application_entry,                    // requests, mutations, adoption, workflow lifecycle
    application_installation,             // in_memory_program, rosters, limits
    domain,                               // installed schema, workflow vocabulary installation
    primary_graph,                        // handlers, seeding, inspection types
    runtime,                              // resource profiles
};
```

| Module | Names you will look for there |
|---|---|
| `…decl::facade::application_program` | `ApplicationFeature`, `ApplicationFeatureSpec`, `ApplicationProgramDefinition`, `ApplicationProgramAuthoring`, `ValidatedApplicationProgram`, ports and connections, `ApplicationWorkflowSpec`, `ApplicationWorkflowDefinitionBuilder` |
| `…decl::facade::application_operation` | `ApplicationMutationBinding`, `ApplicationMutationIntent`, `ApplicationCapabilityMutationBinding` |
| `…host::facade::application_installation` | `in_memory_program`, `in_memory_rostered_program`, `WorthQueryInMemoryApplicationLimits`, `WorthQueryProgramApplicationRuntime`, `WorthQueryProgramOwner` (a trait: import it to call `owned_revision()`) |
| `…host::facade::application_entry` | `WorthQueryApplicationRequestExt`, `WorthQueryApplicationMutationOutcome`, `WorthQueryBranchAdoptionPublicationOutcome`, `Workflow*Outcome`, `Published*Ref` |
| `…host::facade::application_discovery` | `WorthQueryWorkflowDefinitionDiscovery` |
| `…host::facade::domain` | `WorthQueryApplicationWorkflowSpecInstallation`, `WorthQueryApplicationWorkflowResourceCeiling`, `WorthQueryInstalledApplicationSchema` |
| `…host::facade::primary_graph` | `OperationHandler`, `DecisionReader`, `CandidateWriter`, `WorthQuerySelectedProgramInspection` |
| `…host::facade::admission` | `authenticated_principal::{admit_authentication_adapter, WorthQueryRequestScope}`, `authentication_event::install_authentication_event_owner` |

`worth_query_host::facade::declaration` exposes the same declaration modules,
so a host crate can reach `declaration::application_program::*` without a
second import. Most excerpts below do that. The host facade re-exports only
some declaration macros. `worth_query_mutation_binding!`,
`worth_query_query_binding!`, `worth_query_capability!`,
`worth_query_policy!`, and `worth_query_feature_spec!` are only on
`worth_query_decl::facade`.

**Before you conclude a capability is missing,** follow the procedure in the
[Capability Index](capability-index.md).

---

## Contents

- [0. The whole journey on one screen](#0-the-whole-journey-on-one-screen)
- [1. The model: six things, kept apart](#1-the-model-six-things-kept-apart)
- [2. Declare the schema and its contributions](#2-declare-the-schema-and-its-contributions)
- [3. Declare features and the program](#3-declare-features-and-the-program)
- [4. Install the application graph](#4-install-the-application-graph)
- [5. Run the program](#5-run-the-program)
- [6. Author, publish, and run workflows](#6-author-publish-and-run-workflows)
- [7. Adopt a new program on a branch](#7-adopt-a-new-program-on-a-branch)
- [8. Rules for AI agents](#8-rules-for-ai-agents)
- [9. Use the consumer vocabulary](#9-use-the-consumer-vocabulary)
- [10. Where to go deeper](#10-where-to-go-deeper)

---

## 0. The whole journey on one screen

| Step | What you do | The call | Section |
|---|---|---|---|
| 1 | Declare the schema and its contributions | `worth_query_application!`, `worth_query_application_contribution!`, `impl WorthQueryApplicationContribution` | [§2](#2-declare-the-schema-and-its-contributions) |
| 2 | Declare features and the program | `impl ApplicationFeature`, `ApplicationFeatureSpec::root::<S, F>()…finish()`, `impl ApplicationProgramDefinition` | [§3](#3-declare-features-and-the-program) |
| 3 | Validate the program | `ApplicationProgramAuthoring::<S, P>::begin().validated_program()` | [§3.5](#35-validate) |
| 4 | Install the application graph | `application_installation::in_memory_program(..)` or `in_memory_rostered_program(..)` | [§4](#4-install-the-application-graph) |
| 5 | Run program-owned actions | `request.mutate(intent)…idempotency_owned(key).retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)` | [§5](#5-run-the-program) |
| 6 | Install the workflow vocabulary | `WorthQueryApplicationWorkflowSpecInstallation::begin(..)…finish()`, then `retain_workflow_spec(..)` | [§6.2](#62-install-the-workflow-vocabulary-once) |
| 7 | Author and validate a workflow | `ApplicationWorkflowDefinitionBuilder::<Spec>::new(..)`, nodes, connections, `finish()?.validate()?` | [§6.3](#63-author-a-definition) |
| 8 | Publish, start, and advance it | `bind_definition`, `prepare_workflow_publication`, `prepare_workflow_instance_start`, `prepare_workflow_advance` | [§6.6](#66-publish-discover-start-advance) |
| 9 | Adopt a new program on one branch | `.on_branch(b).programs()`, `compare(&target)`, `adopt(&requirements)?`, `prepare(max)`, `programs.publish(prepared)` | [§7](#7-adopt-a-new-program-on-a-branch) |

The running example is the **Bank reference application**
(`workspaces/worth-query-bank-world`). It is the only complete application
in the repository that uses nothing but the two facades. Banking is
incidental: every step below has the same shape for documents, orders,
tickets, or anything else. Where Bank does not exercise a feature, the guide
uses the document-retention certification model or a runnable example.

---

## 1. The model: six things, kept apart

| Thing | What it is | Where it lives | Rust name |
|---|---|---|---|
| **Schema** | The shape of the application graph: entities, aspects, fields, relations, operations, queries | Declared once, installed once | `worth_query_application!`, `ApplicationSchemaDeclaration` |
| **Contribution** | One named slice of the schema, plus the host setup it needs (handlers, invariants, conditional nodes) | Declared and configured at install | `worth_query_application_contribution!`, `WorthQueryApplicationContribution` |
| **Feature** | A named unit of application meaning that *owns* actions (mutations, operations) and may have typed input and output ports | Part of a program | `ApplicationFeature`, `ApplicationFeatureSpec` |
| **Program** | A content-addressed revision of the application's authored meaning: its contributions, features, connections, output graph, and rules | Validated before install; rostered on a host | `ApplicationProgramDefinition`, `ValidatedApplicationProgram`, `ApplicationProgramRevision` |
| **Workflow** | A branch-local, versioned graph of governed steps, authored against a vocabulary the host installed | Data on a branch, published at run time | `ApplicationWorkflowSpec`, `ValidatedWorkflowDefinition` |
| **Activation** | Which program revision one branch runs | One record per branch | Read through `.programs().inspect()` |

Three separations matter most:

1. **Declaration grants nothing.** A validated program or a validated workflow
   definition carries no install, publication, or execution authority. The
   host turns meaning into a running application.
2. **Programs are static; workflows are data.** A program is compiled Rust,
   validated and installed by the host. A workflow definition is published on
   a branch at run time, even when it was written in Rust and ships with the
   application.
3. **Revision, support, and activation are three different facts.** The
   revision says what the program means. Support says this host can run it
   (it is rostered). Activation says a given branch runs it. Changing
   activation is *adoption* ([§7](#7-adopt-a-new-program-on-a-branch)).

Terms: [Glossary](glossary.md) — see **Program**, **Feature**,
**Contribution**, **Adoption**, **Workflow**, **Installation**.

---

## 2. Declare the schema and its contributions

A schema is composed from contributions. Each contribution declares its own
entities, fields, operations, queries, and invariants.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query/crates/worth-query-certification/examples/product_workflow_support/schema.rs (abridged)
worth_query_application! {
    pub TemporalHostSchema {
        owner: "temporal_host_courtroom",
        version: (1, 0),
        contributions: [TemporalHostContribution],
    }
}

worth_query_application_contribution! {
    pub contribution TemporalHostContribution in TemporalHostSchema {
        identity: "temporal_host_courtroom.example.v1",
        members: |schema| {
            let schema = schema
                .entity(TemporalIntent::reference())
                .aspect(TemporalIntent::reference(), IntentFacts::reference())
                .field(TemporalIntent::reference(), IntentRevisionField::reference())
                // ... more fields, relations, a principal binding ...
                .operation(
                    AmendTemporal::reference()
                        .definition()
                        .no_external_effect()   // typestate: you must choose
                        .no_aftermath()         // typestate: you must choose
                        .finish(),
                )
                .operation_read_field(AmendTemporal::reference(), IntentRevisionField::reference())
                .operation_write(AmendTemporal::reference(), IntentRevisionField::reference())
                .invariant(integrity::definition());
            super::application_entry::declare(schema)
        }
    }
}
```

The host side of the same contribution registers what runs: handlers,
invariants, and conditional nodes.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// .../product_workflow_support/conditional_contribution.rs
impl WorthQueryApplicationContribution<TemporalHostSchema> for TemporalHostContribution {
    type Configuration = TemporalContributionConfiguration;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        contracts.conditional::<TemporalConditional>()?;
        Ok(())
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        setup.invariant(
            TemporalIntegrity::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            integrity::resolve_rule,
        )?;
        setup.handler::<super::application_entry::AmendTemporalBinding, _>(
            super::application_entry::AmendTemporalHandler,
        )?;
        setup.conditional::<TemporalConditional>(configuration)
    }
}
```

- `contracts` is optional and defaults to `Ok(())`.
- A contribution with nothing to configure uses `type Configuration = ();`.
  Bank has three such contributions and installs them with the configuration
  tuple `((), (), ())`.
- A missing handler fails installation before any initial state is written.

The schema and contribution APIs are covered in depth by the
[`worth-query-host` README](../workspaces/worth-query/crates/worth-query-host/README.md)
and the [`worth-query-decl` README](../workspaces/worth-query/crates/worth-query-decl/README.md).
The rest of this guide is about what sits on top of the schema.

---

## 3. Declare features and the program

A **program** is a type that implements `ApplicationProgramDefinition<Schema>`.
It names its contributions, lists its features, and declares its output graph
and its rules. Everything is typed: a connection between two ports whose
values differ does not compile.

All names in this section are on
`worth_query_decl::facade::application_program`, which is also reachable as
`worth_query_host::facade::declaration::application_program`.

### 3.1 Declare a feature

A feature is a marker type with a stable identity and a list of input ports.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query-bank-world/crates/bank-server/src/application_definition/composition.rs
pub struct BankAccountsFeature;
pub struct BankPaymentsFeature;

impl ApplicationFeature<BankSchema> for BankAccountsFeature {
    type Inputs = ApplicationFeatureInputLeaf;   // no input ports
    const IDENTITY: &'static str = "worth.bank.feature.accounts.v1";
}

impl ApplicationFeature<BankSchema> for BankPaymentsFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.bank.feature.payments.v1";
}
```

`ApplicationFeature` also has `const MAJOR: u16 = 1` and `const MINOR: u16 = 0`,
which you may override.

### 3.2 Attach actions to features

`ApplicationFeatureSpec` records which feature owns which actions. Build one
per feature with `ApplicationFeatureSpec::root::<Schema, Feature>()`, add its
actions, and call `finish()`.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/application_definition/composition.rs (abridged)
vec![
    ApplicationFeatureSpec::root::<BankSchema, BankAccountsFeature>()
        .mutation::<CreatePersonalAccountMutationBinding>()
        .mutation::<GrantAccountAccessMutationBinding>()
        .finish(),
    ApplicationFeatureSpec::root::<BankSchema, BankPaymentsFeature>()
        .mutation::<DepositMutationBinding>()
        .mutation::<SendMoneyMutationBinding>()
        .conditional_operation::<PublishApprovedPaymentAssessment>()
        .finish(),
    ApplicationFeatureSpec::root::<BankSchema, BankEstateFeature>()
        .mutation::<FreezeEstateAccountMutationBinding>()
        .operation::<RequestEstateEmergencyAccessOperation>()
        .finish(),
]
```

Builder methods on `ApplicationFeatureSpecBuilder`:

| Method | Attaches |
|---|---|
| `mutation::<Binding>()` | A mutation binding the feature owns |
| `mutation_with_requirement_and_external_input::<B, Rule, Provider>()` | A mutation with a rule requirement and an external input provider |
| `mutation_with_locality_and_change::<B, Scope, Shape>()` | A mutation with a declared locality and change contract |
| `repeated_optional_member::<B, Corr>()` and its `_with_external_input` / `_output_with_locality_and_change` variants | Actions over a repeated, optional member |
| `operation::<Op>()` | A declared operation (on the root-composition builder only) |
| `conditional_operation::<Op>()` and `conditional_operation_with_requirement_and_external_input` | A conditional (clock- or signal-driven) operation |
| `provides::<Port>()` | An output port this feature provides |
| `derived_artifact`, `managed_computation`, `derived_collection` | Derived outputs the feature produces |
| `finish()` | Returns the `ApplicationFeatureSpec` |

`ApplicationFeatureSpec::at::<Schema, Instance, Feature>()` places a feature
inside a named composition instance instead of the root
([§3.4](#34-ports-connections-and-the-output-graph)).

There is also a `worth_query_feature_spec!` macro on the decl facade. It covers
only part of the builder and is not re-exported by the host facade. **Use the
builder.**

### 3.3 Declare the program

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/application_definition/composition.rs
pub(crate) type BankRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationLocalRuleRef<BankSchema, BankPaymentsFeature, BankPostingIntegrity>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

impl ApplicationProgramDefinition<BankSchema> for BankApplication {
    type Contributions = (
        BankAccountsProvider,
        BankPaymentsProvider,
        BankEstateProvider,
    );
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = BankRules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.bank.application.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        bank_feature_specs()
    }
}
```

| Item | Meaning |
|---|---|
| `type Contributions` | A tuple of the contributions this program installs, in order. It fixes the type of the `configuration` argument at install. |
| `type Outputs` | The program's output graph, wrapped in `ApplicationProgramOutputs<..>`. Use `ApplicationNoOutputGraph` when there is none. |
| `type Rules` | A type-level list of rules: `ApplicationRuleList<ApplicationRuleAt<Ref, Point>, Tail>`, ended by `ApplicationRuleLeaf`. |
| `const IDENTITY` | The program's stable identity. |
| `const DERIVED_ARTIFACT_GOVERNANCE` | Optional. `ApplicationDerivedArtifactGovernance::{Compatible, Required}`; defaults to compatible. |
| `fn feature_specs()` | The features and the actions each one owns. |

**Rules.** A rule reference is one of:

- `ApplicationLocalRuleRef<Schema, Feature, Invariant>`: owned by one feature;
- `ApplicationSharedRuleRef<Schema, Invariant>`: owned by no single feature;
- `ApplicationLocalRuleInstanceRef` / `ApplicationSharedRuleInstanceRef`: the
  same, inside a composition instance.

The execution point is `ApplicationCommitBoundary`,
`ApplicationMutationSensitive`, or `ApplicationSnapshotPublication`. The rule's
handler is still registered in the contribution's `configure`, with
`setup.invariant(..)` ([§2](#2-declare-the-schema-and-its-contributions)). A
program selects which declared rules govern its branches.

### 3.4 Ports, connections, and the output graph

Features can pass typed values to each other. A feature *provides* an output
port; another feature *requires* an input port; a connection binds the two;
the program's output graph says which connections produce settled outputs.

The block below is a **shape with illustrative names**. The compiled
counterpart is the consumer fixture at
`workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/application_program.rs`
(features and ports) and
`.../fixtures/consumer_entry/consumer_root/src/application_program/roots.rs`
(the output graph). That fixture takes every Query name from the two facades.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Shape. Illustrative names; see the fixture above for compiled code.
pub struct DraftFeature;
pub struct SummaryFeature;
pub struct DraftOutput;
pub struct DraftInput;
pub struct DraftToSummary;

impl ApplicationFeature<MySchema> for DraftFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "example.draft-feature.v1";
}
impl ApplicationFeature<MySchema> for SummaryFeature {
    type Inputs = ApplicationFeatureInputList<DraftInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "example.summary-feature.v1";
}

impl ApplicationOutputPort<MySchema, DraftFeature> for DraftOutput {
    type Value = DraftValueBinding;            // an ApplicationStructuredValueBinding
    const IDENTITY: &'static str = "draft";
}
impl ApplicationInputPort<MySchema, SummaryFeature> for DraftInput {
    type Value = DraftValueBinding;            // must equal the output's Value
    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}

impl ApplicationOccurrenceConnectionBinding<MySchema, DraftFeature, SummaryFeature>
    for DraftToSummary {}
impl ApplicationConnectionIdentity for DraftToSummary {
    const IDENTITY: &'static str = "example.draft-to-summary.v1";
}

type DraftConnection = ApplicationConnectionRef<
    MySchema, DraftFeature, DraftOutput, SummaryFeature, DraftInput, DraftToSummary,
>;

// The output graph: a root connection and its dependents.
pub type MyOutputRoot = ApplicationOutputGraph<DraftConnection, ApplicationOutputLeaf>;

// In the program:  type Outputs = ApplicationProgramOutputs<MyOutputRoot>;
```

- `ApplicationConnectionRef` only exists when the target port's `Value`
  equals the source port's `Value`. A mismatch is a **compile error**, not a
  validation denial.
- Dependents chain with `ApplicationOutputEdge<Connection, Dependents>`.
  Several roots are a tuple: `ApplicationProgramOutputs<(RootA, RootB)>`.
- **Composition instances.** `trait ApplicationCompositionInstance { const PATH: &'static str; }`
  names a place in the program; `ApplicationRootComposition` is `"root"`. A
  feature placed with `ApplicationFeatureSpec::at::<S, Instance, F>()` lives
  there. A connection that crosses instances uses `ApplicationConnectionInstanceRef`
  and must set `const EXPORTS_ACROSS_INSTANCES: bool = true` on its binding.
  A PATH does not allocate a live occurrence; it names a position in meaning.
- The host side of an output connection implements
  `WorthQueryApplicationRequiredOutputConnection<Schema>` (on
  `worth_query_host::facade::primary_graph`), which turns the source mutation's
  input into an output demand.

### 3.5 Validate

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/application_definition/composition.rs
pub(crate) fn validated_bank_application() -> Result<
    ValidatedApplicationProgram<BankSchema, BankApplication>,
    ApplicationProgramValidationDenial,
> {
    ApplicationProgramAuthoring::<BankSchema, BankApplication>::begin().validated_program()
}
```

- The result is a `ValidatedApplicationProgram<Schema, Program>`. Its fields
  are private. Read it back with `identity()`, `revision()`, `features()`,
  `actions()`, `connections()`, `rules()`, `semantic_description()`, and
  `normalized_manifest()`.
- `revision()` is the content-addressed `ApplicationProgramRevision`: the same
  meaning always gives the same revision.
- A denial is an `ApplicationProgramValidationDenial` with `kind()` and
  `subject()`. Kinds include `DuplicateFeature`, `DuplicateAction`,
  `DanglingFeature`, `MissingRequiredInput`, `UndeclaredOutput`,
  `UnexportedCrossInstanceConnection`, `CyclicConnection`, `DuplicateRule`,
  `IncompleteActionChangeContract`, and `UngovernedDerivedOutput`.

### 3.6 Declare and install a partitioned managed computation

A managed computation declares its input, output artifact, partition key,
execution posture, determinism contract, and finite resource ceiling. Add it
with `managed_computation::<Computation>()` beside its `derived_artifact` in the
feature specification. Install its owner with
`setup.partitioned_computation::<Feature, Computation, _>(owner)` and retain the
returned handle in the handler that uses it.

The owner separates reads from computation. `partitions`, `partition_key`, and
`gather` receive an operation-scoped reader on the caller thread. They can read
only what the operation declares. `compute_partition` receives gathered data
and a checkpoint, with no graph reader. `partitions`, `partition_key`, `gather`, and
`compute_partition` must be pure in the declared input, partition key, and
facts read through the reader: no ambient clocks, mutable globals, changing
owner state, or effects. A completed producer run keeps its partition state.
The next run of the same installed producer, on an equal input value, reuses
an unchanged partition without gathering or computing it again. An impure
function can therefore yield a stale reused result. `complete` receives the
reduced result back on the caller thread. Preparation precedes kernel dispatch; completion
follows successful computation and reduction.

In the executable examples below, `run_inventory` and `run_peak` call
`installed.prepare(reader, input)` and return
`prepared.into_decision_plan(..)`. The plan carries compute inputs, not a live
reader. Its completion closure handles the typed computation result and returns
a `HandlerResult`. Use this shape when a producer's computation can run beside
another required producer: Query dispatches compute and invokes completion in
canonical member order. Override `OperationHandler::prepare_decision` to return
this plan. Keep graph reads in preparation and return the decision from
completion; the ordinary candidate-building and commit steps follow on the
owner. A handler that does not compute can keep its `decide` implementation:
the default `prepare_decision` wraps it in an immediate plan.

Partition reuse is distinct from clean reuse of an entire output and from
input cutoff. It applies only to the runs of a producer. A producer is a
`WorthQueryApplicationProducerBinding` declared with
`contracts.producer::<Binding>()` and registered with `setup.producer`; its
`Operation` is the mutation binding the producer runs. Installation decides
retention once: it compares the owner's `Operation` type with the `Operation`
of each declared producer's mutation binding. When a declared producer runs that
operation, a run made under that producer leaves its partition state on the
record its attempt publishes. When no declared producer runs it, a run is never
retained: no input or item digest is taken, and every run gathers and computes
every partition.

A retained computation still gathers and computes every partition when any of
these holds:

- No output record is published at the run's address: the first run.
- The operation is invoked outside a producer invocation, or an earlier
  partitioned computation in the same handler already took the kept state. A
  handler that runs several partitioned computations in one attempt keeps none
  of them.
- The installed owner differs, the installed producer edition differs, or the
  input value's canonical encoding differs. A reinstalled owner with the same
  types is a different installation.
- Observing the kept facts could exceed the declared work the run has left.
- The selected record holds no computation state. A run that stopped before it
  completed, whose charge could not be measured, or whose reservation was
  refused leaves none. A previous attempt that ran no partitioned computation
  leaves none. Checkpoint restoration and republication carry none. A record
  whose successor took over its state holds none.

Otherwise the run names and keys again only what changed, and gathers and
computes again only the partitions whose members or facts changed. A partition
is carried only when every fact its owner calls read in the previous run has
the same content at this attempt's snapshot; a fact that cannot be observed
counts as changed. A run's result, charged work, structural span, and the
partition a work ceiling names never depend on reuse: carried charges replay in a full run's
order, `charged_work()` counts every partition's kernel and every combine
whether the partition was computed again or carried, and a work ceiling names
the partition a full run would name. A carried partition replays its accepted
kernel charges; if the remaining ceiling lies inside those charges, it replays
the original checkpoint steps to preserve the full run's stop. Memory reserves
what is physically held, so a memory denial may depend on reuse. An in-flight
cancellation or deadline can also distinguish physical runs; neither is a
promise that a computation reaches the same checkpoint at the same time.

The example groups warehouse quantities by storage zone and sums them. Stable
item IDs identify inventory lines; the zone key defines partition membership.
Neither depends on worker assignment. The generic schema, feature, artifact,
and operation parameters stand for your application's declarations. The
`Reuse` marker names declaration vocabulary; partition reuse depends on the
state kept from the previous run and the purity contract above. Both examples
in this section are compiled as written: the consumer fixture builds and runs
`workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/warehouse_inventory.rs`
and `.../sensor_telemetry.rs`, and a test fails when a block here and its file
differ.

The executable examples are exercised by
[`compiled_examples_warehouse_inventory_total` and `compiled_examples_sensor_telemetry_peak`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/compiled_examples.rs).
They assert a warehouse sum of `13.0` and a sensor maximum of `850` through a
host mutation.

<!-- compiled: warehouse_inventory.rs -->
```rust
use serde::Serialize;
use std::marker::PhantomData;
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{
        application_operation::ApplicationMutationBinding, application_program::*,
        application_schema::ApplicationSchema,
    },
    primary_graph::{
        DecisionReader, HandlerResult, WorthQueryDecisionPlan,
        WorthQueryPrimaryGraphInstallationDenial,
    },
};

#[derive(Clone, Serialize)]
pub struct InventoryLine {
    id: u64,
    zone: u64,
    quantity: f64,
}
impl ChargedBytes for InventoryLine {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl ApplicationComputationPartition for InventoryLine {
    const IDENTITY: &'static str = "inventory.line.v1";
}

pub struct InventoryInput;
impl ApplicationComputationInput for InventoryInput {
    type Value = Vec<InventoryLine>;
    const IDENTITY: &'static str = "inventory.input.v1";
}
#[derive(Serialize)]
pub struct Zone(u64);
impl ApplicationComputationPartition for Zone {
    const IDENTITY: &'static str = "inventory.zone.v1";
}
pub struct InventoryReuse;
impl ApplicationComputationReuse for InventoryReuse {
    const IDENTITY: &'static str = "inventory.reuse.v1";
}
pub struct InventoryStopped;
impl ApplicationComputationStopped for InventoryStopped {
    const IDENTITY: &'static str = "inventory.stopped.v1";
}

pub struct InventoryTotal<A>(PhantomData<fn() -> A>);
impl<S, F, A> ApplicationManagedComputation<S, F> for InventoryTotal<A>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
{
    type Input = InventoryInput;
    type Output = A;
    type Partition = Zone;
    type Reuse = InventoryReuse;
    type Stopped = InventoryStopped;
    const IDENTITY: &'static str = "inventory.total.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(100_000, 1_048_576);
    // DETERMINISM defaults to canonical bitwise equality.
}

pub struct InventoryOwner<Op>(PhantomData<fn() -> Op>);
impl<S, F, A, Op> WorthQueryPartitionedComputationOwner<S, F, InventoryTotal<A>>
    for InventoryOwner<Op>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
{
    type Operation = Op;
    type Item = InventoryLine;
    type Gathered = Vec<f64>;
    type PartitionResult = f64;
    type Output = f64;
    type Stopped = u32;

    fn partitions(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        input: &Vec<InventoryLine>,
    ) -> Result<
        WorthQueryComputationPartitionPlan<InventoryLine>,
        WorthQueryComputationInputDenial<u32>,
    > {
        Ok(WorthQueryComputationPartitionPlan::keyed(
            input.iter().cloned(),
            |line| PartitionItemId(line.id),
        ))
    }

    fn partition_key(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        _input: &Vec<InventoryLine>,
        item: &InventoryLine,
    ) -> Result<Zone, WorthQueryComputationInputDenial<u32>> {
        Ok(Zone(item.zone))
    }

    fn gather(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        _input: &Vec<InventoryLine>,
        partition: WorthQueryComputationPartitionMembers<'_, Zone, InventoryLine>,
    ) -> Result<Vec<f64>, WorthQueryComputationInputDenial<u32>> {
        Ok(partition.items().map(|(_, line)| line.quantity).collect())
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Zone, Vec<f64>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<u32>> {
        let mut total = 0.0;
        for quantity in partition.gathered() {
            checkpoint.advance(1)?;
            total += quantity;
        }
        Ok(total)
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<f64> {
        WorthQueryDeterministicReducer::canonical(|| 0.0, |left, right| left + right)
    }

    fn complete(&self, reduced: f64) -> Result<f64, u32> {
        Ok(reduced)
    }
}

pub fn declare_inventory<S, F, A>() -> ApplicationFeatureSpec
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
{
    ApplicationFeatureSpec::root::<S, F>()
        .derived_artifact::<A>()
        .managed_computation::<InventoryTotal<A>>()
        .finish()
}

pub fn install_inventory<S, F, A, Op>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
) -> Result<
    WorthQueryInstalledPartitionedComputation<S, F, InventoryTotal<A>, InventoryOwner<Op>>,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
{
    setup.partitioned_computation::<F, InventoryTotal<A>, _>(InventoryOwner(PhantomData))
}

pub fn run_inventory<S, F, A, Op, B, Value: 'static, Domain: 'static>(
    installed: &WorthQueryInstalledPartitionedComputation<
        S,
        F,
        InventoryTotal<A>,
        InventoryOwner<Op>,
    >,
    reader: &mut DecisionReader<'_, '_, '_, S, B>,
    input: &Vec<InventoryLine>,
    complete: impl FnOnce(
            Result<(f64, u64), WorthQueryPartitionedComputationDenial<u32>>,
        ) -> HandlerResult<Value, Domain>
        + 'static,
) -> WorthQueryDecisionPlan<Value, Domain>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
    B: ApplicationMutationBinding<S, Operation = Op>,
{
    match installed.prepare(reader, input) {
        Ok(prepared) => prepared.into_decision_plan(move |computed| {
            complete(computed.and_then(|computed| {
                let charged_work = computed.charged_work();
                computed.complete().map(|value| (value, charged_work))
            }))
        }),
        Err(denial) => WorthQueryDecisionPlan::immediate(complete(Err(denial))),
    }
}
```

A managed computation declares only itself. `managed_computation` adds its
declaration to the feature; the feature's operations, ports, and artifacts are
declared on the same builder. Program validation refuses a computation whose
output artifact its feature does not declare
(`MissingManagedComputationArtifact`), whose partition key does not fit its
execution posture (`MismatchedManagedComputationPartition`), whose resource
ceiling has a zero (`InvalidManagedComputationResources`), or whose identity
repeats within the feature (`DuplicateManagedComputation`). Installation
refuses, with a `WorthQueryPrimaryGraphInstallationDenial`, an owner installed
through the call for the other execution posture, a computation whose
determinism contract is `ContractEquivalent`, and a second owner for one
computation.

Supply a unique, stable ID for every line: duplicate item identities are
denied. Keys use canonical encoding, and partitions are ordered by the
identity derived from that encoding, not by the numeric order of zones. The
reducer fixes its association tree, so floating-point reduction preserves the
same result bits at every admitted worker count.

Choose partition membership from the input and observed facts before placement.
The host's installed policy bounds worker count, memory and aggregate request
work. Serial placement uses a bounded inline lease with no worker pool; leased
placement can use the process authority's workers. Neither may rename a partition
or change its reduction association.

`checkpoint.advance` charges kernel work and checks interruption. Framework
planning and reduction also contribute charged work; `charged_work()` reports
the computation's total. Handle the typed denial before authoring a candidate
from the result. Computing and completing a value do not commit it.

`prepare`, `compute`, and `complete` each fail with a
`WorthQueryPartitionedComputationDenial<Stopped>`, where `Stopped` is the
owner's own stop type. A denied run has no result. Handle every variant:

| Denial | What it reports |
|---|---|
| `Owner(stopped)` | The owner refused to name the input's items, to key one, or to complete the reduced result. |
| `Read(..)` | A read was refused while the owner named the items or keyed one. |
| `Partition { partition, cause }` | One partition stopped. `cause` is a `WorthQueryComputationPartitionStop`: `Owner(stopped)`, `Read(..)`, `Panicked`, `Resource(..)`, `Interrupted(..)`, or `NestedPatternStopped`. When several partitions stop, the one with the least partition identity is reported. No partition is computed unless every partition gathers. |
| `PartitionIdentityCollision { partition }` | Two different keys derive one partition identity. Every run is denied while both keys are in the input. |
| `DuplicateItem { item }` | The plan named one item identity twice; the least such identity is reported. |
| `InputNotEncodable(..)`, `ItemNotEncodable { item, denial }` | The input value or an item refused canonical encoding, so a retained run has no digest for it. |
| `KeyNotEncodable { item, denial }` | An item's partition key refused canonical encoding. |
| `Resource(..)` | A `WorthQueryManagedComputationResourceDenial`: planning, deriving partition identities, or reducing did not fit the declared work or bytes, or the request's execution refused it. |
| `Interrupted(..)` | `Cancelled` or `DeadlineExceeded`: the request was canceled or ran out of time outside any one partition. |
| `NestedPatternStopped` | A pattern started inside a partition's kernel stopped, outside any one partition. |
| `ReducerPanicked` | The reducer panicked. |
| `ReducedEncodingInvalid` | A reduced value's canonical bits disagree with the length it declares. |
| `ReductionInputInvalid(..)` | The reduction refused the identities or values it was given. The platform builds both from one canonical list, so no run is expected to produce it; match it for exhaustiveness. |

Inside `compute_partition`, `checkpoint.advance(work)?` converts a refused
checkpoint (`WorthQueryManagedComputationCheckpointDenial`: `Resource`,
`Interrupted`, or `NestedPatternStopped`) into the kernel's
`WorthQueryManagedComputationDenial`. A refusal attributed to a partition
surfaces as `Partition { partition, cause }`; one that belongs to no partition
surfaces as the top-level `Resource`, `Interrupted`, or `NestedPatternStopped`.

A second example, from sensor telemetry, keeps the same traits and changes the
item, the key, and the reducer. It groups readings by station, takes each
station's peak, and reduces the peaks to one maximum. Integer maximum is exact,
so the reducer needs no floating-point care.

<!-- compiled: sensor_telemetry.rs -->
```rust
use serde::Serialize;
use std::marker::PhantomData;
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{
        application_operation::ApplicationMutationBinding, application_program::*,
        application_schema::ApplicationSchema,
    },
    primary_graph::{
        DecisionReader, HandlerResult, WorthQueryDecisionPlan,
        WorthQueryPrimaryGraphInstallationDenial,
    },
};

#[derive(Clone, Serialize)]
pub struct Reading {
    id: u64,
    station: u32,
    millivolts: u64,
}
impl ChargedBytes for Reading {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl ApplicationComputationPartition for Reading {
    const IDENTITY: &'static str = "telemetry.reading.v1";
}

pub struct TelemetryInput;
impl ApplicationComputationInput for TelemetryInput {
    type Value = Vec<Reading>;
    const IDENTITY: &'static str = "telemetry.input.v1";
}
#[derive(Serialize)]
pub struct Station(u32);
impl ApplicationComputationPartition for Station {
    const IDENTITY: &'static str = "telemetry.station.v1";
}
pub struct TelemetryReuse;
impl ApplicationComputationReuse for TelemetryReuse {
    const IDENTITY: &'static str = "telemetry.reuse.v1";
}
pub struct TelemetryStopped;
impl ApplicationComputationStopped for TelemetryStopped {
    const IDENTITY: &'static str = "telemetry.stopped.v1";
}

pub struct PeakReading<A>(PhantomData<fn() -> A>);
impl<S, F, A> ApplicationManagedComputation<S, F> for PeakReading<A>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
{
    type Input = TelemetryInput;
    type Output = A;
    type Partition = Station;
    type Reuse = TelemetryReuse;
    type Stopped = TelemetryStopped;
    const IDENTITY: &'static str = "telemetry.peak.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(100_000, 1_048_576);
}

pub struct PeakOwner<Op>(PhantomData<fn() -> Op>);
impl<S, F, A, Op> WorthQueryPartitionedComputationOwner<S, F, PeakReading<A>> for PeakOwner<Op>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
{
    type Operation = Op;
    type Item = Reading;
    type Gathered = Vec<u64>;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u32;

    fn partitions(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        input: &Vec<Reading>,
    ) -> Result<WorthQueryComputationPartitionPlan<Reading>, WorthQueryComputationInputDenial<u32>>
    {
        Ok(WorthQueryComputationPartitionPlan::keyed(
            input.iter().cloned(),
            |reading| PartitionItemId(reading.id),
        ))
    }

    fn partition_key(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        _input: &Vec<Reading>,
        item: &Reading,
    ) -> Result<Station, WorthQueryComputationInputDenial<u32>> {
        Ok(Station(item.station))
    }

    fn gather(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        _input: &Vec<Reading>,
        partition: WorthQueryComputationPartitionMembers<'_, Station, Reading>,
    ) -> Result<Vec<u64>, WorthQueryComputationInputDenial<u32>> {
        Ok(partition
            .items()
            .map(|(_, reading)| reading.millivolts)
            .collect())
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Station, Vec<u64>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        let mut peak = 0_u64;
        for millivolts in partition.gathered() {
            checkpoint.advance(1)?;
            peak = peak.max(*millivolts);
        }
        Ok(peak)
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, |left, right| (*left).max(*right))
    }

    fn complete(&self, reduced: u64) -> Result<u64, u32> {
        Ok(reduced)
    }
}

pub fn declare_peak<S, F, A>() -> ApplicationFeatureSpec
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
{
    ApplicationFeatureSpec::root::<S, F>()
        .derived_artifact::<A>()
        .managed_computation::<PeakReading<A>>()
        .finish()
}

pub fn install_peak<S, F, A, Op>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
) -> Result<
    WorthQueryInstalledPartitionedComputation<S, F, PeakReading<A>, PeakOwner<Op>>,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
{
    setup.partitioned_computation::<F, PeakReading<A>, _>(PeakOwner(PhantomData))
}

pub fn run_peak<S, F, A, Op, B, Value: 'static, Domain: 'static>(
    installed: &WorthQueryInstalledPartitionedComputation<S, F, PeakReading<A>, PeakOwner<Op>>,
    reader: &mut DecisionReader<'_, '_, '_, S, B>,
    input: &Vec<Reading>,
    complete: impl FnOnce(
            Result<(u64, u64), WorthQueryPartitionedComputationDenial<u32>>,
        ) -> HandlerResult<Value, Domain>
        + 'static,
) -> WorthQueryDecisionPlan<Value, Domain>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
    B: ApplicationMutationBinding<S, Operation = Op>,
{
    match installed.prepare(reader, input) {
        Ok(prepared) => prepared.into_decision_plan(move |computed| {
            complete(computed.and_then(|computed| {
                let charged_work = computed.charged_work();
                computed.complete().map(|value| (value, charged_work))
            }))
        }),
        Err(denial) => WorthQueryDecisionPlan::immediate(complete(Err(denial))),
    }
}
```

Planning does not depend on the order the input holds its readings in: items
are met in item identity order and partitions in partition identity order.

To use either example, declare the artifact and managed computation on the
feature, install the owner during contribution setup, and store the returned
handle with that operation's handler. Call its `run_inventory` or `run_peak`
function from `prepare_decision`. Declare every graph fact read by
`partitions`, `partition_key`, and `gather` on the operation. For retained
per-partition reuse, also declare and install the producer that runs that
operation; a direct mutation computes successfully but keeps no producer state.
The inventory example sums warehouse quantities; the telemetry example reduces
sensor peaks. Both use the same consumer API, through `worth-query-host`'s
facade and the declarations exported by `worth-query-decl`.

**The handler contract.** Within `OperationHandler<Schema, Binding>`, the
current signature and default implementation are:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
    fn prepare_decision(
        &self,
        input: &Binding::Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> WorthQueryDecisionPlan<Binding::Decision, Binding::Denial> {
        WorthQueryDecisionPlan::immediate(self.decide(input, reader))
    }
```

Override this method to return the prepared computation's decision plan. The
following complete handler excerpt is compiled in the planar geometry example.
Its `Decision` alias expands the reader's three lifetimes and binding. Its
installed `run` prepares the warehouse or telemetry computation shown above;
completion maps the computed result to the operation's decision. `decide`
resolves a declared scope and builds its mutation target. `build_candidate`
authors the field change after computation completes.

The fixture's schema declares `Body`, `PositionY`, and the entry-set facts;
`RegionTotalsDemandBinding` binds those reads and writes. The full declarations
and installation accompany this excerpt in the consumer fixture.

<!-- compiled: checkpoint_recovery/computation_partition/demand.rs -->
```rust
pub(super) type Decision<'borrow, 'reader, 'runtime, Schema> =
    DecisionReader<'borrow, 'reader, 'runtime, Schema, RegionTotalsDemandBinding<Schema>>;

/// What the handler runs over the demanded set inside its decision.
type Run<Schema> = Box<
    dyn Fn(
            &mut Decision<'_, '_, '_, Schema>,
            &WorthQueryInvariantEntityIdentity<Schema, EntrySet>,
        ) -> WorthQueryDecisionPlan<(), PlanarMutationDenial>
        + Send
        + Sync,
>;

/// Decides a demand: runs what it was installed with over the demanded set,
/// then moves the scope's source so every decision is a new one.
pub(super) struct RegionTotalsHandler<Schema: TopologySchemaBinding>(Option<Run<Schema>>);

impl<Schema: TopologySchemaBinding> RegionTotalsHandler<Schema> {
    /// The topology's own handler: nothing to run.
    pub(super) const fn idle() -> Self {
        Self(None)
    }

    pub(super) fn running(
        run: impl Fn(
                &mut Decision<'_, '_, '_, Schema>,
                &WorthQueryInvariantEntityIdentity<Schema, EntrySet>,
            ) -> WorthQueryDecisionPlan<(), PlanarMutationDenial>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self(Some(Box::new(run)))
    }
}

impl<Schema: TopologySchemaBinding> OperationHandler<Schema, RegionTotalsDemandBinding<Schema>>
    for RegionTotalsHandler<Schema>
{
    fn decide(
        &self,
        input: &RegionTotalsDemand,
        reader: &mut Decision<'_, '_, '_, Schema>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Body>, PlanarMutationDenial> {
        let decided = (|| {
            let scope = reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            if reader.field(&scope, PositionY::reference())?.is_none() {
                return Ok(None);
            }
            reader.mutation_target(&scope).map(Some)
        })();
        match decided {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn prepare_decision(
        &self,
        input: &RegionTotalsDemand,
        reader: &mut Decision<'_, '_, '_, Schema>,
    ) -> WorthQueryDecisionPlan<WorthQueryInvariantMutationTarget<Schema, Body>, PlanarMutationDenial>
    {
        let plan = if let Some(run) = &self.0 {
            let set = match reader.resolve_entity(EntrySetKey::reference(), input.entries.clone()) {
                Ok(set) => set,
                Err(error) => {
                    return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(error))
                }
            };
            run(reader, &set)
        } else {
            WorthQueryDecisionPlan::immediate(HandlerResult::Completed(()))
        };
        let decided = self.decide(input, reader);
        plan.and_then(move |()| decided)
    }

    fn candidate_requirements(
        &self,
        _: &RegionTotalsDemand,
        _: &WorthQueryInvariantMutationTarget<Schema, Body>,
    ) -> ApplicationCandidateRequirements {
        RegionTotalsDemandBinding::<Schema>::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &RegionTotalsDemand,
        target: WorthQueryInvariantMutationTarget<Schema, Body>,
        writer: &mut CandidateWriter<'_, Schema, RegionTotalsDemandBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let written = writer.projected_entity(&target).and_then(|entity| {
            writer.write_field(&entity, PositionY::reference(), input.replacement_y)
        });
        match written {
            Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                changed_vertices: 1,
            }),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
```

---

## 4. Install the application graph

Installation turns a validated program, a schema declaration, the
contributions' configuration, and resource limits into a running runtime. The
`initial_state` closure seeds the primary graph before the first publication.

### 4.1 One program

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query/crates/worth-query-certification/examples/product_workflow_support/application.rs
use worth_query_host::facade::{application_installation, primary_graph, runtime};

let declaration = TemporalHostSchema::declaration().expect("the example schema is valid");
let runtime = application_installation::in_memory_program(
    program::validated_program(),                  // ValidatedApplicationProgram<S, P>
    declaration,                                   // ApplicationSchemaDeclaration<S>
    (TemporalContributionConfiguration { clock_source },), // one entry per contribution
    example_limits(),                              // WorthQueryInMemoryApplicationLimits
    |_phase, graph, installed| {                           // seed the primary graph
        let principal_binding = installed
            .principal_binding(TemporalPrincipalBinding::reference())
            .expect("the temporal principal binding must install");
        seed_graph(graph, &principal_binding, gate);
        Ok(())
    },
)
.expect("the validated temporal program must install");
```

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Same file: limits.
application_installation::WorthQueryInMemoryApplicationLimits::new(
    product_world_resources(),
    runtime::WorthQueryApplicationCandidateResourceProfile::physical_resources(5_120, 2_048)
        .expect("valid candidate limits"),
    runtime::WorthQueryApplicationQueryResourceProfile::bounded(5_120, 2_048, usize::MAX, 128)
        .expect("valid query limits"),
    primary_graph::SignalConditionalEvaluationBudget::development(),
)
```

The result is a `WorthQueryProgramApplicationRuntime<Schema, Program>`.

### 4.2 Several program revisions: the roster

A host that must be able to adopt a successor program rosters it at install.
Bank installs `BankApplication` and rosters `BankApplicationP1` beside it:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/identity_runtime/installation.rs (abridged)
let program = validated_bank_application()?;
let successor = validated_bank_application_p1()?;
let declaration = BankSchema::declaration()?;
let runtime = in_memory_rostered_program(
    program,
    WorthQueryApplicationProgramRoster::new().support(successor),
    declaration,
    ((), (), ()),          // three contributions, none configured
    limits,
    initialize,            // |_phase, graph, installed| { ... Ok(()) }
)?;
```

Other constructors on `application_installation`:
`in_memory_program_with_authorization_time_source`,
`in_memory_rostered_program_with_authorization_time_source`, and the
`*_from_checkpoint` variants.

A reopened downstream demand may consume a checkpoint root before any separate
root demand. Query first verifies the complete retained producer facts and native
output witness, then retains bounded custody of that exact settlement through
downstream publication. This static custody contains no executable source query:
changing the root invalidates the old result and requires genuine typed source
admission before it can refresh. Ordinary missing or retired managed settlements
continue to refuse publication.

### 4.3 What the runtime gives you

`WorthQueryProgramApplicationRuntime` dereferences to the primary-graph
application runtime, so `request(..)`, `discovery()`, `current_world()`,
`installed_schema()`, `branches()`, and `conditional::<T>()` are all available
on it. Its own methods:

| Method | Returns |
|---|---|
| `runtime()` | The underlying primary-graph application runtime |
| `installed_program()` | The installed program meaning: `identity`, `revision`, `features`, `actions`, `connections`, `rules` |
| `contains_action::<Binding>()`, `contains_output_root::<Root>()` | Whether the installed program owns an action or output root |
| `supported_program::<P>()` | `Option<WorthQuerySupportedProgramHandle>` for a rostered program. The handle implements the `WorthQueryProgramOwner` trait; import the trait to call `owned_revision()`, which is the adoption target |
| `retire_program_support(&revision)` | Retires support for a revision no branch still needs |
| `retain_workflow_spec(..)` | Binds a workflow vocabulary ([§6.2](#62-install-the-workflow-vocabulary-once)) |
| `close_conditional_runtime()` | Closes conditional resources at shutdown |

`discovery()` lists query requests, mutations, queries, and fields.
Features, connections, and rules come from `installed_program()`, not from
discovery.

---

### 4.4 Checkpoint program transitions

Ordinary `in_memory_rostered_program_from_checkpoint` requires the recovered
activation to name a supported program. For an app-owned predecessor mapping,
`in_memory_rostered_program_from_checkpoint_with_transition` checks an exact
`WorthQueryCheckpointProgramPredecessor` rendering and admits the current target
through the ordinary program/roster installation path. The predecessor rendering
never becomes a program revision or a roster member.

Its bounded `WorthQueryCheckpointMigrationWriter` authors new typed entity seeds
and relations between entities created in that batch. It currently cannot read or
rewrite recovered records, or link a new record to an existing endpoint. Query
commits those effects, the target activation and complete supported entity
revalidation in one native candidate before exposing a World. Accepted outputs,
retained workflows and relation-scoped rules require further migration support
and are refused before authoring. Native candidate/publication limits remain in
force alongside explicit selection and authoring bounds.

A deferred native settlement returns `CheckpointTransitionDeferred` with the
exact unpublished repair custody. Consuming `repair_to_checkpoint` returns a
target checkpoint after acknowledgment, or the same capsule if repair/capture
stops. `CheckpointTransitionCaptureStopped` specifically retains the acknowledged
phase when its first checkpoint capture stops. A later installation failure returns
`CheckpointTransitionAcknowledged`
with the already acknowledged target checkpoint and the original phase denial;
fix the installation configuration and ordinary-restore that checkpoint. A
terminal performed settlement failure has its own typed denial and issues no
acknowledged successor.

---

## 5. Run the program

Requests start from the runtime with `WorthQueryApplicationRequestExt`
(`worth_query_host::facade::application_entry`). Every request names two
things: **who** is asking (an authenticated principal) and **how long** it may
run (a request scope).

### 5.1 Who is asking: principal and request scope

A **principal** comes from an authentication adapter the host admits against
the installed schema. The adapter validates a credential and returns an
external identity. Query maps that identity to an application principal
through the schema's principal binding.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// .../examples/product_workflow_support/application.rs and read.rs (abridged)
use worth_query_host::facade::admission::authenticated_principal::{
    admit_authentication_adapter, WorthQueryAuthenticationAdapterAdmission,
    WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod,
};

let adapter = admit_authentication_adapter(
    application.runtime.installed_schema(),
    WorthQueryAuthenticationAdapterAdmission::new(
        WorthQueryAuthenticationAudience::new("host")?,
        WorthQueryAuthenticationMethod::new("example")?,
    ),
    IdentityAdapter,          // your impl of WorthQueryAuthenticationAdapter
)?;
let principal = adapter.authenticate(credential, &scope).await?;
// -> WorthQueryAuthenticatedExternalPrincipal<Schema>
```

`IdentityAdapter` implements `WorthQueryAuthenticationAdapter`: it names its
`Credential` type and returns a `WorthQueryValidatedExternalPrincipal` from
`validate` (see `.../product_workflow_support/adapters.rs`). The external
identity must already be bound to a principal in the graph. The example does
that while seeding, in the `initial_state` closure from §4.1:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// .../product_workflow_support/application.rs
graph.bind_principal(
    principal_binding,        // installed.principal_binding(MyPrincipalBinding::reference())
    primary_graph::WorthQueryApplicationPrincipalKey::new("product-example")?,
    1_u64,                    // the application's own principal id
    declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
        "https://issuer.example",
        "product-example",
    )?,
    declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
)?;
```

Each executing host call opens one advancement under its installed execution
policy before its first read. Bootstrap installation lends that one request to
preparation, recovery, seeding, and publication. Its memory and aggregate work
limits apply in serial placement as well as leased placement. Internal phases
borrow custody; staged products carry facts and never retain the request.

Reentering an executing public door from an active host callback or its
descendant worker is refused with `ExecutionRequest(NestedOpening)`. An unrelated
execution lease remains the separate `Resource(NestedLeaseMisuse)` cause.
An unrelated thread's public call owns its own request. Borrow the callback's phase
for internal execution rather than opening another budget. A request
already canceled or past its deadline is refused at request admission as
`ExecutionRequest(Interrupted(Cancelled))` or
`ExecutionRequest(Interrupted(DeadlineExceeded))`, before principal resolution
or a reader. Mutation opening refusals use the same cause projection as
queries. An open live subscription is the exception: its `next` ends the
subscription with the `Cancelled` or `DeadlineExceeded` outcome, and every
later `next` reports `Closed`.
An opening refusal is `ExecutionRequest`, not a handler failure: no handler ran.
`ExecutionRequest` also carries resource and misuse refusals encountered while
an admitted request is executing.
An interruption observed after admission keeps the executing owner's cancellation
or timeout outcome.
For an output demand refresh these are `Cancelled` and `TimedOut`; they are
separate from admission's `ExecutionRequest(Interrupted(..))` refusals.
Serial and leased placement both use checked leases. A reservation denied by
the policy reports `MemoryLimit { requested, admitted, level }` with the policy
level; the request's memory limit is not waived by serial placement.

A stopped read is not a missing record or a stale index. Entity resolution
separates `Stopped(cause)` from `Refused(kind)`; handle the typed stop instead
of retrying it as absence. A pinned read checks the live request when the pin
is acquired and holds the selected root and index payload for later reads.

Zero memory reports a policy `MemoryLimit { requested, admitted: 0, level }`
at every placement, including its requested bytes.

A **request scope** carries an optional deadline and a cancellation token. With no deadline,
there is no time limit; work and memory remain policy-bounded, and the request
remains cancellable. It is
not an entity and grants nothing:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// .../product_workflow_support/adapters.rs
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

let cancellation = WorthQueryCancellationSource::new();
let scope = WorthQueryRequestScope::new(Some(Instant::now() + Duration::from_secs(60)), cancellation.token());
```

The entity an operation acts on is a different thing: the intent's *scope
binding*, shown next.

### 5.2 Mutations: intent, binding, handler

A mutation has three parts:

| Part | Trait | Written by | Role |
|---|---|---|---|
| Intent | `ApplicationMutationIntent<Schema>` | Your request code | The value you pass to `mutate(..)`: its input and the entity it targets |
| Binding | `ApplicationMutationBinding<Schema>` | Declared once | Ties the operation to its input, result, denial, decision type, idempotency key, principal binding, scope field, and candidate ceilings |
| Handler | `OperationHandler<Schema, Binding>` | Registered in the contribution's `configure` | Decides, then builds the candidate |

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// .../examples/product_workflow_support/application_entry.rs (abridged)
impl ApplicationMutationBinding<TemporalHostSchema> for AmendTemporalBinding {
    type Input = AmendTemporalInput;
    type Result = AmendTemporalResult;
    type Denial = AmendTemporalDenial;
    type IdempotencyKey = u64;
    type Operation = AmendTemporal;              // the operation declared in the schema
    type Decision = WorthQueryInvariantMutationTarget<TemporalHostSchema, TemporalIntent>;
                                                 // what decide() hands to build_candidate
    type ScopeBinding = AmendTemporalScope;      // which entity field selects the target
    type SourceExpectation = NoApplicationMutationSource;
    // ... value bindings for input, result, and denial; principal binding types ...
    const IDENTITY: &'static str = "worth.query.example.temporal-amend.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.example.temporal-amend-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.example.temporal-amend-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = /* fixed ceilings */;
    // fn scope_field, principal_binding
}

impl ApplicationMutationIntent<TemporalHostSchema> for AmendTemporalIntent {
    type Binding = AmendTemporalBinding;
    fn input(&self) -> &AmendTemporalInput { &self.amendment }
    fn scope_binding(&self) -> AmendTemporalScope {
        AmendTemporalScope::new(IntentIdentityField::reference(), self.identity.clone())
    }
}

impl OperationHandler<TemporalHostSchema, AmendTemporalBinding> for AmendTemporalHandler {
    fn decide(&self, input: &AmendTemporalInput, reader: &mut DecisionReader<'_, '_, '_, _, _>)
        -> HandlerResult<Decision, AmendTemporalDenial> { /* read, then decide */ }
    // `Decision` here is the binding's Decision type (see above).
    fn candidate_requirements(&self, ..) -> ApplicationCandidateRequirements { .. }
    fn build_candidate(&self, input: &AmendTemporalInput, target: Decision, writer: &mut CandidateWriter<'_, _, _>)
        -> HandlerResult<AmendTemporalResult, AmendTemporalDenial> { /* write fields */ }
}
```

The full, compiling version of all three is
`.../examples/product_workflow_support/application_entry.rs` (about 320
lines). The schema registers the binding with
`.application_mutation_binding::<AmendTemporalBinding>()`, and the
contribution registers the handler with `setup.handler::<Binding, _>(..)`
([§2](#2-declare-the-schema-and-its-contributions)).
`worth_query_mutation_binding!` (decl facade) writes the binding for you in
common cases.

The input and idempotency key types derive `serde::Serialize`, and that is all
a binding says about retries. Query derives both identities from a canonical
encoding of what `Serialize` emits, so a retry that reuses a key with any
changed serialized field is refused as `IdempotencyIntentDrift` rather than
replayed. Bindings never write these hashes: a request encodes its key and
input once, at the entry point, into
`ApplicationMutationIdentities::<Schema, Binding>::encode(&key, &input)`. Every
later step (admission, the handler, the commit) reuses those identities and
never encodes again, and a denial says whether the key or the input failed to
encode. A workflow operation encodes its input once when it binds to the
workflow transition, as an `ApplicationEncodedInput`, and the request's
identities reuse it. The two derivations are reported in the receipt's
admission-phase canonical work, for a replayed retry as for a fresh commit.

Application values outside mutation requests can use the same canonical
encoder through `application_value_identity`. Choose `DomainValue` and a
namespace that names the value's meaning and semantic version:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
use worth_query_decl::facade::application_operation::{
    application_value_identity, ApplicationValueIdentityDomain,
};

let label = application_value_identity(
    ApplicationValueIdentityDomain::DomainValue,
    "example.label.v1",
    "catalog-item",
)?;
```

For example, a catalog group can derive its identity from every field that
determines membership, including optional fields:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
#[derive(serde::Serialize)]
struct CatalogGroup<'a> {
    product: &'a str,
    axis: Option<u8>,
}

let group = application_value_identity(
    ApplicationValueIdentityDomain::DomainValue,
    "example.catalog-group.v1",
    &CatalogGroup { product: "catalog-item", axis: None },
)?;
let digest = group.identity();
let encoding_work = group.work();
```

An absent axis differs from `Some(2)`, and changing the namespace or version
changes the identity. Maps are ordered by their encoded keys; sequences keep
their supplied order and floats keep their exact bits. Serialization rejection
returns `CodecRejected` without a digest. The reported work describes this
encoding; it does not charge a live request automatically. A digest grants no
authorization, currentness or publication authority, and does not replace a
mutation request's input/idempotency identities. Include every meaningful
distinction in the serialized value: skipped fields, lossy custom serializers
and untagged variants can erase distinctions the encoder never sees. Use an
explicit version change when the application's identity meaning changes.

The binding is part of the intent too. Only typed constructors build a
`WorthQueryApplicationIdempotencyBinding`, and a mutation request builds its
own with `WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities)`
from the `ApplicationMutationIdentities` it encoded, which names the binding
without encoding anything again. The handler takes that same
`ApplicationMutationIdentities`, which holds the key and input it was encoded
from, so the identities and the input the handler decides on cannot come from
different requests.

The program owner binds `Binding` into the idempotency binding it is given, so
a caller of `compare_and_commit_program_action::<Binding>` passes the
`&ApplicationMutationIdentities` it encoded, plus a closure that adds anything
the request also binds (`std::convert::identity` when nothing), and the door
builds the idempotency binding itself; a caller never names the mutation. At
commit, a candidate built by another binding's handler is refused as
`MutationBindingMismatch`, and one whose handler decided on a different input
than the idempotency binding derives from is refused as `MutationInputMismatch`.
A capability admission governs the request's own input identity rather than
encoding the input again, and the handler refuses a request whose identities
encode a different input (`MutationHandlerExecutionDenial::InputNotAdmitted`).
Hosts that author a commit without a mutation request use
`for_host_commit::<Schema, Operation, _, _>(&key, &intent)`, the only host
constructor: the same key and intent under two operations never replay each
other, and it names no mutation binding. Capability workflows (elevation,
review, delegation and revocation requests) build their binding inside Query:
the input is encoded once, its identity is both the governed input and the
intent, and the key is scoped to the operation and the requesting principal,
so the same key from another principal or under another operation never
replays the request.

**Provisional.** `compare_and_commit_program_undo::<Binding>` and
`compare_and_commit_program_redo::<Binding>` take the effect program, the same
mutation identities and idempotency extension, and the sealed progression
handoff. Both entries preserve the program membership, selected occurrence,
workflow authority, conditionals and operation posture checks of an action
commit. A handoff authorizes only the admission to which it is bound. Linear
undo and redo are provisional experiments, not an accepted application
contract: Milestone 9.18 owns any accepted public correction-history contract
([Ordinary Application Front Door, Current Limits](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#current-limits)).
Do not build on them as governed correction.

The identity is exactly the serialized value, so keep that value complete and
ordered:

- Do not hide meaning from it. A field marked `#[serde(skip)]`,
  `skip_serializing_if`, or written by a lossy `serialize_with` or custom
  `Serialize` does not take part, and two inputs that differ only there
  replay each other.
- Put anything that must matter in a serialized field. `PhantomData` and
  generic type parameters emit no data, so a `Money<C>` currency marker is not
  in the identity.
- Use ordered collections. Map entries are sorted by their encoded keys, so a
  `HashMap` and a `BTreeMap` with the same entries agree. A `HashSet` and every
  sequence keep iteration order, so a retry of the same set can be refused as
  drift. Prefer `BTreeSet` or a sorted `Vec`.
- Keep variants distinguishable. `#[serde(untagged)]` emits no variant name, so
  variants with the same payload share one identity. Floats encode their exact
  bits, so `-0.0` and `0.0` differ.
- Integers encode by value, so a `u8` and a `u64` holding the same number
  agree, but a signed and an unsigned integer holding the same number differ.
  Keep a field's integer type stable across versions.
- Treat serialized type, field and variant names as part of the durable
  identity. Renaming one changes the identity of every in-flight retry and
  of every workflow requirement that names the input; version the input
  binding identity when you do.

### 5.3 Execute through the program

Every ordinary application installs a program with `in_memory_program` or
`in_memory_rostered_program`. Declare a no-output action with
`operation_action::<Binding, Delivery>()`; its delivery mode is
`ApplicationSourceCompletion` or `ApplicationRetainedCommit`. Select the complete
`ApplicationActionReference`, rather than a first matching binding.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Illustrative: not compiled.
let operation = request.mutate(intent)
    .without_source()
    .preconditions(original_conditions)
    .idempotency_owned(key)
    .retain_operation::<Program, Action>(&runtime)?;
let next = operation.advance(&fresh_request);
// Retain Progress/Refused custody for a later fresh-authorized attempt.
// Completed owns evidence and lawful close; Terminal has no advance method.
```

The owner performs native prepare/commit, unpublished recovery and keyed resolution.
Capability authorization follows the binding, including recovery; an authenticated
principal alone grants no permission. Native policy supplies each attempt's
allocation and bounded work. Callers neither reset a failed attempt nor recreate
its original descriptor. Source-only completion may be followed by a separate
declared demand or assessment; it cannot replace required-output settlement.

The runnable `ordinary_product_workflow` declares a real source-only action,
advances its owner, observes native fresh receipt/result evidence, closes cleanup,
and reads both current and retained product truth through the public facade.

#### Capture complete original meaning

Install a program and declare the exact typed action occurrence and its delivery.
`request.mutate(intent)` returns `WorthQueryOperationBuilder`; it exposes original
source preparation, preconditions and `idempotency_owned(key)`, followed by
`retain_operation::<Program, Action>(&runtime)`. Fixed and source-only callers
cannot execute, prepare or recover through that builder or its borrowed-key view.
The selected action carries composition instance, feature, binding and delivery;
a binding label alone cannot select its contract.

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

<!-- compiled: owning_entry.rs -->
```rust
pub fn capture_original<S, I>(
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    intent: I,
    preconditions: TypedMutationPreconditions<S, <I::Binding as ApplicationMutationBinding<S>>::Operation,
        <<I::Binding as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope>,
    source: WorthQueryObservedResultSet<<<I::Binding as ApplicationMutationBinding<S>>::SourceExpectation as ApplicationMutationSourceExpectation<S>>::Query>,
    key: <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
) -> WorthQueryOperationDescription<S, I>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone,
{
    request
        .mutate(intent)
        .preconditions(preconditions)
        .expect_result_set(source)
        .idempotency_owned(key)
}
```

The real result-set court calls this function with nondefault preconditions before
an actual post-effect append failure. It reads the resulting source and outputs
independently after native recovery. No original intent clone is required per pump.

#### Advanced native mutation control

Use the [ordinary owning entry](#53-execute-through-the-program) for fixed and
source-only actions. Deliberate native callers select `request.advanced().mutate(intent)`.
Only that type exposes manual fixed/source-only execute, two-phase prepare,
unpublished recovery, keyed resolution and fixed output start/recovery.
The ordinary workflow binding retains `WorthQueryWorkflowOperation` through
`for_workflow_operation(..)?.retain_operation::<Program, Action>(program)`.
Advance it with a fresh request; retain the returned owner on refusal or progress.
The original slot, native recovery and cleanup remain owned by Query. Deliberate
native workflow execution requires `.advanced().execute_in_program(..)`.
A runtime installed without a
program (`application_installation::in_memory`) is advanced native control;
its `execute` cannot be an ordinary application default. On a program runtime
non-program execution refuses with `ApplicationProgramRequired`.
Native calls take an explicit `ExecutionAllocationPolicy`; the owning attempt
uses its admitted native policy. Unsupported predecessors refuse before effects
or executable readmission and preserve original bytes; no compatibility entrance
is retained.

### 5.4 Mutations with required outputs

For a fixed required-output action, declare its exact composition occurrence,
output root and projection. `ApplicationFixedActionDelivery<Schema, Binding>`
names `Root` and `Projection`. For a query, choose `ApplicationQueryProjection<Read>`
and implement `ApplicationFixedQueryDelivery` with `Read` and
`read_input(&Binding::Input)`. For required outputs without a projected read,
choose `ApplicationNoProjection`; it has no query binding or selector. The
`ApplicationActionDelivery` contract supplies the single stable `IDENTITY` and
`ApplicationFixedDeliveryMode<Root, Projection>`.
The installed declaration binds the concrete `Delivery` implementation. Another
type cannot substitute its selector by copying the delivery identity, root, and
read metadata. The identity still supplies stable canonical meaning; runtime type
stamps are not persisted.

Author that source with `fixed_output_action::<Binding, Delivery>()`; use an
`ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery>` to
select it. Program validation requires the delivery root to belong to that
program and to start at the action's actual source occurrence.

An action with no native output-source obligation declares
`ApplicationActionDelivery::Mode = ApplicationSourceCompletion` for no projection,
or `ApplicationRetainedCommit` for its authentic commit observation. Author it
with `operation_action::<Binding, Delivery>()` and select its exact
`ApplicationActionReference` through the same `retain_operation`. A copied
identity cannot replace a required-output delivery, and the runtime's actual
output-source inventory must also permit the no-output lane.

The description freezes encoded input custody once. Native initial admission,
recovery and keyed resolution share that immutable input; attempts borrow the
original preconditions and fresh request authority. Source completion returns
receipt-only success, while retained commit also returns its native observation.
Both execution paths derive allocation policy from the supplied admitted phase.

Binding-selected admission uses `ApplicationMutationAdmissionBinding::Admission`:
`ApplicationConventionalAdmission`, or `ApplicationCapabilityAdmission<Capability>`
with the existing `ApplicationCapabilityMutationBinding`. Generated conventional
mutation bindings supply the conventional association; the macro's explicit
`admission` type selects another declared association. Fresh native initial,
unpublished-recovery and keyed-read handoffs borrow the original intent, key,
source and full preconditions. They store no principal or request scope.

For fixed delivery, an idempotent `AlreadyCommitted` source advances through
`SourceReplayObserved` and then `RegisteredOutputsRecovered`. Each attempt uses
fresh binding-selected admission and the exact installed action. Query resolves
the original key and rejoins the registered native output demand from the
original input; it never reruns the source handler or supplies a missing handler
result. Keep the returned owner on `Refused` and retry with a fresh request.
Missing, retired or superseded registration returns terminal historical source
evidence instead of output settlement. Terminal `close` releases actual cleanup
obligations; it does not mean the declared outputs completed.

The executable [fixed reply-loss journey](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/fixed_replay.rs)
checks registered recovery, real observation-pressure refusal and retry, and
historical replay after completion through the same ordinary entrance.

The SourceOnly owning adapter delegates the shared native admission, recovery
and keyed-resolution spines. Source completion uses advance with a fresh request;
retained commit additionally requires the explicit NonZeroUsize history-work
bound on each advance. Replay/recovery select the exact original World observation.
A bounded history refusal retains operation and evidence for a fresh funded retry.
See the [retained delivery guide](../workspaces/worth-query/crates/worth-query/docs/foundations/source-only-retained-delivery.md).
Ordinary consumer cutover remains separate work.

`WorthQueryOperationCleanup` retains actual pending records and release failures.
Its phase-taking close returns a cleanup receipt, not whole-operation completion.
Only the owning lane may report closed after its other native obligations are
lawfully discharged. Refused cleanup retains custody for a fresh admitted retry.

The owning entrance retains the original intent, key, typed preconditions,
row or result-set expectation, and branch. Construction admits installed meaning
without performing an effect. A refused constructor returns the original
`WorthQueryOperationDescription`, so it can be retained or lawfully constructed
against the correct declaration. It retains no authenticated principal or grant.

Shape, with illustrative names; the executable counterpart is
[`required_unpublished.rs`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/required_unpublished.rs):

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Shape. Illustrative names.
let description = request
    .mutate(AdjustDraft { /* ... */ })
    .expect_source(source)                 // or expect_result_set(result_set)
    .preconditions(original_conditions)
    .idempotency_owned(key);
let operation = description.retain_operation::<MyProgram, AdjustDraftAction>(&runtime);
// On construction refusal, retain the returned (description, denial).
let operation = retain_or_handle_construction(operation);

match operation.advance(&fresh_request) {
    WorthQueryOperationAdvance::Progress { operation, progress } =>
        retain_for_next_request(operation, progress),
    WorthQueryOperationAdvance::Refused { operation, denial } =>
        retain_refused_operation(operation, denial),
    WorthQueryOperationAdvance::Terminal { operation, denial } =>
        retain_terminal_native_custody(operation, denial),
    WorthQueryOperationAdvance::Settled(settled) => {
        consume_or_retry_delivery(settled.read_current(&fresh_request));
        retain_or_finish_close(settled.close(&fresh_request));
    }
}
```

Each advance receives fresh request authority for the same runtime and branch.
Query borrows the original description into its existing native admission and
execution owners. A real unpublished source continues its original partial;
recovery and promotion never invoke the source handler again. Genuine native
handoffs are reported as progress. Native output `Pending` is reported only
when the existing output owner reports it.

Settlement does not invent a recovered handler reply: `handler_result()` and
`recovered_progress()` remain distinct. `read_current` retains one fresh
observation, requires every settled output's native lineage at that observation,
and executes the declared read there. Refusal retains settlement and original
meaning for a later read; it does not perform the source or outputs again.
`read_current_with_policy` accepts `WorthQueryOperationDeliveryPolicy` for one
attempt: currentness work is capped by installed native policy, and query result
and work ceilings must narrow the declared native query limits. Retry uses fresh
request authority on the same settled owner. Native consumed-fact movement
returns `Stale`; unrelated progress does not itself make settlement stale.
`read_historical` reads the original declared projection at the retained settlement
basis with fresh query authorization. Neither history nor close advances outputs.
For `ApplicationNoProjection`, `deliver_current` checks every settlement at one
authentic observation and returns that observation without issuing a query.

Start with a small finite attempt. This shape uses application-specific names;
the real compiled consumer counterpart is
[`fixed_delivery.rs`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/fixed_delivery.rs).

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Illustrative: not compiled.
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::WorthQueryOperationDeliveryPolicy;

let policy = WorthQueryOperationDeliveryPolicy::currentness(
    NonZeroUsize::new(64).unwrap(),
).query(NonZeroUsize::new(1).unwrap(), NonZeroUsize::new(32).unwrap());
let delivery = settled.read_current_with_policy(&fresh_request, policy);
// Keep settled on refusal; a later fresh request may use another finite policy.
```

For a realistic retained-output journey, settle the original action once, then
handle delivery separately. A currentness work refusal or query work refusal
leaves that same settled owner available for retry. A `Stale` result instead
means native consumed-source facts moved: read the original declared projection
with `settled.read_historical(&fresh_request)` when historical data is useful,
then close the owner. Historical reads still require fresh query authorization;
an installed-action mismatch on current delivery is not permission to execute a
replacement selector. `close` can refuse and return the owner with its remaining
cleanup, so retain it for lawful retry.

When no projected rows are needed, declare `ApplicationNoProjection` and use
`settled.deliver_current(&fresh_request, policy)`. This still settles and validates
the real required outputs. It returns an authentic observation, not query
permission or rows. The compiled public API example and negative counterpart are
on [`deliver_current`](../workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/current_delivery.rs).
`ApplicationFixedProjection` and `WorthQueryFixedProjectionInstallation` are
sealed composition contracts; applications choose the provided projection shape
rather than implementing another installation or authorization policy.

Do not rerun the source or rebuild the descriptor to retry delivery. A caller
cannot enlarge installed ceilings, replace the declared selector, or refresh
closed output owners by asking for a current read.

For several reads with the same installed query binding, declare
`ApplicationQueryBatchProjection<Read, N>` and implement
`ApplicationFixedBatchDelivery<Schema, Binding, N>`. Its pure `read_inputs`
selector borrows the original mutation input and returns an ordered array of
exactly `N` query inputs. `N` must be positive. The binding and cardinality are
installed canonical meaning; the concrete delivery implementation is also
checked at construction and current delivery. Callers cannot supply another
selector or replace the batch at read time.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Illustrative: not compiled.
fn read_inputs(input: &SourceAdjustment) -> [OutputRead; 2] {
    [OutputRead { body_key: input.scope_key.clone() },
     OutputRead { body_key: format!("final:{}", input.scope_key) }]
}
```

`settled.read_batch_current::<N>(&fresh_request)` checks every actual settlement
and executes all items at that same authentic observation in one admitted phase.
The installed binding and native host profile supply finite aggregate work and
custody allowances. Each item still receives independent ordinary scope and
disclosure authorization. The aggregate allowance covers read work and retained
result/source custody; it does not claim to meter all authorization or
application-authored selector work.

Use `read_batch_current_with_policy::<N>` to narrow an attempt. The policy's
`batch(native_limits)` restricts aggregate work, retained bytes and per-item result
bytes; `query(results, work)` narrows each installed query allowance. A larger
aggregate request cannot enlarge installed policy. No item starts another phase
or renews the shared loan. A late refusal returns no partial collection and
releases staged results; the settled operation and cleanup remain yours.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Illustrative: not compiled.
let policy = WorthQueryOperationDeliveryPolicy::currentness(currentness_work)
    .batch(narrower_batch_limits);
match settled.read_batch_current_with_policy::<2>(&fresh_request, policy) {
    Ok(batch) => inspect_ordered_items(batch.items()),
    Err(denial) => retain_for_fresh_authorized_retry(denial),
}
```

The real two-output example is
[`fixed_batch.rs`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/fixed_batch.rs).
It reads two distinguishable required-output values, refuses when the common
loan cannot fund both, then retries on the same owner with fresh authority.
Inspect each item's ordinary receipt and the collection receipt; the latter is
accounting evidence, not combined scope or mutation authority.
`read_batch_historical::<N>` uses the original declared selectors at the retained
settlement basis with fresh authorization. It does not claim currentness or
advance outputs. This batch shape is homogeneous: all items use one installed
query binding. Scalar query and explicit no-projection delivery remain separate
closed shapes.


`close` funds native recovery-record release and owner cleanup. A refusal returns
the settled owner with the exact remaining cleanup. Holding a real World caller
capability can lawfully refuse record release; dropping that capability and
supplying a fresh request permits retry. A closed result reports discharged
cleanup, rather than treating a failed release as success.

The fixed owning entrance selects conventional or capability admission from the
declared mutation binding and checks fresh request authority through the existing
native owner. It does not reinterpret a capability or workflow denial as ordinary
permission. Other lifecycle entrances retain their own native custody and support
boundaries.

When the owner reaches output traversal, one native `advance` can process the caller's dependency chain and other queued
required outputs in several waves. The required-set owner admits a wave only
against a basis where its consumed upstream outputs have committed. Compute
steps of independent members can overlap; completion, commits, and publications
follow canonical keys that order by producer identity, then by observed source
epoch. Every wave and nested computation
shares this call's request bounds. Match the typed stop rather than treating
work exhaustion, memory refusal, cancellation, deadline, panic, or a domain
denial as interchangeable. Earlier performed effects remain recoverable; no
member beyond the applied prefix is published. A queued stopped member does
not prevent unrelated required work from advancing.
An owner-completion failure does not refund compute already settled, even if
that failure is earlier than the compute stop. Treat the applied prefix and
canonical compute prefix as different boundaries.

Closing a continuation releases its interest, while other holders can keep an
output required. Drop releases this holder's interest and that row's own claims,
follows closed claims through at most two rows, charges no work, starts no
computation, and never scans the whole registry. It may hand on at most
one existing closed-leaf claim; remaining cleanup stays recorded for a later
funded call. Readers still see the effective successor of a deferred claim.
The advancement's execution accounting covers all waves, including a failed
call; ordinary output progress remains `Pending` or `Settled`, with failures
returned as typed denials.

**Two independent members in one wave.** The planar geometry application has
two separate source rings, `diamond-left` and `diamond-right`. Each producer
reads its own ring and returns a managed computation plan. A third output,
`diamond-join`, consumes both outputs. The application declares and installs
this program in the consumer fixture's `concurrent_waves/application.rs`.

Before this excerpt, the fixture seeds the three rings, authenticates the
principal, opens demands for both roots and the dependent, and settles their
initial outputs. The root demands stay open. The excerpt changes both source
rings, advances the dependent once, and reads both published results. The
fixture verifies that both root computations finish before either root's
owner completion runs, and that each runs once: the roots share one wave.
The same calls work with serial backing or a host policy admitting parallel
workers; the host chooses placement, not the application.

`Program`, `PlanarRead`, `PlanarSourceAdjustment`, `PlanarOutputRead`, and
`length` are this application's declarations and value constructor. The
imported Query API comes from `worth-query-decl` and `worth-query-host`.
`application`, `principal`, `scope`, and the dependent's demand `join` come from
the setup above.

#### Advanced native required-output control

Use the [ordinary owning entry](#53-execute-through-the-program) for fixed actions.
The following fixture deliberately coordinates native output custody through
`advanced()`; it is not an ordinary lifecycle recipe.

<!-- compiled: checkpoint_recovery/required_chain/concurrent_waves/application.rs -->
```rust
    use worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome;

    let request = application.request(&principal, &scope);
    for (key, y, idempotency) in [
        ("diamond-left", 5, 0x9176_8200_u64),
        ("diamond-right", 54, 0x9176_8201_u64),
    ] {
        let selected = request
            .query(PlanarRead {
                body_key: key.to_owned(),
            })
            .execute()
            .unwrap();
        let changed = request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: key.to_owned(),
                replacement_y: length(y),
            })
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&idempotency)
            .execute_performed::<Program, program::ChainRoot>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(matches!(
            changed,
            WorthQueryApplicationPerformedMutationOutcome::Performed(_)
        ));
    }
    assert!(matches!(
        join.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    for (key, expected) in [("diamond-left", 6), ("diamond-right", 55)] {
        let output = request
            .query(PlanarOutputRead {
                body_key: key.to_owned(),
            })
            .execute()
            .unwrap();
        assert_eq!(output.rows()[0].value, length(expected));
    }
```

---

## 6. Author, publish, and run workflows

Guarded no-output effects use the original descriptor and issued requirement:
`for_workflow_operation(..)?.retain_operation::<Program, Action>(program)`.
Every `advance(&fresh_request)` rechecks current native authority. Effect
completion retains native cleanup; workflow acceptance returns that completed
owner on refusal and keeps it alongside successful native progression.
Assessment has a separate retained demand: `into_assessment_demand(..)?`
`.controls(controls).retain_operation(program)?`, then `advance(&fresh_request)`.
Observer close, producer cleanup, evidence acceptance and approval are distinct.

A workflow is a branch-local, versioned graph of governed steps. The host
installs a **vocabulary** once: which operations, assessments, conditions,
and approvals a workflow may use, and which capabilities control it. After
that, definitions are *data*: authored, validated, published on a branch,
discovered, started, and advanced at run time.

Every workflow action is a governed mutation. There is no background
scheduler, and no timer or callback advances an instance: **the caller pumps
progress**, one request per step. An external effect completes through an
authenticated inbound occurrence (`inbound_occurrences().receive(..)` on the
installed runtime), which completes the effect's owner. After a caller accepts
the owner's receipt (`accept_operation_from_owner`), an `AwaitInbound` node
observes that owner result on the next advance. Callback bytes never resume
an instance. See
[Workflows: Caller-pumped progression](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#caller-pumped-progression)
and [Waiting for an inbound completion](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#waiting-for-an-inbound-completion).

### 6.1 The spec marker

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query-bank-world/crates/bank-domain/src/schema/workflow.rs
pub struct ApprovedBusinessPaymentWorkflow;

impl ApplicationWorkflowSpec for ApprovedBusinessPaymentWorkflow {
    type Schema = BankSchema;

    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.bank.approved-business-payment-workflow.v1");
}
```

### 6.2 Install the workflow vocabulary once

The vocabulary is installed against the running program, then retained on
the program runtime. The result is a
`WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>`, bound to one
spec. `retain_workflow_spec` consumes the program runtime, and the workflow
runtime dereferences back to it.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/identity_runtime/installation.rs (error mapping removed; Bank names the result `runtime`)
use worth_query_host::facade::domain::WorthQueryApplicationWorkflowSpecInstallation;

let workflow = WorthQueryApplicationWorkflowSpecInstallation::<
    BankSchema,
    ApprovedBusinessPaymentWorkflow,
    BankApplication,
>::begin(
    runtime.runtime().installed_schema(),
    runtime.installed_program(),
    payment_workflow_resources(),          // WorthQueryApplicationWorkflowResourceCeiling
)
.operation::<ApprovedBusinessPaymentAuthoringBinding>()?
.operation::<ApprovePaymentMutationBinding>()?
.assessment::<PaymentDetailQueryBinding>()?
.condition_operand::<PaymentAmountQueryBinding>()?
.approval::<ApprovedBusinessPaymentApproval, ApprovedBusinessPaymentApprovalOperation, ApprovePayment>()?
.authoring_capability::<ApprovedBusinessPaymentAuthoring, ApprovedBusinessPaymentAuthoringOperation, ApprovePayment>()?
.instance_start_capability::<ApprovedBusinessPaymentInstanceStart, ApprovedBusinessPaymentInstanceStartOperation, ApprovePayment>()?
.advance_capability::<ApprovedBusinessPaymentAdvance, ApprovedBusinessPaymentAdvanceOperation, ApprovePayment>()?
.finish()?;

let workflow_runtime = runtime.retain_workflow_spec(workflow, approval_authentication.signing_owner())?;
```

- `.operation::<Binding>()`, `.assessment::<Binding>()`, and
  `.condition_operand::<Binding>()` admit the actions and reads a definition
  may use.
- `approval` and the three `*_capability` calls each take three types:
  `<Capability, Operation, Input>`. `Capability` is the capability marker the
  caller must hold. `Operation` is the control operation it gates, declared
  in the schema with `worth_query_operation!`. `Input` is that operation's
  input value type. Bank's four control operations share one input type,
  `ApprovePayment`.
- The three **control capabilities** gate every control step. The authoring
  capability publishes and retires definitions. The instance-start capability
  starts, cancels, migrates, and continues instances on a fork. The advance
  capability advances and navigates back.
- The **signing owner** makes approvals verifiable. Install it with
  `admission::authentication_event::install_authentication_event_owner::<Schema, Clock, _, _>(schema, clock_source, verifier, policy, capacity)`
  and pass its `signing_owner()` here. Bank does this in
  `bank-server/src/approval_authentication.rs`, with a 60-second single-use
  event policy.
- A host supporting several programs calls `support_workflow_spec` on the
  returned workflow runtime for each extra program.

**Control bindings.** Each control step (authoring, instance start, advance,
approval) is a mutation intent whose binding sets `WORKFLOW_CONTROL = true`.
The workflow kernel authorizes and records these steps itself. No handler may
serve one, and installing one is refused. `worth_query_mutation_binding!` does
not set that constant, so a control binding is written by hand. Bank writes
its four with one local macro. This is the part that differs from an ordinary
binding:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query-bank-world/crates/bank-domain/src/schema/workflow/control.rs (abridged)
worth_query_operation!(pub $operation for BankSchema, input ApprovePaymentInputBinding);
worth_query_operation_reads!($operation => [PaymentIdentityField]);

pub struct $binding;

impl ApplicationMutationBinding<BankSchema> for $binding {
    type Input = ApprovePayment;
    type Operation = $operation;
    type Decision = ();                       // no handler decides a control step
    // ... the same associated types as any mutation binding (§5.2) ...
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const WORKFLOW_CONTROL: bool = true;
    // ... identities, CANDIDATES, scope_field, and principal_binding ...
}

impl ApplicationCapabilityMutationBinding<BankSchema> for $binding {
    type Capability = $capability;            // the marker passed to *_capability above
}

impl ApplicationMutationIntent<BankSchema> for $intent { /* input, scope_binding */ }
impl ApplicationCapabilityRequest<BankSchema, $capability> for ApprovePayment {
    /* project the request onto the capability's entity and context */
}
```

The macro is invoked once for each control step, for example
`ApprovedBusinessPaymentAuthoringOperation`, `…AuthoringBinding`,
`…AuthoringIntent`, and `ApprovedBusinessPaymentAuthoring`. The whole file is
244 lines.

### 6.3 Author a definition

`ApplicationWorkflowDefinitionBuilder<Spec>` creates typed nodes, then
connects them. This is Bank's approved-payment workflow. The excerpt keeps
every node, connection, and limit of
`approved_business_payment_definition()`. It puts each `.control(..)` call on
one line, adds the `//` annotations, and omits the function signature and the
denial enum:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query-bank-world/crates/bank-server/src/application_definition/workflows.rs (abridged)
use bank_domain::queries::{PaymentAmountQuery, PaymentDetailQuery};
use bank_domain::schema::{
    approved_payment_inbound_binding, ApprovePaymentMutationBinding,
    ApprovedBusinessPaymentApproval, ApprovedBusinessPaymentAuthoringOperation,
    ApprovedBusinessPaymentWorkflow, ApprovedPaymentSettlementEffect,
};
use std::time::Duration;
use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowAuthoringDenial, ApplicationWorkflowComponentLimits,
    ApplicationWorkflowConditionOperands, ApplicationWorkflowControlOutcome,
    ApplicationWorkflowDefinitionBuilder, ApplicationWorkflowDefinitionLimits,
    ApplicationWorkflowEvidenceJoinPolicy, ApplicationWorkflowInboundWait,
    ApplicationWorkflowValidationDenial, ValidatedWorkflowDefinition,
};

/// The operand the approval limit reads: the payment's amount in cents.
pub const APPROVAL_LIMIT_OPERAND: &str = "amount_cents";

/// Workflow approval covers payments up to $150.00; a larger payment is
/// rejected before any approver is asked.
pub const APPROVAL_LIMIT: &str = "amount_cents <= 15000";

let mut builder = ApplicationWorkflowDefinitionBuilder::<ApprovedBusinessPaymentWorkflow>::new(
    "approved-business-payment",
    ApplicationWorkflowDefinitionLimits::new(
        10,                                                  // nodes
        21,                                                  // connections
        2,                                                   // effects
        ApplicationWorkflowComponentLimits::new(8, 2, 16, 32, 32).unwrap(),
        16 * 1_024,                                          // canonical bytes
    )
    .and_then(|limits| limits.with_total_deadline(Duration::from_secs(7 * 24 * 60 * 60)))
    .expect("approved-payment limits and deadline are valid"),
)
.expect("approved-payment identity is valid");

// Nodes. Each call returns a typed node reference.
let propose =
    builder.operation::<ApprovedBusinessPaymentAuthoringOperation>("propose", false)?;
let payment = builder.assessment::<PaymentDetailQuery>("review/payment")?;
let independent = builder.assessment::<PaymentDetailQuery>("review/independent")?;
let evidence = builder.evidence_join(
    "review/evidence",
    ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
)?;
let limit = builder.condition(
    "approval/limit",
    APPROVAL_LIMIT,
    ApplicationWorkflowConditionOperands::new()
        .query::<PaymentAmountQuery>(APPROVAL_LIMIT_OPERAND),
)?;
let approval = builder.approval::<ApprovedBusinessPaymentApproval>("approval")?;
let apply = builder.operation_binding::<ApprovePaymentMutationBinding>("apply")?;
let await_completion = builder.await_inbound::<ApprovedPaymentSettlementEffect>(
    "await-inbound",
    &apply,
    approved_payment_inbound_binding(),
    ApplicationWorkflowInboundWait::UntilInstanceDeadline,
)?;
let completed = builder.terminal("completed")?;
let rejected = builder.terminal("rejected")?;

// Control: where each outcome goes.
builder
    .start(&propose)
    .control(&propose, ApplicationWorkflowControlOutcome::Completed, &payment)
    .control(&payment, ApplicationWorkflowControlOutcome::Completed, &independent)
    .control(&independent, ApplicationWorkflowControlOutcome::Completed, &evidence)
    .control(&evidence, ApplicationWorkflowControlOutcome::EvidenceSatisfied, &limit)
    .control(&limit, ApplicationWorkflowControlOutcome::ConditionSatisfied, &approval)
    .control(&limit, ApplicationWorkflowControlOutcome::ConditionUnsatisfied, &rejected)
    .control(&evidence, ApplicationWorkflowControlOutcome::EvidenceFailed, &rejected)
    .control(&approval, ApplicationWorkflowControlOutcome::Approved, &apply)
    .control(&approval, ApplicationWorkflowControlOutcome::Rejected, &rejected)
    .control(&apply, ApplicationWorkflowControlOutcome::Completed, &await_completion)
    .control(&await_completion, ApplicationWorkflowControlOutcome::Completed, &completed)
    // Data: what each step receives.
    .proposal_for_assessment(&propose, &payment)
    .proposal_for_assessment(&propose, &independent)
    .assessment_evidence(&payment, &evidence)
    .assessment_evidence(&independent, &evidence)
    .condition_subject(&propose, &limit)
    .proposal_for_approval(&propose, &approval)
    .joined_evidence(&evidence, &approval)
    .approval_authority(&approval, &apply)
    .operation_input(&propose, &apply);

Ok(builder.finish()?.validate()?)
```

`apply` is a guarded effect whose external settlement goes into owner custody;
`await_completion` waits for that owner's inbound completion. An
`AwaitInbound` node requires an instance deadline, which is why the limits
carry `with_total_deadline` (without one, validation refuses with
`MissingInboundDeadline`).

**Node kinds.** There are exactly seven (`ApplicationWorkflowNodeKind`):

| Builder method | Node | Notes |
|---|---|---|
| `operation::<Op>(id, false)`, or `operation_binding::<Binding>(id)` for an unguarded binding | Operation: proposal slot | The caller records proposed input with `prepare_workflow_proposal`. No handler runs and no business state changes. |
| `operation::<Op>(id, true)`, or `operation_binding::<Binding>(id)` for a `workflow_authority required` binding | Operation: guarded effect | An advance issues authority for this occurrence (`AwaitingOperation`); the caller executes the business mutation through `for_workflow_operation`. `operation_binding` takes the authority flag from the binding. |
| `assessment::<Query>(id)`, `assessment_for`, `assessment_when_related_relation_present` | Assessment | A query whose settled result becomes evidence |
| `await_inbound::<Effect>(id, &origin, inbound, wait)` | AwaitInbound | Waits for the inbound completion of the named preceding operation's external effect. `wait` is `ApplicationWorkflowInboundWait::UntilInstanceDeadline`, so the definition needs a total deadline. |
| `condition(id, source, operands)` | Condition | A Bool expression over named, typed query results |
| `approval::<Capability>(id)` | Approval | A signed human decision under a capability |
| `evidence_join(id, policy)` | Evidence join | `AllRequiredPassing` or `AllRequiredCompleted` |
| `terminal(id)` | Terminal | An end state |

There is **no** query, read, branch, or retry node. A read that feeds a
decision is an assessment or a condition. Branching is routing different
`ApplicationWorkflowControlOutcome`s. A retry is a connection:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/main.rs
let revise = ApplicationWorkflowRetry::new(
    ApplicationWorkflowControlOutcome::Rejected,
    "revise the proposal",
    2,                                   // maximum attempts
)
.expect("the revision bound is valid");
// ...
workflow.retry(&approval, revise, &propose);
```

A retry source must also route `RetryExhausted`.

**Connections** on the definition builder: `start`, `control`, `retry`, and
the data flows `proposal_for_assessment`, `condition_subject`,
`proposal_for_approval`, `assessment_evidence`, `joined_evidence`,
`approval_authority`, `operation_input`. Endpoints are typed: an edge that
makes no sense does not compile.

**Reusable components.** `ApplicationWorkflowComponentBuilder<Spec>` builds a
fragment with named `input_port` and `output_port`s. Place it with
`builder.expand_component("checks", &component)?`, then connect to
`expanded.input(&port)?` and `expanded.output(&port)?`. Expanded node paths
are prefixed with the occurrence name (`checks/structural`). See
`examples/authored_workflow/review.rs`.

**Three front ends, one builder.** All of these produce the same
`AuthoredWorkflowDefinition<Spec>`, and all must still be validated:

1. The typed builder shown above.
2. The `worth_query_workflow!` macro. It is a thin block around the same
   builder, applies `?` to `new`, and returns the unvalidated `finish()`
   result:

   <!-- illustrative -->
   ```rust,ignore
   // Illustrative: not compiled.
   let authored = worth_query_workflow! {
       spec: ReviewedChange;
       identity: "reviewed-change";
       limits: limits();
       build: |builder| {
           let propose = builder.operation::<ProposeChange>("propose", false)?;
           // ... nodes and connections, exactly as with the builder ...
       }
   }?;
   let validated = authored.validate()?;
   ```

3. `ApplicationWorkflowCommandAdapter::author::<Spec>(identity, limits, commands)`,
   which lowers a sequence of `ApplicationWorkflowAuthoringCommand`s
   (`Node`, `Start`, `Connection`). Use it when a UI, an API, or an AI agent
   authors a workflow as data.

**Validation.** `validate()` returns `ValidatedWorkflowDefinition<Spec>` or an
`ApplicationWorkflowValidationDenial`. Kinds include `MissingStart`,
`UnreachableNode`, `MissingControlOutcome`, `AmbiguousControlOutcome`,
`ControlCycle`, `MissingTerminal`, `MissingWorkflowAuthority`,
`UnexpectedWorkflowAuthority`, `MissingOperationInput`, `IncompleteApproval`, and
`MissingInboundDeadline`. `MissingWorkflowAuthority` means a guarded operation
has the wrong number of `approval_authority` edges for its installed
requirement: more than one, or not exactly one when the requirement asks for
an approval (including the legacy default). `UnexpectedWorkflowAuthority`
means an unguarded operation carries an approval edge or an execution
requirement. See [Who may run a guarded operation](#65-who-may-run-a-guarded-operation).
`MissingOperationInput` means a guarded operation has zero or more than one
`operation_input` source. It needs exactly one compatible source, including when
its installed requirement needs no approval. Unguarded proposal slots have no
input-cardinality requirement. Unsupported stored definitions are refused before
executable readmission; their original bytes are preserved.
A validated definition carries no branch, publication status, or authority to
start anything.

### 6.4 Fixed and user-authored workflows are the same thing

There is no separate "fixed workflow" construct. Bank's approved-payment
workflow is written in Rust and ships with the application, but it is
published at run time through the same kernel as a workflow a user authors
in a UI. What is static is the spec marker, the installed vocabulary, and the
control bindings. What is data is every definition, instance, proposal,
piece of evidence, and approval.

### 6.5 Who may run a guarded operation

An Operation node is one of two things, decided by its binding:

- **Proposal slot** (unguarded binding): the caller records proposed input with
  `prepare_workflow_proposal`. This runs no handler and writes no business state.
- **Guarded effect** (`workflow_authority required`): an advance issues authority for the
  occurrence; the caller then executes the business operation through `for_workflow_operation`.

A workflow-bound business operation passes up to three checks. They answer
different questions, are declared in different places, and none grants
another.

| Check | Question it answers | Declared by | Checked when |
|---|---|---|---|
| Operation authorization | May this principal perform this business operation on this scope? | The operation's capability or policy binding | Ordinary mutation admission, on every execution, including workflow-bound ones |
| Workflow occurrence authority | Is this exactly the operation, input, and branch the instance is waiting for now, and not already spent? | `workflow_authority required` on the mutation binding | Issued by an advance that reaches a guarded Operation node (`AwaitingOperation`); bound by `for_workflow_operation`; rechecked at commit |
| Required approval evidence | Must a current signed Approve decision also exist? | The binding's `workflow_requirements` plus any `approval_authority` edge authored in the definition | Resolved by the advance before authority is issued; the approver's grant is rechecked at commit |

Decision procedure:

1. An unguarded business mutation needs check 1 only.
2. A guarded mutation (`workflow_authority required`) needs checks 1 and 2, always.
3. It also needs check 3 when the installed requirement asks for an approval,
   or when the definition wires an `approval_authority` edge to the operation.
4. Approving needs the approval capability plus a fresh signature. Advancing
   needs the advance capability. Neither satisfies check 1. Permission to pump
   an instance is not permission to execute the business operations in it.
5. An approval is evidence, not permission. Workflow authority is a binding to
   one occurrence, not permission.

The installed requirement (`workflow_requirements` inside
`worth_query_mutation_binding!`):

| Declared | Meaning |
|---|---|
| nothing (`workflow_execution_requirements()` returns `None`) | Legacy guarded contract: exactly one approval edge is required. |
| `ApplicationWorkflowExecutionRequirements::new()` | No additional evidence. Zero or one approval edge; an authored edge is still enforced. |
| `ApplicationWorkflowExecutionRequirements::requiring_approval::<Cap>()` | An approval by `Cap` is required, and an edited definition cannot remove it. |

> **Warning.** Leaving `workflow_requirements` out does **not** mean "no
> approval". It means "exactly one approval". To run a guarded operation
> without a signature, declare `new()`.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query/crates/worth-query-certification/tests/application_graph/document_retention_model/retention_entry/occurrence_write.rs (abridged)
worth_query_decl::facade::worth_query_mutation_binding!(
    pub OccurrenceSetRetentionBinding for SetRetentionInput, schema DocumentRetentionSchema,
    // ... identity, input, operation, result, idempotency, decision, handler ...
    workflow_authority required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::new(),
    // ... outputs, principal, scope, candidates, resources ...
);
```

The sequence, from proposal to settlement:

1. **Propose.** At an unguarded Operation node, a proposal records the
   operation, input type, an input-identity digest, and the node path. No
   handler runs and no business state changes. The input value itself is not
   retained: it is the caller's domain data.
2. **Assess, branch, join.** Each assessment and condition takes its subject
   from the proposal through `proposal_for_assessment` or
   `condition_subject`: the scope resource the proposal covers, or a related
   or context entity in its coverage. Its queries read that subject's current
   state on the branch. The proposed input value is not retained, so no
   assessment or condition can read it.
3. **Approve (optional).** A signed Approve or Reject decision about the
   proposal. Evidence, not permission.
4. **Authorize.** An advance at a guarded Operation node returns
   `AwaitingOperation` carrying single-use authority for that occurrence.
   Nothing is committed. The authority lives in memory; if it is lost, advance
   again.
5. **Execute and settle.** `for_workflow_operation(..)` then
   `execute_in_program(..)` runs ordinary admission (check 1), requires the
   input to hash to the proposal's digest, and commits the business write
   together with the node's settlement. If the handler denies, nothing commits
   and the node still waits. The authority is spent once ordinary admission
   passes, so a retry after a handler denial needs a fresh advance.
6. **External effects** settle through owner custody and an `AwaitInbound` node.
7. **Advance again** to move to the successor or terminal.

Advancing at a proposal slot returns `PreparationDenied` with
`WorkflowTransitionNodeUnsupported` and the node path. It means "this node
needs a proposal", not a fault. There is no `AwaitingProposal` outcome (known
gap).

Full reference: [Workflows: Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations).
The runnable unsigned, occurrence-only journey is
[`examples/authored_workflow/information.rs`](../workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/information.rs):
propose, advance to `AwaitingOperation`, execute through
`for_workflow_operation`, and advance to the terminal, with no approval node.

### 6.6 Publish, discover, start, advance

Every lifecycle call is a method on the ordinary mutation request chain. The
mutation intent is the matching control intent (authoring, instance start,
advance, or approval), and the workflow runtime from §6.2 is passed in.

`workflow_runtime` dereferences to the program runtime, so
`workflow_runtime.request(&principal, &scope)` is the request entry for every
call below. (Bank wraps it in its own `runtime.request(..)`, which maps its
principal type and then makes exactly this call.)

**Bind and publish a definition** on a branch. Publication names the
predecessor it expects to replace; there is no last-writer-wins overwrite.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/approved_payment_workflow.rs
let contract = workflow_runtime
    .workflow_spec()
    .bind_definition(approved_business_payment_definition()?)?;
let outcome = workflow_runtime
    .request(&principal, &scope)
    .mutate(ApprovedBusinessPaymentAuthoringIntent { input: authority })
    .without_source()
    .idempotency(&command_key)
    .prepare_workflow_publication(contract, expected_predecessor)
    .map(|request| request.execute())?;   // WorkflowDefinitionPublicationOutcome

// Take the published definition out of the outcome. The other variants are
// Application(uncommitted), where nothing landed, and ProjectionDenied(receipt),
// where the commit landed but no reference could be projected.
let WorkflowDefinitionPublicationOutcome::Published(published) = outcome else {
    return Err(/* your error for a refused publication */);
};
let definition: PublishedWorkflowDefinitionRef = published.definition().clone();
// published.receipt() is the commit receipt; published.replayed() is true when
// the idempotency key replayed an already-landed commit.
```

`expected_predecessor` is `WorkflowDefinitionExpectedPredecessor::Absent` for
the first revision, or `Published(previous_ref)` for a revision.

**Discover** the current definition on a branch:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// examples/authored_workflow/main.rs (`discover`)
// `request_execution` is the caller's `ExecutionRequest`
// (`worth_query_host::facade::primary_graph::ExecutionRequest`; the example
// opens a serial one in `run`).
application
    .with_host_advancement(|host_phase| {
        application
            .runtime()
            .on_branch(application.runtime().current_world())
            .select(&host_phase)
            .expect("the main branch selects its exact occurrence")
    })
    .expect("installed host call opens")
    .discover_workflow_definition::<ReviewedDocumentWorkflow>(request_execution, identity)
    .expect("the workflow lineage reads within its bound")
// -> WorthQueryWorkflowDefinitionDiscovery::{Current(ref), Retired, Unpublished}
```

Branch selection needs a host advancement phase (`with_host_advancement`);
discovery reads under the caller's execution request. A refused discovery is
a `WorthQueryWorkflowDefinitionDiscoveryDenial` (for example `ForeignLineage`,
`Cancelled`, or a stopped `ExecutionRequest(..)`).

**Start** an instance, **propose**, and **advance**:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/approved_payment_workflow.rs and approved_payment_workflow/progression.rs
let started = workflow_runtime
    .request(&principal, &scope)
    .mutate(ApprovedBusinessPaymentInstanceStartIntent { input: authority })
    .without_source()
    .idempotency(&start_key)
    .prepare_workflow_instance_start(&workflow_runtime, definition)
    .map(|request| request.execute())?;      // WorkflowInstanceStartOutcome
let WorkflowInstanceStartOutcome::Started(started) = started else {
    // Superseded(..) names the current definition; Retired(..), Application(..),
    // and ProjectionDenied(..) are the other refusals. Nothing was started.
    return Err(/* your error */);
};
let instance: PublishedWorkflowInstanceRef = started.instance().clone();

let proposed = workflow_runtime
    .request(&principal, &scope)
    .mutate(ApprovedBusinessPaymentAuthoringIntent { input: proposal_input })
    .without_source()
    .idempotency(&proposal_key)
    .prepare_workflow_proposal(&workflow_runtime, instance.clone())
    .map(|request| request.execute())?;      // WorkflowProposalOutcome
let WorkflowProposalOutcome::Published(proposed) = proposed else {
    return Err(/* Application(..) or ProjectionDenied(..) */);
};
let proposal: PublishedWorkflowProposalRef = proposed.proposal().clone();
// proposed.transition() is the transition entity; proposed.node_path() the node.

let progress = workflow_runtime
    .request(&principal, &scope)
    .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
    .without_source()
    .idempotency(&advance_key)
    .prepare_workflow_advance(&workflow_runtime, instance.clone())
    .map(|request| request.execute())?;      // WorkflowProgressOutcome
```

Use a fresh idempotency key per logical command. Reusing a key replays the
recorded outcome instead of performing a second step.

`WorkflowProgressOutcome` says what happened or what the instance waits for:

| Variant | Meaning |
|---|---|
| `Completed` | A step was recorded |
| `AwaitingActor` | The caller is not authorized for the next step; an authorized caller pumps the same instance |
| `AwaitingAssessment`, `AwaitingCondition`, `AwaitingEvidence` | Settle the named read, then accept it |
| `AwaitingApproval` | A signed approval is required |
| `AwaitingOperation` | The guarded effect must run ([§6.5](#65-who-may-run-a-guarded-operation)) |
| `Application`, `PreparationDenied`, `AuthenticationDenied`, `IdempotencyDenied` | No step was recorded; each says why |
| `ProjectionDenied` | The commit landed and carries its receipt; only its projection was refused |
| `ProjectionReadStopped { receipt, cause }` | The commit landed and its receipt is retained; the projection read it requires stopped (`cause` is a `RelationalExecutionDenialCause`) |

These are all thirteen variants. There is no `AwaitingProposal` outcome:
advancing at a proposal slot returns `PreparationDenied` whose kind is
`WorkflowTransitionNodeUnsupported` and names the node path. It means "this
node needs a proposal" (`prepare_workflow_proposal`), not a fault to retry.

**Assessments and conditions.** Turn an advance into an assessment demand
with `.into_assessment_demand(demand)`, settle it, then accept it with
`.accept_assessment(&settlement)`. A condition is accepted by naming each
operand's published result:
`.condition(&required).operand::<Binding, _>("name", result).accept()`.

**Approve or reject.** An approval needs a fresh authentication event and a
signature. Replaying a recorded key is the only exception.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// bank-server/src/approved_payment_workflow.rs
let signing = workflow_runtime
    .request(&principal, &scope)
    .mutate(ApprovedBusinessPaymentApprovalIntent { input: authority })
    .without_source()
    .idempotency(&approval_key)
    // `required` is the RequiredWorkflowApproval from
    // WorkflowProgressOutcome::AwaitingApproval(required).
    .prepare_workflow_approval(&workflow_runtime, instance, &required, &proposal, decision)?;
let Some(intent) = signing.authentication_intent().cloned() else {
    return signing.execute_replay();
};
let event = authenticate(credential, intent).await?;   // your authentication adapter
signing.sign(&event).map(|request| request.execute())
```

`decision` is `WorkflowApprovalDecision::{Approve, Reject}`.

**Run the guarded effect** when an advance returns `AwaitingOperation`. Bank's
`ApprovePaymentMutationBinding` declares `workflow_authority required` and no
`workflow_requirements`, so the legacy contract applies: exactly one approval,
and authority is issued only after it is approved. A binding that declares
`ApplicationWorkflowExecutionRequirements::new()` runs the same way with no
approval ([§6.5](#65-who-may-run-a-guarded-operation)).

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Exact original meaning is retained once; construction performs no effect.
let owner = workflow_runtime.request(&principal, &scope)
    .on_branch(instance.branch())
    .mutate(ApprovedBusinessPaymentApplyIntent { input: operation })
    .without_source()
    .preconditions(original_conditions)
    .idempotency_owned(operation_key)
    .for_workflow_operation(&workflow_runtime, &required)?
    .retain_operation::<BankApplication, ApprovedPaymentEffectAction>(workflow_runtime.program_runtime())?;
let fresh = workflow_runtime.request(&fresh_principal, &fresh_scope)
    .on_branch(instance.branch());
match owner.advance(&fresh) {
    WorthQueryWorkflowOperationAdvance::Progress(owner) => retain_for_next_request(owner),
    WorthQueryWorkflowOperationAdvance::Refused { operation: owner, denial } => {
        report_refusal(denial);
        retain_for_next_request(owner);
    }
    WorthQueryWorkflowOperationAdvance::Completed(completed) => {
        // Native Fresh result and Recovered evidence remain distinguishable.
        let accepted = fresh.mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source().idempotency(&accept_key)
            .prepare_workflow_advance(&workflow_runtime, instance)?
            .accept_owned_operation(completed)?;
        record_native_progress(accepted.progress);
        // Refusal returns the actual completed owner, never a replacement receipt.
        accepted.operation.close(&fresh)?;
    }
    WorthQueryWorkflowOperationAdvance::Terminal(terminal) => {
        record_native_stop(&terminal);
        terminal.close(&fresh)?;
    }
}
```

Local guarded completion already includes the operation settlement in its business
commit. A caller may close that actual completed owner and freshly advance the
successor directly. Optional completed-owner acceptance authenticates and projects
the original node/receipt as replay; it does not perform another settlement or
execute the successor, and it does not record a new confirmation command key.
For external effects, genuine native recovery/authenticated inbound custody must
first permit the existing settlement path. Closing source cleanup or disposing a
dispatch handle does not substitute for that evidence, assessment acceptance or approval.

**Everything else in the lifecycle:**

| Call | Does |
|---|---|
| `prepare_workflow_navigate_back` | Records `NavigatedBack` |
| `prepare_workflow_instance_cancellation` | Ends an instance where it stands. Not a rollback: the outcome names every performed step. |
| `prepare_workflow_instance_migration(workflow, instance, target, resume_at)` | Continues an instance on a later definition from a named node |
| `prepare_workflow_fork_continuation` | Continues an instance on a forked branch |
| `prepare_workflow_definition_retirement` | Retires a definition lineage |
| `accept_owned_operation(completed)` | Fresh selected admission confirms original local atomic settlement as replay without a new commit/key write; external settlement requires actual recovery/inbound evidence. Success and refusal retain the same completed owner and cleanup. |
| `operation.advance(&fresh_request)` | Continues original native unpublished/keyed/dispatch custody without reconstructing input, key, authority or recovery handle. |
| `completed.close(&fresh_request)` | Releases actual cleanup; refusal returns the same owner. Physical release does not make the workflow terminal. |
| `mutate(i).workflow(&phase, workflow, authored)…`, `start_workflow`, `run_workflow` | The ordinary convenience lane, which validates, binds, publishes, starts, and runs in fewer calls. `phase` is the `WorthQueryAdvancementPhase` of an open advancement, such as the one `with_host_advancement` passes its closure. |

**Observing an instance.** There is no instance-status query. You learn state
from each typed outcome, from `discover_workflow_definition`, from an
assessment demand's notifications, and from the adoption workflow inventory
([§7.4](#74-workflows-and-custody-during-adoption)).

**What refuses:**

- Starting from a superseded revision returns
  `WorkflowInstanceStartOutcome::Superseded(..)`, which names the current
  definition. Nothing is written. A retired lineage returns `Retired`.
- Passing another workflow's runtime is refused with `RuntimeMismatch` by
  the instance-start, proposal, advance (and approval), definition-retirement,
  and operation-binding preparations. Definition publication binds its
  contract to the spec up front, so it has no such variant.
- A canceled instance refuses further steps
  (`WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled`).
- A control intent sent through the ordinary mutation lane is refused
  (`MutationHandlerExecutionDenial::WorkflowControl`).

Run the full authored-workflow journey. The example first runs
[`information.rs`](../workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/information.rs),
an unsigned, occurrence-only operation, then the signed review journey in
`main.rs`:

```bash
cargo run --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification --example authored_workflow
```

---

## 7. Adopt a new program on a branch

Adoption moves **one exact branch** from the program it runs to a rostered
target revision, in one product publication. Other branches, including
earlier forks, keep their programs.

### 7.1 The call chain

The program currently activated on the branch declares its adoption permission
through `ApplicationProgramDefinition::adoption_authorization`. Its
`ApplicationProgramAdoptionBinding` names the installed capability, operation,
input binding and principal binding. Declare the corresponding capability
contract in the schema and issue real grants through your application’s grant
owner. A program with no declaration refuses adoption with
`WorthQueryProgramAdoptionAuthorizationDenial::CapabilityNotDeclared`.

Bank uses `DeployBankProgram`, scoped to the exact currently governing source
program revision. Its deployment grants are separate from estate and payment
permissions. Installation issues only explicitly supplied
`BankProgramDeploymentGrantSeed` values; it grants nothing to an actor merely
because that actor authenticated. The recorded issuer can revoke one through
`revoke_program_deployment_grant`. An old source-revision grant does not permit
deployment after that branch adopts another revision.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Illustrative: not compiled.
let grant = BankProgramDeploymentGrantSeed::for_program::<BankApplication>(
    "operator-p0-deployment", issuer, operator, valid_from, valid_through,
)?;
let seed = BankWorldSeed::new(snapshot).program_deployment_grant(grant);
```

`compare`, `adopt`, `prepare`, publication and recovery each admit the declared
capability in their current request phase. The source program governs; the
target cannot select a weaker permission for its own adoption. Grants and
authentication must still be current at publication.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Illustrative: not compiled.
// workspaces/worth-query/crates/worth-query-certification/tests/application_graph/adoption/branch_adoption.rs
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

let target = *host
    .supported_program::<RetentionProgramP1>()
    .expect("P1 is rostered")
    .owned_revision();
let programs = host
    .runtime()
    .request(&principal, &scope)
    .on_branch(main)
    .programs();
let requirements = programs
    .compare(&target)
    .expect("the host must describe P0 to P1 requirements");
let prepared = programs
    .adopt(&requirements)
    .expect("the source capability admits adoption")
    .prepare(64)                                   // maximum selection work
    .expect("the seed satisfies P1 and adoption must prepare");
match host.runtime().request(&principal, &scope).on_branch(main).programs().publish(prepared) {
    WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, prepared } => {
        /* no adoption effects; retain prepared for a fresh authorized request */
    }
    WorthQueryBranchAdoptionPublicationOutcome::AdmissionDenied { cause, prepared } => {
        /* execution admission refused; retain the reservation */
    }
    WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied(cause) => {
        /* the native publication attempt stopped */
    }
    WorthQueryBranchAdoptionPublicationOutcome::Performed(performed) => { /* branch runs P1 */ }
    WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => { /* e.g. stale head */ }
    WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
        /* recover: unpublished.into_recovery(), then programs().recover(..) */
    }
}
```

The same chain in production code is
`bank-server/src/program_adoption.rs` (`prepare_branch_program_adoption`).

| Step | Call | Returns |
|---|---|---|
| Target | `host.supported_program::<P>().expect(..).owned_revision()` (needs the `WorthQueryProgramOwner` import) | `&ApplicationProgramRevision` |
| Inspect | `.on_branch(b).programs().inspect()` | `Result<WorthQuerySelectedProgramInspection, WorthQueryApplicationProgramInspectionDenial>` |
| Compare | `.compare(&target)` | `WorthQueryProgramAdoptionRequirements` or `WorthQueryApplicationProgramAdoptionPreparationDenial` |
| Adopt | `.adopt(&requirements)` | `Result` containing a preparation request or typed authorization denial |
| Optional | `.migration(prepared_migration)`, `.workflow(dispositions)` | The same request |
| Prepare | `.prepare(maximum_selection_work)` | `WorthQueryPreparedBranchAdoption` (move-only, fields private) |
| Publish | `programs.publish(prepared)` | `WorthQueryBranchAdoptionPublicationOutcome` |
| Recover | `programs().recover(unpublished.into_recovery())` | `Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryApplicationProgramAdoptionRecoveryFailure>` |

`compare` comes first because the requirements are the input to `adopt`.
`WorthQueryProgramAdoptionRequirements` exposes `semantic_diff()`,
`validation_scopes()`, `added_rules()`, and the predicates
`requires_existing_state_validation()`, `requires_migration_assessment()`,
`requires_custody_inventory()`, and `semantically_equivalent()`.

### 7.2 What each outcome means

| Situation | What you get |
|---|---|
| Declared permission is missing, revoked or expired | Typed `Authorization(..)` refusal; publication returns the exact prepared owner in `AuthorizationDenied` |
| Existing state satisfies the target | `prepare` is `Ok`; `selected_entity_count()` says how much was validated |
| Existing state violates a target rule | `TargetRuleRejected { identity }` |
| The target needs a migration | `requires_migration_assessment()`, then `MigrationAssessmentRequired(..)` if you did not supply one |
| The migration's source changed after it was prepared | `MigrationSourceChanged` |
| The migration was prepared for a different target | `MigrationTargetMismatch` |
| Requirements went stale | `RequirementsChanged`: compare again |
| Selection is too large | `SelectionLimitExceeded { maximum_work_units, consumed_work_units }` |
| The branch head moved before publish | `NoEffect(..)` whose `cause()` is `NoEffectCause::StaleExpectedProductHead` |
| Publication could not complete | `ProductUnpublished(..)`: recover it; recovery never re-runs migration authoring |
| Adopted | `Performed(..)` with `source()`, `target()`, `selected_entity_count()`, `migration()`, `custody()` |

The preparation denials in this table (`TargetRuleRejected`,
`MigrationAssessmentRequired`, `MigrationSourceChanged`,
`MigrationTargetMismatch`, `RequirementsChanged`, `SelectionLimitExceeded`)
and `WorkflowDispositionRequired` ([§7.4](#74-workflows-and-custody-during-adoption))
are variants of `WorthQueryBranchAdoptionPreparationDenial`. They arrive
wrapped as
`WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(..)`; its
`Authorization(..)` variant reports the capability refusal, and
`ProductSelection(..)` means the branch could not be selected.

A writer still running under the old program after adoption lands gets a
stale commit outcome, never a silent commit under the wrong law.

### 7.3 Migrations
#### Advanced program migration preparation

Use the [ordinary owning entry](#53-execute-through-the-program) for application actions.
Migration candidates are deliberate native control selected with `advanced()`.


When the target needs existing state repaired, prepare the migration as a
candidate under the target program, then hand it to the adoption:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Illustrative: not compiled.
// tests/application_graph/adoption/migration.rs (abridged)
let migration = match host
    .runtime()
    .request(&principal, &scope)
    .on_branch(branch)
    .advanced().mutate(SetRetentionIntent { input })
    .without_source()
    .idempotency(&key)
    .prepare_program_migration(&target)?
{
    WorthQueryApplicationProgramMigrationPreparationOutcome::Prepared(prepared) => prepared,
    other => { /* DomainDenied, Cancelled, DeadlineExceeded */ }
};
let programs = host
    .runtime()
    .request(&principal, &scope)
    .on_branch(branch)
    .programs();
let requirements = programs.compare(&target)?;
let prepared = programs
    .adopt(&requirements)?
    .migration(migration)
    .prepare(64)?;       // migration and validation prepare atomically
```

A prepared migration commits nothing and registers no idempotency record
until the adoption publishes.

### 7.4 Workflows and custody during adoption

Every workflow definition and instance on the branch needs a legal
disposition. Ask for the inventory, decide each item, and pass the decisions:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Illustrative: not compiled.
// tests/application_graph/adoption/workflow_custody.rs (abridged)
let adoption = programs.adopt(&requirements)?;
let inventory = adoption.workflow_inventory(1_024)?;
let decisions = inventory
    .dispositions()
    .definition(&inventory.definitions()[0], WorthQueryWorkflowDefinitionDisposition::Retire)?;
let prepared = adoption.workflow(decisions).prepare(1_024)?;
```

- Definitions are `Carry` or `Retire`; instances are `Carry` or `Cancel`. Each
  occurrence's `legal_dispositions()` lists what is allowed.
- An instance that must be migrated refuses a plain disposition
  (`MigrationRequired`).
- Missing decisions deny preparation with
  `WorkflowDispositionRequired { inventory }`.
- Custody dispositions for owner-held effects are derived by the runtime, not
  chosen by you.

### 7.5 Several branches

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Illustrative: not compiled.
let prepared = request
    .on_branches(coverage, &ordered_targets)?     // WorthQueryProgramAdoptionCoverageDenial
    .programs()
    .adopt(&target)?
    .prepare(maximum_selection_work_per_branch)?; // WorthQueryPreparedBranchSetAdoption
```

This prepares adoption across a set of branches. It is **not atomic**:

- Every remaining branch uses its own current source-program permission.
  Advancement rechecks it in a fresh request and returns a refused pending
  native owner to the set, preserving the completed prefix.
- Preparation preflights every branch before any branch publishes. One
  incompatible branch denies the whole preparation and names that branch.
- `prepared.advance(runtime, &principal, &scope)` publishes the next branch and returns its progress
  (`None` when every branch is done). After a `NoEffect` or unpublished
  branch it is denied with `ResolutionRequired` until you resolve it.
- `prepared.cancel()` stops the rest; branches already adopted stay adopted.
  It returns `Result<WorthQueryBranchSetAdoptionCancellation, Self>`: while a
  branch awaits resolution it hands the adoption back, so resolve it and
  `advance` first.
- `resume(stopped, max)` continues a stopped set.

Coverage comes from `runtime.branches().program_adoption_coverage(..)`;
`on_branches` orders it against your target list and refuses a mismatch with
`WorthQueryProgramAdoptionCoverageDenial`. A branch-set member that committed
but did not publish is recovered with
`request.recover_branch_set_adoption(recovery)`.

### 7.6 Siblings, history, and retirement

- **Siblings keep their program.** Adoption writes only the adopting branch.
  A branch forked earlier keeps its own activation, so it keeps running and
  enforcing the old program's rules.
- **Retained reads pin their program.** A retained selection or read keeps
  the old program's support and occurrence alive for as long as it is held.
- **Retirement.** `retire_program_support(&revision)` succeeds only when no
  current branch, retained interpretation, or mandatory custody still uses
  that revision.

---

## 8. Rules for AI agents

**Do:**

- Import only `worth_query_decl::facade` and `worth_query_host::facade`.
- Build features with `ApplicationFeatureSpec::root::<S, F>()` (or `at`) and
  the builder methods. Call `finish()`.
- Validate programs with
  `ApplicationProgramAuthoring::<S, P>::begin().validated_program()` and
  install with `application_installation::in_memory_program` or
  `in_memory_rostered_program`.
- Run fixed and source-only actions through `idempotency_owned(key)` and
  `retain_operation::<Program, Action>(&runtime)`, then supply fresh ordinary
  requests for advancement, delivery and close. Native admission checks the
  original runtime, branch and full action meaning on every attempt.
- Retain guarded workflow effects with their original issued requirement; use a
  separate owning assessment demand for observation. Neither replaces evidence
  acceptance or approval. Keep discovered work on its native entrance until its
  owner lands; do not add a consumer retry engine.
- Match every outcome variant. Treat `NoEffect`, `ProductUnpublished`,
  `Awaiting*`, and `Commit(..)` as states to handle, not errors to retry.
- `compare` before `adopt`, and pass the requirements you got.
- Name the expected predecessor when you publish a workflow definition.
- Pump workflows yourself: one advance per request, until an advance returns
  `Completed(t)` with `t.terminal() == true`. `Completed` alone means one step
  was recorded.

**Do not:**

- Do not import `worth_query_declaration`, `worth_query_installation`,
  `worth_query_admission`, `worth_query_execution`, `worth_query_publication`,
  or `worth_query`. Their consumer names are re-exported through the two
  facades (for example `worth_query_host::facade::domain` and
  `worth_query_host::facade::admission`).
- Do not install a program-less runtime as an ordinary application or coordinate
  fixed/source-only retries manually. Deliberate native control is described under
  [Advanced](#advanced-native-mutation-control).
- Do not substitute an action or program meaning on retry. Native admission compares the retained action with the branch's selected program.
- Do not expect adoption to change sibling branches, or a branch set to adopt
  atomically.
- Do not register a handler for a workflow control binding.
- Do not wait for a background scheduler to run work for you. There is none,
  and no timer or callback advances a workflow instance: the caller pumps.
  Time- and signal-driven work is declared with `conditional_operation`: the
  host submits clock or predicate observations, Signal decides whether a wake
  is eligible, and Query freshly admits the installed operation. Do not build
  a host scheduler or wake registry.
- Do not treat a validated program or validated workflow definition as
  permission to do anything.
- Do not treat names found only in `plans/` as real. Check §9.
- Do not build a substitute for a capability you could not find: no parallel
  registry, copied authority, host executor or scheduler, or host persistence
  of platform truth. Follow the missing-capability procedure in the
  [Capability Index](capability-index.md).
- Do not present `compare_and_commit_program_undo` / `_redo` as an accepted
  correction contract. They are provisional ([§5.2](#52-mutations-intent-binding-handler)).

---

## 9. Use the consumer vocabulary

Plans describe intent; they do not define the consumer API. Use the declarations
and host functions exported by `worth-query-decl` and `worth-query-host`.
The table separates authoring tasks from unsupported shortcuts.

| Task or shortcut | Status | Use instead |
|---|---|---|
| A separate composition or module specification | Unsupported | `impl ApplicationProgramDefinition` plus `ApplicationFeatureSpec` |
| A stand-alone action specification | Unsupported | Builder methods on `ApplicationFeatureSpec` (`mutation`, `operation`, ...); `ApplicationActionDeclaration` is the read-back type |
| Adopting a revision before comparing it | Unsupported | `compare(&revision)` → `adopt(&requirements)` → `[.migration(..)] [.workflow(..)]` → `prepare(max)` |
| "unchanged / additive / restricting / meaning-changing" diff classes | Not the API | `ApplicationSemanticChangeKind::{Added, Removed, Changed}` per family |
| A separately owned published-definition value | Unsupported | `PublishedWorkflowDefinitionRef` |
| `CompiledWorkflowDefinition`, `AdmittedWorkflowTransition` | Crate-private | Nothing to call: the kernel uses them internally |
| A workflow slot on the program | Does not exist | `retain_workflow_spec` on the program runtime, then `support_workflow_spec` on the workflow runtime it returns |

---

## 10. Where to go deeper

| Topic | Guide |
|---|---|
| Programs, rosters, adoption, branch sets, retirement, full denial lists | [Programs and Adoption](../workspaces/worth-query/crates/worth-query/docs/foundations/programs-and-adoption.md) |
| Every workflow call, budget, denial, and a worked example | [Workflows](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md) |
| Feature capsules, ports, connections, and the authoring rule | [Feature Capsule Authoring](../workspaces/worth-query/crates/worth-query/docs/authoring/feature-capsule-authoring.md) |
| The ordinary application path, anti-patterns, current limits | [Ordinary Application Front Door](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md) |
| Contribution-composed apps, output demand, host may / must-not | [`worth-query-host` README](../workspaces/worth-query/crates/worth-query-host/README.md) |
| Declaration surface | [`worth-query-decl` README](../workspaces/worth-query/crates/worth-query-decl/README.md) |
| The complete pure-facade reference application | [Bank public consumer contract](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md) |
| What the runtimes guarantee underneath all of this | [How WORTH Works §9, §12, §13](how-it-works.md#9-query-the-life-of-one-request) |
| Find the call for a task; what to do before concluding a capability is missing | [Capability Index](capability-index.md) |

These four Query guides live under the `worth-query` crate's `docs/`
directory for historical reasons. They are **consumer guides**: every name
they use is on the two facades.
