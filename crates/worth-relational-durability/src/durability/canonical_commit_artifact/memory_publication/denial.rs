//! The original request can refuse Memory framing independently of codec correctness.
use worth_relational_contracts::internal_port::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial as Denial;
#[derive(Debug)]
pub enum MemoryFramePreparationDenial {
    Request(worth_execution::WorkCeilingDenial),
    Artifact(Denial),
}
impl From<Denial> for MemoryFramePreparationDenial {
    fn from(cause: Denial) -> Self {
        Self::Artifact(cause)
    }
}
impl From<worth_execution::ExecutionRequestArrayDenial> for MemoryFramePreparationDenial {
    fn from(cause: worth_execution::ExecutionRequestArrayDenial) -> Self {
        match cause {
            worth_execution::ExecutionRequestArrayDenial::Request(cause) => Self::Request(cause),
            worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => {
                Self::Artifact(Denial::Allocation(cause))
            }
        }
    }
}
