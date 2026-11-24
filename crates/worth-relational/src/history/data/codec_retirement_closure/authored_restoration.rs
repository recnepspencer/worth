//! Historical aspect grammar shared by inverse publication preparation.
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use super::CodecRetirementClosureStop as Stop;
use crate::transactions::data::outcomes::transaction_commit_error::{
    CommitExecutionDenial, CommitExecutionDenialKind, TransactionCommitError,
};
mod creation_patch;
mod inverse;
mod live;
pub(super) use creation_patch::creation_patch;

fn failure(stop: Stop) -> TransactionCommitError {
    match stop {
        Stop::Payload(detail) => {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(detail)
        }
        Stop::Request(cause) => TransactionCommitError::execution(CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(cause),
            partition_identity: None,
        }),
    }
}
