//! The original request can refuse Memory framing independently of codec correctness.
use worth_relational_contracts::internal_port::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial as Denial;
use worth_relational_contracts::internal_port::execution::request_or::ExecutionOr;

pub type MemoryFramePreparationDenial = ExecutionOr<Denial>;

pub(super) fn allocation(
    cause: worth_execution::ExecutionRequestArrayDenial,
) -> MemoryFramePreparationDenial {
    match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => ExecutionOr::Request(cause),
        worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => {
            ExecutionOr::Inner(Denial::Allocation(cause))
        }
    }
}
