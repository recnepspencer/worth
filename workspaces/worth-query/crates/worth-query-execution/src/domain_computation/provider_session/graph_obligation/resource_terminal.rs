//! Resource completion never carries a Query read review or Query receipt.
use worth_query_admission::{
    facade::graph_obligation::WorthQueryGraphWorkPlanIdentity,
    integration::WorthQueryExecutionCapacityReleaseReceipt,
};
use worth_query_installation::facade::WorthQueryInstalledResourceAccessContract;
use {
    crate::domain_computation::vocabulary::primary_graph::basis_registry::WorthQueryApplicationBasisIdentity,
    crate::domain_computation::vocabulary::primary_graph::basis_registry::WorthQueryApplicationBasisReleaseReceipt,
};
use {
    crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkManagedRunIdentity,
    crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity,
};

#[expect(
    dead_code,
    reason = "opaque completion retains its admitted plan, contract and release receipts"
)]
pub(in crate::domain_computation) struct WorthQueryResourceReadCompletion {
    pub(super) session: WorthQueryGraphWorkSessionIdentity,
    pub(super) managed_run: WorthQueryGraphWorkManagedRunIdentity,
    pub(super) plan: WorthQueryGraphWorkPlanIdentity,
    pub(super) contract: std::sync::Arc<WorthQueryInstalledResourceAccessContract>,
    pub(super) basis: WorthQueryApplicationBasisIdentity,
    pub(super) basis_release: WorthQueryApplicationBasisReleaseReceipt,
    pub(super) release: WorthQueryExecutionCapacityReleaseReceipt,
}
