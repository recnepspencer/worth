//! Descriptive ownership conflicts and preservation of their original request stops.
use crate::identity::data::{EntityId, KindId};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RelationalModelOwnershipConflict {
    pub subject: EntityId,
}

impl RelationalModelOwnershipConflict {
    pub const VALUE_IDENTITY: &'static str = "worth.model-ownership.conflict.v1";
    pub fn encode(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
    }
}

#[derive(Debug)]
pub enum RelationalModelOwnershipDenial {
    ForeignBasis,
    Custody(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError),
    TargetNotCurrent,
    SchemaChanged,
    ModelRootKind { model: EntityId, actual: KindId },
    EntityUnavailable(EntityId),
    Conflict(RelationalModelOwnershipConflict),
    Read(crate::validation::data::structural_read::StructuralReadError),
    Adjacency(crate::validation::data::custom_rule::errors::CustomInvariantTraversalError),
    Traversal(crate::validation::data::structural_walk_denial::StructuralWalkDenialKind),
    Request(worth_execution::WorkCeilingDenial),
    Native(crate::execution::denial_cause::RelationalExecutionDenialCause),
}

impl RelationalModelOwnershipDenial {
    /// Keep custody-owner interruptions visible without flattening its typed refusal.
    pub fn kernel_stop(&self) -> Option<worth_execution::MapKernelStop> {
        match self {
            Self::Request(worth_execution::WorkCeilingDenial::Stopped(stop)) => Some(*stop),
            Self::Native(cause) => cause.as_kernel_stop(),
            Self::Custody(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::Execution {
                denial,
                ..
            }) => {
                let crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause) =
                    denial.kind;
                cause.as_kernel_stop()
            }
            _ => None,
        }
    }
}
