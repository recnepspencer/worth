use super::super::WorthQueryApplicationQueryAdmissionDenial;
use crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial;

/// Stops retain the existing authorization/query cause and disclose no protected counts.
#[derive(Debug)]
pub enum WorthQueryApplicationHistoryDenial {
    Query(WorthQueryApplicationQueryAdmissionDenial),
    Read(super::super::WorthQueryApplicationOneShotDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    SubjectMismatch,
    MetadataDisclosureRefused,
    CaptureLimitExceeded,
    StorageLimitExceeded,
    SizeOverflow,
    Native(worth_relational::facade::history::RelationalRecordHistoryDenial),
    World(worth_runtime_world::facade::ProductExactHistoryDenial),
    Principal(crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial),
    Scope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    Selection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    InvalidContinuation,
    BasisReleaseFailed,
    Request(worth_execution::WorkCeilingDenial),
    Allocation(worth_execution::ExecutionAllocationDenial),
}

impl From<super::super::WorthQueryApplicationOneShotDenial> for WorthQueryApplicationHistoryDenial {
    fn from(cause: super::super::WorthQueryApplicationOneShotDenial) -> Self {
        Self::Read(cause)
    }
}

impl From<super::super::authorized_read::WorthQueryAuthorizedApplicationReadDenial>
    for WorthQueryApplicationHistoryDenial
{
    fn from(
        cause: super::super::authorized_read::WorthQueryAuthorizedApplicationReadDenial,
    ) -> Self {
        Self::Read(super::super::one_shot::map_authorized_read_denial(
            cause, "history",
        ))
    }
}

impl From<worth_execution::ExecutionRequestArrayDenial> for WorthQueryApplicationHistoryDenial {
    fn from(cause: worth_execution::ExecutionRequestArrayDenial) -> Self {
        match cause {
            worth_execution::ExecutionRequestArrayDenial::Request(cause) => Self::Request(cause),
            worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => {
                Self::Allocation(cause)
            }
        }
    }
}
