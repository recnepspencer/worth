//! Checkpoint format and retention refusals retain their distinct causes.
use super::{denial, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind};
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StoreDenial,
    application_checkpoint::FactDecodeDenial,
};

pub(super) fn decode_denial(error: FactDecodeDenial) -> WorthQueryOutputDemandDenial {
    match error {
        FactDecodeDenial::Format(message) => denial(Kind::IncompleteDependencyCoverage, message),
        FactDecodeDenial::Retention(StoreDenial::Allocation(allocation)) => {
            let kind = match allocation.kind() {
                worth_execution::ExecutionAllocationDenialKind::Cancelled => Kind::Cancelled,
                worth_execution::ExecutionAllocationDenialKind::DeadlineElapsed => Kind::TimedOut,
                _ => Kind::RetentionBudgetExceeded,
            };
            denial(kind, "checkpoint producer facts")
        }
        FactDecodeDenial::Retention(StoreDenial::RequestInterruption(event)) => {
            use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
            let kind = match event {
                WorthQueryRequestInterruption::Cancelled => Kind::Cancelled,
                WorthQueryRequestInterruption::DeadlineExceeded => Kind::TimedOut,
            };
            denial(kind, "checkpoint producer facts")
        }
        FactDecodeDenial::Retention(error) => {
            let kind = match error {
                StoreDenial::ConflictingBody => Kind::IncompleteDependencyCoverage,
                _ => Kind::RetentionBudgetExceeded,
            };
            denial(kind, error.to_string())
        }
    }
}
