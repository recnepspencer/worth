use crate::access::shape::AccessShape;
use crate::strategy::LayoutStrategyFamily;

use super::super::denial::SelectionCandidateRejection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::planning) enum EligibleStrategyOperation {
    LsmLookup,
    LsmRunPublication,
    LsmReplayRecovery,
    LsmCompaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::planning) enum CandidateStrategyFamily {
    BTree,
    Lsm,
}

impl CandidateStrategyFamily {
    pub(in crate::planning) const fn strategy_family(self) -> LayoutStrategyFamily {
        match self {
            Self::BTree => LayoutStrategyFamily::BTreeRange,
            Self::Lsm => LayoutStrategyFamily::BaselineLsmWriteOptimized,
        }
    }
}

pub(in crate::planning) const fn classify_candidate_operation(
    family: CandidateStrategyFamily,
    shape: AccessShape,
) -> Result<EligibleStrategyOperation, SelectionCandidateRejection> {
    let operation = match (family, shape) {
        (CandidateStrategyFamily::Lsm, AccessShape::PointLookup) => {
            EligibleStrategyOperation::LsmLookup
        }
        (CandidateStrategyFamily::Lsm, AccessShape::Append) => {
            EligibleStrategyOperation::LsmRunPublication
        }
        (CandidateStrategyFamily::Lsm, AccessShape::RebuildRead) => {
            EligibleStrategyOperation::LsmReplayRecovery
        }
        (CandidateStrategyFamily::Lsm, AccessShape::CompactionRead) => {
            EligibleStrategyOperation::LsmCompaction
        }
        (
            CandidateStrategyFamily::BTree,
            AccessShape::PointLookup
            | AccessShape::BatchPointLookup
            | AccessShape::SortedBatchLookup
            | AccessShape::MultiRangeLookup
            | AccessShape::GroupedPrefixLookup
            | AccessShape::CoalescedPageRead
            | AccessShape::ChunkTreeWalk
            | AccessShape::ManifestGraphWalk
            | AccessShape::BoundedScan
            | AccessShape::FullDeclaredScan
            | AccessShape::StreamingRead
            | AccessShape::StreamingContinuationRead
            | AccessShape::RangeLookup
            | AccessShape::PrefixLookup
            | AccessShape::RebuildRead
            | AccessShape::Append
            | AccessShape::CompactionRead
            | AccessShape::VerifierRead
            | AccessShape::RepairRead
            | AccessShape::QuarantineRead
            | AccessShape::DegradedExactScan,
        )
        | (
            CandidateStrategyFamily::Lsm,
            AccessShape::BatchPointLookup
            | AccessShape::SortedBatchLookup
            | AccessShape::RangeLookup
            | AccessShape::MultiRangeLookup
            | AccessShape::PrefixLookup
            | AccessShape::GroupedPrefixLookup
            | AccessShape::CoalescedPageRead
            | AccessShape::ChunkTreeWalk
            | AccessShape::ManifestGraphWalk
            | AccessShape::BoundedScan
            | AccessShape::FullDeclaredScan
            | AccessShape::StreamingRead
            | AccessShape::StreamingContinuationRead
            | AccessShape::VerifierRead
            | AccessShape::RepairRead
            | AccessShape::QuarantineRead
            | AccessShape::DegradedExactScan,
        ) => {
            return Err(SelectionCandidateRejection::OperationUnsupported {
                family: family.strategy_family(),
                shape,
            });
        }
    };
    Ok(operation)
}
