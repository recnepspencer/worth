//! Carry the original owned consistency cause into the provider read-set boundary.
use super::*;
use crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind;
use crate::domain_computation::{
    WorthQueryDecisionReadSetDenialKind as Kind, WorthQueryDecisionReadSetFailure,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Interruption;

impl WorthQueryApplicationAttemptDenial {
    pub(in crate::domain_computation::primary_graph) fn into_decision_read_set_failure(
        self,
    ) -> WorthQueryDecisionReadSetFailure {
        use WorthQueryApplicationAttemptDenialKind as Attempt;
        let kind = match self.kind() {
            Attempt::ExecutionRequest(kind) => Kind::ExecutionRequest(kind),
            Attempt::Cancelled => Kind::RequestInterrupted(Interruption::Cancelled),
            Attempt::DeadlineExceeded => Kind::RequestInterrupted(Interruption::DeadlineExceeded),
            _ => Kind::ProviderRejected,
        };
        let failure = WorthQueryDecisionReadSetFailure::new(kind, self.to_string());
        match self.meaning {
            AttemptDenialMeaning::NativeExecution(cause) => failure.with_native_query_read_stop(
                worth_relational::facade::runtime::QueryReadExecutionStop::ReadRequest(cause),
            ),
            AttemptDenialMeaning::Authored {
                cause: AttemptDenialCause::NativeQueryRead(stop),
                ..
            } => failure.with_native_query_read_stop(*stop),
            AttemptDenialMeaning::Authored {
                cause: AttemptDenialCause::ManagedIndex(cause),
                ..
            } => failure.with_managed_index_denial(cause),
            _ => failure,
        }
    }
}
