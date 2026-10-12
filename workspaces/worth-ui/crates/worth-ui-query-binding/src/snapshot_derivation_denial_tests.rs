use worth_query::facade::{
    consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema},
    domain,
    installed::operation,
    read::{
        ProjectionFactExtractionError, WorthQueryProjectionOutcome, WorthQueryProjectionUnavailable,
    },
    runtime,
};

use crate::{
    worth_ui_domain_package, WorthUiQueryAllocationDetail, WorthUiQueryBindingPlan,
    WorthUiQueryConsumerRequirements, WorthUiQueryDenialPresentation,
    WorthUiQueryInspectionRelevance, WorthUiQueryViewShape, WorthUiQueryWorkspaceExt,
    WorthUiSnapshotProjectionConsumptionOutcome,
};

#[test]
fn absent_required_measurement_stops_at_query_consumption() {
    let mut workspace = workspace_with_identity_only("absent-required-measurement");
    let reference = installed_reference(&workspace);
    let outcome = reference
        .enter_snapshot_attempt(&workspace)
        .unwrap()
        .prepare_snapshot_consumer(requirements())
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume();
    let WorthUiSnapshotProjectionConsumptionOutcome::Failed(stop) = outcome else {
        panic!("an absent required value must fail at Query projection consumption")
    };
    let operation::WorthQueryProgressionDenial::Projection(outcome) = *stop else {
        panic!("consumption failure must retain Query's projection stop")
    };
    let WorthQueryProjectionOutcome::Unavailable(WorthQueryProjectionUnavailable::Extraction(
        ProjectionFactExtractionError::MissingRequiredNativeFact { contract_key, .. },
    )) = *outcome
    else {
        panic!("absence must retain Query's exact missing-required-native-fact evidence")
    };

    assert_eq!(
        contract_key,
        worth_foundational::facade::AspectKey::new("measurement").unwrap()
    );
}

#[test]
fn invalid_null_and_non_float_values_stop_at_query_mutation_contract() {
    let denials = [runtime::WorthQueryAuthoredAspectValue::null(), "240".into()]
        .into_iter()
        .map(invalid_measurement_write)
        .collect::<Vec<_>>();

    assert_eq!(denials.len(), 2);
    assert_ne!(denials[0], denials[1]);
}

fn invalid_measurement_write(value: runtime::WorthQueryAuthoredAspectValue) -> String {
    let mut workspace =
        crate::certification::request_custody::with_bootstrap_request(|execution| {
            installed_builder()
                .workspace(execution, "invalid-measurement-shape")
                .unwrap()
        });
    let error = workspace
        .with_execution_request_mut(|execution, workspace| {
            workspace
                .insert(execution, "WorthUiMeasurement", |measurement| {
                    measurement
                        .set_aspect(identity_touch(), "invalid-measurement")
                        .set_aspect(measurement_touch(), value)
                })
                .expect_err("Query must reject a value outside the installed Float32 contract")
        })
        .expect("workspace host call opens");
    let runtime::WorthQueryRuntimeError::MutationContractDenied(denial) = error else {
        panic!("invalid native shape must retain Query's mutation-contract denial")
    };
    assert_eq!(denial.touch(), &measurement_touch());
    denial.detail().to_owned()
}

fn workspace_with_identity_only(label: &str) -> runtime::WorthQueryWorkspace {
    let mut workspace =
        crate::certification::request_custody::with_bootstrap_request(|execution| {
            installed_builder().workspace(execution, label).unwrap()
        });
    workspace
        .with_execution_request_mut(|execution, workspace| {
            workspace
                .insert(execution, "WorthUiMeasurement", |measurement| {
                    measurement.set_aspect(identity_touch(), "measurement-without-value")
                })
                .unwrap()
        })
        .expect("workspace host call opens");
    workspace
}

fn installed_reference(
    workspace: &runtime::WorthQueryWorkspace,
) -> crate::WorthUiInstalledQueryBindingReference {
    let view = workspace
        .worth_ui()
        .unwrap()
        .measurement_view("dashboard.measurements")
        .unwrap();
    let identity = view.definition().identity().clone();
    WorthUiQueryBindingPlan::default()
        .register_view(view)
        .unwrap()
        .resolve_definition(&identity, WorthUiQueryViewShape::Collection)
        .unwrap()
}

fn requirements() -> WorthUiQueryConsumerRequirements {
    WorthUiQueryConsumerRequirements::new(
        domain::WorthQueryConsumerBoundaryRequirements {
            presentation: domain::WorthQueryConsumerPresentationPosture::Interactive,
            allocation: domain::WorthQueryConsumerAllocationPosture::Borrowed,
        },
        WorthUiQueryAllocationDetail::BorrowedFactSlice,
        WorthUiQueryViewShape::Collection,
        WorthUiQueryDenialPresentation::StructuredStatus,
        WorthUiQueryInspectionRelevance::Relevant,
    )
}

fn installed_builder() -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let schema = WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
        .aspect_contracts(crate::worth_ui_native_aspect_contracts())
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap()
        .aspect("measurement.value", "measurement.value")
        .unwrap();
    crate::install_worth_ui_test_operation_executors(
        in_memory_test_runtime()
            .with_schema(schema)
            .domain_package(worth_ui_domain_package()),
    )
}

fn identity_touch() -> runtime::WorthQueryAspectTouch {
    runtime::WorthQueryAspectTouch::from_authoring_ingress_text("identity.id").unwrap()
}

fn measurement_touch() -> runtime::WorthQueryAspectTouch {
    runtime::WorthQueryAspectTouch::from_authoring_ingress_text("measurement.value").unwrap()
}
