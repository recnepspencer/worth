//! HTTP projection of the managed receiver's original owned denial.
mod native_source;
use super::*;
pub(super) fn managed_index(
    cause: &worth_query_host::facade::primary_graph::WorthQueryManagedIndexDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestInterruption as Stop;
    use worth_query_host::facade::primary_graph::WorthQueryManagedIndexDenialCause as Cause;
    match cause.cause() {
        Cause::CurrentControl(cause) => output_settlement(cause.kind().into()),
        Cause::SourceVerification { kind, authorization, execution } => {
            if let Some(cause) = execution { return crate::http::server::relational_execution_denial::denial(cause); }
            if let Some(cause) = authorization { return authorization_denial(cause.into()); }
            crate::http::server::estate_denial::attempt_kind(kind.into())
        }
        Cause::NativeSource(cause) => native_source::source(cause),
        Cause::NativeAdmission(cause) => crate::http::server::relational_execution_denial::companion(cause.clone()),
        Cause::LiveAdmission(cause) => query_denial(BankApplicationQueryDenial::from_live_open(cause.clone())),
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

fn source(
    cause: &worth_query_host::facade::primary_graph::WorthQuerySourceExpectationDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestInterruption;

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
        Work::Controlled(Stop::Projection(cause)) => {
            use worth_query_host::facade::primary_graph::WorthQueryApplicationProjectionDenialKind as Projection;
            match cause.kind() {
                Projection::ValueDecode(cause) => value_decode(cause),
                Projection::DomainProjectionRejected => permission_denied(),
                Projection::FieldNotProjected
                | Projection::FieldContractMismatch
                | Projection::FieldTypeMismatch
                | Projection::FieldOmitted
                | Projection::RelationNotProjected
                | Projection::RelationContractMismatch
                | Projection::RelationCardinalityMismatch
                | Projection::RelationOmitted => internal_denied(),
            }
        }
        Work::Controlled(Stop::Content(cause)) => {
            crate::http::server::relational_execution_denial::content(*cause)
        }
        Work::Controlled(Stop::MaterialStorage(cause)) => {
            crate::http::server::relational_execution_denial::denial(*cause)
        }
        Work::Controlled(Stop::NativeSource(cause)) => native_source::walk(*cause),
        Work::Controlled(Stop::OrderedStorage(cause)) => {
            use worth_query_host::facade::runtime::ExecutionOrderedMapDenial as Ordered;
            match cause {
                Ordered::Request(cause) => crate::http::server::advancement_denial::request(*cause),
                Ordered::Allocation(cause) => allocation(cause),
                Ordered::CapacityOverflow => internal_denied(),
            }
        }
        Work::Controlled(
            Stop::IncrementalUnavailable
            | Stop::SourceStructureUnavailable
            | Stop::MembershipUnavailable
            | Stop::RefinementUnavailable
            | Stop::SummaryUnavailable,
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
        Work::Controlled(Stop::Request(stop) | Stop::Storage(Storage::Request(stop))) => {
            crate::http::server::advancement_denial::request(*stop)
        }
        Work::Controlled(Stop::Storage(Storage::Allocation(cause))) => allocation(cause),
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
