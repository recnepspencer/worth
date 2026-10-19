# Build an Application

> **The application story, end to end:** declare the program and its
> features, install the application graph, run it, author and run workflows,
> and adopt a new program on a branch. A code block that follows a
> `<!-- compiled: ... -->` marker is checked byte-for-byte by CI against a
> compiled fixture. A `<!-- doctest: crate -->` marker binds a block to the
> crate's rustdoc and CI doctest lane. Illustrative blocks visibly say
> `// Illustrative: not compiled.`. The configured consumer guides require zero
> unclassified Rust fences; CI refuses debt growth.

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
| `…host::facade::application_entry` | `WorthQueryApplicationRequestExt`, `WorthQueryOperationBuilder`, `WorthQueryOperationDescription`, `WorthQueryOperationAdvance`, `WorthQuerySettledFixedOperation`, `WorthQueryTerminalFixedOperation`, `WorthQueryWorkflowOperation`, `WorthQueryApplicationMutationOutcome`, `WorthQueryBranchAdoptionPublicationOutcome`, `Workflow*Outcome`, `Published*Ref` |
| `…host::facade::application_discovery` | `WorthQueryWorkflowDefinitionDiscovery`, `WorthQueryWorkflowInstanceRead`, `WorthQueryWorkflowInstancePage`, `WorthQueryWorkflowHistoryPage`, `WorthQueryWorkflowHistoryMark` |
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
| 5 | Run program-owned actions | `request.mutate(intent)…idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)` | [§5](#5-run-the-program) |
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

`ApplicationFeatureSpec` records which feature owns which actions. Begin a
feature with `root::<Schema, Feature>()` or
`at::<Schema, Instance, Feature>()`, then begin each ordinary action with
`.action::<Binding, Delivery>()`. Delivery is mandatory and names the real
source-completion, retained-commit, fixed-output or discovered-output contract.
Attach the action's optional metadata before `.finish_action()` returns the
feature parent. Only then can you add another action or finish the feature.

The neutral consumer's actual fixed source action retains locality/change,
evaluated requirement and external-input metadata alongside its real output
delivery. This excerpt uses the surrounding imports and source-feature members
from [the compiled declaration owner](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_program/composed_action.rs):

<!-- compiled: composed_action.rs -->
```rust
pub(crate) fn source_feature_spec() -> ApplicationFeatureSpec {
    features::source_builder()
        .action::<PlanarSourceAdjustmentBinding<ConsumerSchema>, ConsumerFixedDelivery>()
        .locality_and_change::<SourceLocality, SourceChange>()
        .evaluated_requirement::<CatalogInputRequirement>()
        .external_input::<external_input::NeutralExternalProvider>()
        .finish_action()
        .finish()
}
```

| On the feature parent | Result |
|---|---|
| `action::<Binding, Delivery>()` | An unfinished ordinary action with explicit delivery |
| `native_action::<Binding>()` | An unfinished deliberately specialized native action, without delivery |
| `operation::<Op>()` | A declared operation on the root-composition builder |
| `conditional_operation::<Op>()` and `conditional_operation_with_requirement_and_external_input` | A conditional operation |
| `provides::<Port>()` | An output port this feature provides |
| `derived_artifact`, `managed_computation`, `derived_collection` | Derived outputs the feature produces |
| `finish()` | The complete `ApplicationFeatureSpec` |

| On the unfinished action | Contract |
|---|---|
| `locality_and_change::<Scope, Shape>()` | The paired locality and change shape |
| `correspondence::<Correspondence>()` | Correspondence for this mutation binding |
| `evaluated_requirement::<Rule>()` | An evaluated rule for the binding's operation |
| `external_input::<Provider>()` | An external-input provider for the binding's operation |
| `finish_action()` | Append exactly one complete declaration and return its feature parent |

Each optional clause consumes its unconfigured state; setting it twice is a
compiler error. All subsets are legal under their existing trait bounds,
including a rule without a provider and a provider without a rule. No incomplete
action is appended, and the action builder has no feature `finish()` or
`provides()` method. Ordinary delivery derives required-output-source meaning;
there is no output-source override. A source-only declaration cannot erase a
binding's actual required-output obligation: owning construction refuses it.

`ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery>`
selects this exact occurrence. Its sealed `ApplicationAction::selector()`
returns selection facts, not a complete declaration. Installation resolves the
complete actual action, including every metadata contract; fresh progression
compares that full installed meaning. Duplicate binding types and duplicate
portable `(instance, feature, binding identity)` keys are rejected. Distinct
legal composition occurrences remain distinct.

Correspondence lookup is binding-scoped. Rule/provider capabilities are scoped
to operation plus rule/provider type across declared actions; they do not become
occurrence-isolated permission. Installing a clause neither evaluates a rule nor
admits an external revision. Use the installed evaluator's result for guidance
and submission enforcement; native authorization remains with its existing owner.
Resolve external input, admit its captured revision and pass the accepted values
into the original mutation intent.

The `worth_query_feature_spec!` macro on the decl facade lowers
`action(Binding, Delivery) { ... };` and `native_action(Binding) { ... };`
through that same builder. Clauses are `locality_and_change(Scope, Shape);`,
`correspondence(Correspondence);`, `evaluated_requirement(Rule);` and
`external_input(Provider);`. Empty blocks are valid. Clause order preserves
canonical meaning; unsupported or repeated clauses fail rather than disappear.
See [Feature Capsule Authoring](../workspaces/worth-query/crates/worth-query/docs/authoring/feature-capsule-authoring.md).

Use `native_action` only for a present specialized native responsibility, such
as staging, authority/phase certification or explicit branch/host control.
Ordinary product mutations and retries use explicit owning delivery, including
when a native caller also consumes that occurrence. Native required-output
membership comes from actual declared roots; there is no native output toggle.

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
- Validation uses `ApplicationProgramDefinition::CANONICAL_REVISION_BUDGET`,
  a finite `CanonicalDigestWorkBudget` available from the same program facade.
  Its default remains 4096 basis entries and 1 MiB encoded bytes. Large authored
  programs may override that associated constant; exceeding either ceiling is
  a typed `CanonicalRevisionBudgetExceeded` denial. The budget governs cold
  declaration work, is not part of the manifest, and does not change the revision
  when the accepted program meaning is identical. It grants no runtime resource
  or publication permission.
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

Capability delegation and revocation use retained native commands. Call
`request.retain_capability_delegation_in_program(..)` or
`request.retain_capability_revocation_in_program(..)` once with the owned original
input and key. Capture refusal returns `(input, key, prepare, denial)` intact.
The preparation closure may repeat under a fresh admission; it projects and
materializes the native command program, while Query alone commits effects.
Advance the returned owner with a fresh request rather than reissuing intent.
A refusal returns the same command. A completed owner exposes native receipt
evidence and still requires `close_with_evidence(&fresh_request)`; close refusal
returns that same completed owner. See [owning capability commands](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#owning-capability-commands).

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
`.action::<Binding, Delivery>() ... .finish_action()`; its delivery mode is
`ApplicationSourceCompletion` or `ApplicationRetainedCommit`. Select the exact
`ApplicationActionReference`, rather than a first matching binding.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let operation = request.mutate(intent)
    .without_source()
    .preconditions(original_conditions)
    .idempotency_owned(key)?
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
`retain_operation::<Program, Action>(&runtime)`. Fixed, discovered and source-only callers
cannot execute, prepare or recover through that builder or its borrowed-key view.
The selector carries composition instance, feature, binding and delivery;
Installation resolves the complete actual action, including every metadata
contract. A binding label alone cannot select its contract.

`idempotency_owned` first opens the initiating request's native capture phase and
returns `Result<Description, (original_builder, owned_key, capture_denial)>`.
Retain all three refusal fields. No business effect or workflow-slot transfer has
occurred. A later capture retry uses that builder and key with a fresh native
opening. Success freezes one typed input payload under the existing Execution
allocation policy; encoded clones and native capability consumers share that
same backing and keep its charge until their last owner releases it. The quote
covers typed payload, not nested caller Clone heaps or allocator/Arc metadata.
The original Intent is moved, never cloned by this capture.
An adapter whose initiating scope ends can consume the returned builder with
`builder.into_capture_refusal(key, denial)`. This transfers the original input,
conditions, source, key and branch into `WorthQueryOperationInputCaptureRefusal`
without retaining the initiating request or principal. Its `retry_capture`
requires a fresh request on the original runtime/branch and delegates the same
native capture; every refusal returns the owning original. It never executes a
source, consumes a workflow occurrence or creates a completed description on
failure. Bank construction returns this owner in
`BankOperationConstructionRefusal::Capture`; declaration refusal retains the
funded description separately. Transport may decline an unexecuted request,
which is distinct from losing a post-effect cleanup obligation.

Fixed, discovered and source-only owners retain the authentic installed runtime
allocation and complete installed action by value. Their returned types have no
construction-runtime lifetime; they can be stored after the constructing borrow
ends or the constructing handle is dropped. Keep a separate genuine runtime
handle for fresh request construction. Operation owners expose no runtime
accessor or `Deref` for admission. Retention shares the existing allocation; it
copies neither the graph nor native obligations and grants no permission.

Configure workflow vocabulary and other unique-runtime setup before cloning
handles or retaining operations. While an operation retains the allocation,
runtime close refuses with `RuntimeKeepaliveRetained`. Successful operation
close releases that operation's keepalive; refusal returns the complete owner.
Dropping an owner abandons custody and does not perform successful cleanup.

For an eligible Required fixed leaf, a producer's terminal domain refusal can
close through the same ordinary terminal owner. `terminal.close(&fresh_request)`
delegates native negative completion before cleanup; a refusal returns the same
owner. If native finish succeeds but cleanup refuses, inspect
`terminal.output_unavailable()` and retry close with fresh authority. Successful
close carries that actual evidence in `closed.output_unavailable()`: the original
source was saved and no current output was produced. This is neither successful
required-output settlement nor permission to publish a positive result. Native
eligibility still refuses unresolved preparation, partial work, already settled
roots, dependent trees and discovered aggregates. No source or producer is
reexecuted to close. The executable ordinary fixed-leaf court is
[`terminal_unavailable.rs`](../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/producer_predicate_checkpoint/ordinary_source/terminal_unavailable.rs).

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
#[allow(
    clippy::result_large_err,
    reason = "The example returns the complete original builder and key without allocation"
)]
pub fn capture_original<'a, 'p, 's, S, I>(
    request: &WorthQueryApplicationRequest<'a, 'p, 's, S>,
    intent: I,
    preconditions: TypedMutationPreconditions<S, <I::Binding as ApplicationMutationBinding<S>>::Operation,
        <<I::Binding as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope>,
    source: WorthQueryObservedResultSet<<<I::Binding as ApplicationMutationBinding<S>>::SourceExpectation as ApplicationMutationSourceExpectation<S>>::Query>,
    key: <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
) -> Result<
    WorthQueryOperationDescription<S, I>,
    (
        WorthQueryOperationBuilder<'a, 'p, 's, S, I, WorthQueryMutationSourcePrepared>,
        <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
        WorthQueryOperationInputCaptureDenial,
    ),
>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
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

The ordinary workflow binding retains `WorthQueryWorkflowOperation` through
`for_workflow_operation(..)?.retain_operation::<Program, Action>(program)`.
Advance it with a fresh request; retain the returned owner on refusal or progress.
The original slot, native recovery and cleanup remain owned by Query.

#### Advanced native mutation control

Use the [ordinary owning entry](#53-execute-through-the-program) for fixed,
discovered, source-only and guarded workflow actions. Deliberate native callers
select `request.advanced().mutate(intent)`. Only that type exposes manual
fixed/discovered/source-only execute, two-phase prepare, unpublished recovery,
keyed resolution and fixed output start/recovery. Borrowed native workflow
execution binds the issued requirement before selecting
`for_workflow_operation(..)?.advanced().execute_in_program(..)`.
A runtime installed without a
program (`application_installation::in_memory`) is advanced native control;
its `execute` cannot be an ordinary application default. On a program runtime
non-program execution refuses with `ApplicationProgramRequired`.
Native calls take an explicit `ExecutionAllocationPolicy`; the owning attempt
uses its admitted native policy. Unsupported predecessors refuse before effects
or executable readmission and preserve original bytes; no compatibility entrance
is retained.

#### Storage beyond the construction borrow

`WorthQueryFixedOperation<Schema, Intent, Program, Root, Action>` and
`WorthQueryDiscoveredOperation<Schema, Intent, Program, Root, Action>` carry no
runtime lifetime parameter. Return or store the actual owner across commands;
retain the operation returned on each progress or refusal. Fresh requests must
come from a separate genuine handle for that same allocation and original
branch, not from the operation itself.

The fixed consumer below first clones its authentic runtime handle for request
construction, moves the original handle into `constructing`, retains the
operation, and drops `constructing` before advancement. It uses the still-owned
original allocation to construct request authority, resolves and admits the external
provider's actual value and checks the resulting source and outputs. This is
the same installed fixed journey, not a second runtime or a fabricated lifetime.

### Compiled owning call bodies

These excerpts are byte-bound to their actual compiled consumer fixtures or
registered examples. Their enclosing sources supply the installed program,
authenticated request, schema imports and real seed world. They are not
standalone programs. The existing bounded fixture pump calls one fresh native
advancement at a time; production code retains the returned owner between
requests, including every refusal and pending result.

Fixed: the two-instance court selects its complete primary action, uses admitted
external input and the installed evaluator, drops the constructing runtime
handle, advances the original owner, observes source/output truth and closes.

<!-- compiled: owning_fixed.rs -->
```rust
    use crate::application_program::{
        composed_action::CatalogInputRequirement,
        external_input::{NeutralExternalDenial, NeutralExternalProvider},
    };
    use worth_query_topology_entry::AdjustPlanarSource;
    let external = constructing
        .installed_program()
        .external_input_provider::<AdjustPlanarSource, NeutralExternalProvider>()
        .unwrap();
    let provider = NeutralExternalProvider::new(7);
    let before_input_refusal = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let captured = external.resolve(&provider, "replacement-y").unwrap();
    provider.change(2);
    assert_eq!(
        captured.admit(&provider).err(),
        Some(NeutralExternalDenial::Changed)
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before_input_refusal
    );
    let accepted = external
        .resolve(&provider, "replacement-y")
        .unwrap()
        .admit(&provider)
        .unwrap()
        .into_resolution();
    assert_eq!(accepted.revision(), &2);
    let requirement = constructing
        .installed_program()
        .evaluated_requirement::<AdjustPlanarSource, CatalogInputRequirement>()
        .unwrap()
        .evaluate(accepted.revision());
    assert_eq!(
        requirement.required_input_guidance(),
        Some(&"catalog-value")
    );
    assert_eq!(
        requirement.enforce_submission(false).unwrap_err().missing(),
        &"catalog-value"
    );
    requirement.enforce_submission(true).unwrap();
    let primary_input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(*accepted.values()),
    };
    reset_planar_source_adjustment_contacts(&primary_input);
    let source = request
        .query(PlanarRead {
            body_key: primary_input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let primary = request
        .mutate(primary_input.clone())
        .expect_source(source)
        .idempotency_owned(10_193)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<ActionInstanceProgram, ConsumerFixedAction>(&constructing)
        .unwrap_or_else(|(_, denial)| panic!("primary exact contract: {denial:?}"));
    drop(constructing);
    assert_eq!(world.application.close_conditional_runtime().unwrap_err().kind(),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained);
    // Fresh requests come from the independently retained authentic allocation.
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    let request = world.application.request(&fresh_principal, &fresh_scope);
    let mut primary = Some(primary);
    let primary = (0..32)
        .find_map(|_| match primary.take().unwrap().advance(&request) {
            Advance::Progress { operation, .. } => {
                primary = Some(operation);
                None
            }
            Advance::Settled(settled) => Some(settled),
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                panic!("native fixed advancement: {denial:?}")
            }
        })
        .expect("the declared fixed graph settles");
    assert_eq!(primary.declared_action(), &primary_declaration);
    assert_eq!(
        primary.declared_action().delivery().unwrap().identity(),
        <ConsumerFixedDelivery as ApplicationActionDelivery<ConsumerSchema, Binding>>::IDENTITY
    );
    assert_eq!(primary.settlement().output_count(), 4);
    let output_delivery = primary.read_current(&request).unwrap();
    assert_eq!(output_delivery.rows()[0].body_key, "anchor-a");
    assert_eq!(output_delivery.rows()[0].value, length(3));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: primary_input.scope_key.clone()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(*accepted.values())
    );
    assert_eq!(planar_source_adjustment_contacts(&primary_input), (1, 1));
    primary
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("primary close: {cause:?}"));
    world.application.close_conditional_runtime().unwrap();
```

Discovered: the existing public journey carries its original input through required-set settlement, guarded delivery and close.

<!-- compiled: owning_discovered.rs -->
```rust
    let operation = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency_owned(10_030)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(&world.application)
        .unwrap_or_else(|(_, denial)| panic!("declared discovered construction: {denial:?}"));
    let mut operation = Some(operation);
    let settled = settle(|| match operation.take().unwrap().advance(&request) {
        WorthQueryOperationAdvance::Progress {
            operation: next, ..
        } => {
            operation = Some(next);
            None
        }
        WorthQueryOperationAdvance::Settled(settled) => Some(settled),
        WorthQueryOperationAdvance::Refused { denial, .. }
        | WorthQueryOperationAdvance::Terminal { denial, .. } => {
            panic!("ordinary discovery: {denial:?}")
        }
    });
    assert_eq!(
        settled.read_current(&request).unwrap().rows()[0].value,
        length(42)
    );
    let roots = settled.settlement().root_outputs().collect::<Vec<_>>();
    assert_eq!(roots.len(), 3);
    assert_eq!(
        roots
            .iter()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["remote-b", "sibling-b", "sibling-c"]
    );
    assert_eq!(settled.settlement().output_count(), 6);
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0,
        "settling all discovered roots releases the performed source and observation"
    );
    for (key, expected) in [
        ("sibling-b", length(22)),
        ("sibling-c", length(31)),
        ("remote-b", length(42)),
    ] {
        assert_eq!(
            request
                .at(settled.settlement().observation())
                .query(PlanarOutputRead {
                    body_key: key.to_owned()
                })
                .execute()
                .expect("each settled output is readable")
                .rows()[0]
                .value,
            expected
        );
    }
    assert_eq!(
        request
            .at(&prior)
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the prior authored source remains coherent")
            .rows()[0]
            .y,
        length(1)
    );
    settled
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("lawful discovered close: {denial:?}"));
```

Source-only: the runnable ordinary product example retains a no-output action, distinguishes real Fresh evidence from recovery and closes.

<!-- compiled: owning_source_only.rs -->
```rust
    let operation = request
        .mutate(AmendTemporalIntent {
            identity: "intent-1".to_owned(),
            amendment: AmendTemporalInput {
                revision: 2,
                due: 11,
                lifecycle: "active".to_owned(),
                input: "published-through-application-entry".to_owned(),
                gate: "ready".to_owned(),
            },
        })
        .without_source()
        .idempotency_owned(0x51_u64).unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<product_workflow_support::program::TemporalExampleProgram,
            product_workflow_support::program::AmendTemporalAction>(&application.runtime)
        .unwrap_or_else(|(_, denial)| panic!("declared temporal action: {denial:?}"));
    let completed = match operation.advance(&request) {
        WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
            panic!("fresh attempt refused: {denial:?}")
        }
        WorthQuerySourceOnlyAdvance::Progress(_) => {
            panic!("this successful native attempt has no pending publication")
        }
        WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
            panic!("native terminal: {:?}", terminal.native_outcome())
        }
    };
    let committed_version = match completed.evidence() {
        WorthQuerySourceCompletion::Fresh {
            receipt, result, ..
        } => {
            assert_eq!(result.revision, 2);
            receipt.commit_reference().version_id.as_u64()
        }
        WorthQuerySourceCompletion::Recovered { .. } => {
            panic!("a new key must produce the actual handler result")
        }
    };
    completed
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("native cleanup refused: {denial:?}"));
```

Workflow: the runnable authored example obtains the actual awaited slot, captures before binding it, advances the exact installed action and closes. A later workflow advance remains separate.

<!-- compiled: owning_workflow.rs -->
```rust
    let required = match advance_instance(application, instance.clone(), key)
        .expect("the approved effect requirement prepares")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("the approved effect must be awaited, got {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let effect = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: days,
            },
        })
        .without_source()
        .idempotency_owned(key + 1).unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .for_workflow_operation(application, &required)
        .unwrap_or_else(|_|panic!("exact original approved occurrence"))
        .retain_operation::<document_retention_model::programs::WorkflowOwnerProgram,document_retention_model::programs::ReviewedAction>(application.program_runtime())
        .unwrap_or_else(|_|panic!("installed approved effect action"));
    let effect=match effect.advance(&runtime.request(&principal,&scope).on_branch(instance.branch())) {
        worth_query_host::facade::application_entry::WorthQueryWorkflowOperationAdvance::Completed(done)=>done,
        _=>panic!("lawful approved effect"),
    };
    effect
        .close(
            &runtime
                .request(&principal, &scope)
                .on_branch(instance.branch()),
        )
        .unwrap_or_else(|_| panic!("native effect cleanup"));
    assert_eq!(read_retention(runtime, instance.branch()), days);
```

### 5.4 Mutations with required outputs

Fixed owning construction validates the installed governed root artifact's
actual source locality, change/succession and dependencies before minting an
effect-capable owner. Refusal returns the original operation description and
`WorthQueryOperationConstructionDenial::OutputSource`; the borrowed authentic
runtime remains with the caller, and no source effect has run. This uses the
same native artifact-source guard as later required-output preparation.


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

Author that source with `.action::<Binding, Delivery>() ... .finish_action()`; use an
`ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery>` to
select it. Program validation requires the delivery root to belong to that
program and to start at the action's actual source occurrence.

An action with no native output-source obligation declares
`ApplicationActionDelivery::Mode = ApplicationSourceCompletion` for no projection,
or `ApplicationRetainedCommit` for its authentic commit observation. Author it
with `.action::<Binding, Delivery>() ... .finish_action()` and select its exact
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
Ordinary SourceOnly callers retain this owner rather than coordinating manual
execution and recovery.

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
    .idempotency_owned(key)?;
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
`ApplicationOutputProjection` and `WorthQueryOutputProjectionInstallation` are
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

### 5.5 Own a complete no-source output tree

Use the ordinary demand builder when a read request needs the complete installed
output graph without an authored source mutation:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Concrete call bodies live in the neutral tree fixture.
let operation = request.demand(original_demand)
    .controls(WorthQueryOutputDemandControls::host_policy())
    .retain_program_outputs::<Program, FullTree>(&program);
```

Success returns `WorthQueryProgramOutputOperation<Schema, Program, FullTree>`.
Construction refusal owns the original demand, controls, branch, authentic program
runtime and any already issued observation. Retain it past the initiating scope
and call `retry(&fresh_request)`. A retry never replaces an issued basis;
`into_parts()` returns the actual original inputs for an explicit correction.
No principal, request scope or phase is retained.

Call `advance(&fresh_request)` explicitly. `Pending` and `Refused` return the same
original request custody; `Settled` owns all original output settlements and
complete discovery evidence; `Terminal` owns the genuine refusal and cleanup.
A root-only `start_in_program::<Program, FullTree>` demand does not traverse the
dependent tree and cannot substitute for this contract.

On the settled owner, `read_current(&fresh_request, typed_query_intent)` checks the
fresh selected program/root, every original settlement and original discovery
membership, then executes the actual typed query at that same native observation
in the same phase. `read_current_with_policy` narrows installed native limits;
`deliver_current` returns the checked observation without a query. Refusal leaves
the owner intact. These calls do not create another traversal or replace the
original source/demand.

Initial, settled, terminal and construction-refusal owners expose consuming
`close(&fresh_request)`. Success returns `WorthQueryClosedProgramOutputRequest`:
this proves discharge of the caller's read custody, never positive output,
source completion or platform shutdown. Close validates the original allocation
and branch and fresh cleanup authority. It does not require the original source
or root to remain current after supersession or program adoption.

Close refusal owns the whole original context and any already discharged native
prefix. Retain `WorthQueryProgramOutputCloseRefusal` and call `retry_close` with
fresh original-runtime/branch authority. That stage cannot advance released
interests. Actual producer cleanup, typed required successors and selected
unpublished checkpoints remain owned until their native obligations permit
disposal. An actual producer unpublished partial releases through its original
native recovery owner; refusal retains that same owner for fresh close retry.
When native release requires settlement, close may finish that original owner's
retained settlement before asking native release again. Neither release nor
close reruns the producer or publishes a current result.
Physical release never erases unresolved producer recovery. Dropping an owner is
abandonment, not a successful close. Performed-source required and
discovered aggregate completion rules, including fixed-leaf unavailable
completion, are unchanged.

#### Delivery after native close

When final publication must follow cleanup, call the settled parent's
`close_for_delivery(&fresh_cleanup_request)`. A Fixed parent returns
`WorthQueryClosedFixedDelivery`; a complete no-source parent returns
`WorthQueryClosedProgramOutputDelivery`. Each sealed carrier owns the original
complete settlement, original installed meaning and authentic runtime, beside
actual native close evidence. Its immutable `settlement()` includes the genuine
root correspondence even for `StableReused` with `root_receipt() == None`.
No original demand, controls or extra source-basis lease survives no-source close.

Close validates original allocation/branch and fresh cleanup authority; stale
sources or changed selected programs do not prevent disposal. Fixed refusal
returns the same settled parent and typed cause. No-source refusal owns the
positive settlement and all remaining native custody, including any discharged
prefix; `WorthQueryProgramOutputDeliveryCloseRefusal::retry_close` is close-only.
It cannot deliver, advance, or convert back into a live tree.

Only after actual native close succeeds, use the carrier's fresh `deliver_current`
or `read_current`. Fixed declared query/batch shapes retain their original
selectors and authorization; no-query delivery requires `ApplicationNoProjection`.
These calls reuse the live owner's exact complete output/discovery and installed
context validation, then read at that native observation in the same phase.
Any refusal borrows and preserves the already-closed carrier. It cannot rerun
source/producer work, create another demand or reopen cleanup. Plain `close`
remains available for cleanup-only intent.

Complete-tree currentness is as of that fresh native observation. Additional
application projection facts must be read again at the delivered observation;
old frozen views and descriptive commit equality provide no substitute.
Closed carriers and returned observations pin readonly lineage/history and the
runtime; they do not certify whole-application shutdown or own uncleared cleanup.
Drop them promptly after final publication, before whole-runtime close or OS join.
The neutral `application_program::closed_delivery` fixture exercises both owners,
including a real native source perturbation for Stable/no-receipt reuse. Its
rostered-successor court selects compatible meaning before capturing the original
operation, then checks genuine post-close current delivery. Later changed/removed
actions deny delivery while original native read interests still close. Restoring
a program does not restore an unavailable exact native output basis: matching
action meaning is necessary, and complete-tree currentness remains a separate
requirement. These checks do not replay a source or producer.
Any program adoption after settlement may make the original retained basis
terminally unavailable (`RetainedBasisUnavailable`). Matching action meaning
does not repair that basis; the application must run a fresh operation.

To present several closed carriers together, retain one observation and deliver
every carrier at it with `deliver_current_at(&request.at(&observation), policy)`.
It exists on closed no-projection Fixed and closed no-source carriers. It runs
the same native gate as `deliver_current` at that caller's observation:
no-projection Fixed re-resolves its original installed action, no-source checks
its selected typed root, and both validate their complete original lineage under
the installed currentness ceiling. A closed Fixed carrier with a declared query
projection delivers only through its declared read (`read_current`,
`read_batch_current`); it has no common-observation delivery today. That is a
known gap, not permission to pair a unit gate with a replacement selector. The request's scope is checked fresh. A
foreign runtime, branch or observation is refused before any installed meaning is
read. Bare `request.at(&observation).require_current_program_output(...)` checks
lineage only. It neither revalidates the selected action or root nor replaces
this gate. Success returns `()` because the caller already holds the
observation. Read every displayed fact at that same observation.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let observation = request.retain_current_read()?;
let pinned = request.at(&observation);
for carrier in &closed_no_projection_fixed {
    carrier.deliver_current_at(&pinned, policy)?; // installed action + full lineage
}
for carrier in &closed_trees {
    carrier.deliver_current_at(&pinned, policy)?; // selected typed root + full lineage
}
let membership = pinned.query(membership_read).execute()?; // facts at the same basis
```

A refusal for one carrier leaves every carrier closed and unchanged. Only a
typed native `Superseded` (Fixed `Stale`, no-source `Currentness(Output(..))`
of kind `Superseded`) tells the application to run a fresh operation for that
carrier. `RetainedBasisUnavailable` and other causes keep their own posture.
The compiled shape is in Host `closed_delivery_contract`.

#### Original root output evidence

A settled parent's `settlement().root_settlement()` borrows its actual immutable
`WorthQueryApplicationOutputDemandSettlement<RootQuery>`. Inspect its existing
`outputs_of::<Contract>()` and typed `entity::<Role>()` correspondence to validate
which native entity fills the original root role. A genuinely reused root can
have `root_receipt() == None` and still carry that correspondence; no receipt,
new demand, key lookup or source-query identity needs to replace it.

This borrow is root-only retained evidence. It performs no fresh currentness
check and discharges no tree or cleanup custody. Keep the parent operation;
use its `read_current` or `deliver_current` for complete-tree validation, and
its `close` for lawful original read-request disposal. Projection at the
aggregate basis uses `settlement().observation()`, not `root_observation()`.
The executable `checkpoint_recovery::root_delivery` fixture exercises the
no-receipt case and closes the same original parent and runtime.

### 5.6 Retain preview custody across refusal

An application preview starts from an exact retained read observation:
`runtime.begin_preview(&phase, &observation, WorthQueryApplicationPreviewRequest::new(identity))`.
The request and session types are exported from
`worth_query_host::facade::application_installation` as
`WorthQueryApplicationPreviewRequest` and `WorthQueryApplicationPreviewSession`.

`preview.readmit(&phase, &runtime)` returns either the readmitted owner or
`(original_preview, WorthQueryApplicationPreviewReadmissionDenial)`. A foreign
phase/runtime, stopped selection or stale source leaves that original preview
available for explicit discard. Each preview retains one authentic installed
runtime clone until its native owner and source custody are discharged; closing
the installed runtime while a preview retains it returns `RuntimeKeepaliveRetained`.
Readmission never replaces its selected source or automatically discards it. A new request can retry the same owner when the
cause permits; stale source does not become fresh by retrying.

`readmitted.promote()` finalizes the original admitted native session and execution
proof. Failure returns `(same_readmitted_owner, BridgeSpeculationError)`.
Promotion is the preview evidence transition; it does not reselect current Query
source, save source, settle required outputs, or certify current publication.
An admitted source action may legitimately commit between readmission and preview
promotion; that action's authentic source and output contracts still govern its
publication. Successful promotion consumes the original native preview once.

Both preview phases expose `discard() -> Result<(), (Self, BridgeSpeculationError)>`.
Discard uses the original native owner and can release a stale preview without
selecting a new source. On refusal, retain the returned owner and its original
application context; retry lawful explicit discard on that owner. Dropping either
phase is abandonment, and does not post a successful discard record. Preview
liveness observation is not proof of cleanup or promotion.

## 6. Author, publish, and run workflows

Guarded no-output effects use the original descriptor and issued requirement:
`for_workflow_operation(..)?.retain_operation::<Program, Action>(program)`.
A refusal from this effect-free `retain_operation` returns the bound owner.
Use its consuming `into_parts()` to recover the original description and issued
requirement outside the borrowed workflow runtime, then correct the declared
action/runtime and rebind that same description and requirement. No effect has
run and no slot has been consumed at this construction boundary.
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

For explicitly driven discovered programs, retain the completed aggregate and use
`request.at(&observation).require_current_discovered_program_outputs(&settlement,
maximum_work)` to validate that exact original required set. The check includes
root and dependent discovery consumption and every retained output settlement.
Changing membership refuses delivery even when every surviving root remains
current. An originally empty set still carries its original negative facts;
retiring a native root does not erase those facts or invent an output settlement.
Refusal preserves the borrowed aggregate and its historical observation. This
native guard does not run a projection. Ordinary discovered actions use
`retain_operation::<Program, Action>`, then fresh-request `advance`; their settled
owner composes this complete guard and the installed declared query or batch at
one authentic observation and phase. See [owning discovered actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-discovered-actions).


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
// Direct Query vocabulary setup; Bank retains refusal custody in identity_runtime/workflow_binding.rs.
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

Bind this vocabulary, and support any extra program vocabularies, before cloning
runtime handles or retaining operations. `retain_workflow_spec` returns
`WorthQueryWorkflowRuntimeBindingRefusal<Schema, Spec, Program>` on any refusal.
Inspect `denial()` and consume `into_parts()` to recover the original runtime,
installed spec, signing owner and denial, in that order. Release the real
keepalives before retrying a `RuntimeKeepaliveRetained` refusal. No refusal
silently drops those inputs or requires a replacement runtime or signing owner.
`support_workflow_spec(&mut self, installed)` returns `Err((installed, denial))`;
the caller still owns the borrowed workflow runtime and the returned original
spec. Signing custody must not be logged as diagnostic material.

Bank retains its surrounding application inputs too:
`BankIdentityRuntimeBuildError::WorkflowRuntimeBinding { binding, denial }`
returns an opaque `BankIdentityRuntimeWorkflowBinding` alongside the denial.
Keep that stage. When correcting vocabulary, `replace_workflow_spec` returns
the original installed spec to the caller. After releasing actual dependent
operations, `(*binding).bind()` retries the same authentic runtime, spec,
signing owner and Bank companions. A repeated refusal returns the complete
stage again; keeping only its denial would discard retry custody.

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

#### Create and enroll through one retained action

When the subject must be born with its workflow, declare
`creation.enroll_in_workflow::<Spec, Binding, CreatedRole>()` on the actual
creation body and install `.birth_operation::<Binding, CreatedRole>()` in the
vocabulary. The creator has its own container-scoped capability; it does not use
the unborn subject's current-operation authority. Publish a real definition and
prepare `RequiredWorkflowBirth` through native workflow control.

Ordinary creation and retry then use the original captured descriptor:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let original = workflow.request(&principal, &scope)
    .on_branch(required.branch()).mutate(intent)
    .without_source().idempotency_owned(logical_key)?;
let operation = original
    .for_workflow_birth::<Spec, CreatedRole>(&workflow, &required)?
    .retain_operation::<Program, CreateAction>(workflow.program_runtime())?;
// Supply fresh authority to advance; preserve Progress/Refused or close completion custody.
```

Use SourceCompletion or RetainedCommit delivery, not a native-action declaration
for ordinary mutation retry. Binding checks the frozen original input and key;
construction failures return original custody without consuming birth authority.
Native advancement owns atomic creation/enrollment and recovery under fresh
authorization. The application must not coordinate separate create/start commits
or repeatedly reissue advanced mutations after interrupted completion. See
[workflow-enrolled creations](../workspaces/worth-query/crates/worth-query/docs/authoring/workflow-enrolled-creations.md).

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

**Node authoring.** The [source-derived node inventory](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#source-derived-vocabulary) owns the complete `ApplicationWorkflowNodeKind` list. The following table explains builder usage:

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
| Guarded requirement omitted | Compile refusal; manual bindings must implement the requirement method explicitly. |
| `ApplicationWorkflowExecutionRequirements::not_applicable()` | Explicit non-workflow applicability; refused for a guarded binding. |
| `ApplicationWorkflowExecutionRequirements::occurrence_only()` | No additional evidence. Zero or one approval edge; an authored edge is still enforced. |
| `ApplicationWorkflowExecutionRequirements::requiring_approval::<Cap>()` | An approval by `Cap` is required, and an edited definition cannot remove it. |

Guarded bindings require an explicit declaration. Omission does not select a
legacy policy. Manual implementations return the direct requirement type.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// workspaces/worth-query/crates/worth-query-certification/tests/application_graph/document_retention_model/retention_entry/occurrence_write.rs (abridged)
worth_query_decl::facade::worth_query_mutation_binding!(
    pub OccurrenceSetRetentionBinding for SetRetentionInput, schema DocumentRetentionSchema,
    // ... identity, input, operation, result, idempotency, decision, handler ...
    workflow_authority required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::occurrence_only(),
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
5. **Retain and advance the approved operation.** Capture the original input
   fallibly with `idempotency_owned`, bind that description with
   `for_workflow_operation`, then select the installed declared action with
   `retain_operation`. Capture refusal returns the original builder and key
   before the issued slot transfers. Advance the retained owner with a fresh
   request; it delegates native admission, proposal-input matching and the
   atomic business-write/node-settlement funnel. Keep the returned owner on
   refusal or pending publication; do not rebuild after an uncertain effect.
   See [the owning workflow entry](#6-author-publish-and-run-workflows) for the call shape.
6. **External effects** settle through owner custody and an `AwaitInbound` node.
7. **Advance again** to move to the successor or terminal.

`AwaitingProposal(RequiredWorkflowProposal)` names the node, operation and input
type. Publish the proposal through fresh `prepare_workflow_proposal` admission;
the need grants no authority and waiting records no transition.

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

**Discover** metadata only through an explicitly governed retained read. The
workflow must declare a `definition_metadata`
rule naming the concrete installed read binding. Subject permission does not
open this separate route.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let basis = request.retain_current_read()?;
let target = ApplicationWorkflowDefinitionReadTarget::<PurchaseWorkflowSpec>::new(identity);
let found = request.at(&basis)
    .discover_workflow_definition::<PurchaseWorkflowSpec, DefinitionMetadataRead>(
        workflow, &target, independently_supplied_read_input,
    )?;
// found.basis() identifies the exact data; found.definition() is Current,
// Retired or Unpublished. Neither the result nor a reference authorizes a start.
```

Fresh current capability admission precedes any existence-dependent lookup.
Foreign-Spec names are undisclosed as `Unpublished`; technical/native corruption
failures remain typed errors. A revoked disclosure grant cannot read even an old
retained basis. There is no execution-request-only discovery overload.

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
| `AwaitingProposal(RequiredWorkflowProposal)` | Names the node, operation and input type; publish through fresh `prepare_workflow_proposal` admission |
| `AwaitingActor` | The caller is not authorized for the next step; an authorized caller pumps the same instance |
| `AwaitingAssessment`, `AwaitingCondition`, `AwaitingEvidence` | Settle the named read, then accept it |
| `AwaitingApproval` | A signed approval is required |
| `AwaitingOperation` | The guarded effect must run ([§6.5](#65-who-may-run-a-guarded-operation)) |
| `Application`, `PreparationDenied`, `AuthenticationDenied`, `IdempotencyDenied` | No step was recorded; each says why |
| `ProjectionDenied` | The commit landed and carries its receipt; only its projection was refused |
| `ProjectionReadStopped { receipt, cause }` | The commit landed and its receipt is retained; the projection read it requires stopped (`cause` is a `RelationalExecutionDenialCause`) |

`AwaitingProposal(RequiredWorkflowProposal)` names the node, operation and input
type. Publish the proposal through fresh `prepare_workflow_proposal` admission;
the need grants no authority and waiting records no transition.

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
`ApprovePaymentMutationBinding` explicitly requires its approval capability.
Authority is issued only after that exact approval is satisfied. A binding that
declares `ApplicationWorkflowExecutionRequirements::occurrence_only()` can run
without an approval ([§6.5](#65-who-may-run-a-guarded-operation)).

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Exact original meaning is retained once; construction performs no effect.
let owner = workflow_runtime.request(&principal, &scope)
    .on_branch(instance.branch())
    .mutate(ApprovedBusinessPaymentApplyIntent { input: operation })
    .without_source()
    .preconditions(original_conditions)
    .idempotency_owned(operation_key)?
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

### 6.7 Capture, approve, archive, and observe one workflow

The neutral [observable workflow example](../workspaces/worth-query/crates/worth-query-certification/examples/observable_workflow.rs)
connects the ordinary owners in one application. It captures a document's
retention, checks the retained value, requires a different admitted approver,
and waits for the original archive operation's inbound completion. Use this
shape for a process that combines local data and an external effect.

Run the complete example target:

```sh
cargo run --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification --example observable_workflow
```

The application imports `worth-query-decl` for meaning and `worth-query-host`
for requests, owners and observations. Its [definition](../workspaces/worth-query/crates/worth-query-certification/examples/observable_workflow/definition.rs)
explicitly installs subject, definition, metadata and audit read rules and a
recorded-actor policy. Capture declares occurrence-only requirements; archival
requires the authored approval. The admitted proposal and approval supply their
actors. Business input never supplies an actor override.

#### Keep a basis, reacquire a locator

This smallest call sequence assumes the example's installed vocabulary and read
bindings. Retain the original `prepared.discovery_intent()` before executing the
start. A later request uses that typed data; it does not recompute start identity
from the currently installed program. The receiver owns admission internally:
`retain_current_read()` takes no phase argument. Inside an already admitted host
call, the separate advanced `retain_read(&phase)` receiver remains available.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let request = runtime.request(&principal, &scope).on_branch(branch);
let basis = request.retain_current_read()?;
let found = request.at(&basis)
    .read_original_workflow::<MixedWorkflow, SubjectRead, AuditRead>(
        application.vocabulary(), &original_start, subject_input, audit_input,
    )?;
let target = found.target(); // Option<&WorthQueryWorkflowReadTarget>
```

Match Live, Historical or Missing before choosing what to do. Only Live returns
a usable instance locator; a historical view is data. For an existing result,
clone `observation.target()` or `found.target()` rather than assembling an
identity. Discovery intent, locators, targets and pages retain data custody;
none retains a grant, principal, request, approval need or command permission.
Every read uses fresh current disclosure admission even at an old data basis.
Subject read and audit are independent: withholding actors must not hide an
approval, pending owner or incomplete recovery state.

For current-format restore, retain the typed original start intent as ordinary
intent data and use a new authenticated request against the restored runtime's
own retained basis. Reacquire references there; do not serialize live locators,
reuse an old runtime's retained basis, or interpret a format refusal as Missing.
Unsupported displaced artifacts are refused intact before installation. Logical
migration and native fork continuation remain separate current operations; see
[discovery and observation](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md).

#### Publish a proposal when the owner asks for one

The example starts from the installed definition, then checks the actual
`AwaitingProposal` need before publishing a proposal with an admitted request.
The full principal identity read here is an independent example assertion
against the native receipt, not business input or a source of authority.

<!-- compiled: observable_proposal.rs -->
```rust
    let waiting = advance(&application, &instance, &[10, 11]);
    let WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingProposal(need)) =
        waiting.stop()
    else {
        panic!("the first node must expose its actual proposal need: {waiting:?}");
    };
    assert_eq!(need.node_path(), "propose");
    assert_eq!(
        need.operation(),
        WorkflowDefinitionAuthoringOperation::reference().name()
    );
    assert_eq!(
        need.input_type(),
        WorkflowDefinitionAuthoringOperation::reference()
            .input_identity()
            .as_str()
    );
    assert!(waiting.transitions().is_empty());
    let proposal = runtime
        .request(&actor, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowDefinitionAuthoringIntent { input: input() })
        .without_source()
        .idempotency(&20_u64)
        .prepare_workflow_proposal(&application, instance.clone())
        .unwrap()
        .execute();
    let (proposal, expected_proposer) = match proposal {
        WorkflowProposalOutcome::Published(published) => {
            let principal = published.receipt().principal_scope().principal();
            (
                published.proposal().clone(),
                [
                    u64::from(principal.partition_id()),
                    principal.local_slot(),
                    u64::from(principal.generation()),
                ],
            )
        }
        other => panic!("original proposal publication: {other:?}"),
    };
```

Next it advances to capture, runs the original occurrence-bound owner, and
independently reads retention `9`. The real condition selects the approval
branch. A separately authenticated reviewer answers the original
`RequiredWorkflowApproval` challenge and signs it. Observed actors and history
never stand in for that required need or authentication.

#### Keep the original external operation through settlement and cleanup

The archive transport launches this same executable as a local OS child and
writes the original payload through owned pipes to an actual file. Its installed
verifier authenticates that bounded local channel. This is a local producer,
not a remote cryptographic service or durable crash-recovery demonstration.
The declared inbound operation retains a continuing owner until native ingress.
Before dispatch, the caller drives that same owner while the transport has no child result.
As soon as authenticated child output exists, it admits genuine ingress through
the installed verifier before any further recovery advance. Transport failure is
an error, not an instruction to retry a possibly performed external effect.
Receiving the declared inbound completion is distinct from corrective recovery
or permission to retry the external effect. The archive declares no corrective
aftermath; the caller supplies genuine ingress and leaves settlement to native
inspection of the original dispatch.
At this example's real pending seam, `exercise_pending_custody` is an
example-local assertion phase: close refuses while retaining the owner, a
cancelled fresh advance returns typed cancellation with that owner, and a legal
fresh advance still waits before ingress. It checks the committed head and
actual handler/transport entry counts to reject repeated work. It returns the
original owner; it is not a framework recovery or authority API.
For this declared inbound effect, native `Progress` retains the original owner
until authenticated ingress is admitted. The example requires that pending
custody exercise before accepting the original performed operation under fresh
admission and independently closing cleanup:

<!-- compiled: observable_operation.rs -->
```rust
    let mut operation = request
        .mutate(crate::effect::ArchiveIntent {
            input: crate::journey::input(),
        })
        .without_source()
        .idempotency_owned(80_u64)
        .unwrap_or_else(|(_, _, cause)| panic!("original archive capture: {cause:?}"))
        .for_workflow_operation(application, &required)
        .unwrap_or_else(|_| panic!("exact approved occurrence"))
        .retain_operation::<MixedProgram, crate::effect::ArchiveAction>(
            application.program_runtime(),
        )
        .unwrap_or_else(|_| panic!("installed archive action"));
    let receive = |envelope: &[u8]| {
        application
            .runtime()
            .inbound_occurrences()
            .receive(verifier, envelope, &scope)
            .execute()
            .expect("native provenance admission of the real completion");
    };
    let mut ingress_received = false;
    let completed = loop {
        match operation.advance(
            &application
                .runtime()
                .request(&actor, &scope)
                .on_branch(instance.branch()),
        ) {
            WorthQueryWorkflowOperationAdvance::Progress(next) => {
                operation = next;
                // Progress can precede dispatch. Actual child output, not this
                // variant, determines when ingress must precede further recovery.
                if !ingress_received
                    && let Some(envelope) = archive.envelope().expect(
                        "inspect actual child completion without retrying a failed dispatch",
                    )
                {
                    operation = exercise_pending_custody(operation, application, archive);
                    receive(&envelope);
                    ingress_received = true;
                }
            }
            WorthQueryWorkflowOperationAdvance::Completed(done) => {
                if !ingress_received {
                    let envelope = archive
                        .envelope()
                        .expect("inspect actual synchronous child completion")
                        .expect("native completion must retain genuine authenticated child output");
                    receive(&envelope);
                }
                break done;
            }
            WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
                let _original_owner = operation;
                panic!("archive advance refused with original custody: {denial:?}");
            }
            WorthQueryWorkflowOperationAdvance::Terminal(terminal) => {
                panic!(
                    "archive terminal: source={:?}, recovery={:?}, abort={:?}",
                    terminal.denial(),
                    terminal.recovery_denial(),
                    terminal.original_abort()
                );
            }
        }
    };
    assert!(
        ingress_received,
        "the original owner must exercise pending custody before ingress/completion"
    );
    assert_eq!(archive.dispatch_calls(), 1, "one original child dispatch");
    assert_eq!(
        crate::effect::handler_invocations(),
        [1, 1],
        "one original handler execution"
    );
    let accepted = request
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.into(),
            },
        })
        .without_source()
        .idempotency(&90_u64)
        .prepare_workflow_advance(application, instance.clone())
        .unwrap()
        .accept_owned_operation(completed)
        .unwrap_or_else(|(_owner, cause)| panic!("exact original external settlement: {cause:?}"));
    assert!(
        matches!(&accepted.progress, WorkflowProgressOutcome::Completed(performed) if performed.node_path() == "archive")
    );
    accepted
        .operation
        .close(&request)
        .unwrap_or_else(|_| panic!("original dispatch and native cleanup must close"));
```

A refused advance returns its original owner; retain it before deciding whether
to retry with a fresh request. `Completed` is not permission to abandon native
cleanup. Before ingress, this declared inbound wait mints no corrective recovery
handle or retry authority. Close refuses `NativeObligationRetained` with the same
owner until a fresh advance observes native settlement. This example requires
that wait and closes only after genuine ingress and acceptance.
After settlement the caller advances through `AwaitInbound` to the terminal and
checks that exactly one file contains the original identity and retention.
[Operations and ownership](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations)
explains the other stop postures; callers do not resubmit effects or decode
private receipts to drive their own retry engine.

#### A live mark does not refresh a retained read

The example first reacquires a history target at an old basis. The operator has
subject access; the reviewer independently has audit access. History preserves
proposal and outcome rows while returning `WorthQueryWorkflowHistoryAudit::Withheld`
for the operator. `page(..)` below calls the public
`read_workflow_history::<MixedWorkflow, SubjectRead, AuditRead>` with both inputs,
a width of `16`, `PublishedOnly`, and no prior page. A wider history continues
with the returned page under the same exact basis and selection; each request
rechecks disclosure. No page is authority to advance.

The closure `approve()` performs the actual original approval with fresh reviewer
authentication. Polling uses the existing native envelope subscription. `Changed`
marks relevant changes without altering `old`. The caller explicitly retains
`now`, reacquires a target and requests audit again:

<!-- compiled: observable_history.rs -->
```rust
    let mut subscription = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .at(&old)
        .subscribe_workflow_history::<MixedWorkflow, SubjectRead>(
            application.vocabulary(),
            &target,
            input(),
        )
        .expect("native envelope interest at the old data basis");
    assert_eq!(
        runtime
            .request(&operator, &scope)
            .on_branch(instance.branch())
            .at(&old)
            .poll_workflow_history::<MixedWorkflow, SubjectRead>(
                application.vocabulary(),
                &mut subscription,
                input(),
            )
            .unwrap(),
        WorthQueryWorkflowHistoryMark::Pending
    );
    let expected_approver = approve();
    assert_eq!(
        runtime
            .request(&operator, &scope)
            .on_branch(instance.branch())
            .at(&old)
            .poll_workflow_history::<MixedWorkflow, SubjectRead>(
                application.vocabulary(),
                &mut subscription,
                input(),
            )
            .unwrap(),
        WorthQueryWorkflowHistoryMark::Changed
    );
    // A mark reports relevant change. It does not mutate the old retained read.
    let historical = page(application, &reviewer, &scope, &old, &target);
    assert_eq!(historical.basis().selected_commit(), old.selected_commit());
    assert!(historical
        .entries()
        .iter()
        .all(|row| row.approval().is_none()));
    // Refresh is an explicit caller choice with new current admission.
    let now = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .retain_current_read()
        .expect("explicitly retain a fresh post-approval basis");
    let fresh_discovery = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .at(&now)
        .read_original_workflow::<MixedWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            original,
            input(),
            input(),
        )
        .unwrap();
    let fresh_target = fresh_discovery
        .target()
        .expect("same native instance at fresh data");
    let fresh = page(application, &reviewer, &scope, &now, fresh_target);
    assert_eq!(fresh.basis().selected_commit(), now.selected_commit());
    let approved = fresh
        .entries()
        .iter()
        .find(|row| row.approval().is_some())
        .expect("actual recorded approval");
    assert_eq!(
        approved.approval().unwrap().decision(),
        WorkflowApprovalDecision::Approve
    );
    assert_eq!(
        approved.approval().unwrap().proposal_identity(),
        proposal.identity()
    );
    match approved.audit() {
        WorthQueryWorkflowHistoryAudit::Visible {
            approver: Some(actor),
            ..
        } => {
            let actor = actor.entity_id();
            assert_eq!(
                [
                    actor.partition_value_u64(),
                    actor.local_slot_value(),
                    u64::from(actor.generation_value())
                ],
                expected_approver,
                "the performed native approval is the actor oracle"
            );
            assert_ne!(actor, proposer);
        }
        _ => panic!("independent audit reveals the performed approver"),
    }
    let fresh_withheld = page(application, &operator, &scope, &now, fresh_target);
    assert_eq!(fresh.entries().len(), fresh_withheld.entries().len());
    for (audited, withheld) in fresh.entries().iter().zip(fresh_withheld.entries()) {
        assert_eq!(audited.occurrence(), withheld.occurrence());
        assert_eq!(audited.node_path(), withheld.node_path());
        assert_eq!(audited.outcome(), withheld.outcome());
        assert_eq!(audited.proposal(), withheld.proposal());
    }
    let approved_without_audit = fresh_withheld
        .entries()
        .iter()
        .find(|row| row.approval().is_some())
        .expect("withholding cannot hide approval state");
    assert!(matches!(
        approved_without_audit.audit(),
        WorthQueryWorkflowHistoryAudit::Withheld
    ));
    // Releases native subscription custody only; never operation cleanup proof.
    subscription.close();
```

`subscription.close()` takes no request or execution argument: it releases native
subscription custody. It does not close an operation, settle an inbound effect,
accept workflow evidence or prove cleanup. Cancellation and read/reconstruction
resource refusals remain typed; do not convert them to absent workflow state.

The full example includes installation, authentic principals/grants, policies,
producer verification and assertions outside these excerpts. It covers the
positive mixed path; current restore, malformed/unsupported formats, revoked
reads, fork/migration and cleanup failures have their own native courts. Follow
the [canonical consumer prohibitions](capability-index.md#consumer-prohibitions)
for the shared constraints rather than creating a host actor/status registry.

---

### 6.8 Retain capability commands instead of reissuing them

Delegation and revocation use ordinary request-level command owners, separately
from workflow-bound domain effects. Begin with
`request.retain_capability_delegation_in_program(..)` or
`request.retain_capability_revocation_in_program(..)`. Each captures the original
input, owned key, preconditions and native preparation closure against the
installed program and principal binding. Capture refusal returns those original
values. The [workflow guide's command example](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#owning-capability-commands)
shows the typed arguments; do not substitute a domain mutation handler.

| Returned owner state | Caller responsibility |
| --- | --- |
| `WorthQueryCapabilityCommandAdvance::Progress(command)` | Retain that command. Release any pending managed cleanup through its receiver before the next advancement. Supply a fresh request to `command.advance(&request)`. |
| `Refused { command, denial }` | Keep the returned original command and inspect the typed denial. Correct the request/admission or resolve the native condition before choosing another advancement; never recapture the command as a retry. |
| `Completed(done)` | Inspect `done.evidence()` for the native Fresh/Recovered result. Independently call `done.close_with_evidence(&request)`; success returns the closed operation and original evidence, while refusal returns the same completed owner and denial. |
| `Terminal(terminal)` | Stop effect advancement. Inspect the native outcome/idempotency resolution, and independently release cleanup or close the returned terminal owner. Terminality does not authorize resubmission. |

A later request must still match runtime, principal and command meaning and
pass fresh current authorization. Deferred publication and ambiguous commits
recover the exact original native result through this owner. Do not interpret
native replay receipts in a consumer retry loop. Command cleanup, workflow
operation cleanup and history subscription release have distinct receivers.

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

The runtime privately checks every current definition and live instance before
adoption. That complete safety inventory is not a consumer read surface.
For disclosure, select one typed definition with `read_workflow_instances`,
its installed Definition read binding and fresh principal/input. A metadata
read does not grant instance enumeration.

Submit explicit choices for opaque references already possessed, or request
native carry-compatible handling of every fact found by the fresh safety check:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
use worth_query_host::facade::primary_graph::{
    WorthQueryWorkflowDispositions, WorthQueryWorkflowDefinitionDisposition,
};
let choices = WorthQueryWorkflowDispositions::new()
    .definition(&definition, WorthQueryWorkflowDefinitionDisposition::Retire);
let prepared = programs.adopt(&requirements)?.workflow(choices).prepare(1_024)?;
// When every fresh fact should be carried, use instead:
let choices = WorthQueryWorkflowDispositions::carry_compatible();
```

Explicit choices must totally and legally decide the fresh native inventory.
An omitted live instance refuses; retiring a definition does not cancel its
instances. A performed effect or pending external-owner/approval obligation
cannot be cancelled away. The existing source-program adoption authority is
still required. References and read views grant no adoption permission.

`WorkflowDispositionRequired` and `WorkflowDispositionRejected` carry no
inventory or occurrence identities. Native candidate preparation logs also stay
private; execution cancellation/resource causes retain their typed category. A new start can invalidate explicit
choices; take a fresh governed view and decide again. A start after preparation
is still fenced by the native expected head at publication. The carry-compatible
policy explicitly applies to fresh native truth; it is not a cached inventory.

### 7.5 Several branches

<!-- illustrative -->
```rust,ignore
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
- Run fixed, discovered and source-only actions through `idempotency_owned(key)` and
  `retain_operation::<Program, Action>(&runtime)`, then supply fresh ordinary
  requests for advancement, delivery and close. Native admission checks the
  original runtime, branch and full action meaning on every attempt.
- Retain guarded workflow effects with their original issued requirement; use a
  separate owning assessment demand for observation. Neither replaces evidence
  acceptance or approval. Fixed, discovered and source-only actions use their
  declared ordinary owners; do not add a consumer retry engine.
- Match the owning lane's `Progress`, `Refused`, `Terminal`, `Settled` or `Completed`
  variants, retaining each returned owner. `NoEffect`, `ProductUnpublished` and
  `Commit(..)` are native advanced outcomes; workflow `Awaiting*` progress is
  distinct from business-effect completion. Do not automatically retry a stop.
- `compare` before `adopt`, and pass the requirements you got.
- Name the expected predecessor when you publish a workflow definition.
- Pump workflows yourself: one advance per request, until an advance returns
  `Completed(t)` with `t.terminal() == true`. `Completed` alone means one step
  was recorded.

Consumer constraints are maintained in the
[canonical consumer prohibitions](capability-index.md#consumer-prohibitions).


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
