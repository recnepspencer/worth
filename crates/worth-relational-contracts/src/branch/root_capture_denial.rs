//! Branch root capture and authoritative payload refusal vocabulary.
use crate::identity::data::PartitionId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchRootCaptureDenial {
    Execution(crate::execution::denial_cause::RelationalExecutionDenialCause),
    PublishedPartitionMissing(PartitionId),
    DuplicateRegion(PartitionId),
    InvalidAdjacencyProjection(PartitionId),
    InvalidHistoryCoverage(PartitionId),
    ContentEncoding(PartitionId),
    AllocationAccountingOverflow(PartitionId),
    StorageUnavailable(crate::storage::data::storage_unavailable::RelationalStorageUnavailable),
    SlotCountOverflow,
    AllocationLayoutOverflow,
    Request(crate::execution::denial_cause::RelationalExecutionDenialCause),
    CanonicalEnvelopeMismatch,
    CanonicalPayloadBacking(worth_execution::ExecutionAllocationDenial),
    UnadmittedContent,
    Content(crate::content::denial::RelationalContentDenial),
    StorageContentMismatch {
        descriptor_root: [u8; 32],
        reconstructed_root: [u8; 32],
    },
    SchemaRootMismatch {
        descriptor_root: [u8; 32],
        canonical_root: [u8; 32],
    },
    VisibilityCommitmentMismatch {
        committed: [u8; 32],
        reconstructed: [u8; 32],
    },
    UnresolvedContentSymbol(crate::symbols::data::Symbol),
    RootIdentityExhausted,
    SchemaAuthorityIdentityExhausted,
    RegionIdentityExhausted,
    ReachabilityIdentityExhausted,
}

impl From<crate::storage::substrate::maintenance::refusal::MaintenanceRefusal>
    for RelationalBranchRootCaptureDenial
{
    fn from(cause: crate::storage::substrate::maintenance::refusal::MaintenanceRefusal) -> Self {
        match cause {
            crate::storage::substrate::maintenance::refusal::MaintenanceRefusal::Execution(cause) => Self::Execution(cause),
            crate::storage::substrate::maintenance::refusal::MaintenanceRefusal::MissingPredecessor | crate::storage::substrate::maintenance::refusal::MaintenanceRefusal::UnavailableChange => {
                Self::CanonicalEnvelopeMismatch
            }
        }
    }
}

impl From<crate::branch::root::schema::quote_denial::SchemaCompilationQuoteDenial>
    for RelationalBranchRootCaptureDenial
{
    fn from(
        cause: crate::branch::root::schema::quote_denial::SchemaCompilationQuoteDenial,
    ) -> Self {
        match cause {
            crate::branch::root::schema::quote_denial::SchemaCompilationQuoteDenial::AllocationLayoutOverflow => {
                Self::AllocationLayoutOverflow
            }
            crate::branch::root::schema::quote_denial::SchemaCompilationQuoteDenial::Request(cause) => {
                Self::Request(cause)
            }
        }
    }
}
