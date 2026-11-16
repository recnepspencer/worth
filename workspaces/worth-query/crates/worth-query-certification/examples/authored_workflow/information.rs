//! Information gathering retains its native occurrence without requesting a signature.

use super::document_retention_model::{
    operator_identity::{authenticate_operator, request_scope},
    readback::read_retention,
    retention_entry::{OccurrenceSetRetentionBinding, DOCUMENT_IDENTITY},
    schema::SetRetentionInput,
    workflow::{
        advance_instance, definition_limits, propose_authoring_instance_with_retention,
        publish_definition, start_instance, ReviewedDocumentWorkflow,
        WorkflowDefinitionAuthoringOperation,
    },
};
use worth_query_decl::facade::application_program::{
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
};
use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
    WorthQueryApplicationRequestExt,
};

pub(super) fn run() {
    let directory = super::installation::location("information");
    let (home, application) = super::installation::open(&directory);
    let mut definition = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "record-information",
        definition_limits(),
    )
    .unwrap();
    let propose = definition
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .unwrap();
    let record = definition
        .operation_binding::<OccurrenceSetRetentionBinding>("record")
        .unwrap();
    let done = definition.terminal("done").unwrap();
    definition
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &record,
        )
        .control(&record, ApplicationWorkflowControlOutcome::Completed, &done)
        .operation_input(&propose, &record);
    let definition = definition.finish().unwrap().validate().unwrap();
    let published = match publish_definition(
        &application,
        definition,
        WorkflowDefinitionExpectedPredecessor::Absent,
        30001,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("information definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, published, 30002).unwrap() {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("information instance did not start: {other:?}"),
    };
    assert!(matches!(
        propose_authoring_instance_with_retention(&application, instance.clone(), 30003, 8)
            .unwrap(),
        WorkflowProposalOutcome::Published(_)
    ));
    let required = match advance_instance(&application, instance.clone(), 30004).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("information operation is not executable: {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch());
    let outcome = request
        .mutate(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: 8,
        })
        .without_source()
        .idempotency_owned(30005_u64).unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .for_workflow_operation(&application, &required)
        .unwrap_or_else(|_|panic!("exact original occurrence"))
        .retain_durable_operation::<super::document_retention_model::programs::RetentionProgramP0,super::actions::OccurrenceAction>(&request, application.program_runtime())
        .unwrap_or_else(|_|panic!("installed owning action"));
    let outcome=match outcome.advance(&request) {
        worth_query_host::facade::application_entry::WorthQueryWorkflowOperationAdvance::Completed(done)=>done,
        _=>panic!("lawful information effect"),
    };
    outcome
        .close(&request)
        .unwrap_or_else(|_| panic!("native effect cleanup"));
    assert_eq!(read_retention(runtime, instance.branch()), 8);
    match advance_instance(&application, instance, 30006).unwrap() {
        WorkflowProgressOutcome::Completed(performed) => assert_eq!(performed.node_path(), "done"),
        other => panic!("information operation did not settle: {other:?}"),
    }
    drop(application);
    super::installation::close(home);
    let (home, application) = super::installation::open(&directory);
    assert_eq!(
        read_retention(application.runtime(), application.runtime().current_world()),
        8
    );
    drop(application);
    super::installation::close(home);
    println!("Information recorded without a signed approval; native operation settled.");
}
