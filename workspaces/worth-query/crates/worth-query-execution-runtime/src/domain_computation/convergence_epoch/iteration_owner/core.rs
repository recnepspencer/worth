//! Epoch state carried intact across direct and workflow iteration phases.

mod lifecycle;
mod report_history;

use std::sync::Arc;

use worth_query_admission::facade::domain_computation::WorthQueryAdmittedConvergenceContract;

use crate::execution_digest::hash_parts;

use super::super::{
    WorthQueryBoundConvergenceReport, WorthQueryRetainedConvergenceCandidateEvidence,
};

pub use lifecycle::WorthQueryConvergenceEpochCounters;
pub use lifecycle::{
    WorthQueryConvergenceEpochLifecycle, WorthQueryConvergenceLifecycleEvent,
};
pub use report_history::{
    admit_assessed_domain_report, WorthQueryConvergenceReportAdmissionFailure,
};
use report_history::{
    WorthQueryConvergenceReportHistory, WorthQueryPreparedConvergenceReportCommit,
};

pub struct WorthQueryConvergenceEpochCore {
    identity: Arc<str>,
    logical_run_identity: Arc<str>,
    contract: WorthQueryAdmittedConvergenceContract,
    lifecycle: WorthQueryConvergenceEpochLifecycle,
    report_history: WorthQueryConvergenceReportHistory,
}

impl WorthQueryConvergenceEpochCore {
    pub(super) fn new(
        operation_binding_identity: &str,
        logical_run_identity: &str,
        managed_run_identity: &str,
        graph_authority_identity: &str,
        contract: WorthQueryAdmittedConvergenceContract,
        lifecycle: WorthQueryConvergenceEpochLifecycle,
    ) -> Self {
        let identity = Arc::from(hash_parts(&[
            "worth_query_convergence_epoch_v1".into(),
            format!("operation-binding:{operation_binding_identity}"),
            format!("logical-run:{logical_run_identity}"),
            format!("managed-run:{managed_run_identity}"),
            format!("graph-authority:{graph_authority_identity}"),
            format!("contract:{}", contract.identity()),
        ]));
        Self {
            identity,
            logical_run_identity: Arc::from(logical_run_identity),
            contract,
            lifecycle,
            report_history: WorthQueryConvergenceReportHistory::empty(),
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn logical_run_identity(&self) -> &str {
        &self.logical_run_identity
    }

    pub fn contract(
        &self,
    ) -> &WorthQueryAdmittedConvergenceContract {
        &self.contract
    }

    pub fn counters(
        &self,
    ) -> &WorthQueryConvergenceEpochCounters {
        self.lifecycle.counters()
    }

    pub fn record_lifecycle_event<E>(
        &mut self,
        event: E,
    ) where
        E: WorthQueryConvergenceLifecycleEvent,
    {
        self.lifecycle.record(event);
    }

    pub fn incumbents(
        &self,
    ) -> &[WorthQueryRetainedConvergenceCandidateEvidence] {
        self.report_history.incumbents()
    }

    pub fn latest_report(
        &self,
    ) -> Option<&WorthQueryBoundConvergenceReport> {
        self.report_history.latest_report()
    }

    fn commit_prepared_report(&mut self, prepared: WorthQueryPreparedConvergenceReportCommit) {
        let event = self.report_history.commit(prepared);
        self.lifecycle.record(event);
    }
}
