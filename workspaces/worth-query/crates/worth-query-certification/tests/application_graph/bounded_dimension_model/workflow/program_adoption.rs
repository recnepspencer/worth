//! Moving a workflow host from the first program to `DimensionProgramP1`, and
//! driving workflows through the P1 vocabulary afterwards.

use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef,
    RequiredWorkflowApproval, WorkflowApprovalDecision, WorkflowInstanceStartOutcome,
    WorkflowProgressOutcome, WorkflowProposalOutcome,
    WorthQueryApplicationProgramAdoptionPreparationDenial, WorthQueryApplicationRequestExt,
    WorthQueryOutputDemandControls, WorthQueryWorkflowAdvancePreparationDenial,
    WorthQueryWorkflowAssessmentAcceptanceDenial, WorthQueryWorkflowAssessmentDemandProgress,
    WorthQueryWorkflowInstanceStartPreparationDenial, WorthQueryWorkflowProposalPreparationDenial,
};
use worth_query_host::facade::declaration::application_program::ApplicationProgramRevision;
use worth_query_host::facade::primary_graph::{
    WorthQueryBranchAdoptionPublicationOutcome, WorthQueryPreparedBranchAdoption,
    WorthQueryWorkflowAdoptionInventory, WorthQueryWorkflowDispositions,
};
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::super::{
    assessment_output::PartAssessmentDemand,
    dimension_entry::PART_IDENTITY,
    host::BoundedDimensionWorkflowRuntime,
    operator_identity::{authenticate_operator, block_on, request_scope},
    programs::DimensionProgramP1,
};
use super::{
    WorkflowAdvanceInput, WorkflowAdvanceIntent, WorkflowApprovalIntent,
    WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringIntent,
    WorkflowInstanceStartInput, WorkflowInstanceStartIntent,
};

/// The caller's decision over the inventory the owner just read.
pub type WorkflowDecision<'decide> =
    &'decide dyn Fn(&WorthQueryWorkflowAdoptionInventory) -> WorthQueryWorkflowDispositions;

pub fn second_program_workflow_inventory(
    application: &BoundedDimensionWorkflowRuntime,
    branch: WorthQueryProductBranch,
) -> WorthQueryWorkflowAdoptionInventory {
    let target = second_revision(application);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let programs = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    programs
        .adopt(&target)
        .requirements(&requirements)
        .workflow_inventory(1_024)
        .expect("the owner inventories the branch's workflow facts")
}

/// Prepares the move to P1, deciding the fresh inventory when `decide` is set.
pub fn prepare_second_program_adoption(
    application: &BoundedDimensionWorkflowRuntime,
    branch: WorthQueryProductBranch,
    decide: Option<WorkflowDecision<'_>>,
) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryApplicationProgramAdoptionPreparationDenial>
{
    let target = second_revision(application);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let programs = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let request = programs.adopt(&target).requirements(&requirements);
    let request = match decide {
        Some(decide) => {
            let inventory = request
                .workflow_inventory(1_024)
                .expect("the owner inventories the branch's workflow facts");
            request.workflow(decide(&inventory))
        }
        None => request,
    };
    request.prepare(1_024)
}

pub fn publish_adoption(
    prepared: Result<
        WorthQueryPreparedBranchAdoption,
        WorthQueryApplicationProgramAdoptionPreparationDenial,
    >,
) {
    match prepared.expect("decided workflow facts adopt").publish() {
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_) => {}
        _ => panic!("the adoption did not publish"),
    }
}

pub fn second_revision(
    application: &BoundedDimensionWorkflowRuntime,
) -> ApplicationProgramRevision {
    application
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision()
        .clone()
}

pub fn advance_on_second(
    application: &BoundedDimensionWorkflowRuntime,
    branch: WorthQueryProductBranch,
    instance: PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial> {
    let vocabulary = application
        .supported_vocabulary::<DimensionProgramP1>()
        .expect("P1 serves its vocabulary");
    let runtime = vocabulary.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_advance(vocabulary, instance)
        .map(|request| request.execute())
}

pub fn start_on_second(
    application: &BoundedDimensionWorkflowRuntime,
    branch: WorthQueryProductBranch,
    definition: PublishedWorkflowDefinitionRef,
    idempotency: u64,
) -> Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial> {
    let vocabulary = application
        .supported_vocabulary::<DimensionProgramP1>()
        .expect("P1 serves its vocabulary");
    let runtime = vocabulary.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_instance_start(vocabulary, definition)
        .map(|request| request.execute())
}

pub fn approve_on_second(
    application: &BoundedDimensionWorkflowRuntime,
    instance: PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &PublishedWorkflowProposalRef,
    decision: WorkflowApprovalDecision,
    idempotency: u64,
) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial> {
    let vocabulary = application
        .supported_vocabulary::<DimensionProgramP1>()
        .expect("P1 serves its vocabulary");
    let runtime = vocabulary.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let signing = runtime
        .request(&principal, &scope)
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_approval(vocabulary, instance, required, proposal, decision)?;
    let Some(authentication_intent) = signing.authentication_intent().cloned() else {
        return signing.execute_replay();
    };
    let event = block_on(application.authentication().authenticate(
        (),
        &principal,
        authentication_intent,
        &scope,
    ))
    .expect("the installed certification factor accepts the exact approval challenge");
    signing.sign(&event).map(|request| request.execute())
}

pub fn propose_on_second(
    application: &BoundedDimensionWorkflowRuntime,
    instance: PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial> {
    let vocabulary = application
        .supported_vocabulary::<DimensionProgramP1>()
        .expect("P1 serves its vocabulary");
    let runtime = vocabulary.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_proposal(vocabulary, instance)
        .map(|request| request.execute())
}

/// Collects and accepts fresh evidence for `node_path` under P1.
pub fn recollect_on_second(
    application: &BoundedDimensionWorkflowRuntime,
    instance: PublishedWorkflowInstanceRef,
    node_path: &str,
    idempotency: u64,
) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAssessmentAcceptanceDenial> {
    let vocabulary = application
        .supported_vocabulary::<DimensionProgramP1>()
        .expect("P1 serves its vocabulary");
    let runtime = vocabulary.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let intent = || WorkflowAdvanceIntent {
        input: WorkflowAdvanceInput {
            part_identity: PART_IDENTITY.to_owned(),
        },
    };
    let branch = instance.branch();
    let mut handle = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(intent())
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_collect_assessment(vocabulary, instance.clone(), node_path)
        .expect("a carried instance collects fresh evidence under P1")
        .into_assessment_demand(PartAssessmentDemand::new(PART_IDENTITY))
        .expect("the typed demand matches the P1 assessment contract")
        .controls(WorthQueryOutputDemandControls::new(
            std::num::NonZeroUsize::new(512).unwrap(),
            std::num::NonZeroUsize::new(1024).unwrap(),
        ))
        .start()
        .expect("the P1 assessment demand starts");
    let settlement = match handle
        .settle(&runtime.request(&principal, &scope).on_branch(branch))
        .expect("the P1 assessment producer advances")
    {
        WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => settled,
        WorthQueryWorkflowAssessmentDemandProgress::Pending => {
            panic!("the bounded assessment producer did not settle within its declared work")
        }
    };
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(intent())
        .without_source()
        .idempotency(&(idempotency + 1))
        .prepare_workflow_collect_assessment(vocabulary, instance, node_path)
        .expect("fresh evidence acceptance is admitted under P1")
        .accept_assessment(&settlement)
}
