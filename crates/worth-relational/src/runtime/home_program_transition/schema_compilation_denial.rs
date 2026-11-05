//! Preserve request and capacity causes at the public Home compilation boundary.
use super::RelationalHomeProgramContractDenial as Denial;
use crate::branch::RelationalBranchRootCaptureDenial as Cause;

pub(super) fn translate(cause: Cause) -> Denial {
    match cause {
        Cause::Request(cause) => Denial::Execution(cause),
        Cause::AllocationLayoutOverflow
        | Cause::AllocationAccountingOverflow(_)
        | Cause::SlotCountOverflow => Denial::CapacityOverflow,
        Cause::SchemaAuthorityIdentityExhausted => Denial::SchemaAuthorityExhausted,
        Cause::PublishedPartitionMissing(_)
        | Cause::DuplicateRegion(_)
        | Cause::InvalidAdjacencyProjection(_)
        | Cause::InvalidHistoryCoverage(_)
        | Cause::ContentEncoding(_)
        | Cause::StorageUnavailable(_)
        | Cause::CanonicalEnvelopeMismatch
        | Cause::StorageContentMismatch { .. }
        | Cause::SchemaRootMismatch { .. }
        | Cause::VisibilityCommitmentMismatch { .. }
        | Cause::UnresolvedContentSymbol(_)
        | Cause::RootIdentityExhausted
        | Cause::RegionIdentityExhausted
        | Cause::ReachabilityIdentityExhausted => Denial::SchemaCapture,
    }
}
