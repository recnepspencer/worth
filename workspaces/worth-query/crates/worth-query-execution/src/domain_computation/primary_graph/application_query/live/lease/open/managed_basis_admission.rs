use std::sync::atomic::{AtomicU64, Ordering};

use super::super::super::outcome::{
    WorthQueryApplicationLiveOpenDenial, WorthQueryApplicationLiveOpenDenialKind,
};
use super::super::validation::open_denial;
use crate::domain_computation::{
    managed_run::{
        admit_managed_lower_execution_basis, WorthQueryManagedLowerBinding,
        WorthQueryManagedLowerExecutionBasis, WorthQueryManagedTruthReadRequest,
    },
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
};

static NEXT_APPLICATION_QUERY_LIVE_LEASE: AtomicU64 = AtomicU64::new(1);

pub(super) fn admit_live_managed_basis<Schema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    live: &worth_query_installation::facade::WorthQueryInstalledApplicationLiveContract,
    graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    subject: &str,
) -> Result<WorthQueryManagedLowerExecutionBasis, WorthQueryApplicationLiveOpenDenial> {
    let descriptor = graph_work
        .query_basis()
        .map(|basis| basis.descriptor().clone())
        .ok_or_else(|| {
            open_denial(
                WorthQueryApplicationLiveOpenDenialKind::ProviderVersionUnavailable,
                subject,
            )
        })?;
    let attempt = NEXT_APPLICATION_QUERY_LIVE_LEASE.fetch_add(1, Ordering::Relaxed);
    let attempt_identity = format!("application-query-live:{attempt}");
    let binding =
        WorthQueryManagedLowerBinding::new(subject, &attempt_identity, live.resource_envelope());
    let request = WorthQueryManagedTruthReadRequest::new(
        descriptor,
        worth_runtime_bridge::facade::SnapshotReadPacket::new(Vec::new()),
    );
    let request_bridge = application.bridge.ordinary().fork_managed_request_lane();
    admit_managed_lower_execution_basis(
        &request_bridge,
        &application.product_runtime.source,
        binding,
        request,
    )
    .map_err(|failure| {
        open_denial(
            WorthQueryApplicationLiveOpenDenialKind::BridgeBasisRejected,
            failure.detail.as_ref(),
        )
    })
}
