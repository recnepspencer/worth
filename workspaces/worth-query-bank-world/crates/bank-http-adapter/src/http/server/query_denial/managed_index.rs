//! HTTP projection of the managed receiver's original owned denial.
use super::*;
pub(super) fn managed_index(
    cause: &worth_query_host::facade::primary_graph::WorthQueryManagedIndexDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestInterruption as Stop;
    use worth_query_host::facade::primary_graph::WorthQueryManagedIndexDenialCause as Cause;
    match cause.cause() {
        Cause::Request(stop) => crate::http::server::advancement_denial::request(stop),
        Cause::Interrupted(Stop::Cancelled) => cancelled(),
        Cause::Interrupted(Stop::DeadlineExceeded) => deadline(),
        Cause::Execution(cause) => crate::http::server::relational_execution_denial::denial(*cause),
        Cause::Installation(cause) => query_denial(BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::BindingInstallation(cause.clone()))),
        Cause::Product(cause) => query_denial(BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ProductSelection(*cause))),
        Cause::Principal(cause) => query_denial(BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::PrincipalResolution(cause.clone()))),
        Cause::Scope(cause) => query_denial(BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ScopeResolution(cause.clone()))),
        Cause::QueryAdmission(cause) => query_denial(BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::Admission(cause.clone()))),
        Cause::QueryRead(cause) => query_denial(BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::Execution(cause.clone()))),
        Cause::Work(cause) => work(cause),
        Cause::Refused(_) | Cause::GraphWork(_) | Cause::Source(_) => unavailable(),
    }
}

fn work(
    cause: &worth_query_host::facade::primary_graph::WorthQueryManagedIndexWorkDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::{
        primary_graph::{
            WorthQueryManagedIndexStop as Stop, WorthQueryManagedIndexWorkDenial as Work,
        },
        runtime::ExecutionRequestArrayDenial as Storage,
    };
    match cause {
        Work::Controlled(Stop::Request(stop) | Stop::Storage(Storage::Request(stop))) => {
            crate::http::server::advancement_denial::request(*stop)
        }
        Work::Controlled(Stop::Storage(Storage::Allocation(cause))) => match cause.classify() {
            worth_query_host::facade::runtime::ExecutionAllocationDenialClass::Request(cause) => {
                crate::http::server::advancement_denial::request(cause)
            }
            worth_query_host::facade::runtime::ExecutionAllocationDenialClass::Physical(cause) => {
                crate::http::server::advancement_denial::physical_allocation(cause)
            }
        },
        Work::Controlled(Stop::ValueDecode(cause)) => value_decode(*cause),
        Work::Controlled(Stop::NodeCapacityExceeded) => exhausted(),
        Work::Controlled(
            Stop::ForeignNode
            | Stop::UnwrittenNode
            | Stop::InvalidRoot
            | Stop::ForeignSourceSlot
            | Stop::SourceSlotUnavailable
            | Stop::CapacityOverflow,
        ) => internal_denied(),
        Work::Provider(_) => unavailable(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_query_host::facade::{
        primary_graph::{
            WorthQueryManagedIndexStop as Stop, WorthQueryManagedIndexWorkDenial as Work,
        },
        runtime::{MapKernelStop, WorkCeilingDenial},
    };
    #[test]
    fn managed_callback_control_stops_keep_their_http_action() {
        for (stop, expected) in [
            (MapKernelStop::Cancelled, Kind::Cancelled),
            (MapKernelStop::DeadlineElapsed, Kind::DeadlineExceeded),
        ] {
            let cause = Work::Controlled(Stop::Request(WorkCeilingDenial::Stopped(stop)));
            assert_eq!(work(&cause), BankHttpDenial::new(expected, Next::Retry));
        }
    }
}
