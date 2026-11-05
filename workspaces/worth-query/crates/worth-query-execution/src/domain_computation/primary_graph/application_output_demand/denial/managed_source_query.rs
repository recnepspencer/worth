//! Original managed Source read refusal stays owned through producer/output handoff.
use super::*;
use std::sync::Arc;
use {
    crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexDenial as Cause,
    crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexDenialCause as Detail,
    crate::domain_computation::primary_graph::output_reuse,
    crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture,
    crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial as Work,
    crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop,
};
impl WorthQueryOutputDemandDenial {
    pub(in crate::domain_computation::primary_graph) fn managed_source_query(
        cause: Arc<Cause>,
        subject: &'static str,
    ) -> Self {
        let mut denial = match cause.cause() {
            Detail::CurrentControl(stop) => stop.clone(),
            Detail::SourceVerification { execution: Some(stop), .. } =>
                output_reuse::read_denial(stop).with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable),
            Detail::SourceVerification { authorization: Some(kind), .. } =>
                Self::new(WorthQueryOutputDemandDenialKind::of_request_authorization(kind), subject)
                    .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable),
            Detail::SourceVerification { kind, .. } => {
                use crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind as Attempt;
                let kind = match kind {
                    Attempt::Cancelled => WorthQueryOutputDemandDenialKind::Cancelled,
                    Attempt::DeadlineExceeded => WorthQueryOutputDemandDenialKind::TimedOut,
                    Attempt::DecisionFactBudgetExceeded => WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    Attempt::ExecutionRequest(stop) => WorthQueryOutputDemandDenialKind::SourceQueryExecution(
                        crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenialKind::ExecutionRequest(stop)),
                    _ => return Self::new(WorthQueryOutputDemandDenialKind::ManagedIndexSourceDenied, subject)
                        .with_managed_source_query(cause),
                };
                Self::new(kind, subject).with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable)
            }
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
                    crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind::Cancelled => WorthQueryOutputDemandDenialKind::Cancelled,
                    crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded => WorthQueryOutputDemandDenialKind::TimedOut,
                    kind => WorthQueryOutputDemandDenialKind::SourceQueryAdmission(kind),
                },
                subject,
            ),
            Detail::QueryRead(stop) => Self::new(
                match stop.kind() {
                    crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenialKind::Cancelled => WorthQueryOutputDemandDenialKind::Cancelled,
                    crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenialKind::DeadlineExceeded => WorthQueryOutputDemandDenialKind::TimedOut,
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

    fn with_managed_source_query(mut self, cause: Arc<Cause>) -> Self {
        self.evidence_mut().managed_source_query = Some(cause);
        self
    }
}
