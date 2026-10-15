//! Genuine approval storage followed by cold current-format authority readmission.
use super::super::document_retention_model::{
    host::restore_on_first_program, programs::validated_second_program, workflow::retain_workflow,
};
use super::*;
use worth_query_host::facade::application_installation::{
    WorthQueryApplicationCheckpoint, WorthQueryApplicationProgramRoster,
    WorthQueryCheckpointCapturePolicy,
};

#[test]
fn approval_field_text_survives_current_checkpoint_and_fresh_authority_readmission() {
    let (application, definition, instance, proposal, required, _) =
        approval_journey("field-text-restored", 98_000);
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
    let restored = retain_workflow(
        restore_on_first_program(
            WorthQueryApplicationCheckpoint::from_untrusted_bytes(checkpoint),
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .expect("current native symbol dictionary and Query checkpoint restore"),
    );
    let rebuilt_definition = definition.held_on(restored.runtime().current_world());
    let restored_instance = match start_instance(&restored, rebuilt_definition, 98_001).unwrap() {
        WorkflowInstanceStartOutcome::Started(performed) => {
            assert!(performed.replayed());
            performed.instance().clone()
        }
        other => panic!("the original native start reacquires its restored instance: {other:?}"),
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
