use worth_query_host::facade::{
    application_entry::{
        PublishedWorkflowDefinitionRef, PublishedWorkflowProposalRef, RequiredWorkflowApproval,
        WorkflowApprovalDecision, WorkflowDefinitionExpectedPredecessor,
        WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome,
        WorkflowProgressOutcome, WorkflowProposalOutcome, WorthQueryApplicationRequestExt,
        WorthQueryWorkflowAdvancePreparationDenial,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
        WorthQueryWorkflowInstanceStartPreparationDenial,
        WorthQueryWorkflowProposalPreparationDenial,
    },
    declaration::application_program::{
        ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
        ApplicationWorkflowDefinitionLimits, ValidatedWorkflowDefinition,
    },
};
use worth_query_installation::facade::{
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryApplicationWorkflowSpecInstallation,
    WorthQueryInstalledWorkflowDefinitionContract,
};

use super::super::{
    dimension_entry::{PartDimensionQueryBinding, PART_IDENTITY},
    host::{BoundedDimensionRuntime, BoundedDimensionWorkflowRuntime},
    operator_identity::{authenticate_operator, request_scope},
    programs::DimensionProgramP0,
    schema::{BoundedDimensionSchema, PartDimensionQuery},
};
use super::{
    ReviewedGeometryWorkflow, WorkflowAdvanceCapability, WorkflowAdvanceInput,
    WorkflowAdvanceIntent, WorkflowAdvanceOperation, WorkflowApprovalCapability,
    WorkflowApprovalIntent, WorkflowDefinitionAuthoringCapability,
    WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringIntent,
    WorkflowDefinitionAuthoringOperation, WorkflowInstanceStartCapability,
    WorkflowInstanceStartInput, WorkflowInstanceStartIntent, WorkflowInstanceStartOperation,
};

pub fn reviewed_geometry_definition(
    completion_identity: &str,
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "reviewed-geometry",
        definition_limits(),
    )
    .expect("the workflow identity is valid");
    let propose = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .expect("the proposal node is valid");
    let structural = builder
        .assessment::<PartDimensionQuery>("checks/structural")
        .expect("the structural assessment node is valid");
    let manufacturability = builder
        .assessment::<PartDimensionQuery>("checks/manufacturability")
        .expect("the manufacturability assessment node is valid");
    let evidence = builder
        .evidence_join("checks/evidence")
        .expect("the evidence node is valid");
    let approval = builder
        .approval::<WorkflowApprovalCapability>("approval")
        .expect("the approval node is valid");
    let apply = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("apply", true)
        .expect("the guarded operation node is valid");
    let completed = builder
        .terminal(completion_identity)
        .expect("the completion node is valid");
    let rejected = builder
        .terminal("rejected")
        .expect("the rejection node is valid");
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &structural,
        )
        .control(
            &structural,
            ApplicationWorkflowControlOutcome::Completed,
            &manufacturability,
        )
        .control(
            &manufacturability,
            ApplicationWorkflowControlOutcome::Completed,
            &evidence,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::Completed,
            &approval,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Approved,
            &apply,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Rejected,
            &rejected,
        )
        .control(
            &apply,
            ApplicationWorkflowControlOutcome::Completed,
            &completed,
        )
        .proposal_for_assessment(&propose, &structural)
        .proposal_for_assessment(&propose, &manufacturability)
        .assessment_evidence(&structural, &evidence)
        .assessment_evidence(&manufacturability, &evidence)
        .proposal_for_approval(&propose, &approval)
        .joined_evidence(&evidence, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&propose, &apply);
    builder
        .finish()
        .expect("the authored definition is complete")
        .validate()
        .expect("the reviewed-geometry definition is valid")
}

pub fn terminal_definition(
    terminal_identity: &str,
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "reviewed-geometry-terminal",
        definition_limits(),
    )
    .expect("the terminal workflow identity is valid");
    let terminal = builder
        .terminal(terminal_identity)
        .expect("the terminal start node is valid");
    builder.start(&terminal);
    builder
        .finish()
        .expect("the terminal definition is complete")
        .validate()
        .expect("the terminal definition is valid")
}

pub fn proposal_terminal_definition() -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "proposal-terminal",
        definition_limits(),
    )
    .expect("the proposal-terminal workflow identity is valid");
    let proposal = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal", false)
        .expect("the proposal node is valid");
    let terminal = builder
        .terminal("completed")
        .expect("the terminal node is valid");
    builder.start(&proposal).control(
        &proposal,
        ApplicationWorkflowControlOutcome::Completed,
        &terminal,
    );
    builder
        .finish()
        .expect("the proposal-terminal definition is complete")
        .validate()
        .expect("the proposal-terminal definition is valid")
}

pub fn repeated_proposal_definition() -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "repeated-proposal",
        definition_limits(),
    )
    .expect("the repeated-proposal workflow identity is valid");
    let first = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal/first", false)
        .expect("the first proposal node is valid");
    let second = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal/second", false)
        .expect("the second proposal node is valid");
    let terminal = builder
        .terminal("completed")
        .expect("the terminal node is valid");
    builder
        .start(&first)
        .control(
            &first,
            ApplicationWorkflowControlOutcome::Completed,
            &second,
        )
        .control(
            &second,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        );
    builder
        .finish()
        .expect("the repeated-proposal definition is complete")
        .validate()
        .expect("the repeated-proposal definition is valid")
}

pub fn bind_definition(
    application: &BoundedDimensionWorkflowRuntime,
    definition: ValidatedWorkflowDefinition<ReviewedGeometryWorkflow>,
) -> WorthQueryInstalledWorkflowDefinitionContract<
    BoundedDimensionSchema,
    ReviewedGeometryWorkflow,
    DimensionProgramP0,
> {
    application
        .workflow_spec()
        .bind_definition(definition)
        .expect("the workflow definition binds to installed vocabulary")
}

pub fn retain_workflow(
    application: BoundedDimensionRuntime<DimensionProgramP0>,
) -> BoundedDimensionWorkflowRuntime {
    let workflow = WorthQueryApplicationWorkflowSpecInstallation::<
        BoundedDimensionSchema,
        ReviewedGeometryWorkflow,
        DimensionProgramP0,
    >::begin(
        application.runtime().installed_schema(),
        application.installed_program(),
        workflow_resources(),
    )
    .operation::<WorkflowDefinitionAuthoringOperation, WorkflowDefinitionAuthoringInput>()
    .expect("the workflow operation is installed")
    .assessment::<PartDimensionQueryBinding>()
    .expect("the workflow assessment is installed")
    .approval::<
        WorkflowApprovalCapability,
        WorkflowAdvanceOperation,
        WorkflowAdvanceInput,
    >()
    .expect("the workflow approval capability is installed")
    .authoring_capability::<
        WorkflowDefinitionAuthoringCapability,
        WorkflowDefinitionAuthoringOperation,
        WorkflowDefinitionAuthoringInput,
    >()
    .expect("the workflow authoring capability is installed")
    .instance_start_capability::<
        WorkflowInstanceStartCapability,
        WorkflowInstanceStartOperation,
        WorkflowInstanceStartInput,
    >()
    .expect("the workflow instance-start capability is installed")
    .advance_capability::<WorkflowAdvanceCapability, WorkflowAdvanceOperation, WorkflowAdvanceInput>()
    .expect("the workflow advance capability is installed")
    .finish()
    .expect("the workflow vocabulary is valid");
    application
        .retain_workflow_spec(workflow)
        .expect("the workflow vocabulary belongs to the runtime")
}

pub fn publish_definition(
    application: &BoundedDimensionWorkflowRuntime,
    definition: ValidatedWorkflowDefinition<ReviewedGeometryWorkflow>,
    expected_predecessor: WorkflowDefinitionExpectedPredecessor,
    idempotency: u64,
) -> Result<
    WorkflowDefinitionPublicationOutcome,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
> {
    let contract = bind_definition(application, definition);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_publication(contract, expected_predecessor)
        .map(|request| request.execute())
}

pub fn start_instance(
    application: &BoundedDimensionWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    idempotency: u64,
) -> Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_instance_start(application, definition)
        .map(|request| request.execute())
}

pub fn advance_instance(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_advance(application, instance)
        .map(|request| request.execute())
}

pub fn approve_instance(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &PublishedWorkflowProposalRef,
    decision: WorkflowApprovalDecision,
    idempotency: u64,
) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_approval(application, instance, required, proposal, decision)
        .map(|request| request.execute())
}

pub fn propose_instance(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_proposal(application, instance)
        .map(|request| request.execute())
}

pub fn propose_instance_on_branch(
    application: &BoundedDimensionWorkflowRuntime,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_proposal(application, instance)
        .map(|request| request.execute())
}

pub(super) fn definition_limits() -> ApplicationWorkflowDefinitionLimits {
    ApplicationWorkflowDefinitionLimits::new(32, 64, 4, 4, 64 * 1024)
        .expect("the workflow definition limits are nonzero")
}

fn workflow_resources() -> WorthQueryApplicationWorkflowResourceCeiling {
    WorthQueryApplicationWorkflowResourceCeiling::new(32, 64, 4, 4, 64 * 1024, 32, 128, 256 * 1024)
        .expect("the workflow installation limits are nonzero")
}
