//! Moved whole from `domain_computation::primary_graph::application_query::history::admission`: the items below and every impl they own.
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::admission::MetadataValidationDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial;

impl From<MetadataValidationDenial> for WorthQueryApplicationHistoryDenial {
    fn from(cause: MetadataValidationDenial) -> Self {
        match cause {
            MetadataValidationDenial::Session => Self::SubjectMismatch,
            MetadataValidationDenial::Authorization(cause) => Self::Authorization(cause),
        }
    }
}

/// Stops retain the existing authorization/query cause and disclose no protected counts.
#[derive(Debug)]
pub enum WorthQueryApplicationHistoryDenial {
    Query(WorthQueryApplicationQueryAdmissionDenial),
    Read(crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    SubjectMismatch,
    MetadataDisclosureRefused,
    CaptureLimitExceeded,
    StorageLimitExceeded,
    SizeOverflow,
    Native(worth_relational::facade::history::RelationalRecordHistoryDenial),
    World(worth_runtime_world::facade::ProductExactHistoryDenial),
    Principal(crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial),
    Scope(crate::domain_computation::primary_graph::vocabulary::entity_resolution_denial::WorthQueryEntityResolutionDenial),
    Selection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    InvalidContinuation,
    BasisReleaseFailed,
    Request(worth_execution::WorkCeilingDenial),
    Allocation(worth_execution::ExecutionAllocationDenial),
}

impl From<crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenial> for WorthQueryApplicationHistoryDenial {
    fn from(cause: crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenial) -> Self {
        Self::Read(cause)
    }
}

impl From<crate::domain_computation::primary_graph::vocabulary::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial>
    for WorthQueryApplicationHistoryDenial
{
    fn from(
        cause: crate::domain_computation::primary_graph::vocabulary::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial,
    ) -> Self {
        Self::Read(crate::domain_computation::primary_graph::vocabulary::application_query::one_shot::map_authorized_read_denial(
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
