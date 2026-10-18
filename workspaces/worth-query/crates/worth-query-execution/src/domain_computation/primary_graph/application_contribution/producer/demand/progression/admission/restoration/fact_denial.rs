//! Checkpoint format and retention refusals retain their distinct causes.
use super::{denial, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind};
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StoreDenial,
    application_checkpoint::FactDecodeDenial,
};

pub(super) fn decode_denial(error: FactDecodeDenial) -> WorthQueryOutputDemandDenial {
    match error {
        FactDecodeDenial::Execution(cause) => {
            crate::domain_computation::primary_graph::output_reuse::read_denial(cause)
        }
        FactDecodeDenial::Format(message) => denial(Kind::IncompleteDependencyCoverage, message),
        FactDecodeDenial::Retention(StoreDenial::Allocation(allocation)) => {
            use worth_execution::{
                ExecutionAllocationDenialClass as Class, MapKernelStop as Stop,
                WorkCeilingDenial as Request,
            };
            let kind = match allocation.classify() {
                Class::Request(Request::Stopped(Stop::Cancelled)) => Kind::Cancelled,
                Class::Request(Request::Stopped(Stop::DeadlineElapsed)) => Kind::TimedOut,
                Class::Request(Request::Admission(_))
                | Class::Request(Request::Stopped(
                    Stop::WorkCounterOverflow | Stop::WorkCeiling | Stop::NestedStopped,
                ))
                | Class::Request(Request::Panicked)
                | Class::Physical(_) => Kind::RetentionBudgetExceeded,
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
