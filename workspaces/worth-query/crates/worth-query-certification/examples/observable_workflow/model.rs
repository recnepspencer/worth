//! Neutral document model reused by the mixed workflow example.
//!
//! The original document model is compiled against this example-owned schema.
//! Its canonical courtroom schema and protected predecessors remain unchanged.
//! The added contribution declares the real inbound archival effect.

#[path = "../../tests/application_graph/document_retention_model/assessment_output.rs"]
pub mod assessment_output;
#[path = "../../tests/application_graph/document_retention_model/assessment_readiness.rs"]
pub mod assessment_readiness;
#[path = "../../tests/application_graph/document_retention_model/correctable_retention.rs"]
pub mod correctable_retention;
#[path = "../../tests/application_graph/document_retention_model/host.rs"]
pub mod host;
#[path = "installation.rs"]
pub mod installation;
#[path = "../../tests/application_graph/document_retention_model/operator_identity.rs"]
pub mod operator_identity;
#[path = "../../tests/application_graph/document_retention_model/presented_request.rs"]
pub mod presented_request;
#[path = "../../tests/application_graph/document_retention_model/program_adoption.rs"]
pub mod program_adoption;
#[path = "../../tests/application_graph/document_retention_model/programs.rs"]
pub mod programs;
#[path = "../../tests/application_graph/document_retention_model/readback.rs"]
pub mod readback;
#[path = "../../tests/application_graph/document_retention_model/retention_days.rs"]
pub mod retention_days;
#[path = "../../tests/application_graph/document_retention_model/retention_entry.rs"]
pub mod retention_entry;
#[path = "../../tests/application_graph/document_retention_model/rules.rs"]
pub mod rules;
#[path = "model/schema.rs"]
pub mod schema;
#[path = "../../tests/application_graph/document_retention_model/settled_verdict.rs"]
pub mod settled_verdict;
#[path = "../../tests/application_graph/document_retention_model/workflow.rs"]
pub mod workflow;

#[path = "../../tests/application_graph/document_retention_model/actor_generation.rs"]
pub mod actor_generation;

/// Retains the example's complete installed vocabulary in the existing owner.
pub fn retain_mixed_workflow(
    application: host::DocumentRetentionRuntime<crate::program::MixedProgram>,
) -> MixedApplication {
    use self::workflow::{
        install_certification_authentication_with_clock, WorkflowAdvanceCapability,
        WorkflowAdvanceInput, WorkflowAdvanceOperation, WorkflowApprovalCapability,
        WorkflowDefinitionAuthoringBinding, WorkflowDefinitionAuthoringCapability,
        WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringOperation,
        WorkflowInstanceStartCapability, WorkflowInstanceStartInput,
        WorkflowInstanceStartOperation,
    };
    use self::{
        retention_days::DocumentRetentionDaysQueryBinding,
        retention_entry::{
            DocumentRetentionConditionQueryBinding, DocumentRetentionQueryBinding,
            ExplicitReviewedSetRetentionBinding, OccurrenceSetRetentionBinding,
            ReviewedSetRetentionBinding,
        },
        schema::DocumentRetentionSchema,
    };
    use worth_query_host::facade::domain::WorthQueryApplicationWorkflowSpecInstallation;

    let schema = application.runtime().installed_schema();
    let program = application.installed_program();
    let resources = workflow::workflow_resources();
    type Program = crate::program::MixedProgram;
    let specification = {
        WorthQueryApplicationWorkflowSpecInstallation::<
        DocumentRetentionSchema,
        crate::definition::MixedWorkflow,
        Program,
    >::begin(schema, program, resources)
    .operation::<crate::effect::ArchiveBinding>()
    .expect("the inbound origin operation is installed")
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
    };
    let (authentication, _authentication_clock) =
        install_certification_authentication_with_clock(application.runtime().installed_schema());
    let workflow = application
        .retain_workflow_spec(specification, authentication.signing_owner())
        .expect("the installed vocabulary belongs to this runtime");
    MixedApplication {
        workflow,
        authentication,
    }
}

/// The example retains the native workflow runtime and its signing owner.
pub struct MixedApplication {
    pub workflow:
        worth_query_host::facade::application_installation::WorthQueryWorkflowApplicationRuntime<
            schema::DocumentRetentionSchema,
            crate::definition::MixedWorkflow,
            crate::program::MixedProgram,
        >,
    pub authentication: workflow::CertificationAuthenticationOwner,
}

impl std::ops::Deref for MixedApplication {
    type Target =
        worth_query_host::facade::application_installation::WorthQueryWorkflowApplicationRuntime<
            schema::DocumentRetentionSchema,
            crate::definition::MixedWorkflow,
            crate::program::MixedProgram,
        >;
    fn deref(&self) -> &Self::Target {
        &self.workflow
    }
}

impl<'application> From<&'application MixedApplication>
    for worth_query_host::facade::application_installation::WorthQueryWorkflowVocabulary<
        'application,
        schema::DocumentRetentionSchema,
        crate::definition::MixedWorkflow,
    >
{
    fn from(application: &'application MixedApplication) -> Self {
        application.workflow.vocabulary()
    }
}
