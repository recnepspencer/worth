//! Original managed Source read refusal stays owned through producer/output handoff.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::managed_index::{
        WorthQueryManagedIndexDenial as Cause, WorthQueryManagedIndexDenialCause as Detail,
        WorthQueryManagedIndexStop as Stop, WorthQueryManagedIndexWorkDenial as Work,
    },
    output_reuse,
};
use std::sync::Arc;
impl WorthQueryOutputDemandDenial {
    pub(in crate::domain_computation::primary_graph) fn managed_source_query(
        cause: Arc<Cause>,
        subject: &'static str,
    ) -> Self {
        let mut denial = match cause.cause() {
            Detail::CurrentControl(stop) => stop.clone(),
            Detail::Work(Work::Controlled(Stop::Request(stop))) => {
                output_reuse::read_denial((*stop).into())
            }
            Detail::Work(Work::Controlled(Stop::MaterialStorage(stop)))
            | Detail::Execution(stop) => output_reuse::read_denial(*stop),
            Detail::Work(work) => Self::managed_index(work.clone(), subject),
            Detail::NativeAdmission(stop) => output_reuse::retained_fact_denial(output_reuse::preflight_stop(*stop), subject),
            Detail::Request(stop) => output_reuse::read_denial(stop.into()),
            Detail::Interrupted(stop) => Self::new(
                match stop {
                    worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled => WorthQueryOutputDemandDenialKind::Cancelled,
                    worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded => WorthQueryOutputDemandDenialKind::TimedOut,
                },
                subject,
            ),
            Detail::Installation(stop) => Self::new(
                WorthQueryOutputDemandDenialKind::SourceQueryInstallation(stop.kind()),
                subject,
            ),
            Detail::Principal(stop) => Self::new(
                WorthQueryOutputDemandDenialKind::of_principal_resolution(stop.kind()),
                subject,
            ),
            Detail::Scope(stop) => Self::new(
                WorthQueryOutputDemandDenialKind::of_scope_resolution(stop.kind()),
                subject,
            ),
            Detail::QueryAdmission(stop) => Self::new(
                match stop.kind() {
                    crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::Cancelled => WorthQueryOutputDemandDenialKind::Cancelled,
                    crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded => WorthQueryOutputDemandDenialKind::TimedOut,
                    kind => WorthQueryOutputDemandDenialKind::SourceQueryAdmission(kind),
                },
                subject,
            ),
            Detail::QueryRead(stop) => Self::new(
                match stop.kind() {
                    crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind::Cancelled => WorthQueryOutputDemandDenialKind::Cancelled,
                    crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind::DeadlineExceeded => WorthQueryOutputDemandDenialKind::TimedOut,
                    kind => WorthQueryOutputDemandDenialKind::SourceQueryExecution(kind),
                },
                subject,
            ),
            _ => Self::new(
                WorthQueryOutputDemandDenialKind::ManagedIndexSourceDenied,
                subject,
            ),
        };
        if let Detail::Work(work) = cause.cause() {
            denial.evidence_mut().managed_index = Some(work.clone());
        }
        denial.evidence_mut().managed_source_query = Some(cause);
        denial
    }
    /// Exact original managed read error, including its typed domain/request/refusal.
    pub fn managed_source_query_denial(&self) -> Option<&Cause> {
        self.evidence.as_ref()?.managed_source_query.as_deref()
    }
}
