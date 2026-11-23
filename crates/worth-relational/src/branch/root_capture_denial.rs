//! Branch root capture and authoritative payload refusal vocabulary.
use crate::identity::data::PartitionId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RelationalBranchRootCaptureDenial {
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
