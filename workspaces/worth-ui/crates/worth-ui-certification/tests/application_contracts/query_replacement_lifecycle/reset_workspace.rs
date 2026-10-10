use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use worth_query::facade::{foundation::WorthQueryEntityIdentity, runtime};
use worth_ui_query_binding::{worth_ui_domain_package, worth_ui_native_aspect_contracts};

pub(crate) fn installed_workspace_without_collection_entity_lookup(
    label: &str,
) -> (runtime::WorthQueryWorkspace, WorthQueryEntityIdentity) {
    let schema = WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
        .aspect_contracts(worth_ui_native_aspect_contracts())
        .expect("native contracts")
        .aspect("identity.id", "identity.id")
        .expect("identity aspect")
        .aspect("measurement.value", "measurement.value")
        .expect("measurement aspect");
    let builder = super::scenario::operation_live_support(
        in_memory_test_runtime()
            .with_schema(schema)
            .domain_package(worth_ui_domain_package()),
    )
    .without_collection_entity_lookup();
    let mut workspace = crate::request_custody::with_bootstrap_request(|execution| {
        worth_ui_query_binding::install_worth_ui_test_operation_executors(builder)
            .workspace(execution, label)
            .expect("installed reset-capable Query workspace")
    });
    let measurement = super::scenario::insert_measurement(&mut workspace);
    (workspace, measurement)
}
