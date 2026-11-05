//! HTTP projection of the managed receiver's original owned denial.
use super::*;
pub(super) fn managed_index(
    cause: &worth_query_host::facade::primary_graph::WorthQueryManagedIndexDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestInterruption as Stop;
    use worth_query_host::facade::primary_graph::WorthQueryManagedIndexDenialCause as Cause;
    match cause.cause() {
        Cause::SourceVerification { execution: Some(cause), .. } => crate::http::server::relational_execution_denial::denial(cause),
        Cause::SourceVerification { authorization: Some(cause), .. } => authorization_denial(cause.into()),
        Cause::SourceVerification { .. } | Cause::NativeSource(_) => unavailable(),
        Cause::LiveAdmission(cause) => query_denial(BankApplicationQueryDenial::from_live_open(cause.clone())),
        Cause::NativeAdmission(cause) => native_admission(cause),
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
        Cause::Source(cause) => source(cause),
        Cause::Refused(_) | Cause::GraphWork(_) => unavailable(),
    }
}

pub(super) fn source(
    cause: &worth_query_host::facade::primary_graph::WorthQuerySourceExpectationDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestInterruption;

    if let Some(authorization) = cause.authorization_denial() {
        return authorization_denial(authorization.kind().into());
    }
    if let Some(stop) = cause.retention_interruption() {
        return match stop {
            WorthQueryRequestInterruption::Cancelled => cancelled(),
            WorthQueryRequestInterruption::DeadlineExceeded => deadline(),
        };
    }
    if let Some(stop) = cause.execution_denial() {
        return crate::http::server::advancement_denial::request(*stop);
    }
    match cause.allocation_denial() {
        Some(cause) => allocation(cause),
        None => unavailable(),
    }
}

fn allocation(
    cause: &worth_query_host::facade::runtime::ExecutionAllocationDenial,
) -> BankHttpDenial {
    match cause.classify() {
        worth_query_host::facade::runtime::ExecutionAllocationDenialClass::Request(cause) => {
            crate::http::server::advancement_denial::request(cause)
        }
        worth_query_host::facade::runtime::ExecutionAllocationDenialClass::Physical(cause) => {
            crate::http::server::advancement_denial::physical_allocation(cause)
        }
    }
}

pub(in crate::http::server) fn work(
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
        Work::Controlled(Stop::Storage(Storage::Allocation(cause))) => allocation(cause),
        Work::Controlled(Stop::MaterialStorage(cause)) => {
            crate::http::server::relational_execution_denial::denial(*cause)
        }
        Work::Controlled(Stop::OrderedStorage(cause)) => ordered_storage(cause),
        Work::Controlled(Stop::ValueDecode(cause)) => value_decode(*cause),
        Work::Controlled(Stop::NativeSource(cause)) => native_walk(cause),
        Work::Controlled(Stop::NodeCapacityExceeded) => exhausted(),
        Work::Controlled(
            Stop::ForeignNode
            | Stop::UnwrittenNode
            | Stop::InvalidRoot
            | Stop::ForeignSourceSlot
            | Stop::SourceSlotUnavailable
            | Stop::CapacityOverflow,
        ) => internal_denied(),
        Work::Controlled(
            Stop::MembershipUnavailable
            | Stop::RefinementUnavailable
            | Stop::SummaryUnavailable
            | Stop::IncrementalUnavailable
            | Stop::SourceStructureUnavailable,
        ) => unavailable(),
        Work::Controlled(
            Stop::Coverage(_)
            | Stop::ForeignCoverage
            | Stop::IncompleteCoverage
            | Stop::IncompleteRefinement
            | Stop::IncompleteTraversal
            | Stop::SummaryMismatch
            | Stop::ForeignSummarySource
            | Stop::SourceStructureIncomplete
            | Stop::DuplicateSourceSlot
            | Stop::DuplicateNodeParent
            | Stop::SourceStructureCycle,
        ) => internal_denied(),
        Work::Provider(_) => unavailable(),
    }
}

fn native_walk(
    cause: &worth_query_host::facade::primary_graph::StructuralWalkDenialKind,
) -> BankHttpDenial {
    use worth_query_host::facade::{
        application_invariants::CustomInvariantInterruption as Stop,
        primary_graph::StructuralWalkDenialKind as Denial,
        runtime::{LeaseDenial, WorkCeilingDenial},
    };
    match cause {
        Denial::Interrupted(Stop::Cancelled) => cancelled(),
        Denial::Interrupted(Stop::DeadlineElapsed) => deadline(),
        Denial::MemoryAdmission(cause) => crate::http::server::advancement_denial::request(
            WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(*cause)),
        ),
        Denial::MemoryBudgetExceeded => exhausted(),
        Denial::Read(_) => unavailable(),
    }
}

fn ordered_storage(
    cause: &worth_query_host::facade::runtime::ExecutionOrderedMapDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::runtime::ExecutionOrderedMapDenial as Denial;
    match cause {
        Denial::Request(cause) => crate::http::server::advancement_denial::request(*cause),
        Denial::Allocation(cause) => allocation(cause),
        Denial::CapacityOverflow => internal_denied(),
    }
}
pub(super) fn native_admission(
    cause: &worth_query_host::facade::primary_graph::CompanionPreflightStop,
) -> BankHttpDenial {
    use worth_query_host::facade::primary_graph::{
        CompanionPreflightStop as Denial, RelationalOperationInterruption as Stop,
    };
    match cause {
        Denial::Execution(cause) => {
            crate::http::server::relational_execution_denial::denial(*cause)
        }
        Denial::Interrupted(cause) => match cause.interruption() {
            Stop::Cancelled => cancelled(),
            Stop::TimedOut => deadline(),
        },
        Denial::ManagedSourceChanged
        | Denial::SelectedSourceMismatch
        | Denial::SelectedPositionUnavailable { .. }
        | Denial::RegistrationChanged => stale(),
        Denial::ManagedSourceNotPrepared | Denial::TopologyPending => unavailable(),
        Denial::ForeignCell
        | Denial::WorkCounterOverflow
        | Denial::PreparationMemoryCounterOverflow => internal_denied(),
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
    #[test]
    fn native_managed_callback_interruptions_keep_their_http_action() {
        use worth_query_host::facade::{
            application_invariants::CustomInvariantInterruption as Interruption,
            primary_graph::StructuralWalkDenialKind as Native,
        };
        for (stop, expected) in [
            (Interruption::Cancelled, Kind::Cancelled),
            (Interruption::DeadlineElapsed, Kind::DeadlineExceeded),
        ] {
            let cause = Work::Controlled(Stop::NativeSource(Native::Interrupted(stop)));
            assert_eq!(work(&cause), BankHttpDenial::new(expected, Next::Retry));
        }
    }
}
