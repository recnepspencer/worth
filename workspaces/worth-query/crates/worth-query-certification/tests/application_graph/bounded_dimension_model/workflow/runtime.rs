use worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition;
use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
use worth_query_installation::facade::{
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryApplicationWorkflowSpecInstallation,
    WorthQueryInstalledApplicationProgram, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledApplicationWorkflowSpec,
};

use super::super::{
    dimension_entry::{
        PartDimensionConditionQueryBinding, PartDimensionQueryBinding,
        ReviewedSetPartDimensionBinding,
    },
    host::{BoundedDimensionRuntime, BoundedDimensionWorkflowRuntime},
    programs::DimensionProgramP0,
    schema::BoundedDimensionSchema,
};
use super::{
    install_certification_authentication_with_clock, ReviewedGeometryWorkflow,
    WorkflowAdvanceCapability, WorkflowAdvanceInput, WorkflowAdvanceOperation,
    WorkflowApprovalCapability, WorkflowDefinitionAuthoringBinding,
    WorkflowDefinitionAuthoringCapability, WorkflowDefinitionAuthoringInput,
    WorkflowDefinitionAuthoringOperation, WorkflowInstanceStartCapability,
    WorkflowInstanceStartInput, WorkflowInstanceStartOperation,
};

pub fn retain_workflow(
    application: BoundedDimensionRuntime<DimensionProgramP0>,
) -> BoundedDimensionWorkflowRuntime {
    retain_workflow_with_resources(application, workflow_resources())
}

pub fn retain_workflow_with_resources(
    application: BoundedDimensionRuntime<DimensionProgramP0>,
    resources: WorthQueryApplicationWorkflowResourceCeiling,
) -> BoundedDimensionWorkflowRuntime {
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
    BoundedDimensionWorkflowRuntime {
        workflow,
        authentication,
        authentication_clock,
    }
}

/// Installs the reviewed-geometry vocabulary against one installed program,
/// so the same spec can serve every program the host rosters.
pub fn install_workflow_spec<Program>(
    schema: &WorthQueryInstalledApplicationSchema<BoundedDimensionSchema>,
    program: &WorthQueryInstalledApplicationProgram<BoundedDimensionSchema, Program>,
    resources: WorthQueryApplicationWorkflowResourceCeiling,
) -> WorthQueryInstalledApplicationWorkflowSpec<
    BoundedDimensionSchema,
    ReviewedGeometryWorkflow,
    Program,
>
where
    Program: ApplicationProgramDefinition<BoundedDimensionSchema>,
{
    WorthQueryApplicationWorkflowSpecInstallation::<
        BoundedDimensionSchema,
        ReviewedGeometryWorkflow,
        Program,
    >::begin(schema, program, resources)
    .operation::<WorkflowDefinitionAuthoringBinding>()
    .expect("the workflow operation is installed")
    .operation::<ReviewedSetPartDimensionBinding>()
    .expect("the workflow apply operation is installed")
    .assessment::<PartDimensionQueryBinding>()
    .expect("the workflow assessment is installed")
    .condition::<PartDimensionConditionQueryBinding>()
    .expect("the workflow condition is installed")
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
pub fn support_workflow_program<Program>(application: &mut BoundedDimensionWorkflowRuntime)
where
    Program: ApplicationProgramDefinition<BoundedDimensionSchema> + 'static,
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
        .expect("the vocabulary belongs to a rostered program");
}

fn workflow_resources() -> WorthQueryApplicationWorkflowResourceCeiling {
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
