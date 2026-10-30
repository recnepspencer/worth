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
        plan: WorthQueryGraphWorkPlanIdentity,
        target: &std::sync::Arc<WorthQueryApplicationInspectionTarget>,
    ) -> bool {
        self.session == session && self.plan == plan && std::sync::Arc::ptr_eq(&self.target, target)
    }

    pub(in crate::domain_computation) const fn session_identity(
        &self,
    ) -> WorthQueryGraphWorkSessionIdentity {
        self.session
    }
}

/// Preserve the original request stop separately from session affinity refusals.
#[derive(Debug)]
pub(in crate::domain_computation) enum WorthQueryInspectionReadTerminalDenial {
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
