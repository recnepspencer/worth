use worth_foundational::facade::{AspectValue, CanonicalF32};
use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use worth_query::facade::runtime;
use worth_ui_query_binding::{
    worth_ui_domain_package, worth_ui_native_aspect_contracts, WorthUiQueryAllocationDetail,
    WorthUiQueryConsumerRequirements, WorthUiQueryDenialPresentation,
    WorthUiQueryInspectionRelevance, WorthUiQueryViewShape,
};

pub(super) fn installed_measurement_workspace(label: &str) -> runtime::WorthQueryWorkspace {
    installed_measurement_workspace_with(label, false)
}

pub(super) fn partial_measurement_workspace(label: &str) -> runtime::WorthQueryWorkspace {
    installed_measurement_workspace_with(label, true)
}

pub(super) fn unsupported_measurement_workspace(label: &str) -> runtime::WorthQueryWorkspace {
    crate::request_custody::with_bootstrap_request(|execution| {
        worth_ui_query_binding::install_worth_ui_test_operation_executors(
        measurement_runtime_builder().consumer_support_posture(
            worth_query::facade::domain::WorthQueryConsumerSupportDimension::ProjectionConsumption,
            worth_query::facade::domain::WorthQueryConsumerSupportPosture::Unsupported,
        ),
    )
    .workspace(execution, label)
    .expect("unsupported Query workspace")
    })
}

fn installed_measurement_workspace_with(
    label: &str,
    partial: bool,
) -> runtime::WorthQueryWorkspace {
    let builder = operation_live_support(measurement_runtime_builder());
    let builder = if partial {
        worth_ui_query_binding::install_worth_ui_partial_test_operation_executors(builder)
    } else {
        worth_ui_query_binding::install_worth_ui_test_operation_executors(builder)
    };
    let mut workspace = crate::request_custody::with_bootstrap_request(|execution| {
        builder
            .workspace(execution, label)
            .expect("installed Query workspace")
    });
    insert_measurement(&mut workspace);
    workspace
}

fn operation_live_support(
    builder: worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder,
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    use worth_query::facade::domain::{
        WorthQueryConsumerSupportDimension as Dimension,
        WorthQueryConsumerSupportPosture as Posture,
    };
    [
        Dimension::Live,
        Dimension::Sharing,
        Dimension::Invalidation,
        Dimension::DependencyImpact,
        Dimension::CollectionDelivery,
    ]
    .into_iter()
    .fold(builder, |builder, dimension| {
        builder.consumer_support_posture(dimension, Posture::Supported)
    })
}

fn measurement_runtime_builder(
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let schema = WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
        .aspect_contracts(worth_ui_native_aspect_contracts())
        .expect("native contracts")
        .aspect("identity.id", "identity.id")
        .expect("identity aspect")
        .aspect("measurement.value", "measurement.value")
        .expect("measurement aspect");
    in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(worth_ui_domain_package())
}

fn insert_measurement(workspace: &mut runtime::WorthQueryWorkspace) {
    workspace
        .with_execution_request_mut(|execution, workspace| {
            workspace
                .insert(execution, "WorthUiMeasurement", |measurement| {
                    measurement
                        .set_aspect(
                            runtime::WorthQueryAspectTouch::from_authoring_ingress_text(
                                "identity.id",
                            )
                            .expect("identity touch"),
                            runtime::WorthQueryAuthoredAspectValue::string("measurement"),
                        )
                        .set_aspect(
                            runtime::WorthQueryAspectTouch::from_authoring_ingress_text(
                                "measurement.value",
                            )
                            .expect("measurement touch"),
                            runtime::WorthQueryAuthoredAspectValue::native(AspectValue::Float32(
                                CanonicalF32::from_f32(240.0),
                            )),
                        )
                })
                .expect("measurement insertion")
        })
        .expect("workspace host call opens");
}

pub(super) fn interactive_borrowed_collection_requirements() -> WorthUiQueryConsumerRequirements {
    WorthUiQueryConsumerRequirements::new(
        worth_query::facade::domain::WorthQueryConsumerBoundaryRequirements {
            presentation:
                worth_query::facade::domain::WorthQueryConsumerPresentationPosture::Interactive,
            allocation: worth_query::facade::domain::WorthQueryConsumerAllocationPosture::Borrowed,
        },
        WorthUiQueryAllocationDetail::BorrowedFactSlice,
        WorthUiQueryViewShape::Collection,
        WorthUiQueryDenialPresentation::StructuredStatus,
        WorthUiQueryInspectionRelevance::Relevant,
    )
}
