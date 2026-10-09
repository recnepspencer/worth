use super::{aspect_touch, installed_builder, measurement_value_path, settled_identity_count};
use crate::installed_domain::measurement_recording::{
    WorthUiMeasurementRecording, WorthUiMeasurementRecordingFamily, IDENTIFY_STAGE, RECORD_STAGE,
};
use crate::installed_domain::snapshot_measurement::{
    WorthUiSnapshotMeasurement, WorthUiSnapshotMeasurementFamily,
};
use crate::WorthUiQueryWorkspaceExt;
use worth_foundational::facade::{AspectValue, CanonicalF32};
use worth_query::facade::domain;
#[test]
fn registered_snapshot_and_recording_workflow_execute_real_query_mechanics() {
    let mut workspace = installed_builder()
        .workspace("worth-ui-installed-operations")
        .expect("Worth UI operation executors should install");
    let installed = workspace
        .worth_ui()
        .expect("Worth UI domain should install");
    let recording = workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(WorthUiMeasurementRecordingFamily)
        .bind(installed.handle(), WorthUiMeasurementRecording)
        .expect("measurement recording should bind")
        .admit_workflow_resources(
            crate::installed_domain::execution_resources::operation_execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            IDENTIFY_STAGE,
            domain::WorthQueryWorkflowValue::Text("measurement-17".into()),
            &mut workspace,
        )
        .unwrap()
        .advance(
            RECORD_STAGE,
            domain::WorthQueryWorkflowValue::U64(u64::from(CanonicalF32::from_f32(42.0).bits())),
            &mut workspace,
        )
        .unwrap()
        .complete()
        .unwrap();
    assert_eq!(recording.stage_receipts().len(), 2);
    let effects = recording.stage_receipts()[1].effect_evidence();
    assert_eq!(effects.len(), 1);
    assert_eq!(
        effects[0].family(),
        domain::WorthQueryOperationEffectFamily::Mutation
    );
    let mutation_receipt = effects[0]
        .mutation_receipt()
        .expect("the declared mutation effect must retain Query's write receipt");
    let recorded_touches = mutation_receipt
        .declared_aspect_operations()
        .iter()
        .map(|operation| operation.aspect_touch())
        .collect::<Vec<_>>();
    assert_eq!(recorded_touches.len(), 2);
    assert!(recorded_touches.contains(&&aspect_touch("identity.id")));
    assert!(recorded_touches.contains(&&aspect_touch("measurement.value")));

    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(WorthUiSnapshotMeasurementFamily)
        .bind(installed.handle(), WorthUiSnapshotMeasurement)
        .expect("snapshot measurement should bind");
    let consumer = bound
        .consumer_projection_contract()
        .expect("snapshot operation should mint one consumer contract");
    let settled = bound
        .admit_execution_resources(
            (),
            crate::installed_domain::execution_resources::operation_execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(
            consumer,
            worth_query::facade::read::project_facts()
                .entity_identities()
                .display_field(measurement_value_path()),
        )
        .unwrap()
        .settle()
        .unwrap();
    assert_eq!(settled.authority().facts().entity_identities().len(), 1);
    let display_fields = settled.authority().facts().display_fields();
    assert_eq!(display_fields.len(), 1);
    assert_eq!(
        display_fields[0].native_value().scalar(),
        Some(&AspectValue::Float32(CanonicalF32::from_f32(42.0)))
    );
    assert_eq!(settled.counters().primary_read_contacts, 1);
}

#[test]
fn recording_workflow_rejects_invalid_value_without_a_partial_write() {
    let mut workspace = installed_builder()
        .workspace("worth-ui-recording-atomic-denial")
        .unwrap();
    let installed = workspace.worth_ui().unwrap();
    let run = workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(WorthUiMeasurementRecordingFamily)
        .bind(installed.handle(), WorthUiMeasurementRecording)
        .unwrap()
        .admit_workflow_resources(
            crate::installed_domain::execution_resources::operation_execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            IDENTIFY_STAGE,
            domain::WorthQueryWorkflowValue::Text("must-not-commit".into()),
            &mut workspace,
        )
        .unwrap();

    assert!(!run
        .advance(
            RECORD_STAGE,
            domain::WorthQueryWorkflowValue::U64(u64::MAX),
            &mut workspace,
        )
        .is_success());
    assert_eq!(settled_identity_count(&mut workspace), 0);
}
