//! Inspection completion carries the actual original target and its own release.
use super::{WorthQueryGraphWorkManagedRunIdentity, WorthQueryGraphWorkSessionIdentity};
use crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionTarget;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisIdentity, WorthQueryApplicationBasisReleaseReceipt,
};
use worth_query_admission::{
    facade::graph_obligation::WorthQueryGraphWorkPlanIdentity,
    integration::WorthQueryExecutionCapacityReleaseReceipt,
};
mod court;

pub(in crate::domain_computation) struct WorthQueryInspectionReadCompletion {
    pub(super) session: WorthQueryGraphWorkSessionIdentity,
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    pub(super) managed_run: WorthQueryGraphWorkManagedRunIdentity,
    pub(super) plan: WorthQueryGraphWorkPlanIdentity,
    pub(super) target: std::sync::Arc<WorthQueryApplicationInspectionTarget>,
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    pub(super) basis: WorthQueryApplicationBasisIdentity,
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    pub(super) basis_release: WorthQueryApplicationBasisReleaseReceipt,
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
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
