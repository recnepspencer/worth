//! Descriptive diagnostics derive solely from the actual native physical receipt.
pub(super) fn describe(
    receipt: &worth_query_host::facade::runtime::WorthQueryDirectRunCleanupReceipt,
) -> super::super::protocol::BankHttpNativeCleanupDescription {
    let inspection = receipt.inspection();
    super::super::protocol::BankHttpNativeCleanupDescription {
        logical_run_identity: inspection.logical_run_identity().into(),
        run_identity: inspection.run_identity().into(),
        provider_session_identity: inspection.provider_session_identity().into(),
        resources_released: inspection.resources_released(),
    }
}

pub(super) fn failure(
    cause: &worth_query_host::facade::primary_graph::WorthQueryMutationCleanupFailure,
) -> super::super::protocol::BankHttpNativeCleanupFailureDescription {
    super::super::protocol::BankHttpNativeCleanupFailureDescription {
        run_identity: cause.run_identity().into(),
        graph_session_identity: format!("{:?}", cause.session_identity()),
        failure: format!("{:?}", cause.kind()),
    }
}
