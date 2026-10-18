use super::*;
#[test]
fn current_contract_restores_original_identity_and_issues_fresh_operation_authority() {
    // This existing program declares the owning occurrence action; canonical P0
    // and its retained predecessor artifact keep their original meaning.
    use crate::document_retention_model::{
        operator_identity::{authenticate_operator, authenticate_reviewer, request_scope},
        workflow::{
            governed_read::{AuditRead, GovernedDocumentWorkflow, SubjectRead},
            prepare_instance_start, reacquisition, retain_workflow_for, workflow_resources,
        },
    };
    use worth_query_host::facade::application_discovery::{
        WorthQueryWorkflowAudit, WorthQueryWorkflowHistoryAudit, WorthQueryWorkflowIndexRead,
        WorthQueryWorkflowInstanceRead,
    };
    let program =
        worth_query_host::facade::declaration::application_program::ApplicationProgramAuthoring::<
            DocumentRetentionSchema,
            WorkflowOwnerProgram,
        >::begin()
        .validated_program()
        .unwrap();
    let application = retain_workflow_for::<_, GovernedDocumentWorkflow>(
        crate::document_retention_model::host::publish(
            program,
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .unwrap(),
        workflow_resources(),
    );
    use worth_query_package_archive::facade::{
        decode_workflow_definition_draft, encode_workflow_definition_draft,
        WorthQueryPackageArchiveLimits,
    };
    let definition = occurrence_definition().unwrap();
    let identity = definition.content_identity().clone();
    let bytes =
        encode_workflow_definition_draft(&definition, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap();
    let definition =
        decode_workflow_definition_draft(&bytes, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap()
            .author(application.workflow_spec())
            .unwrap()
            .validate()
            .unwrap();
    assert_eq!(definition.content_identity(), &identity);
    assert_eq!(
        encode_workflow_definition_draft(&definition, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap(),
        bytes
    );
    let published = match publish_definition(
        &application,
        definition,
        WorkflowDefinitionExpectedPredecessor::Absent,
        94011,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("current definition must publish: {other:?}"),
    };
    assert_eq!(published.content_identity(), &identity);
    let prepared = prepare_instance_start(&application, published.clone(), 94012).unwrap();
    let original_start = prepared.discovery_intent();
    let started = match prepared.execute() {
        WorkflowInstanceStartOutcome::Started(performed) => performed,
        other => panic!("current native definition must start: {other:?}"),
    };
    let original_instance = started.instance().clone();
    let actor = started.receipt().principal_scope().principal();
    let expected_start = [
        u64::from(actor.partition_id()),
        actor.local_slot(),
        u64::from(actor.generation()),
    ];
    drop(started);
    let original_proposal = match propose_authoring_instance_with_retention(
        &application,
        original_instance.clone(),
        94013,
        8,
    )
    .unwrap()
    {
        WorkflowProposalOutcome::Published(performed) => performed,
        other => panic!("current native proposal must publish: {other:?}"),
    };
    let restored = restore(capture(&application));
    let rebuilt = published.held_on(restored.runtime().current_world());
    assert_eq!(rebuilt.content_identity(), &identity);
    // Discovery uses original start DATA; it neither reissues Start nor needs an old receipt.
    let instance = match reacquisition::read(&restored, &original_start) {
        WorthQueryWorkflowInstanceRead::Live {
            instance,
            observation,
        } => {
            assert_eq!(instance.entity_id(), original_instance.entity_id());
            assert_eq!(
                instance.program_revision(),
                original_instance.program_revision()
            );
            match observation.audit() {
                WorthQueryWorkflowAudit::Visible { start, .. } => {
                    let actor = start.entity_id();
                    assert_eq!(
                        [
                            actor.partition_value_u64(),
                            actor.local_slot_value(),
                            u64::from(actor.generation_value())
                        ],
                        expected_start
                    );
                }
                _ => panic!("fresh audit must disclose original admitted start actor"),
            }
            let scope = request_scope();
            let reader = authenticate_reviewer(restored.runtime().installed_schema(), &scope);
            let request = restored.runtime().request(&reader, &scope);
            let basis = request.retain_current_read().unwrap();
            let found = request
                .at(&basis)
                .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                    restored.vocabulary(),
                    &original_start,
                    reacquisition::input(),
                    reacquisition::input(),
                )
                .unwrap();
            let page = request
                .at(&basis)
                .read_workflow_history::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                    restored.vocabulary(),
                    found.target().unwrap(),
                    reacquisition::input(),
                    reacquisition::input(),
                    16,
                    WorthQueryWorkflowIndexRead::PublishedOnly,
                    None,
                )
                .unwrap();
            assert!(page.continuation().is_none());
            let row = page
                .entries()
                .iter()
                .find(|entry| entry.proposal().is_some())
                .expect("original canonical proposal history");
            let proposal = row.proposal().unwrap();
            assert_eq!(
                proposal.entity_id(),
                original_proposal.proposal().entity_id()
            );
            assert_eq!(proposal.identity(), original_proposal.proposal().identity());
            let original_actor = original_proposal.receipt().principal_scope().principal();
            match row.audit() {
                WorthQueryWorkflowHistoryAudit::Visible {
                    proposer: Some(actor),
                    ..
                } => {
                    let actor = actor.entity_id();
                    assert_eq!(
                        [
                            actor.partition_value_u64(),
                            actor.local_slot_value(),
                            u64::from(actor.generation_value())
                        ],
                        [
                            u64::from(original_actor.partition_id()),
                            original_actor.local_slot(),
                            u64::from(original_actor.generation())
                        ]
                    );
                }
                _ => panic!("original proposal actor survives current native restore"),
            }
            instance
        }
        _ => panic!("current original start must be reacquired as live DATA"),
    };
    let required = match advance_instance(&restored, instance.clone(), 94014).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("restored proposal must supply fresh operation authority: {other:?}"),
    };
    use crate::document_retention_model::{
        readback::read_retention, retention_entry::DOCUMENT_IDENTITY, schema::SetRetentionInput,
    };
    use worth_query_host::facade::application_entry::{
        WorthQueryApplicationRequestExt, WorthQuerySourceCompletion,
        WorthQueryWorkflowOperationAdvance,
    };
    let runtime = restored.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch());
    let operation = request
        .mutate(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: 8,
        })
        .without_source()
        .idempotency_owned(94015u64)
        .unwrap_or_else(|(_, _, cause)| panic!("restored original input capture: {cause:?}"))
        .for_workflow_operation(&restored, &required)
        .unwrap_or_else(|_| panic!("restored occurrence must bind the actual original input"))
        .retain_operation::<WorkflowOwnerProgram, OccurrenceAction>(restored.program_runtime())
        .unwrap_or_else(|_| panic!("the restored installed owning action must retain custody"));
    let completed = match operation.advance(&request) {
        WorthQueryWorkflowOperationAdvance::Completed(completed) => completed,
        _ => panic!("the fresh restored operation must really complete"),
    };
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Fresh { .. }
    ));
    completed
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("actual restored operation cleanup: {cause:?}"));
    assert_eq!(read_retention(runtime, instance.branch()), 8);
    let performed_restore = restore(capture(&restored));
    assert_eq!(
        read_retention(
            performed_restore.runtime(),
            performed_restore.runtime().current_world()
        ),
        8
    );
}

fn occurrence_definition() -> Result<
    worth_query_host::facade::declaration::application_program::ValidatedWorkflowDefinition<
        crate::document_retention_model::workflow::governed_read::GovernedDocumentWorkflow,
    >,
    worth_query_host::facade::declaration::application_program::ApplicationWorkflowValidationDenial,
> {
    use crate::document_retention_model::workflow::{
        definition_limits, governed_read::GovernedDocumentWorkflow,
        WorkflowDefinitionAuthoringOperation,
    };
    use worth_query_host::facade::declaration::application_program::{
        ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
    };
    let mut builder = ApplicationWorkflowDefinitionBuilder::<GovernedDocumentWorkflow>::new(
        "information-recording",
        definition_limits(),
    )
    .unwrap();
    let propose = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .unwrap();
    let perform = builder
        .operation_binding::<OccurrenceSetRetentionBinding>("perform")
        .unwrap();
    let done = builder.terminal("done").unwrap();
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &perform,
        )
        .control(
            &perform,
            ApplicationWorkflowControlOutcome::Completed,
            &done,
        )
        .operation_input(&propose, &perform);
    builder.finish().unwrap().validate()
}
