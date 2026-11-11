pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_approval::field_text_readmission::*;

#[test]
fn approval_field_text_survives_current_checkpoint_and_fresh_authority_readmission() {
    use worth_query_host::facade::application_discovery::{
        WorthQueryWorkflowAudit, WorthQueryWorkflowHistoryAudit, WorthQueryWorkflowIndexRead,
        WorthQueryWorkflowInstanceRead,
    };
    use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::authenticate_reviewer;
    let application = retain_workflow_for::<_, GovernedDocumentWorkflow>(
        publish_on_first_program(),
        workflow_resources(),
    );
    // Capture only descriptive identities; no old installed handle survives restore.
    let original_capability = application
        .runtime()
        .installed_schema()
        .capability(
            worth_query_testkit_document_retention::facade::document_retention_model::workflow::WorkflowApprovalCapability::reference(),
            worth_query_testkit_document_retention::facade::document_retention_model::workflow::WorkflowAdvanceOperation::reference(),
        )
        .unwrap();
    let original_capability_identity = *original_capability.identity().bytes();
    let original_installation_seal = original_capability.authority_identity().to_owned();
    drop(original_capability);
    let definition = match publish_definition(
        &application,
        governed_reviewed_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        98_000,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("real governed approval definition: {other:?}"),
    };
    let prepared = prepare_instance_start(&application, definition.clone(), 98_001).unwrap();
    let original_start = prepared.discovery_intent();
    let started = match prepared.execute() {
        WorkflowInstanceStartOutcome::Started(performed) => performed,
        other => panic!("real original governed start: {other:?}"),
    };
    let instance = started.instance().clone();
    let original_instance = instance.entity_id();
    let original_revision = *instance.program_revision();
    let actor = started.receipt().principal_scope().principal();
    let expected_start = [
        u64::from(actor.partition_id()),
        actor.local_slot(),
        u64::from(actor.generation()),
    ];
    drop(started);
    let (proposal, required, _) =
        super::journey::approval_requirement(&application, instance.clone(), 98_000);
    let original_proposal = proposal.identity().to_owned();
    let performed = match approve_instance(
        &application,
        instance,
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        98_010,
    )
    .unwrap()
    {
        WorkflowProgressOutcome::Completed(performed) => performed,
        other => panic!("the original admitted approval must publish: {other:?}"),
    };
    let expected_approver = performed.approval().unwrap().approver();
    let original_approval = performed
        .approval()
        .expect("the receipt projects a genuine approval")
        .entity();
    assert!(performed
        .receipt()
        .committed_changes()
        .entity_changes()
        .any(|(entity, _)| entity == original_approval));
    let checkpoint = application
        .runtime()
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    // Neither the old request nor its native live ref supplies the restored authority.
    drop(performed);
    drop(required);
    drop(proposal);
    drop(application);
    let retained = checkpoint;
    let intact = retained.bytes().to_vec();
    let restored = retain_workflow_for::<_, GovernedDocumentWorkflow>(
        restore_on_first_program(
            retained.clone(),
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .expect("current native symbol dictionary and Query checkpoint restore"),
        workflow_resources(),
    );
    assert_eq!(retained.bytes(), intact);
    let current_capability = restored
        .runtime()
        .installed_schema()
        .capability(
            worth_query_testkit_document_retention::facade::document_retention_model::workflow::WorkflowApprovalCapability::reference(),
            worth_query_testkit_document_retention::facade::document_retention_model::workflow::WorkflowAdvanceOperation::reference(),
        )
        .unwrap();
    assert_eq!(
        current_capability.identity().bytes(),
        &original_capability_identity
    );
    assert_ne!(
        current_capability.authority_identity(),
        original_installation_seal
    );
    drop(current_capability);

    let restored_instance = match reacquisition::read(&restored, &original_start) {
        WorthQueryWorkflowInstanceRead::Live {
            instance,
            observation,
        } => {
            assert_eq!(instance.entity_id(), original_instance);
            assert_eq!(instance.program_revision(), &original_revision);
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
                _ => panic!("fresh audited read must retain original admitted start actor"),
            }
            let scope = request_scope();
            let reviewer = authenticate_reviewer(restored.runtime().installed_schema(), &scope);
            let request = restored.runtime().request(&reviewer, &scope);
            let basis = request.retain_current_read().unwrap();
            let fresh = request
                .at(&basis)
                .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                    restored.vocabulary(),
                    &original_start,
                    reacquisition::input(),
                    reacquisition::input(),
                )
                .unwrap();
            let WorthQueryWorkflowInstanceRead::Live {
                observation: fresh, ..
            } = fresh
            else {
                panic!("fresh history basis must select the restored original instance");
            };
            let page = request
                .at(&basis)
                .read_workflow_history::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                    restored.vocabulary(),
                    fresh.target(),
                    reacquisition::input(),
                    reacquisition::input(),
                    16,
                    WorthQueryWorkflowIndexRead::PublishedOnly,
                    None,
                )
                .unwrap();
            assert!(page.continuation().is_none());
            let approved = page
                .entries()
                .iter()
                .find(|row| row.approval().is_some())
                .expect("original native approval survives restore");
            assert_eq!(
                approved.approval().unwrap().proposal_identity(),
                original_proposal
            );
            assert_eq!(
                approved.approval().unwrap().decision(),
                WorkflowApprovalDecision::Approve
            );
            match approved.audit() {
                WorthQueryWorkflowHistoryAudit::Visible {
                    approver: Some(actor),
                    ..
                } => assert_eq!(actor.entity_id(), expected_approver),
                _ => panic!(
                    "actual performed approver remains canonical after last old-runtime pin drops"
                ),
            }
            instance
        }
        _ => panic!("original native start must be reacquired under fresh read authority"),
    };
    // Preparing and performing this actual guarded operation reads the stored
    // five envelopes and readmits principal/grant/support under a fresh request.
    let receipt =
        super::instance_migration::perform_approved_effect(&restored, &restored_instance, 98_020);
    assert!(receipt
        .committed_changes()
        .entity_changes()
        .next()
        .is_some());
    assert_eq!(
        read_retention(restored.runtime(), restored_instance.branch()),
        8
    );
}

#[test]
fn cancelled_workflow_readmits_the_actual_ending_principal_with_fresh_audit_authority() {
    use worth_query_host::facade::application_discovery::{
        WorkflowEndingActorProvenance, WorthQueryWorkflowAudit,
        WorthQueryWorkflowHistoricalPosture, WorthQueryWorkflowInstanceRead,
        WorthQueryWorkflowObservedState,
    };
    use worth_query_host::facade::application_entry::WorkflowInstanceCancellationOutcome;
    use worth_query_testkit_document_retention::facade::document_retention_model::workflow::{
        WorkflowInstanceStartInput, WorkflowInstanceStartIntent,
    };
    let application = retain_workflow_for::<_, GovernedDocumentWorkflow>(
        publish_on_first_program(),
        workflow_resources(),
    );
    let definition = match publish_definition(
        &application,
        governed_reviewed_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        98_100,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("genuine definition must publish: {other:?}"),
    };
    let start = prepare_instance_start(&application, definition, 98_101).unwrap();
    let original = start.discovery_intent();
    let instance = match start.execute() {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("genuine instance must start: {other:?}"),
    };
    let expected = {
        let scope = request_scope();
        let runtime = application.runtime();
        let principal = authenticate_operator(runtime.installed_schema(), &scope);
        let cancelled = runtime
            .request(&principal, &scope)
            .on_branch(instance.branch())
            .mutate(WorkflowInstanceStartIntent {
                input: WorkflowInstanceStartInput {
                    document_identity: DOCUMENT_IDENTITY.to_owned(),
                },
            })
            .without_source()
            .idempotency(&98_102_u64)
            .prepare_workflow_instance_cancellation(&application, instance)
            .unwrap()
            .execute();
        let WorkflowInstanceCancellationOutcome::Cancelled(performed) = cancelled else {
            panic!("the actual admitted cancellation must publish: {cancelled:?}");
        };
        let actor = performed.receipt().principal_scope().principal();
        [
            u64::from(actor.partition_id()),
            actor.local_slot(),
            u64::from(actor.generation()),
        ]
    };
    let checkpoint = application
        .runtime()
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    drop(application);
    let restored = retain_workflow_for::<_, GovernedDocumentWorkflow>(
        restore_on_first_program(
            checkpoint,
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .unwrap(),
        workflow_resources(),
    );
    // This fixture admits a fresh reader and also proves the ungranted reader refuses.
    let found = reacquisition::read(&restored, &original);
    let WorthQueryWorkflowInstanceRead::Historical {
        posture: WorthQueryWorkflowHistoricalPosture::Ended,
        observation,
    } = found
    else {
        panic!("the same original intent must select the ended instance");
    };
    assert_eq!(
        observation.state(),
        WorthQueryWorkflowObservedState::Cancelled
    );
    match observation.audit() {
        WorthQueryWorkflowAudit::Visible {
            end: Some(WorkflowEndingActorProvenance::Principal(actor)),
            ..
        } => {
            assert_eq!(
                [
                    u64::from(actor.partition_id.0),
                    actor.local_slot.0,
                    u64::from(actor.generation.0)
                ],
                expected
            );
            let provenance = WorkflowEndingActorProvenance::Principal(*actor);
            assert_eq!(
                WorkflowEndingActorProvenance::from_recorded_bytes(
                    provenance.recorded_bytes().as_bytes()
                ),
                Ok(provenance)
            );
        }
        WorthQueryWorkflowAudit::Visible {
            end: Some(WorkflowEndingActorProvenance::HomeWorldOwner(_)),
            ..
        } => panic!("the cancelling actor was the admitted principal"),
        WorthQueryWorkflowAudit::Visible { end: None, .. } => {
            panic!("an ended instance requires complete ending provenance")
        }
        WorthQueryWorkflowAudit::Withheld => {
            panic!("the fresh independent audit grant must disclose the actual ending actor")
        }
    }
}
