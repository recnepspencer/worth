//! Resource completion never carries a Query read review or Query receipt.
use super::{WorthQueryGraphWorkManagedRunIdentity, WorthQueryGraphWorkSessionIdentity};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisIdentity, WorthQueryApplicationBasisReleaseReceipt,
};
use worth_query_admission::{
    facade::graph_obligation::WorthQueryGraphWorkPlanIdentity,
    integration::WorthQueryExecutionCapacityReleaseReceipt,
};
use worth_query_installation::facade::WorthQueryInstalledResourceAccessContract;

pub(in crate::domain_computation) struct WorthQueryResourceReadCompletion {
    pub(super) session: WorthQueryGraphWorkSessionIdentity,
    pub(super) managed_run: WorthQueryGraphWorkManagedRunIdentity,
    pub(super) plan: WorthQueryGraphWorkPlanIdentity,
    pub(super) contract: std::sync::Arc<WorthQueryInstalledResourceAccessContract>,
    pub(super) basis: WorthQueryApplicationBasisIdentity,
    pub(super) basis_release: WorthQueryApplicationBasisReleaseReceipt,
    pub(super) release: WorthQueryExecutionCapacityReleaseReceipt,
}

impl WorthQueryResourceReadCompletion {
    pub(in crate::domain_computation) const fn session_identity(
        &self,
    ) -> WorthQueryGraphWorkSessionIdentity {
        self.session
    }

    pub(in crate::domain_computation) fn purpose(
        &self,
    ) -> worth_query_installation::facade::WorthQueryResourceAccessPurpose {
        self.contract.purpose()
    }
}
