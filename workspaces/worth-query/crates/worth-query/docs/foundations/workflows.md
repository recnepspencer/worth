# Workflows

This guide explains the Query workflow kernel for application code that uses the
audience facades `worth-query-decl` and `worth-query-host`. Every name below is
a public name reachable through those facades unless the guide says otherwise.

## In one paragraph

A workflow is branch-local, versioned data that describes a bounded graph of
steps: operations, assessments, conditions, approvals, evidence joins, and
terminals. An application declares a workflow vocabulary (`ApplicationWorkflowSpec`),
installs it against one installed program, authors a definition with
`ApplicationWorkflowDefinitionBuilder`, binds it to the installed vocabulary,
and publishes it on one branch with an explicit expected predecessor. An
instance is one run of one pinned definition revision on one branch. Nothing
moves on its own: the caller pumps every step with an ordinary Query request,
and each request returns a typed outcome that says what the instance now needs.
The workflow kernel authorizes and records its own control steps. It enforces
step, evidence, live-instance, and deadline budgets, and it keeps performed
effects when an instance is canceled, migrated, or continued on a fork.

## Contents

1. [Terms](#terms)
2. [Where the names live](#where-the-names-live)
3. [Lifecycle at a glance](#lifecycle-at-a-glance)
4. [Declare a workflow vocabulary](#declare-a-workflow-vocabulary)
5. [Control steps and `WORKFLOW_CONTROL`](#control-steps-and-workflow_control)
6. [Install the vocabulary](#install-the-vocabulary)
7. [Give the runtime its vocabulary per program](#give-the-runtime-its-vocabulary-per-program)
8. [Node kinds](#node-kinds)
9. [Author a definition](#author-a-definition)
10. [Bind and publish a definition](#bind-and-publish-a-definition)
11. [Discover and retire definitions](#discover-and-retire-definitions)
12. [Start an instance](#start-an-instance)
13. [Propose and advance](#propose-and-advance)
14. [Assessments](#assessments)
15. [Conditions](#conditions)
16. [Approvals](#approvals)
17. [Operations](#operations)
18. [Navigate back](#navigate-back)
19. [The ordinary convenience lane](#the-ordinary-convenience-lane)
20. [Caller-pumped progression](#caller-pumped-progression)
21. [Cancellation](#cancellation)
22. [Owner custody](#owner-custody)
23. [Budgets and deadlines](#budgets-and-deadlines)
24. [Observations and notifications](#observations-and-notifications)
25. [Migration and fork continuation](#migration-and-fork-continuation)
26. [Portable definition drafts (WQWD)](#portable-definition-drafts-wqwd)
27. [Denials](#denials)
28. [You can](#you-can)
29. [You cannot](#you-cannot)
30. [Misconceptions](#misconceptions)
31. [Worked example: purchase-request approval](#worked-example-purchase-request-approval)
32. [Related documents](#related-documents)

## Terms

| Term | Meaning |
|---|---|
| Workflow spec | A pure marker type that implements `ApplicationWorkflowSpec`. It names a workflow vocabulary and its schema. Declaring it installs and authorizes nothing. |
| Vocabulary | The operations, assessments, conditions, approvals, and control capabilities a spec may use, installed against one installed program. |
| Definition | A validated, bound, published workflow graph with an identity (`ApplicationWorkflowDefinitionIdentity`). It lives on one branch. |
| Lineage (definition lineage) | The chain of definition revisions that share one identity. A new revision names its predecessor. Live-instance budgets and retirement apply per definition lineage. |
| Instance | One run of one pinned definition revision on one branch. |
| Instance lineage | An instance together with the instances it was migrated or continued from. The step and evidence budgets and the deadline belong to the instance lineage, so a successor inherits what its sources spent. |
| Node | One step in a definition. Each node has a kind (see [Node kinds](#node-kinds)). |
| Node path | The string that names a node inside a definition, for example `approval` or `checks/budget`. Nodes inside an expanded component are prefixed with the component occurrence. |
| Transition | One recorded step of an instance: the node it settled and the control outcome it took. `PerformedWorkflowTransition` is its public receipt. |
| Control step | A workflow action that the kernel authorizes and records itself, with no application handler: publishing or retiring a definition, starting, canceling, migrating, or continuing an instance, advancing, and approving. Its binding sets `WORKFLOW_CONTROL`. See [Control steps](#control-steps-and-workflow_control). |
| Pump | One caller request that asks the kernel to take the next step. |
| Workflow kernel | The part of Query that authorizes, adjudicates, and records workflow steps. Application code reaches it only through the workflow runtime. |
| Workflow runtime | `WorthQueryWorkflowApplicationRuntime`: the host value that carries the installed vocabularies. Instance requests take it (`&workflow` in the examples); definition publication does not. |
| Owner custody | The state of an external operation that committed but whose settlement the owner has not yet accepted. |

Two older `worth-query` modules also use the word *workflow*, and neither is
this kernel. `worth_query::facade::workflow` declares preview, promotion,
writeback, and branch merge. `worth_query::facade::installed::workflow`
traces staged runs of installed domain operations and compares their replays.
The audience facades re-export neither, and neither can start, advance, or
read a workflow instance.

## Where the names live

| Facade module | What it holds |
|---|---|
| `worth_query_decl::facade::application_program` and `worth_query_host::facade::declaration::application_program` | `ApplicationWorkflowSpec`, builders, node kinds, control outcomes, limits, retry |
| `worth_query_decl::facade` and `worth_query_host::facade` (crate root) | The `worth_query_workflow!` authoring macro |
| `worth_query_host::facade::domain` | `WorthQueryApplicationWorkflowSpecInstallation`, `WorthQueryApplicationWorkflowResourceCeiling`, `WorthQueryInstalledApplicationWorkflowSpec`, `WorthQueryInstalledWorkflowDefinitionContract`, installation denials |
| `worth_query_host::facade::application_installation` | `WorthQueryProgramApplicationRuntime`, `WorthQueryWorkflowApplicationRuntime`, `WorthQueryWorkflowRuntimeBindingDenial`, `WorthQueryWorkflowVocabulary` |
| `worth_query_host::facade::application_discovery` | `WorthQueryWorkflowDefinitionDiscovery` |
| `worth_query_host::facade::application_entry` | Every workflow request, outcome, and preparation denial, and every `Required*` value an outcome carries (`RequiredWorkflowActor`, `RequiredWorkflowApproval`, and the rest) |
| `worth_query_host::facade::primary_graph` | `WorthQueryApplicationAttemptDenialKind`, adoption inventory types |

## Lifecycle at a glance

| Phase | Who acts | Public entry point | Result |
|---|---|---|---|
| Declare | Application author | `impl ApplicationWorkflowSpec` | A marker type |
| Install | Host setup | `WorthQueryApplicationWorkflowSpecInstallation::begin(..)...finish()` | `WorthQueryInstalledApplicationWorkflowSpec` |
| Retain | Host setup | `retain_workflow_spec`, then `support_workflow_spec` per extra program | `WorthQueryWorkflowApplicationRuntime` |
| Author | Application code | `ApplicationWorkflowDefinitionBuilder` | `AuthoredWorkflowDefinition`, then `ValidatedWorkflowDefinition` |
| Bind | Application code | `bind_definition` | `WorthQueryInstalledWorkflowDefinitionContract` |
| Publish | Caller request | `prepare_workflow_publication` | `WorkflowDefinitionPublicationOutcome` |
| Discover | Caller | `discover_workflow_definition` | `WorthQueryWorkflowDefinitionDiscovery` |
| Start | Caller request | `prepare_workflow_instance_start` | `WorkflowInstanceStartOutcome` |
| Propose | Caller request | `prepare_workflow_proposal` | `WorkflowProposalOutcome` |
| Advance | Caller request | `prepare_workflow_advance` | `WorkflowProgressOutcome` |
| Settle a need | Caller request | assessment, condition, approval, or operation lanes | `WorkflowProgressOutcome` or a mutation outcome |
| End | Caller request | terminal node, `prepare_workflow_instance_cancellation`, or migration | Terminal transition, cancellation, or successor |
| Retire a lineage | Caller request | `prepare_workflow_definition_retirement` | `WorkflowDefinitionRetirementOutcome` |

## Declare a workflow vocabulary

A workflow spec is a marker type:

```rust,ignore
pub struct PurchaseWorkflowSpec;

impl ApplicationWorkflowSpec for PurchaseWorkflowSpec {
    type Schema = PurchaseSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("example.purchase-workflow.v1");
}
```

- `ApplicationWorkflowSpec` requires `Sized + 'static`, an associated `Schema: ApplicationSchema`, and a constant `IDENTITY`.
- The marker grants nothing. Only installation (next sections) makes members usable.
- Operations, queries, and capabilities that the vocabulary will use are ordinary application declarations of the same schema.

## Control steps and `WORKFLOW_CONTROL`

Every workflow action is a declared application mutation intent. Its binding
(`ApplicationMutationBinding`) has three boolean constants that matter here:

| Constant | Default | Meaning when `true` |
|---|---|---|
| `REQUIRES_APPLICATION_PROGRAM` | `false` | The mutation must commit under the program the branch runs. |
| `REQUIRES_WORKFLOW_AUTHORITY` | `false` | The mutation is a guarded effect. It runs only with authority issued by a workflow operation step. |
| `WORKFLOW_CONTROL` | `false` | The mutation is a workflow control step (see the table below). |

Each control step is authorized by one installed control capability:

| Control step | Request method | Authorizing capability |
|---|---|---|
| Publish or retire a definition | `prepare_workflow_publication`, `prepare_workflow_definition_retirement` | `authoring_capability` |
| Start, cancel, migrate, or continue an instance on a fork | `prepare_workflow_instance_start`, `prepare_workflow_instance_cancellation`, `prepare_workflow_instance_migration`, `prepare_workflow_fork_continuation` | `instance_start_capability` |
| Advance, navigate back | `prepare_workflow_advance`, `prepare_workflow_navigate_back` | `advance_capability` |
| Approve or reject | `prepare_workflow_approval` | the approval's own capability (`.approval::<Capability, ..>()`) |

Why no handler serves a control step:

- The kernel authorizes and records a control step itself. The step's meaning is the kernel's transition, not application code.
- Installing a mutation handler for a `WORKFLOW_CONTROL` binding is refused.
- The ordinary mutation lane refuses a `WORKFLOW_CONTROL` binding with `WorthQueryApplicationRequestMutationDenial::Handler(MutationHandlerExecutionDenial::WorkflowControl)`. Control intents reach the kernel only through the workflow request methods (`prepare_workflow_publication`, `prepare_workflow_instance_start`, `prepare_workflow_advance`, `prepare_workflow_approval`, and the rest).

A guarded effect (`REQUIRES_WORKFLOW_AUTHORITY = true`) is different. It has an
ordinary handler, but it commits only when the request carries authority from
the workflow step that requires it. See [Operations](#operations).

## Install the vocabulary

Install the spec against one installed schema and one installed program:

```rust,ignore
let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
    32,                  // maximum_definition_nodes
    64,                  // maximum_definition_connections
    4,                   // maximum_definition_effects
    ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
    64 * 1024,           // maximum_canonical_bytes
    32,                  // maximum_live_instances (per definition lineage)
    128,                 // maximum_retained_transitions_per_instance (per instance lineage)
    256 * 1024,          // maximum_evidence_bytes (per instance lineage)
)
.expect("every ceiling is nonzero");

let installed = WorthQueryApplicationWorkflowSpecInstallation::<
    PurchaseSchema,
    PurchaseWorkflowSpec,
    PurchaseProgramV1,
>::begin(schema, program, resources)
.operation::<DraftRequestBinding>()?
.operation::<PlaceOrderBinding>()?
.assessment::<BudgetAvailableBinding>()?
.assessment::<VendorApprovedBinding>()?
.approval::<ManagerApprovalCapability, AdvanceOperation, AdvanceInput>()?
.authoring_capability::<AuthoringCapability, AuthoringOperation, AuthoringInput>()?
.instance_start_capability::<StartCapability, StartOperation, StartInput>()?
.advance_capability::<AdvanceCapability, AdvanceOperation, AdvanceInput>()?
.finish()?;
```

- `begin(schema, program, resources)` names the installed schema, the installed program, and a `WorthQueryApplicationWorkflowResourceCeiling`.
- `.operation::<Binding>()`, `.assessment::<Binding>()`, and `.condition::<Binding>()` add members. Each member must already be installed in the program.
- `.approval::<Capability, Operation, Input>()` adds an approval capability.
- `.authoring_capability`, `.instance_start_capability`, and `.advance_capability` are required. Without them, `finish()` refuses.
- `finish()` returns `WorthQueryInstalledApplicationWorkflowSpec`. It exposes `schema_binding()`, `program_revision()`, `resources()`, and `bind_definition(..)`.
- `WorthQueryApplicationWorkflowResourceCeiling::new` returns `None` when a ceiling is zero. `with_history_reconstruction_budget(..)` sets how much retained history a request may reconstruct (`WorthQueryWorkflowHistoryReconstructionBudget::standard()` is the default budget).

Installation refusals are `WorthQueryApplicationWorkflowInstallationDenial`,
read through `kind()` and `subject()`:

| Kind | Cause |
|---|---|
| `OperationNotInProgram` | A member's or control capability's operation is not in the named program. |
| `OperationNotInstalled`, `AssessmentNotInstalled`, `ApprovalNotInstalled` | The member was not installed. A condition is an installed query like an assessment, so an uninstalled condition member is also `AssessmentNotInstalled`. |
| `AuthoringCapabilityNotInstalled`, `InstanceStartCapabilityNotInstalled`, `AdvanceCapabilityNotInstalled` | The control capability was not installed. |
| `MissingAuthoringCapability`, `MissingInstanceStartCapability`, `MissingAdvanceCapability` | `finish()` ran without that control capability. |
| `DuplicateVocabularyMember` | A member was added twice. |
| `EmptyVocabulary` | No member was added. |
| `DefinitionLimitExceeded` | A definition passed to `bind_definition` exceeds the installed ceiling. |
| `UnsupportedDefinitionMember` | A definition names a member this vocabulary does not hold. |
| `ForeignSchemaBinding` | The schema passed to `begin` is not the program's schema. |

## Give the runtime its vocabulary per program

A workflow request never names its program. The runtime reads the program the
selected branch runs and uses the vocabulary installed for that program.

- `WorthQueryProgramApplicationRuntime::retain_workflow_spec(installed, signing_owner)` consumes the program runtime and returns `WorthQueryWorkflowApplicationRuntime`. The second argument is the signing owner of the installed authentication event owner (`WorthQueryInstalledAuthenticationEventOwner::signing_owner`). Approvals are signed through it.
- `WorthQueryWorkflowApplicationRuntime::support_workflow_spec(installed)` adds the same spec installed against another rostered program. Use it so a branch keeps a vocabulary after it adopts that program.
- `program_runtime()` returns the underlying `WorthQueryProgramApplicationRuntime`, which guarded effects execute through.
- `WorthQueryProgramApplicationRuntime::supported_program::<Program>()` (reach it as `workflow.program_runtime().supported_program::<Program>()`) returns `Option<WorthQuerySupportedProgramHandle>`. It is `None` unless this host rostered that program and still supports it. The handle's `installed_program()` is what you install the extra vocabulary against.

`WorthQueryWorkflowRuntimeBindingDenial`:

| Variant | Cause |
|---|---|
| `ForeignSchema` | The installed spec belongs to another schema. |
| `ForeignProgram` | The spec was not installed against a program this host rosters and still supports. |
| `ForeignAuthenticationOwner` | The signing owner belongs to another authentication owner. |
| `AlreadySupported` | The program already has a vocabulary on this runtime. The initial program always has one. |
| `ConflictingVocabularyCoverage` | The spec was installed for the program with different node vocabulary. |

A branch whose program has no installed vocabulary is refused before any effect.

## Node kinds

`ApplicationWorkflowNodeKind` has six variants:

| Kind | Builder method | What it does | Outgoing control outcomes |
|---|---|---|---|
| `Operation { operation, requires_workflow_authority }` | `operation::<Operation>(id, requires_workflow_authority)` or `operation_binding::<Binding>(id)` | Runs one installed operation. With `requires_workflow_authority`, the step issues authority that a guarded effect consumes. `operation_binding` takes the flag from `Binding::REQUIRES_WORKFLOW_AUTHORITY`. | `Completed` |
| `Assessment(..)` | `assessment::<Query>(id)`, `assessment_for::<Query>(id, selector)`, `assessment_when_related_relation_present::<..>(..)` | Evaluates one installed query and records its result as evidence. | `Completed` |
| `Condition(..)` | `condition::<Query>(id)` | Evaluates a pure predicate. The query's result binding must produce `bool`. It is not an effect. | `ConditionSatisfied`, `ConditionUnsatisfied` |
| `Approval(..)` | `approval::<Capability>(id)` | Waits for a signed decision by an authorized principal. | `Approved`, `Rejected` |
| `EvidenceJoin(policy)` | `evidence_join(id, policy)` | Joins assessment evidence. `AllRequiredPassing` or `AllRequiredCompleted`. | `EvidenceSatisfied`, `EvidenceFailed` |
| `Terminal` | `terminal(id)` | Ends the instance. | none |

`ApplicationWorkflowControlOutcome` has nine variants: `Completed`, `Approved`,
`Rejected`, `EvidenceSatisfied`, `EvidenceFailed`, `ConditionSatisfied`,
`ConditionUnsatisfied`, `RetryExhausted`, and `NavigatedBack`. The kernel
publishes `NavigatedBack` itself when a caller navigates back; you never author
a route on it.

Validation rules for routes (`ApplicationWorkflowValidationDenialKind`):

- Every non-terminal node routes each of its outcomes exactly once. A missing route is `MissingControlOutcome`; a duplicate is `AmbiguousControlOutcome`; an outcome the kind does not produce is `UnexpectedControlOutcome`.
- A node with a retry also routes `RetryExhausted` exactly once.
- Control routes must not cycle. A retry is the only route back to an earlier node, and its target must lead back to its source; otherwise validation refuses with `ControlCycle`.

There is no separate read-only node kind. A read that feeds a decision is an
assessment or a condition.

## Author a definition

### Limits

`ApplicationWorkflowDefinitionLimits::new(maximum_nodes, maximum_connections,
maximum_effects, component_limits, maximum_canonical_bytes)` returns `None` when
a limit is zero.
`ApplicationWorkflowComponentLimits::new(..)` takes five limits for expanded
components. `with_total_deadline(Duration)` adds a total deadline (see
[Budgets and deadlines](#budgets-and-deadlines)).

### Builder

`ApplicationWorkflowDefinitionBuilder::<Spec>::new(identity, limits)` returns
`Result<Self, ApplicationWorkflowAuthoringDenial>`. Every node method returns
`Result<ApplicationWorkflowNodeRef<..>, ApplicationWorkflowAuthoringDenial>`.
The node reference is typed by node kind, so the compiler rejects a connection
that makes no sense (for example, joined evidence into an operation).

Connection methods return `&mut Self` and chain:

| Method | Connects |
|---|---|
| `start(&node)` | Marks the first node. |
| `control(&source, outcome, &target)` | Routes one control outcome. |
| `retry(&source, retry, &target)` | Routes a bounded retry (see below). |
| `sequence(&[operations])` | Chains operations on `Completed`. Returns `Result`. |
| `proposal_for_assessment(&operation, &assessment)` | The proposal an operation produced is the assessment's subject. |
| `condition_subject(&operation, &condition)` | The proposal is the condition's subject. |
| `proposal_for_approval(&operation, &approval)` | The proposal is what the approver decides on. |
| `assessment_evidence(&assessment, &join)` | Feeds evidence into a join. |
| `joined_evidence(&join, &approval)` | The approver sees the joined evidence. |
| `approval_authority(&approval, &operation)` | The approval authorizes that operation. |
| `operation_input(&operation, &operation)` | One operation's input comes from another's proposal. |

`finish()` returns `Result<AuthoredWorkflowDefinition<Spec>,
ApplicationWorkflowAuthoringDenial>`. `validate()` checks the whole graph and
returns `ValidatedWorkflowDefinition<Spec>` (with `identity()` and
`component_expansions()`).

The `worth_query_workflow!` macro is shorthand for the same builder, not a
second grammar. It creates the builder, runs your block, and evaluates to the
`finish()` result. Use it inside a function that returns
`Result<_, ApplicationWorkflowAuthoringDenial>`, because it applies `?` to
`new`:

```rust,ignore
let authored = worth_query_workflow! {
    spec: PurchaseWorkflowSpec;
    identity: identity;
    limits: limits;
    build: |builder| {
        let draft = builder.operation_binding::<DraftRequestBinding>("draft")?;
        let done = builder.terminal("done")?;
        builder.start(&draft).control(&draft, ApplicationWorkflowControlOutcome::Completed, &done);
    }
}?;
```

### Bounded retry

`ApplicationWorkflowRetry::new(trigger, reason, maximum_attempts)` returns
`None` when `maximum_attempts` is zero or `reason` is blank. A retry routes a
trigger outcome (for example `Rejected`) back to an earlier node at most
`maximum_attempts` times. After that, the source node takes its
`RetryExhausted` route.

### Reusable components

`ApplicationWorkflowComponentBuilder::<Spec>::new(id)` builds a reusable
subgraph with the same node methods. Its connection methods return `Result`.
Declare ports with `input_port(name, &node)` and `output_port(name, &node)`,
then `finish()` to get `AuthoredWorkflowComponent<Spec>`. In a definition,
`expand_component(occurrence, &component)` returns an
`ExpandedWorkflowComponent`; its `input(&port)` and `output(&port)` return node
references you connect like any other. Expanded nodes have paths prefixed with
the occurrence name.

## Bind and publish a definition

Binding checks the validated definition against the installed vocabulary:

```rust,ignore
let contract = installed.bind_definition(validated)?; // WorthQueryInstalledWorkflowDefinitionContract
```

Publishing is a control step on one branch:

```rust,ignore
let outcome = runtime
    .request(&principal, &scope)
    .on_branch(branch)
    .mutate(AuthoringIntent { input })
    .without_source()
    .idempotency(&key)
    .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
    .map(|request| request.execute())?;
```

- `WorkflowDefinitionExpectedPredecessor` is `Absent` for the first revision or `Published(PublishedWorkflowDefinitionRef)` for a successor. There is no overwrite: the kernel refuses a publication whose expected predecessor is not the current one.
- A successor publishes on its predecessor's branch (`PublishedWorkflowDefinitionRef::branch()`).
- `WorkflowDefinitionPublicationOutcome`:

| Variant | Meaning |
|---|---|
| `Published(PerformedWorkflowDefinitionPublication)` | The definition is current. `definition()` returns the `PublishedWorkflowDefinitionRef`. |
| `Application(WorthQueryApplicationUncommitted)` | The commit did not land. |
| `ProjectionDenied(..)` | The commit landed, but the projection was refused. |

Preparation refusals are `WorthQueryWorkflowDefinitionPublicationPreparationDenial`.

## Discover and retire definitions

Discovery asks one selected branch occurrence which revision is current for an identity:

```rust,ignore
let found = runtime
    .on_branch(branch)
    .select()?
    .discover_workflow_definition::<PurchaseWorkflowSpec>(&identity)?;
```

`WorthQueryWorkflowDefinitionDiscovery`:

| Variant | Meaning |
|---|---|
| `Current(PublishedWorkflowDefinitionRef)` | This revision is current on this occurrence. |
| `Retired` | The lineage was retired. It admits no new starts. |
| `Unpublished` | Nothing was published under this identity on this occurrence. |

`WorthQueryWorkflowDefinitionDiscoveryDenial`:

| Variant | Meaning |
|---|---|
| `ForeignLineage` | Another spec published this identity. Discover it through that spec. |
| `LineageUnreadable` | The lineage records cannot be read within their fixed bounds. Retrying the same discovery cannot succeed. |
| `IndexUnavailable` | The lineage index could not be made current within its reconstruction budget. |

The answer is exact for that occurrence only. A later start that finds the
definition superseded or retired says so with a typed outcome. A discovered
reference grants no authority.

Retirement: `prepare_workflow_definition_retirement(workflow, definition)`
retires one exact current definition so its lineage admits no new starts.
History, custody, and instances pinned to it are untouched. The outcome is
`WorkflowDefinitionRetirementOutcome { Retired(..), Application(..) }`;
preparation refusals are `WorthQueryWorkflowDefinitionRetirementPreparationDenial`.

## Start an instance

```rust,ignore
let started = runtime
    .request(&principal, &scope)
    .on_branch(definition.branch())
    .mutate(StartIntent { input })
    .without_source()
    .idempotency(&key)
    .prepare_workflow_instance_start(&workflow, definition)?
    .execute();
```

`WorkflowInstanceStartOutcome`:

| Variant | Meaning |
|---|---|
| `Started(PerformedWorkflowInstanceStart)` | `instance()` returns the `PublishedWorkflowInstanceRef`. |
| `Superseded(SupersededWorkflowDefinitionStart)` | The named revision is no longer current. The outcome names the current one. |
| `Retired(RetiredWorkflowDefinitionStart)` | The lineage was retired. |
| `Application(WorthQueryApplicationUncommitted)` | The commit did not land. |
| `ProjectionDenied(..)` | The commit landed, but the projection was refused. |

Preparation refusals are `WorthQueryWorkflowInstancePreparationDenial`. The
instance is pinned to the revision it started on. Publishing a successor does
not move running instances.

## Propose and advance

**Propose.** `prepare_workflow_proposal(&workflow, instance)` publishes a
proposal: the typed input of the proposal intent. Assessments, conditions,
approvals, and guarded operations read it through the data connections you
authored. `WorkflowProposalOutcome` is `Published(PerformedWorkflowProposal)`
(with `proposal()` returning `PublishedWorkflowProposalRef`), `Application(..)`,
or `ProjectionDenied(..)`.

**Advance.** `prepare_workflow_advance(&workflow, instance)` returns
`Result<WorthQueryWorkflowAdvanceRequest, WorthQueryWorkflowAdvancePreparationDenial>`.
`execute()` takes one step and returns `WorkflowProgressOutcome`:

| Variant | What the instance needs next |
|---|---|
| `Completed(PerformedWorkflowTransition)` | Nothing; a step was recorded. `node_path()` names it; `terminal()` says whether the instance ended. |
| `AwaitingActor(RequiredWorkflowActor)` | The pumping principal is not authorized for this step. `instance()` names the instance; `denial()` says why. A principal authorized for the step pumps the same instance again. |
| `AwaitingAssessment(RequiredWorkflowAssessment)` | Evidence from an assessment. `node_path()` names it. |
| `AwaitingCondition(RequiredWorkflowCondition)` | A condition result. |
| `AwaitingOperation(RequiredWorkflowOperation)` | A guarded effect under the issued authority. |
| `AwaitingEvidence(RequiredWorkflowEvidence)` | More evidence before a join can decide. |
| `AwaitingApproval(RequiredWorkflowApproval)` | A signed approval decision. |
| `Application(WorthQueryApplicationUncommitted)` | The commit did not land. |
| `ProjectionDenied(..)` | The commit landed, but the projection was refused. |
| `PreparationDenied(WorthQueryApplicationAttemptDenial)` | The kernel refused the step. Read `kind()` (see [Denials](#denials)). |
| `AuthenticationDenied(..)` | The authentication event was refused. |
| `IdempotencyDenied(..)` | The key conflicts with a different recorded request. |

`WorthQueryWorkflowAdvancePreparationDenial` has `RuntimeMismatch`,
`RequestAdmission(..)`, `TransitionPreparation(..)`, `Authentication(..)`, and
`AwaitingActor(RequiredWorkflowActor)`. The preparation-time `AwaitingActor`
is observation-time readiness: no transition attempt was prepared.

## Assessments

An assessment step settles through an output demand, then records evidence:

```rust,ignore
let mut handle = runtime
    .request(&principal, &scope)
    .on_branch(instance.branch())
    .mutate(AdvanceIntent { input })
    .without_source()
    .idempotency(&key)
    .prepare_workflow_advance(&workflow, instance.clone())?
    .into_assessment_demand(BudgetAvailableDemand::new(request_id))?
    .controls(WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(512).unwrap(),
        NonZeroUsize::new(1024).unwrap(),
    ))
    .start()?;

let settlement = match handle.settle(&runtime.request(&principal, &scope).on_branch(instance.branch()))? {
    WorthQueryWorkflowAssessmentDemandProgress::Settled(settlement) => settlement,
    WorthQueryWorkflowAssessmentDemandProgress::Pending => return Ok(()), // pump again later
};

let outcome = runtime
    .request(&principal, &scope)
    .on_branch(instance.branch())
    .mutate(AdvanceIntent { input })
    .without_source()
    .idempotency(&next_key)
    .prepare_workflow_advance(&workflow, instance)?
    .accept_assessment(&settlement)?;
```

- `into_assessment_demand` (and `into_assessment_demand_for`) refuses a demand whose contract does not match the installed assessment with `WorthQueryWorkflowAssessmentDemandPreparationDenial`.
- `settle(&fresh_request)` runs bounded attempts and returns `Settled` or `Pending`. `Pending` means pump again.
- `accept_assessment` returns `WorkflowProgressOutcome` or `WorthQueryWorkflowAssessmentAcceptanceDenial` (`NotAwaitingAssessment`, `RequirementMismatch`, `Replay(..)`, `Attempt(..)`).
- `prepare_workflow_collect_assessment(&workflow, instance, node_path)` collects evidence for a declared assessment before the instance reaches it.

## Conditions

A condition node evaluates a query whose result binding produces `bool`. When
the instance returns `AwaitingCondition(required)`, run that query with an
ordinary query request on the instance's branch, then pass its published result
to a fresh advance request:
`prepare_workflow_advance(..)?.accept_condition::<Binding>(&required, result)`.
The binding, query, parameter type, and result type must match `required`. Refusals are
`WorthQueryWorkflowConditionAcceptanceDenial` (`NotAwaitingCondition`,
`RequirementMismatch`, `Replay(..)`, `Attempt(..)`). The instance then follows
`ConditionSatisfied` or `ConditionUnsatisfied`.

## Approvals

An approval is a signed control step. Each decision needs a fresh
authentication event:

```rust,ignore
let signing = runtime
    .request(&principal, &scope)
    .on_branch(instance.branch())
    .mutate(ApprovalIntent { input })
    .without_source()
    .idempotency(&key)
    .prepare_workflow_approval(&workflow, instance, &required, &proposal, WorkflowApprovalDecision::Approve)?;

let Some(intent) = signing.authentication_intent().cloned() else {
    return signing.execute_replay(); // this key already recorded a decision
};
let event = block_on(authentication.authenticate((), &principal, intent, &scope))?;
let outcome = signing.sign(&event).map(|request| request.execute())?;
```

- `required` is the `RequiredWorkflowApproval` from `AwaitingApproval`. `proposal` is the `PublishedWorkflowProposalRef` the approver decides on.
- `WorkflowApprovalDecision` is `Approve` or `Reject`.
- `prepare_workflow_approval` returns `WorthQueryWorkflowApprovalSigningRequest`. When the key already recorded a decision, `authentication_intent()` is `None` and `execute_replay()` returns the recorded outcome without a new event.
- On `Approved`, the approval's authority flows to the operation connected by `approval_authority`. On `Rejected`, a retry route (if authored) sends the instance back.

Approval refusals surface as attempt denial kinds: `WorkflowApprovalPrincipalStale`,
`WorkflowApprovalGrantUnavailable`, `WorkflowApprovalExpired`,
`WorkflowApprovalDelegationChanged`, and `WorkflowApprovalAuthorityDenied`.

## Operations

When advance returns `AwaitingOperation(required)`, run the guarded effect
through an ordinary mutation request bound to that requirement:

```rust,ignore
let committed = runtime
    .request(&principal, &scope)
    .on_branch(instance.branch())
    .mutate(PlaceOrderIntent { input })
    .without_source()
    .idempotency(&key)
    .for_workflow_operation(&workflow, &required)?
    .execute_in_program(workflow.program_runtime())?;
```

- `for_workflow_operation` returns `WorthQueryWorkflowOperationBindingDenial` if the intent does not match the requirement or no authority was issued (`AuthorityUnavailable`).
- `execute_in_program` resolves the program from the request's branch and returns `WorthQueryApplicationMutationOutcome` (for example `Committed { .. }`).
- After the effect commits, advance again. The kernel records the step and follows `Completed`.
- For an external effect whose owner holds custody, `accept_operation_from_owner` accepts the owner-resolved receipt and returns `WorkflowProgressOutcome`, or `WorthQueryWorkflowOperationOwnerAcceptanceDenial`. `WorthQueryWorkflowOperationOwnerPosture` reports what the owner saw.
- Recovery of an owner operation binds only the exact performed operation. It never issues authority for a new effect. First bind the request with `for_workflow_operation_recovery(&workflow, &required)`, which returns `WorthQueryWorkflowOperationBindingDenial` if the intent does not match. Without that binding, recovery preparation is refused with `NotWorkflowBound`. `prepare_workflow_operation_recovery_from_owner` returns a `WorthQueryPreparedWorkflowOperationRecovery` or `WorthQueryWorkflowOperationRecoveryPreparationDenial`. Its `safe_retry()` returns the safe-retry admission or `WorthQueryWorkflowOperationRecoveryDenial`, which hands the prepared recovery back through `into_recovery()` when it can be tried again. `accept_recovered_operation_from_owner` then accepts the recovered receipt, with the same outcome and denial as `accept_operation_from_owner`.

## Navigate back

`prepare_workflow_navigate_back(&workflow, instance)` returns a request whose
`execute()` returns `Result<PerformedWorkflowTransition, WorkflowProgressOutcome>`.
It publishes one Back settlement; the kernel records it as `NavigatedBack`. The
instance head and authority are rechecked at commit. No effect or receipt is
replayed. Back spends a step. Back is refused with
`WorkflowTransitionOperationUnsettled` when it would abandon an approved
operation that has not settled.

## The ordinary convenience lane

The ordinary lane wraps the common steps on a mutation request:

| Call | Does |
|---|---|
| `mutate(intent).workflow(&workflow, authored)` | Validates the authored definition and binds it against the vocabulary for the branch's program. Returns `WorthQueryOrdinaryWorkflowDraft` or `WorthQueryOrdinaryWorkflowPublicationDenial` (`RuntimeMismatch`, `InvalidDefinition`, `UnsupportedDefinition`, `Preparation`). |
| `.publish(expected).idempotency(&key).execute()` | Publishes. Returns `WorkflowDefinitionPublicationOutcome`. |
| `mutate(intent).start_workflow(&workflow, definition).idempotency(&key).execute()` | Starts. Returns `WorkflowInstanceStartOutcome`. |
| `mutate(intent).run_workflow(&workflow, instance).idempotency_keys(&keys).execute()` | Advances repeatedly, one key per attempted step. Returns `WorthQueryOrdinaryWorkflowRunProgress`. |

`WorthQueryOrdinaryWorkflowRunProgress` exposes `transitions()`,
`attempted_steps()`, and `stop()`. `WorthQueryOrdinaryWorkflowRunStop`:

| Variant | Meaning |
|---|---|
| `Terminal` | The instance reached a terminal node. |
| `CallerKeysExhausted` | The key list ended. The instance may still be live. |
| `InstalledStepLimit` | This run reached the installed step ceiling. It is not proof of a terminal. |
| `Interrupted(..)` | The request was interrupted. |
| `PreparationDenied(..)` | A step could not be prepared. |
| `Outcome(WorkflowProgressOutcome)` | A step returned a wait or denial. Settle it with the matching lane. |

`run_workflow` only advances. An assessment, condition, approval, or operation
still needs its own lane and its own key.

## Caller-pumped progression

- There is no background scheduler. No timer advances an instance.
- There is no callback, resume message, or inbound completion API.
- Every step happens because a caller sent a request. A caller that stops pumping leaves the instance where it is.
- Each outcome is typed, so a caller (or an AI agent) can read the `Awaiting*` variant and choose the next request.
- Each request carries its own idempotency key. Retrying a key replays the recorded outcome.

## Cancellation

`prepare_workflow_instance_cancellation(&workflow, instance)` ends a live
instance where it stands, on the request's branch.

- Cancellation is not rollback. Every effect the instance performed remains, and the outcome names each one.
- The start capability authorizes cancellation.
- The same key replays exactly. Any other request for a canceled instance is refused with `WorkflowInstanceCancelled`.
- A step admitted before the cancellation commits goes stale before its effect. A cancellation prepared before a step settles goes stale in turn.
- Other instances keep running.
- Cancellation still works when the instance has spent its step budget or passed its deadline.

`WorkflowInstanceCancellationOutcome`: `Cancelled(PerformedWorkflowInstanceCancellation)`,
`Application(WorthQueryApplicationUncommitted)`, or `ProjectionDenied(..)`. A
completed instance is refused with `WorkflowInstanceCompleted`.

## Owner custody

An external operation that commits into its owner's custody has not settled
until the owner's receipt is accepted. While it is unsettled:

- cancellation is refused with `WorkflowOperationInOwnerCustody`;
- migration and fork continuation are refused;
- program adoption reports the instance as `OperationInOwnerCustody` and gives it no legal disposition.

Accept the owner's receipt (see [Operations](#operations)), then retry.

## Budgets and deadlines

| Budget | Where it is set | Refusal |
|---|---|---|
| Steps per instance lineage | `maximum_retained_transitions_per_instance` in the resource ceiling | `WorkflowInstanceCapacityUnavailable` |
| Evidence bytes per instance lineage | `maximum_evidence_bytes` in the resource ceiling | `WorkflowInstanceEvidenceCapacityUnavailable` |
| Live instances per definition lineage | `maximum_live_instances` in the resource ceiling | `WorkflowLineageCapacityUnavailable` |
| Total deadline | `ApplicationWorkflowDefinitionLimits::with_total_deadline` | `WorkflowInstanceDeadlineElapsed`, `WorkflowTrustedTimeUnavailable` |
| History reconstruction | `with_history_reconstruction_budget` in the resource ceiling | `WorkflowHistoryReconstructionBudgetExceeded` |

- **Steps.** One instance lineage shares one step budget, despite the `_per_instance` in the ceiling's name: a successor counts the steps its sources took. Recorded steps still replay and the instance can still be canceled after the budget is spent. Ending other instances frees nothing for it.
- **Evidence.** When accepting evidence would exceed the ceiling, the instance stays awaiting. It can still be canceled or navigate without new evidence.
- **Live instances.** A new start is refused when the definition lineage already holds the maximum. A retry of a recorded start still replays. Canceling or completing an instance frees room.
- **Deadline.** The deadline is whole, nonzero milliseconds of trusted time from the start, measured on the installed clock. A successor keeps the earlier of its source's deadline and its own. After it elapses, the instance takes no further step, migration, or fork continuation; it can still be canceled. If the clock cannot be read, the step is refused with `WorkflowTrustedTimeUnavailable`. A late step that reaches commit after the deadline surfaces as the commit denial kind `WorkflowSettlementDenied { kind }`.

## Observations and notifications

An assessment demand handle (`WorthQueryWorkflowAssessmentDemandHandle`) is an
observation of one assessment run.

- `notifications()` returns `WorthQueryOutputDemandNotifications` for that observation.
- `close()` releases this observer only. It does not cancel the instance and does not settle its head.
- A notification is a hint to pump. It moves nothing by itself.

## Migration and fork continuation

**Migration.** `prepare_workflow_instance_migration(&workflow, instance, target, resume_at)`
ends the instance and continues its work as a new instance on the current
`target` definition, starting at the node `resume_at` names.

- The successor carries only performed effects, as history it can never repeat.
- Proposals, evidence, and approvals are re-established by running the nodes that produce them. Each one the successor consumes must run before its consumer.
- An approval whose operation has not yet run blocks migration until it settles under the source.
- The start capability authorizes migration.
- A mapping that would repeat or drop a performed effect is refused with `WorkflowInstanceMigrationUnmapped`. Later requests on the source get `WorkflowInstanceMigrated`.
- The outcome is `WorkflowInstanceStartOutcome`.

**Fork continuation.** A fork copies history, not execution. The copy's
approvals and receipts open nothing on the fork.
`prepare_workflow_fork_continuation(&workflow, instance, target, resume_at)`,
issued on the fork, continues the fork's copy as a new instance on the fork's
current `target` definition under the same law. It ends only the fork's copy.
The instance on its own branch is untouched. `target` may be the copied
definition itself, named by `PublishedWorkflowDefinitionRef::held_on(fork)`,
while the fork holds it current.

See [branches-and-previews.md](branches-and-previews.md) for forks and
[programs-and-adoption.md](programs-and-adoption.md) for what happens to
instances when a branch adopts a new program.

## Portable definition drafts (WQWD)

`worth-query-package-archive` has a versioned, authority-free codec for
authored definitions. It is not part of the `worth-query-decl` or
`worth-query-host` facades. The in-repo users are certification and release
tooling. This guide does not state that application crates may depend on it.

- `encode_workflow_definition_draft(&validated, limits)` writes a `WQWD` byte stream (protocol version `WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION`, currently `1`).
- `decode_workflow_definition_draft(bytes, limits)` returns `WorthQueryUntrustedWorkflowDefinitionDraft`. Every count is checked against the draft's limits and the remaining bytes before allocation. Node identities must be in ascending order.
- `draft.author::<Schema, Spec>(&installed)` rebuilds an `AuthoredWorkflowDefinition` against an installed spec. It refuses with `WorthQueryWorkflowDefinitionDraftDenial` and names the node it refuses.
- A draft carries no instances, approvals, or authority. Decoded bytes mint nothing. The rebuilt definition still validates, binds, and publishes through the ordinary path.

## Denials

Workflow refusals come from several families.

### Request-time (`WorthQueryApplicationRequestMutationDenial`)

These are refused before any effect, and the refusal does not claim the
idempotency key.

| Variant | Cause |
|---|---|
| `ProgramSelection(..)` | The branch's adopted program could not be inspected, so no installed owner could be chosen. |
| `ApplicationProgramRequired` | The binding needs an application program: send it through `execute_in_program`. On that lane, it means neither the branch's program nor the initial installation supplies this action. |
| `ApplicationProgramMismatch` | The program runtime passed is not the request's runtime. |
| `RequiresWorkflowTransition` | The binding requires workflow authority and the request is not bound to a workflow step. |
| `WorkflowAuthoritySpent` | The issued workflow authority was already consumed. |
| `WorkflowTransitionCurrentness(..)` | The workflow step is no longer current. It carries a `WorthQueryApplicationAttemptDenial`. |

### Attempt denial kinds (`WorthQueryApplicationAttemptDenialKind`)

Read with `WorthQueryApplicationAttemptDenial::kind()` and `subject()`.

| Family | Kinds |
|---|---|
| Definition | `WorkflowDefinitionAffinityMismatch`, `WorkflowDefinitionAuthorityMismatch`, `WorkflowDefinitionIntentIdentityUnavailable`, `WorkflowLineageUnavailable`, `WorkflowDefinitionCompilationUnavailable`, `WorkflowDefinitionSuperseded`, `WorkflowDefinitionRetired` |
| Instance lifecycle | `WorkflowInstanceCancelled`, `WorkflowInstanceCompleted`, `WorkflowInstanceMigrated`, `WorkflowInstanceMigrationUnmapped`, `WorkflowInstanceHistoryUnavailable`, `WorkflowOperationInOwnerCustody` |
| Budgets and time | `WorkflowInstanceCapacityUnavailable`, `WorkflowInstanceEvidenceCapacityUnavailable`, `WorkflowLineageCapacityUnavailable`, `WorkflowInstanceDeadlineElapsed`, `WorkflowTrustedTimeUnavailable`, `WorkflowHistoryReconstructionBudgetExceeded` |
| Evidence | `WorkflowAssessmentEvidenceIncomplete`, `WorkflowAssessmentEvidenceMismatch` |
| Approval | `WorkflowApprovalPrincipalStale`, `WorkflowApprovalGrantUnavailable`, `WorkflowApprovalExpired`, `WorkflowApprovalDelegationChanged`, `WorkflowApprovalAuthorityDenied` |
| Transition | `WorkflowTransitionAlreadySettled`, `WorkflowTransitionNodeUnsupported`, `WorkflowTransitionOperationUnsettled`, `WorkflowTransitionIdentityUnavailable` |
| Affinity and authority | `WorkflowInstanceAffinityMismatch`, `WorkflowInstanceAuthorityMismatch`, `WorkflowInstanceIntentIdentityUnavailable`, `WorkflowTransitionAffinityMismatch`, `WorkflowTransitionAuthorityMismatch` |

### Commit-time kinds (`WorthQueryApplicationCommitDenialKind`)

| Kind | Cause |
|---|---|
| `ApplicationProgramRequired` | The binding needs a program and none was resolved. |
| `WorkflowAuthorityRequired` | A guarded effect reached commit without workflow authority. |
| `ProgramNotActiveOnOccurrence { active }` | The branch adopted another program between selection and commit. |
| `ProgramSupportRetired` | The host retired support for the program. |
| `ProgramActivationUnresolved` | The branch's program activation could not be resolved. |
| `WorkflowSettlementDenied { kind }` | The workflow settlement was refused at commit, for example after the deadline. |

### Other families

| Type | Where |
|---|---|
| `ApplicationWorkflowAuthoringDenial` | Builder methods |
| `ApplicationWorkflowValidationDenial` | `validate()` |
| `WorthQueryApplicationWorkflowInstallationDenial` | Installation and `bind_definition` |
| `WorthQueryWorkflowRuntimeBindingDenial` | `retain_workflow_spec`, `support_workflow_spec` |
| `WorthQueryWorkflowDefinitionDiscoveryDenial` | Discovery |
| `WorthQueryWorkflowAdvancePreparationDenial` | `prepare_workflow_advance` |
| `WorthQueryWorkflowAssessmentAcceptanceDenial`, `WorthQueryWorkflowConditionAcceptanceDenial` | Accepting assessment evidence and condition results |
| `WorthQueryWorkflowOperationOwnerAcceptanceDenial` | `accept_operation_from_owner`, `accept_recovered_operation_from_owner`. Variants: `Binding`, `Request`, `Inspection`, `Owner`, `RecoveryNotRequired`, and `Acceptance(WorthQueryWorkflowOperationAcceptanceDenial)` |
| `WorthQueryWorkflowOperationRecoveryPreparationDenial` | `prepare_workflow_operation_recovery_from_owner` |
| `WorthQueryWorkflowOperationRecoveryDenial` | `safe_retry()` on a prepared operation recovery |
| `WorthQueryWorkflowOperationBindingDenial` | `for_workflow_operation`, `for_workflow_operation_recovery` |
| `WorthQueryOrdinaryWorkflowPublicationDenial` | Ordinary lane publication |

## You can

- Publish several definition revisions in one lineage, each naming its expected predecessor.
- Keep running instances on the revision they started on while new starts use the current one.
- Reuse a subgraph across definitions with `ApplicationWorkflowComponentBuilder`.
- Bound a rejection loop with `ApplicationWorkflowRetry`.
- Collect declared assessment evidence early with `prepare_workflow_collect_assessment`.
- Retry any request with the same key and get the recorded outcome.
- Cancel an instance after its step budget is spent or its deadline has passed.
- Keep a branch's workflows usable after adoption by installing the same spec for the new program with `support_workflow_spec`.

## You cannot

- Serve a control step with your own handler.
- Push a completion into the kernel. You pump it with a request.
- Publish without naming the predecessor you expect.
- Approve without a fresh authentication event, except when replaying a recorded key.
- Run a guarded effect without the authority its workflow step issued.
- Act on an instance from another branch.
- Undo a performed effect by canceling.
- Cancel, migrate, or continue on a fork while an owner operation is unsettled.
- Exceed the step, evidence, or live-instance budgets by retrying.
- Hold or construct compiled definitions. `CompiledWorkflowDefinition` is internal.
- Get authority from a decoded WQWD draft.

## Misconceptions

| Belief | Fact |
|---|---|
| "The workflow runs in the background." | Nothing moves until a caller pumps a request. |
| "Cancel undoes the work." | Cancellation keeps every performed effect and names each one. |
| "A rejected approval fails the instance." | With `ApplicationWorkflowRetry`, a rejection routes back up to a bounded count. After that, `RetryExhausted` routes where you authored. |
| "Publishing a new revision updates running instances." | Instances stay pinned. Only new starts use the current revision; migration is explicit. |
| "Closing the last observer stops the workflow." | `close()` releases one observer only. |
| "Migration resets the budgets." | Successors inherit the step and evidence budgets and the earlier deadline. |
| "A fork's copy of an instance can act on the fork." | A fork copies history, not execution. Use `prepare_workflow_fork_continuation`. |
| "A workflow transition is a managed run." | Workflow steps are kernel transitions. They are not managed runs. |

## Worked example: purchase-request approval

This example is illustrative. The domain types (`PurchaseSchema`,
`DraftRequest`, `BudgetAvailable`, and so on) are names an application would
declare; they do not exist in this repository. The Query names are real.

### What it models

A requester drafts a purchase request. Two checks run: the budget is available
and the vendor is approved. A manager approves or rejects. A rejection sends the
request back to the draft step at most twice. An approval authorizes one
guarded effect that places the order. The whole request must finish within
seven days.

```text
draft ──Completed──> checks/budget ──Completed──> checks/vendor ──Completed──> checks/evidence
checks/evidence ──EvidenceSatisfied──> approval
checks/evidence ──EvidenceFailed──> declined
approval ──Approved──> place-order ──Completed──> ordered
approval ──Rejected (retry ≤ 2)──> draft
approval ──RetryExhausted──> declined
```

### Vocabulary

| Member | Kind | Installed with |
|---|---|---|
| `DraftRequest` | Proposal operation | `.operation::<DraftRequestBinding>()` |
| `BudgetAvailable` | Assessment query | `.assessment::<BudgetAvailableBinding>()` |
| `VendorApproved` | Assessment query | `.assessment::<VendorApprovedBinding>()` |
| `ManagerApproval` | Approval capability | `.approval::<ManagerApprovalCapability, AdvanceOperation, AdvanceInput>()` |
| `PlaceOrder` | Guarded effect (`REQUIRES_WORKFLOW_AUTHORITY = true`) | `.operation::<PlaceOrderBinding>()` |
| Authoring, start, advance | Control capabilities (`WORKFLOW_CONTROL = true`) | `.authoring_capability`, `.instance_start_capability`, `.advance_capability` |

### Reusable checks component

```rust,ignore
let mut checks = ApplicationWorkflowComponentBuilder::<PurchaseWorkflowSpec>::new("purchase-checks")?;
let budget = checks.assessment::<BudgetAvailable>("budget")?;
let vendor = checks.assessment::<VendorApproved>("vendor")?;
let evidence = checks.evidence_join("evidence", ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing)?;
checks
    .control(&budget, ApplicationWorkflowControlOutcome::Completed, &vendor)?
    .control(&vendor, ApplicationWorkflowControlOutcome::Completed, &evidence)?
    .assessment_evidence(&budget, &evidence)?
    .assessment_evidence(&vendor, &evidence)?;
let budget_in = checks.input_port("budget-subject", &budget)?;
let vendor_in = checks.input_port("vendor-subject", &vendor)?;
let evidence_out = checks.output_port("checked-evidence", &evidence)?;
let checks = checks.finish()?;
```

### Definition

```rust,ignore
let limits = ApplicationWorkflowDefinitionLimits::new(
    16, 32, 2,
    ApplicationWorkflowComponentLimits::new(8, 1, 16, 32, 32).unwrap(),
    16 * 1024,
)
.and_then(|limits| limits.with_total_deadline(Duration::from_secs(7 * 24 * 60 * 60)))
.expect("limits are nonzero");

let mut workflow =
    ApplicationWorkflowDefinitionBuilder::<PurchaseWorkflowSpec>::new("purchase-request", limits)?;
let draft = workflow.operation::<DraftRequest>("draft", false)?;
let approval = workflow.approval::<ManagerApproval>("approval")?;
let place_order = workflow.operation_binding::<PlaceOrderBinding>("place-order")?;
let ordered = workflow.terminal("ordered")?;
let declined = workflow.terminal("declined")?;
let expanded = workflow.expand_component("checks", &checks)?;
let budget = expanded.input(&budget_in)?;
let vendor = expanded.input(&vendor_in)?;
let evidence = expanded.output(&evidence_out)?;
let revise = ApplicationWorkflowRetry::new(
    ApplicationWorkflowControlOutcome::Rejected,
    "revise the request",
    2,
)
.expect("the retry bound is valid");

workflow
    .start(&draft)
    .control(&draft, ApplicationWorkflowControlOutcome::Completed, &budget)
    .control(&evidence, ApplicationWorkflowControlOutcome::EvidenceSatisfied, &approval)
    .control(&evidence, ApplicationWorkflowControlOutcome::EvidenceFailed, &declined)
    .control(&approval, ApplicationWorkflowControlOutcome::Approved, &place_order)
    .retry(&approval, revise, &draft)
    .control(&approval, ApplicationWorkflowControlOutcome::RetryExhausted, &declined)
    .control(&place_order, ApplicationWorkflowControlOutcome::Completed, &ordered)
    .proposal_for_assessment(&draft, &budget)
    .proposal_for_assessment(&draft, &vendor)
    .proposal_for_approval(&draft, &approval)
    .joined_evidence(&evidence, &approval)
    .approval_authority(&approval, &place_order)
    .operation_input(&draft, &place_order);

let validated = workflow.finish()?.validate()?;
let contract = installed.bind_definition(validated)?;
```

### Running it

| Step | Request | Expected outcome |
|---|---|---|
| 1 | `prepare_workflow_publication(contract, Absent)` | `Published` |
| 2 | `prepare_workflow_instance_start(&workflow, definition)` | `Started` |
| 3 | `prepare_workflow_proposal(&workflow, instance)` with the draft request as input | `Published` |
| 4 | `prepare_workflow_advance(..).execute()`, repeated | `Completed` steps until `AwaitingAssessment` at `checks/budget` |
| 5 | Settle the budget demand, then `accept_assessment` | `Completed` |
| 6 | Repeat for `checks/vendor` | `Completed`, then the join decides |
| 7 | Advance | `AwaitingApproval(required)` |
| 8 | `prepare_workflow_approval(.., Reject)`, authenticate, `sign`, `execute` | `Completed` at `approval`; the retry routes to `draft` |
| 9 | Propose a revised request and repeat steps 4 to 7 | `AwaitingApproval` |
| 10 | Approve the same way | `Completed` at `approval` |
| 11 | Advance | `AwaitingOperation(required)` |
| 12 | `mutate(PlaceOrderIntent).for_workflow_operation(..).execute_in_program(..)` | `Committed { .. }` |
| 13 | Advance | `Completed` at `ordered`; `terminal()` is true |

Every row is one caller request with its own key. Each step's exact
intermediate outcomes depend on the graph; the caller reads each outcome and
chooses the next request.

### What the example shows

- **Caller-pumped progress.** Nothing happens between rows unless a caller sends the next request.
- **Bounded retry.** A third rejection takes `RetryExhausted` to `declined`.
- **Evidence budget.** Each accepted check charges the instance lineage's evidence budget.
- **Owner custody.** If `PlaceOrder` is an external effect, the request cannot be canceled or migrated until its owner receipt is accepted.
- **Cancellation.** Canceling after step 12 leaves the placed order recorded; the outcome names it.
- **Deadline.** After seven days of trusted time, no further step runs; cancellation still works.
- **Revisions.** Publishing a successor with `Published(first)` makes new starts on the first revision return `Superseded`. Running instances stay pinned.
- **Adoption.** If a new program revision changes `VendorApproved`, the definition becomes incompatible. See [programs-and-adoption.md](programs-and-adoption.md) for what each instance may do.

### Runnable reference

The certification crate has a runnable example of the same shape (a reusable
component with two assessments, a rejection loop, an approved guarded effect,
and a second revision):

```text
cargo run -p worth-query-certification --example authored_workflow
```

The Bank reference world (`workspaces/worth-query-bank-world`) runs business
payment approval as a workflow. Its control bindings declare `WORKFLOW_CONTROL`.

## Related documents

- [AI_README.md](../AI_README.md): reading order for the Query docs.
- [ordinary-application-front-door.md](ordinary-application-front-door.md): the request API every workflow request uses.
- [branches-and-previews.md](branches-and-previews.md): branches, occurrences, and forks.
- [programs-and-adoption.md](programs-and-adoption.md): programs per branch and adoption, including the workflow inventory.
- [how-it-works.md](../../../../../../docs/how-it-works.md): platform machinery, including the workflow kernel.
- [glossary.md](../../../../../../docs/glossary.md): platform terms.
