# Build an Application

> **The application story, end to end:** declare the program and its
> features, install the application graph, run it, author and run workflows,
> and adopt a new program on a branch. Every code block here is real code
> from this repository, or is marked as a shape with a pointer to its compiled
> counterpart.

This is the centerpiece guide for building on WORTH. Other public documents
explain *why* the platform works the way it does
([Philosophy](philosophy.md)) and *what* each runtime guarantees
([How WORTH Works](how-it-works.md)). This one shows *which names you call, in
which order*.

An application imports exactly two crates, `worth-query-decl` and
`worth-query-host`. Their facades are made of modules, so import from the
module that owns each name:

```rust
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
| `…host::facade::application_installation` | `in_memory` (no program), `in_memory_program`, `in_memory_rostered_program`, `WorthQueryInMemoryApplicationLimits`, `WorthQueryProgramApplicationRuntime`, `WorthQueryProgramOwner` (a trait: import it to call `owned_revision()`) |
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
- [9. Names that do not exist](#9-names-that-do-not-exist)
- [10. Where to go deeper](#10-where-to-go-deeper)

---

## 0. The whole journey on one screen

| Step | What you do | The call | Section |
|---|---|---|---|
| 1 | Declare the schema and its contributions | `worth_query_application!`, `worth_query_application_contribution!`, `impl WorthQueryApplicationContribution` | [§2](#2-declare-the-schema-and-its-contributions) |
| 2 | Declare features and the program | `impl ApplicationFeature`, `ApplicationFeatureSpec::root::<S, F>()…finish()`, `impl ApplicationProgramDefinition` | [§3](#3-declare-features-and-the-program) |
| 3 | Validate the program | `ApplicationProgramAuthoring::<S, P>::begin().validated_program()` | [§3.5](#35-validate) |
| 4 | Install the application graph | `application_installation::in_memory_program(..)` or `in_memory_rostered_program(..)` | [§4](#4-install-the-application-graph) |
| 5 | Run program-owned actions | `request(&principal, &scope).mutate(intent).without_source().idempotency(&key).execute_in_program(&runtime)` | [§5](#5-run-the-program) |
| 6 | Install the workflow vocabulary | `WorthQueryApplicationWorkflowSpecInstallation::begin(..)…finish()`, then `retain_workflow_spec(..)` | [§6.2](#62-install-the-workflow-vocabulary-once) |
| 7 | Author and validate a workflow | `ApplicationWorkflowDefinitionBuilder::<Spec>::new(..)`, nodes, connections, `finish()?.validate()?` | [§6.3](#63-author-a-definition) |
| 8 | Publish, start, and advance it | `bind_definition`, `prepare_workflow_publication`, `prepare_workflow_instance_start`, `prepare_workflow_advance` | [§6.5](#65-publish-discover-start-advance) |
| 9 | Adopt a new program on one branch | `.on_branch(b).programs()`, `compare(&target)`, `adopt(&requirements)`, `prepare(max)`, `publish()` | [§7](#7-adopt-a-new-program-on-a-branch) |

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

```rust
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

```rust
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

```rust
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

```rust
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

```rust
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

```rust
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

```rust
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

---

## 4. Install the application graph

Installation turns a validated program, a schema declaration, the
contributions' configuration, and resource limits into a running runtime. The
`initial_state` closure seeds the primary graph before the first publication.

### 4.1 One program

```rust
// workspaces/worth-query/crates/worth-query-certification/examples/product_workflow_support/application.rs
use worth_query_host::facade::{application_installation, primary_graph, runtime};

let declaration = TemporalHostSchema::declaration().expect("the example schema is valid");
let runtime = application_installation::in_memory_program(
    program::validated_program(),                  // ValidatedApplicationProgram<S, P>
    declaration,                                   // ApplicationSchemaDeclaration<S>
    (TemporalContributionConfiguration { clock_source },), // one entry per contribution
    example_limits(),                              // WorthQueryInMemoryApplicationLimits
    |graph, installed| {                           // seed the primary graph
        let principal_binding = installed
            .principal_binding(TemporalPrincipalBinding::reference())
            .expect("the temporal principal binding must install");
        seed_graph(graph, &principal_binding, gate);
        Ok(())
    },
)
.expect("the validated temporal program must install");
```

```rust
// Same file: limits.
application_installation::WorthQueryInMemoryApplicationLimits::new(
    product_world_resources(),
    runtime::WorthQueryApplicationCandidateResourceProfile::bounded(5_120, 2_048, 5_120)
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

```rust
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
    initialize,            // |graph, installed| { ... Ok(()) }
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

```rust
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

```rust
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

A **request scope** carries only a deadline and a cancellation token. It is
not an entity and grants nothing:

```rust
// .../product_workflow_support/adapters.rs
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

let cancellation = WorthQueryCancellationSource::new();
let scope = WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(60), cancellation.token());
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

```rust
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

On a runtime installed with a program (`in_memory_program` or
`in_memory_rostered_program`), **every** mutation runs through the program.
Plain `execute()` refuses it with
`WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired`.
Plain `execute()` is only for a runtime installed without a program
(`application_installation::in_memory`).

```rust
// workspaces/worth-query/crates/worth-query-certification/examples/ordinary_product_workflow.rs
let request = application.runtime.request(&principal, &scope);
let outcome = request
    .mutate(AmendTemporalIntent {
        identity: "intent-1".to_owned(),
        amendment,
    })
    .without_source()
    .idempotency(&0x51_u64)
    .execute_in_program(&application.runtime)
    .expect("the installed program must prepare the application mutation");
let (receipt, result) = match outcome {
    WorthQueryApplicationMutationOutcome::Committed { receipt, result } => (receipt, result),
    other => panic!("the ordinary application mutation must commit: {other:?}"),
};
```

- **A mutation never names its program.** `execute_in_program` reads which
  program the request's branch runs and commits under it. If an adoption lands
  between selection and commit, the commit is denied before any effect.
- Passing a runtime other than the one the request came from (a different
  runtime instance, compared by identity) is `ApplicationProgramMismatch`.
- `.on_branch(branch)` selects a branch other than the current world. A branch
  token grants nothing; it is re-selected on every use.
- Match every outcome variant. `Commit(..)` is never an error string: see
  [How WORTH Works §11](how-it-works.md#11-outcomes-every-way-a-request-can-end).

Run it:

```bash
cargo run --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification --example ordinary_product_workflow
```

### 5.4 Mutations with required outputs

When a mutation's source feeds the program's output graph, execute it as a
*performed* mutation and settle its required outputs. Shape, with
illustrative names; the compiled counterpart is
`.../fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/settlement.rs`:

```rust
// Shape. Illustrative names.
let outcome = request
    .mutate(AdjustDraft { /* ... */ })
    .expect_source(source)                 // or .without_source()
    .idempotency(&key)
    .execute_performed::<MyProgram, MyOutputRoot>(&runtime)?;
let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
    /* RequiredOutputDenied { .. } or NotPerformed(..): handle it */
};
let mut performed = performed.start_required_outputs(&request, controls)?;
performed.required_output_mut().advance(&request)?; // Pending, later Settled(..)
```

`controls` is `WorthQueryOutputDemandControls::new(NonZeroUsize, NonZeroUsize)`.
Output progress is `WorthQueryApplicationProgramOutputProgress::{Pending, Settled}`.

---

## 6. Author, publish, and run workflows

A workflow is a branch-local, versioned graph of governed steps. The host
installs a **vocabulary** once: which operations, assessments, conditions,
and approvals a workflow may use, and which capabilities control it. After
that, definitions are *data*: authored, validated, published on a branch,
discovered, started, and advanced at run time.

Every workflow action is a governed mutation. There is no background
scheduler, callback, or inbound-completion API: **the caller pumps progress**,
one request per step.

### 6.1 The spec marker

```rust
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

```rust
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

```rust
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
255 lines.

### 6.3 Author a definition

`ApplicationWorkflowDefinitionBuilder<Spec>` creates typed nodes, then
connects them. This is Bank's approved-payment workflow, complete:

```rust
// workspaces/worth-query-bank-world/crates/bank-server/src/application_definition/workflows.rs
use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowComponentLimits, ApplicationWorkflowConditionOperands,
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
    ApplicationWorkflowDefinitionLimits, ApplicationWorkflowEvidenceJoinPolicy,
    ValidatedWorkflowDefinition,
};

let mut builder = ApplicationWorkflowDefinitionBuilder::<ApprovedBusinessPaymentWorkflow>::new(
    "approved-business-payment",
    ApplicationWorkflowDefinitionLimits::new(
        9,                                                   // nodes
        20,                                                  // connections
        2,                                                   // effects
        ApplicationWorkflowComponentLimits::new(8, 2, 16, 32, 32).unwrap(),
        8 * 1_024,                                           // canonical bytes
    )
    .expect("approved-payment limits are nonzero"),
)
.expect("approved-payment identity is valid");

// Nodes. Each call returns a typed node reference.
let propose = builder.operation::<ApprovedBusinessPaymentAuthoringOperation>("propose", false)?;
let payment = builder.assessment::<PaymentDetailQuery>("review/payment")?;
let independent = builder.assessment::<PaymentDetailQuery>("review/independent")?;
let evidence = builder.evidence_join(
    "review/evidence",
    ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
)?;
let limit = builder.condition(
    "approval/limit",
    "amount_cents <= 15000",
    ApplicationWorkflowConditionOperands::new().query::<PaymentAmountQuery>("amount_cents"),
)?;
let approval = builder.approval::<ApprovedBusinessPaymentApproval>("approval")?;
let apply = builder.operation_binding::<ApprovePaymentMutationBinding>("apply")?;
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
    .control(&apply, ApplicationWorkflowControlOutcome::Completed, &completed)
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

let definition: ValidatedWorkflowDefinition<ApprovedBusinessPaymentWorkflow> =
    builder.finish()?.validate()?;
```

**Node kinds.** There are exactly six (`ApplicationWorkflowNodeKind`):

| Builder method | Node | Notes |
|---|---|---|
| `operation::<Op>(id, requires_workflow_authority)` / `operation_binding::<Binding>(id)` | Operation | A governed mutation. `operation_binding` takes the authority flag from the binding. |
| `assessment::<Query>(id)`, `assessment_for`, `assessment_when_related_relation_present` | Assessment | A query whose settled result becomes evidence |
| `condition(id, source, operands)` | Condition | A Bool expression over named, typed query results |
| `approval::<Capability>(id)` | Approval | A signed human decision under a capability |
| `evidence_join(id, policy)` | Evidence join | `AllRequiredPassing` or `AllRequiredCompleted` |
| `terminal(id)` | Terminal | An end state |

There is **no** query, read, branch, or retry node. A read that feeds a
decision is an assessment or a condition. Branching is routing different
`ApplicationWorkflowControlOutcome`s. A retry is a connection:

```rust
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

   ```rust
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
`ControlCycle`, `MissingTerminal`, `MissingWorkflowAuthority`, and
`IncompleteApproval`. A validated definition carries no branch, publication
status, or authority to start anything.

### 6.4 Fixed and user-authored workflows are the same thing

There is no separate "fixed workflow" construct. Bank's approved-payment
workflow is written in Rust and ships with the application, but it is
published at run time through the same kernel as a workflow a user authors
in a UI. What is static is the spec marker, the installed vocabulary, and the
control bindings. What is data is every definition, instance, proposal,
piece of evidence, and approval.

### 6.5 Publish, discover, start, advance

Every lifecycle call is a method on the ordinary mutation request chain. The
mutation intent is the matching control intent (authoring, instance start,
advance, or approval), and the workflow runtime from §6.2 is passed in.

`workflow_runtime` dereferences to the program runtime, so
`workflow_runtime.request(&principal, &scope)` is the request entry for every
call below. (Bank wraps it in its own `runtime.request(..)`, which maps its
principal type and then makes exactly this call.)

**Bind and publish a definition** on a branch. Publication names the
predecessor it expects to replace; there is no last-writer-wins overwrite.

```rust
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

```rust
// examples/authored_workflow/main.rs
application
    .runtime()
    .on_branch(application.runtime().current_world())
    .select()
    .expect("the main branch selects its exact occurrence")
    .discover_workflow_definition::<ReviewedDocumentWorkflow>(identity)
    .expect("the workflow lineage reads within its bound")
// -> WorthQueryWorkflowDefinitionDiscovery::{Current(ref), Retired, Unpublished}
```

**Start** an instance, **propose**, and **advance**:

```rust
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
| `AwaitingOperation` | The guarded effect must run |
| `Application`, `PreparationDenied`, `AuthenticationDenied`, `IdempotencyDenied` | No step was recorded; each says why |
| `ProjectionDenied` | The commit landed and carries its receipt; only its projection was refused |

**Assessments and conditions.** Turn an advance into an assessment demand
with `.into_assessment_demand(demand)`, settle it, then accept it with
`.accept_assessment(&settlement)`. A condition is accepted by naming each
operand's published result:
`.condition(&required).operand::<Binding, _>("name", result).accept()`.

**Approve or reject.** An approval needs a fresh authentication event and a
signature. Replaying a recorded key is the only exception.

```rust
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

**Run the guarded effect** under the approved requirement:

```rust
// bank-server/src/approved_payment_workflow.rs
workflow_runtime
    .request(&principal, &scope)
    .on_branch(instance.branch())
    .mutate(ApprovedBusinessPaymentApplyIntent { input: operation })
    .idempotency(&operation_key)
    // `required` is the RequiredWorkflowOperation from
    // WorkflowProgressOutcome::AwaitingOperation(required).
    .for_workflow_operation(&workflow_runtime, &required)?
    .execute_in_program(workflow_runtime.program_runtime())?
```

**Everything else in the lifecycle:**

| Call | Does |
|---|---|
| `prepare_workflow_navigate_back` | Records `NavigatedBack` |
| `prepare_workflow_instance_cancellation` | Ends an instance where it stands. Not a rollback: the outcome names every performed step. |
| `prepare_workflow_instance_migration(workflow, instance, target, resume_at)` | Continues an instance on a later definition from a named node |
| `prepare_workflow_fork_continuation` | Continues an instance on a forked branch |
| `prepare_workflow_definition_retirement` | Retires a definition lineage |
| `accept_operation_from_owner`, `for_workflow_operation_recovery` | Owner receipts and recovery for external effects |
| `mutate(i).workflow(workflow, authored)…`, `start_workflow`, `run_workflow` | The ordinary convenience lane, which validates, binds, publishes, starts, and runs in fewer calls |

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

Run the full authored-workflow journey:

```bash
cargo run --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification --example authored_workflow
```

---

## 7. Adopt a new program on a branch

Adoption moves **one exact branch** from the program it runs to a rostered
target revision, in one product publication. Other branches, including
earlier forks, keep their programs.

### 7.1 The call chain

```rust
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
    .prepare(64)                                   // maximum selection work
    .expect("the seed satisfies P1 and adoption must prepare");
match prepared.publish() {
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
| Adopt | `.adopt(&requirements)` | A preparation request |
| Optional | `.migration(prepared_migration)`, `.workflow(dispositions)` | The same request |
| Prepare | `.prepare(maximum_selection_work)` | `WorthQueryPreparedBranchAdoption` (move-only, fields private) |
| Publish | `prepared.publish()` | `WorthQueryBranchAdoptionPublicationOutcome` |
| Recover | `programs().recover(unpublished.into_recovery())` | `Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryApplicationProgramAdoptionRecoveryFailure>` |

`compare` comes first because the requirements are the input to `adopt`.
`WorthQueryProgramAdoptionRequirements` exposes `semantic_diff()`,
`validation_scopes()`, `added_rules()`, and the predicates
`requires_existing_state_validation()`, `requires_migration_assessment()`,
`requires_custody_inventory()`, and `semantically_equivalent()`.

### 7.2 What each outcome means

| Situation | What you get |
|---|---|
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
other variant, `ProductSelection(..)`, means the branch could not be selected.

A writer still running under the old program after adoption lands gets a
stale commit outcome, never a silent commit under the wrong law.

### 7.3 Migrations

When the target needs existing state repaired, prepare the migration as a
candidate under the target program, then hand it to the adoption:

```rust
// tests/application_graph/adoption/migration.rs (abridged)
let migration = match host
    .runtime()
    .request(&principal, &scope)
    .on_branch(branch)
    .mutate(SetRetentionIntent { input })
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
    .adopt(&requirements)
    .migration(migration)
    .prepare(64)?;       // migration and validation prepare atomically
```

A prepared migration commits nothing and registers no idempotency record
until the adoption publishes.

### 7.4 Workflows and custody during adoption

Every workflow definition and instance on the branch needs a legal
disposition. Ask for the inventory, decide each item, and pass the decisions:

```rust
// tests/application_graph/adoption/workflow_custody.rs (abridged)
let adoption = programs.adopt(&requirements);
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

```rust
let prepared = request
    .on_branches(coverage, &ordered_targets)?     // WorthQueryProgramAdoptionCoverageDenial
    .programs()
    .adopt(&target)
    .prepare(maximum_selection_work_per_branch)?; // WorthQueryPreparedBranchSetAdoption
```

This prepares adoption across a set of branches. It is **not atomic**:

- Preparation preflights every branch before any branch publishes. One
  incompatible branch denies the whole preparation and names that branch.
- `prepared.advance()` publishes the next branch and returns its progress
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
- On a runtime installed with a program, execute **every** mutation through
  a program lane: `execute_in_program`, `execute_capability_in_program`,
  `execute_retained_in_program`, or `execute_performed::<Program, Root>`.
  Pass the program runtime the request came from (for a workflow runtime,
  `workflow_runtime.program_runtime()`); another one is refused with
  `ApplicationProgramMismatch`.
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
- Do not call plain `execute()` on a runtime installed with a program: every
  mutation there is refused with `ApplicationProgramRequired`. Plain
  `execute()` is only for a runtime installed with `in_memory` (no program).
- Do not name a program on a mutation. The branch decides.
- Do not expect adoption to change sibling branches, or a branch set to adopt
  atomically.
- Do not register a handler for a workflow control binding.
- Do not wait for a callback, timer, or background scheduler. There is none.
- Do not treat a validated program or validated workflow definition as
  permission to do anything.
- Do not treat names found only in `plans/` as real. Check §9.

---

## 9. Names that do not exist

The milestone plans for this work (`plans/WORTH-query/milestone-9.17.4.md`,
`-9.17.5.md`, and `-9.17.6.md`) describe intent, and some of their names never
shipped. If you see one of these, use the real name instead.

| Plan name | Status | Use instead |
|---|---|---|
| `ApplicationCompositionSpec`, `ApplicationModuleSpec` | Do not exist | `impl ApplicationProgramDefinition` plus `ApplicationFeatureSpec` |
| `ApplicationActionSpec` | Does not exist | Builder methods on `ApplicationFeatureSpec` (`mutation`, `operation`, ...); `ApplicationActionDeclaration` is the read-back type |
| `.adopt(revision).requirements(..).idempotency(..).controls(..)` | Not the API | `compare(&revision)` → `adopt(&requirements)` → `[.migration(..)] [.workflow(..)]` → `prepare(max)` |
| "unchanged / additive / restricting / meaning-changing" diff classes | Not the API | `ApplicationSemanticChangeKind::{Added, Removed, Changed}` per family |
| `PublishedWorkflowDefinition` | Does not exist | `PublishedWorkflowDefinitionRef` |
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

These four Query guides live under the `worth-query` crate's `docs/`
directory for historical reasons. They are **consumer guides**: every name
they use is on the two facades.
