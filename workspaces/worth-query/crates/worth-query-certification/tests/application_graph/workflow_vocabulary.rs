//! Operation-only vocabularies need no invented assessment or signed approval.
use super::document_retention_model::{
    host::publish_on_first_program,
    programs::RetentionProgramP0,
    schema::DocumentRetentionSchema,
    workflow::{
        WorkflowAdvanceCapability, WorkflowAdvanceInput, WorkflowAdvanceOperation,
        WorkflowDefinitionAuthoringBinding, WorkflowDefinitionAuthoringCapability,
        WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringOperation,
        WorkflowInstanceStartCapability, WorkflowInstanceStartInput,
        WorkflowInstanceStartOperation,
    },
};
use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowActorPolicy, ApplicationWorkflowObservationPolicy, ApplicationWorkflowSpec,
    ApplicationWorkflowSpecIdentity,
};

struct InformationOnlyWorkflow;
impl ApplicationWorkflowSpec for InformationOnlyWorkflow {
    type Schema = DocumentRetentionSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.query.tests.information-only.v1");
    fn observation_policy() -> ApplicationWorkflowObservationPolicy {
        ApplicationWorkflowObservationPolicy::disabled()
    }
    fn actor_policy() -> ApplicationWorkflowActorPolicy {
        ApplicationWorkflowActorPolicy::not_applicable()
    }
}
use worth_query_host::facade::{
    declaration::application_program::ApplicationWorkflowComponentLimits,
    domain::{
        WorthQueryApplicationWorkflowInstallationDenialKind,
        WorthQueryApplicationWorkflowResourceCeiling,
        WorthQueryApplicationWorkflowSpecInstallation,
    },
};

#[test]
fn operation_only_workflow_vocabulary_installs_but_an_entirely_empty_one_is_denied() {
    let application = publish_on_first_program();
    let installation = || {
        WorthQueryApplicationWorkflowSpecInstallation::<DocumentRetentionSchema, InformationOnlyWorkflow, RetentionProgramP0>::begin(
            application.installed_schema(), application.installed_program(),
            WorthQueryApplicationWorkflowResourceCeiling::new(32, 64, 4,
                ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(), 64 * 1024, 32, 128, 256 * 1024).unwrap())
        .authoring_capability::<WorkflowDefinitionAuthoringCapability, WorkflowDefinitionAuthoringOperation, WorkflowDefinitionAuthoringInput>().unwrap()
        .instance_start_capability::<WorkflowInstanceStartCapability, WorkflowInstanceStartOperation, WorkflowInstanceStartInput>().unwrap()
        .advance_capability::<WorkflowAdvanceCapability, WorkflowAdvanceOperation, WorkflowAdvanceInput>().unwrap()
    };
    let denial = match installation().finish() {
        Ok(_) => panic!("entirely empty vocabulary installed"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationWorkflowInstallationDenialKind::EmptyVocabulary
    );
    let vocabulary = installation()
        .operation::<WorkflowDefinitionAuthoringBinding>()
        .unwrap()
        .finish()
        .unwrap();
    use worth_query_host::facade::declaration::application_program::{
        ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
    };
    let mut definition = ApplicationWorkflowDefinitionBuilder::<InformationOnlyWorkflow>::new(
        "information-only",
        super::document_retention_model::workflow::definition_limits(),
    )
    .unwrap();
    let propose = definition
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .unwrap();
    let done = definition.terminal("done").unwrap();
    definition.start(&propose).control(
        &propose,
        ApplicationWorkflowControlOutcome::Completed,
        &done,
    );
    assert!(vocabulary
        .bind_definition(definition.finish().unwrap().validate().unwrap())
        .is_ok());
    let mut unsupported = ApplicationWorkflowDefinitionBuilder::<InformationOnlyWorkflow>::new(
        "outside-vocabulary",
        super::document_retention_model::workflow::definition_limits(),
    )
    .unwrap();
    let set = unsupported
        .operation::<super::document_retention_model::schema::SetRetention>("set", false)
        .unwrap();
    let done = unsupported.terminal("done").unwrap();
    unsupported
        .start(&set)
        .control(&set, ApplicationWorkflowControlOutcome::Completed, &done);
    let denial = match vocabulary.bind_definition(unsupported.finish().unwrap().validate().unwrap())
    {
        Ok(_) => panic!("definition member outside installed vocabulary was admitted"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationWorkflowInstallationDenialKind::UnsupportedDefinitionMember
    );
}
