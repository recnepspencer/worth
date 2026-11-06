//! Atomic ownership of the accepted convergence report and incumbent set.

mod admission;
mod incumbent_transition;
mod prepared_commit;

use {
    super::super::super::WorthQueryBoundConvergenceReport,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryRetainedConvergenceCandidateEvidence,
};

pub(in crate::domain_computation::convergence_epoch::iteration_owner) use admission::{
    admit_assessed_domain_report, WorthQueryConvergenceReportAdmissionFailure,
};
pub(in crate::domain_computation::convergence_epoch::iteration_owner::core) use prepared_commit::{
    ReportHistoryLifecycleEvent, ReportHistoryLifecycleEventKind,
    WorthQueryPreparedConvergenceReportCommit,
};

pub(super) struct WorthQueryConvergenceReportHistory {
    incumbents: Vec<WorthQueryRetainedConvergenceCandidateEvidence>,
    latest_report: Option<WorthQueryBoundConvergenceReport>,
}

impl WorthQueryConvergenceReportHistory {
    pub(super) const fn empty() -> Self {
        Self {
            incumbents: Vec::new(),
            latest_report: None,
        }
    }

    pub(super) fn incumbents(&self) -> &[WorthQueryRetainedConvergenceCandidateEvidence] {
        &self.incumbents
    }

    pub(super) fn latest_report(&self) -> Option<&WorthQueryBoundConvergenceReport> {
        self.latest_report.as_ref()
    }
}
