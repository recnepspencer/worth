//! Preserve native/request interruption when new alias storage is refused.
use worth_execution::{
    ExecutionAllocationDenialClass as Class, MapKernelStop as Stop, WorkCeilingDenial as Request,
};
use {crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial, crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial as Denial, crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind as Kind};

pub(super) fn storage_denial(cause: StoreDenial) -> Denial {
    let kind = match &cause {
        StoreDenial::ExecutionRequest(stop) => Kind::ExecutionRequest(
            crate::domain_computation::request_execution::execution_stop::advancement_scope_stop(*stop)),
        StoreDenial::Allocation(allocation) => match allocation.classify() {
            Class::Request(Request::Stopped(Stop::Cancelled)) => Kind::Cancelled,
            Class::Request(Request::Stopped(Stop::DeadlineElapsed)) => Kind::TimedOut,
            Class::Request(stop) => Kind::ExecutionRequest(
                crate::domain_computation::request_execution::execution_stop::advancement_scope_stop(stop)),
            Class::Physical(_) => Kind::RetentionBudgetExceeded,
        },
        StoreDenial::RequestInterruption(event) => {
            use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Event;
            match event { Event::Cancelled => Kind::Cancelled, Event::DeadlineExceeded => Kind::TimedOut }
        }
        StoreDenial::ConflictingBody => Kind::IncompleteDependencyCoverage,
        _ => Kind::RetentionBudgetExceeded,
    };
    Denial::new(kind, "stable output fact storage")
}
