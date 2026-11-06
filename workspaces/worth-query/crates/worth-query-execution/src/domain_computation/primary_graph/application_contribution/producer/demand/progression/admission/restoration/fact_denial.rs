//! Checkpoint format and retention refusals retain their distinct causes.
use {crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial, crate::domain_computation::primary_graph::application_checkpoint::FactDecodeDenial};
use {
    super::denial,
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial,
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind as Kind,
};

pub(super) fn decode_denial(error: FactDecodeDenial) -> WorthQueryOutputDemandDenial {
    match error {
        FactDecodeDenial::Execution(cause) => {
            crate::domain_computation::primary_graph::output_reuse::read_denial(cause)
        }
        FactDecodeDenial::Format(message) => denial(Kind::IncompleteDependencyCoverage, message),
        FactDecodeDenial::Retention(StoreDenial::ExecutionRequest(cause)) => denial(
            Kind::ExecutionRequest(crate::domain_computation::request_execution::execution_stop::advancement_scope_stop(cause)),
            "checkpoint producer facts",
        ),
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
                    Stop::WorkCounterOverflow
                    | Stop::AccountingUnavailable
                    | Stop::WorkCeiling
                    | Stop::NestedStopped,
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
