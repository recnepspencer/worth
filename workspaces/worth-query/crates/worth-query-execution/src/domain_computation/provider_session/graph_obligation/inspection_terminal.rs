//! Inspection completion carries the actual original target and its own release.
use crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionTarget;
use worth_query_admission::{
    facade::graph_obligation::WorthQueryGraphWorkPlanIdentity,
    integration::WorthQueryExecutionCapacityReleaseReceipt,
};
use {
    crate::domain_computation::vocabulary::primary_graph::basis_registry::WorthQueryApplicationBasisIdentity,
    crate::domain_computation::vocabulary::primary_graph::basis_registry::WorthQueryApplicationBasisReleaseReceipt,
};
use {
    crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkManagedRunIdentity,
    crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity,
};
mod court;

pub(in crate::domain_computation) struct WorthQueryInspectionReadCompletion {
    pub(super) session: WorthQueryGraphWorkSessionIdentity,
    pub(super) managed_run: WorthQueryGraphWorkManagedRunIdentity,
    pub(super) plan: WorthQueryGraphWorkPlanIdentity,
    pub(super) target: std::sync::Arc<WorthQueryApplicationInspectionTarget>,
    pub(super) basis: WorthQueryApplicationBasisIdentity,
    pub(super) basis_release: WorthQueryApplicationBasisReleaseReceipt,
    pub(super) release: WorthQueryExecutionCapacityReleaseReceipt,
}

impl WorthQueryInspectionReadCompletion {
    pub(in crate::domain_computation) fn matches_report(
        &self,
        session: WorthQueryGraphWorkSessionIdentity,
        managed_run: WorthQueryGraphWorkManagedRunIdentity,
        plan: WorthQueryGraphWorkPlanIdentity,
        target: &std::sync::Arc<WorthQueryApplicationInspectionTarget>,
        basis: &WorthQueryApplicationBasisIdentity,
        capacity_identity: worth_query_admission::integration::WorthQueryGraphCapacityReservationIdentity,
    ) -> bool {
        // Construction checks local basis custody. Admission owns the graph
        // reservation issuer, which only releases GraphWork with count one.
        // Match its exact identity rather than repeat the issuer invariant.
        self.session == session
            && self.managed_run == managed_run
            && self.plan == plan
            && std::sync::Arc::ptr_eq(&self.target, target)
            && self.basis == *basis
            && self.basis_release.identity() == basis
            && self.release.graph_reservation_identity() == Some(capacity_identity)
    }
}

/// Preserve the original request stop separately from session affinity refusals.
#[derive(Debug)]
pub enum WorthQueryInspectionReadTerminalDenial {
    Request(worth_execution::WorkCeilingDenial),
    Read(super::session::WorthQueryManagedGraphReadDenial),
}

impl From<super::session::WorthQueryManagedGraphReadDenial>
    for WorthQueryInspectionReadTerminalDenial
{
    fn from(cause: super::session::WorthQueryManagedGraphReadDenial) -> Self {
        Self::Read(cause)
    }
}
