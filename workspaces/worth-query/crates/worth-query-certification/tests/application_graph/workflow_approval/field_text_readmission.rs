//! Genuine approval storage followed by cold current-format authority readmission.
use super::super::document_retention_model::{
    host::{publish_on_first_program, restore_on_first_program},
    programs::validated_second_program,
    workflow::{
        governed_read::{AuditRead, GovernedDocumentWorkflow, SubjectRead},
        governed_reviewed_definition, prepare_instance_start, reacquisition, retain_workflow_for,
        workflow_resources,
    },
};
use super::*;
use worth_query_host::facade::application_installation::{
    WorthQueryApplicationCheckpoint, WorthQueryApplicationProgramRoster,
    WorthQueryCheckpointCapturePolicy,
};

#[test]
fn approval_field_text_survives_current_checkpoint_and_fresh_authority_readmission() {
    use crate::document_retention_model::operator_identity::authenticate_reviewer;
    use worth_query_host::facade::application_discovery::{
        WorthQueryWorkflowAudit, WorthQueryWorkflowHistoryAudit, WorthQueryWorkflowIndexRead,
        WorthQueryWorkflowInstanceRead,
    };
    let application = retain_workflow_for::<_, GovernedDocumentWorkflow>(
        publish_on_first_program(),
        workflow_resources(),
    );
    // Capture only descriptive identities; no old installed handle survives restore.
    let original_capability = application
        .runtime()
        .installed_schema()
        .capability(
            crate::document_retention_model::workflow::WorkflowApprovalCapability::reference(),
            crate::document_retention_model::workflow::WorkflowAdvanceOperation::reference(),
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
                .bytes()
                .to_vec()
        })
        .unwrap();
    // Neither the old request nor its native live ref supplies the restored authority.
    drop(performed);
    drop(required);
    drop(proposal);
    drop(application);
    let retained = WorthQueryApplicationCheckpoint::from_untrusted_bytes(checkpoint);
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
            crate::document_retention_model::workflow::WorkflowApprovalCapability::reference(),
            crate::document_retention_model::workflow::WorkflowAdvanceOperation::reference(),
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
