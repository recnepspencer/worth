use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
use worth_query_installation::facade::{
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryApplicationWorkflowSpecInstallation,
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
    install_certification_authentication, ReviewedGeometryWorkflow, WorkflowAdvanceCapability,
    WorkflowAdvanceInput, WorkflowAdvanceOperation, WorkflowApprovalCapability,
    WorkflowDefinitionAuthoringBinding, WorkflowDefinitionAuthoringCapability,
    WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringOperation,
    WorkflowInstanceStartCapability, WorkflowInstanceStartInput, WorkflowInstanceStartOperation,
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
    let workflow = WorthQueryApplicationWorkflowSpecInstallation::<
        BoundedDimensionSchema,
        ReviewedGeometryWorkflow,
        DimensionProgramP0,
    >::begin(application.runtime().installed_schema(), application.installed_program(), resources)
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
    .expect("the workflow vocabulary is valid");
    let authentication =
        install_certification_authentication(application.runtime().installed_schema());
    let workflow = application
        .retain_workflow_spec(workflow, authentication.signing_owner())
        .expect("the workflow vocabulary belongs to the runtime");
    BoundedDimensionWorkflowRuntime {
        workflow,
        authentication,
    }
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
