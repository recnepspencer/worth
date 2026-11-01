//! Original-source reconstruction is DATA processing under one admitted operation.
mod branches;
mod canonical;
mod capture;
mod contracts;
mod draft;
mod metadata;
mod prepare;
mod records;
mod regions;
mod roots;
mod schemas;
mod source;
mod symbols;
use crate::durability::data::{
    RelationalNativeCheckpoint, RelationalNativeCheckpointCaptureDenial as Denial,
};

/// The all-root producer's strong result, retaining its original read and ACK custody.
/// Descriptive bytes and ordinary captures have no conversion into this result.
#[derive(Debug)]
pub struct CompleteRelationalNativeCheckpoint {
    checkpoint: RelationalNativeCheckpoint,
}
impl CompleteRelationalNativeCheckpoint {
    pub fn checkpoint(&self) -> &RelationalNativeCheckpoint {
        &self.checkpoint
    }
}

fn root_stop(cause: crate::branch::RelationalBranchRootCaptureDenial) -> Denial {
    match cause {
        crate::branch::RelationalBranchRootCaptureDenial::Request(cause) => Denial::Request(cause),
        crate::branch::RelationalBranchRootCaptureDenial::StorageUnavailable(cause) => cause.into(),
        cause => Denial::Durability(crate::durability::data::DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::CorruptCanonicalArtifact,
            format!("original all-root reconstruction disagrees: {cause:?}"),
        )),
    }
}
fn transaction_stop(cause: crate::transactions::data::TransactionCommitError) -> Denial {
    match cause {
        crate::transactions::data::TransactionCommitError::Execution { denial, .. } => {
            match denial.kind {
                crate::transactions::data::CommitExecutionDenialKind::Cause(cause) => {
                    Denial::Request(cause)
                }
            }
        }
        cause => Denial::Durability(crate::durability::data::DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::CorruptCanonicalArtifact,
            format!("sealed original mutation input refused: {cause:?}"),
        )),
    }
}

impl super::DurabilityAuthority<'_> {
    pub(crate) fn complete_native_checkpoint_with_policy(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        placement: crate::durability::data::RelationalCheckpointArtifactPolicy<'_>,
    ) -> Result<CompleteRelationalNativeCheckpoint, Denial> {
        prepare::prepare(self.runtime, execution, placement)
    }
}
#[cfg(test)]
mod tests;
