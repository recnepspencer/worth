use worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition;
use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
use worth_query_host::facade::domain::{
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryApplicationWorkflowSpecInstallation,
    WorthQueryInstalledApplicationProgram, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledApplicationWorkflowSpec,
};

use super::super::{
    host::{DocumentRetentionRuntime, DocumentWorkflowRuntime},
    retention_days::DocumentRetentionDaysQueryBinding,
    retention_entry::{
        DocumentRetentionConditionQueryBinding, DocumentRetentionQueryBinding,
        ExplicitReviewedSetRetentionBinding, OccurrenceSetRetentionBinding,
        ReviewedSetRetentionBinding,
    },
    schema::DocumentRetentionSchema,
};
use super::{
    ReviewedDocumentWorkflow, WorkflowAdvanceCapability, WorkflowAdvanceInput,
    WorkflowAdvanceOperation, WorkflowApprovalCapability, WorkflowDefinitionAuthoringBinding,
    WorkflowDefinitionAuthoringCapability, WorkflowDefinitionAuthoringInput,
    WorkflowDefinitionAuthoringOperation, WorkflowInstanceStartCapability,
    WorkflowInstanceStartInput, WorkflowInstanceStartOperation,
    install_certification_authentication_with_clock,
};

pub fn retain_workflow<Program: ApplicationProgramDefinition<DocumentRetentionSchema>>(
    application: DocumentRetentionRuntime<Program>,
) -> DocumentWorkflowRuntime<Program> {
    retain_workflow_with_resources(application, workflow_resources())
}

pub fn retain_workflow_with_resources<
    Program: ApplicationProgramDefinition<DocumentRetentionSchema>,
>(
    application: DocumentRetentionRuntime<Program>,
    resources: WorthQueryApplicationWorkflowResourceCeiling,
) -> DocumentWorkflowRuntime<Program> {
    let workflow = install_workflow_spec(
        application.runtime().installed_schema(),
        application.installed_program(),
        resources,
    );
    let (authentication, authentication_clock) =
        install_certification_authentication_with_clock(application.runtime().installed_schema());
    let workflow = application
        .retain_workflow_spec(workflow, authentication.signing_owner())
        .expect("the workflow vocabulary belongs to the runtime");
    DocumentWorkflowRuntime {
        workflow,
        authentication,
        authentication_clock,
    }
}

/// Installs the reviewed-document vocabulary against one installed program,
/// so the same spec can serve every program the host rosters.
pub fn install_workflow_spec<Program>(
    schema: &WorthQueryInstalledApplicationSchema<DocumentRetentionSchema>,
    program: &WorthQueryInstalledApplicationProgram<DocumentRetentionSchema, Program>,
    resources: WorthQueryApplicationWorkflowResourceCeiling,
) -> WorthQueryInstalledApplicationWorkflowSpec<DocumentRetentionSchema, ReviewedDocumentWorkflow>
where
    Program: ApplicationProgramDefinition<DocumentRetentionSchema>,
{
    WorthQueryApplicationWorkflowSpecInstallation::<
        DocumentRetentionSchema,
        ReviewedDocumentWorkflow,
        Program,
    >::begin(schema, program, resources)
    .operation::<WorkflowDefinitionAuthoringBinding>()
    .expect("the workflow operation is installed")
    .operation::<ReviewedSetRetentionBinding>()
    .expect("the workflow apply operation is installed")
    .operation::<OccurrenceSetRetentionBinding>()
    .expect("the workflow occurrence-only operation is installed")
    .operation::<ExplicitReviewedSetRetentionBinding>()
    .expect("the workflow explicit review contract is installed")
    .assessment::<DocumentRetentionQueryBinding>()
    .expect("the workflow assessment is installed")
    .condition_operand::<DocumentRetentionConditionQueryBinding>()
    .expect("the workflow condition is installed")
    .condition_operand::<DocumentRetentionDaysQueryBinding>()
    .expect("the workflow days operand is installed")
    .approval::<WorkflowApprovalCapability, WorkflowAdvanceOperation, WorkflowAdvanceInput>()
    .expect("the workflow approval capability is installed")
    .authoring_capability::<WorkflowDefinitionAuthoringCapability, WorkflowDefinitionAuthoringOperation, WorkflowDefinitionAuthoringInput>()
    .expect("the workflow authoring capability is installed")
    .instance_start_capability::<WorkflowInstanceStartCapability, WorkflowInstanceStartOperation, WorkflowInstanceStartInput>()
    .expect("the workflow instance-start capability is installed")
    .advance_capability::<WorkflowAdvanceCapability, WorkflowAdvanceOperation, WorkflowAdvanceInput>()
    .expect("the workflow advance capability is installed")
    .finish()
    .expect("the workflow vocabulary is valid")
}

/// Gives the workflow runtime its vocabulary for one other rostered program.
pub fn support_workflow_program<Program>(application: &mut DocumentWorkflowRuntime)
where
    Program: ApplicationProgramDefinition<DocumentRetentionSchema> + 'static,
{
    let workflow = install_workflow_spec(
        application.workflow.installed_schema(),
        application
            .workflow
            .supported_program::<Program>()
            .expect("the program is rostered on this host")
            .installed_program(),
        workflow_resources(),
    );
    application
        .workflow
        .support_workflow_spec(workflow)
        .unwrap_or_else(|(_, denial)| {
            panic!("the vocabulary belongs to a rostered program: {denial:?}")
        });
}

pub fn workflow_resources() -> WorthQueryApplicationWorkflowResourceCeiling {
    WorthQueryApplicationWorkflowResourceCeiling::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1024,
        32,
        128,
        256 * 1024,
    )
    .expect("the workflow installation limits are nonzero")
}
