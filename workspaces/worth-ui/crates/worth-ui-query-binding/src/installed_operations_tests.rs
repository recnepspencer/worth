use worth_foundational::facade::{AspectValue, CanonicalF32, CanonicalFieldPath, FieldKey};
use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use worth_query::facade::{domain, foundation};

use crate::installed_domain::{
    collection_text_projection::{
        WorthUiCollectionTextProjection, WorthUiCollectionTextProjectionExecutor,
        WorthUiCollectionTextProjectionFamily,
    },
    executor_registration::install_worth_ui_test_operation_executors,
    measurement_recording::{
        WorthUiMeasurementRecording, WorthUiMeasurementRecordingExecutor,
        WorthUiMeasurementRecordingFamily, IDENTIFY_STAGE, RECORD_STAGE,
    },
    scalar_text_projection::{
        WorthUiScalarTextProjection, WorthUiScalarTextProjectionExecutor,
        WorthUiScalarTextProjectionFamily,
    },
    snapshot_measurement::{
        snapshot_measurement_definition_with_value_alias, WorthUiSnapshotMeasurement,
        WorthUiSnapshotMeasurementFamily, WorthUiSnapshotMeasurementValueAliasExecutor,
    },
};
use crate::{
    worth_ui_domain_package, WorthUiQueryBindingPlan, WorthUiQueryOperationAttemptDenial,
    WorthUiQueryViewShape, WorthUiQueryWorkspaceExt,
};

#[path = "installed_operations_tests/reference_convergence.rs"]
mod reference_convergence;
#[path = "installed_operations_tests/semantic_contract.rs"]
mod semantic_contract;

#[test]
fn installed_snapshot_identity_converges_and_semantic_drift_changes_it() {
    let first_workspace = installed_builder()
        .workspace("worth-ui-operation-identity-first")
        .unwrap();
    let second_workspace = installed_builder()
        .workspace("worth-ui-operation-identity-second")
        .unwrap();
    let drifted_definition = snapshot_measurement_definition_with_value_alias("value");
    let drifted_identity = drifted_definition
        .clone()
        .into_portable()
        .canonical_identity()
        .to_owned();
    let drifted_installation_denial = match installed_builder_with_package(
        crate::domain_package::worth_ui_domain_package_with_snapshot_definition(
            drifted_definition.clone(),
        ),
    )
    .workspace("worth-ui-operation-identity-drifted")
    {
        Ok(_) => panic!("semantic drift cannot retain the old registered executor"),
        Err(denial) => denial,
    };

    let first = bound_snapshot(&first_workspace);
    let second = bound_snapshot(&second_workspace);
    let drifted_workspace = drifted_installed_builder_with_package(
        crate::domain_package::worth_ui_domain_package_with_snapshot_definition(drifted_definition),
    )
    .workspace("worth-ui-operation-identity-drifted-with-matching-executor")
    .expect("semantic drift with its exact executor should install");
    let drifted = bound_snapshot(&drifted_workspace);

    assert_eq!(
        first.definition().canonical_identity(),
        second.definition().canonical_identity()
    );
    assert_eq!(drifted.definition().canonical_identity(), drifted_identity);
    assert_ne!(
        first.definition().canonical_identity(),
        drifted.definition().canonical_identity()
    );
    assert_eq!(
        drifted_installation_denial.kind(),
        worth_query::facade::consumer_kit::WorthQueryTestBackendErrorKind::WorkspaceBuildFailed
    );
    assert!(drifted_installation_denial
        .message()
        .contains("executor read declaration disagrees with installed canonical semantics"));
}

#[test]
fn missing_snapshot_executor_denies_runtime_construction_before_ui_authority_exists() {
    let denial = match base_builder().workspace("worth-ui-missing-operation-executor") {
        Ok(_) => panic!("Query admitted an installed operation without its exact executor"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        worth_query::facade::consumer_kit::WorthQueryTestBackendErrorKind::WorkspaceBuildFailed
    );
    assert!(denial
        .message()
        .contains("installed operation and executor registration sets differ"));
}

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

#[test]
fn gateway_rejects_a_foreign_world_before_binding_or_execution() {
    let owner = installed_builder()
        .workspace("worth-ui-gateway-owner")
        .expect("owner workspace should build");
    let foreign = installed_builder()
        .workspace("worth-ui-gateway-foreign")
        .expect("foreign workspace should build");
    let view = owner
        .worth_ui()
        .expect("owner domain should install")
        .measurement_view("dashboard.measurements")
        .expect("snapshot view should only validate UI declaration meaning");
    let view_identity = view.definition().identity().clone();
    let reference = WorthUiQueryBindingPlan::default()
        .register_view(view)
        .expect("owner view should register")
        .resolve_definition(&view_identity, WorthUiQueryViewShape::Collection)
        .expect("registered definition should resolve");

    assert!(matches!(
        reference.enter_snapshot_attempt(&foreign),
        Err(WorthUiQueryOperationAttemptDenial::InstalledDomainAuthorityMismatch)
    ));
    let bound = reference
        .enter_snapshot_attempt(&owner)
        .expect("exact owner world should admit")
        .bind_snapshot()
        .expect("gateway should bind the installed snapshot operation");
    assert!(!bound.binding_identity().is_empty());
}

fn base_builder() -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let schema = WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
        .aspect_contracts(crate::worth_ui_native_aspect_contracts())
        .expect("Worth UI native aspect contracts should admit")
        .aspect("identity.id", "identity.id")
        .expect("identity aspect should admit")
        .aspect("measurement.value", "measurement.value")
        .expect("measurement aspect should admit");
    in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(worth_ui_domain_package())
}

pub(crate) fn installed_builder(
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    install_worth_ui_test_operation_executors(base_builder())
}

fn installed_builder_with_package(
    package: domain::WorthQueryDomainPackage<crate::WorthUiDomainEntry>,
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let schema = WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
        .aspect_contracts(crate::worth_ui_native_aspect_contracts())
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap()
        .aspect("measurement.value", "measurement.value")
        .unwrap();
    install_worth_ui_test_operation_executors(
        in_memory_test_runtime()
            .with_schema(schema)
            .domain_package(package),
    )
}

fn drifted_installed_builder_with_package(
    package: domain::WorthQueryDomainPackage<crate::WorthUiDomainEntry>,
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let schema = WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
        .aspect_contracts(crate::worth_ui_native_aspect_contracts())
        .expect("Worth UI native aspect contracts should admit")
        .aspect("identity.id", "identity.id")
        .expect("identity aspect should admit")
        .aspect("measurement.value", "measurement.value")
        .expect("measurement aspect should admit");
    in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(package)
        .domain_operation_executor(
            crate::WorthUiDomainEntry,
            WorthUiSnapshotMeasurement,
            WorthUiSnapshotMeasurementFamily,
            WorthUiSnapshotMeasurementValueAliasExecutor,
        )
        .domain_operation_executor(
            crate::WorthUiDomainEntry,
            WorthUiScalarTextProjection,
            WorthUiScalarTextProjectionFamily,
            WorthUiScalarTextProjectionExecutor,
        )
        .domain_operation_executor(
            crate::WorthUiDomainEntry,
            WorthUiCollectionTextProjection,
            WorthUiCollectionTextProjectionFamily,
            WorthUiCollectionTextProjectionExecutor,
        )
        .workflow_stage_executor(
            crate::WorthUiDomainEntry,
            WorthUiMeasurementRecording,
            WorthUiMeasurementRecordingFamily,
            WorthUiMeasurementRecordingExecutor,
        )
}

pub(crate) fn bound_snapshot(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
) -> crate::WorthUiBoundSnapshotMeasurement<foundation::ObservationLaneWitness> {
    let installed = workspace.worth_ui().unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(WorthUiSnapshotMeasurementFamily)
        .bind(installed.handle(), WorthUiSnapshotMeasurement)
        .unwrap()
}

fn settled_identity_count(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> usize {
    let bound = bound_snapshot(workspace);
    let consumer = bound.consumer_projection_contract().unwrap();
    bound
        .admit_execution_resources(
            (),
            crate::installed_domain::execution_resources::operation_execution_resource_request(),
            workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(
            consumer,
            worth_query::facade::read::project_facts().entity_identities(),
        )
        .unwrap()
        .settle()
        .unwrap()
        .authority()
        .facts()
        .entity_identities()
        .len()
}

pub(crate) fn aspect_touch(value: &str) -> worth_query::facade::runtime::WorthQueryAspectTouch {
    worth_query::facade::runtime::WorthQueryAspectTouch::from_authoring_ingress_text(value)
        .expect("static Worth UI aspect touch must admit")
}

fn measurement_value_path() -> foundation::ProjectionFactFieldPath {
    foundation::ProjectionFactFieldPath::from_canonical_field_path(
        CanonicalFieldPath::new([
            FieldKey::new("measurement").expect("static aspect path must admit"),
            FieldKey::new("value").expect("static field path must admit"),
        ])
        .expect("static measurement field path must admit"),
    )
}
