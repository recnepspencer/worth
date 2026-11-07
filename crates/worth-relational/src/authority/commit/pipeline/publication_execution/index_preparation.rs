//! Attach the index owner's funded result to the exact prepared publication.
use crate::branch::RelationalBranchRoot;
use crate::indexes::data::{DerivedIndexMaintenanceDenialKind, DerivedIndexMaintenanceWork};
use crate::indexes::PreparedCandidateIndexPublication;
use crate::publication::{bundle::PublicationStage, data::PublicationError};
use crate::transactions::data::TransactionCommitError;
use std::sync::Arc;

pub(super) fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &crate::runtime::IndexDefinitionReadBinding,
    budget: Option<&crate::execution::RequestWorkBudget>,
    predecessor: &Arc<RelationalBranchRoot>,
    successor: &Arc<RelationalBranchRoot>,
) -> Result<Option<PreparedCandidateIndexPublication>, TransactionCommitError> {
    let change = successor.commit_change().ok_or_else(|| {
        TransactionCommitError::publication(PublicationError::new(
            PublicationStage::BundleAssembly,
            "prepared root lacks its maintenance change",
        ))
    })?;
    let operation = || {
        indexes
            .prepare_publication(execution, predecessor, successor, change)
            .map_err(|denial| match denial.kind {
                DerivedIndexMaintenanceDenialKind::Execution(cause) => {
                    TransactionCommitError::execution(
                        crate::transactions::data::CommitExecutionDenial {
                            kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                                cause,
                            ),
                            partition_identity: None,
                        },
                    )
                }
                kind => TransactionCommitError::publication(PublicationError::new(
                    PublicationStage::BundleAssembly,
                    format!("derived index maintenance denied: {kind:?}"),
                )),
            })
    };
    let budget = match (
        crate::authority::commit::pipeline::request_lease(execution),
        budget,
    ) {
        (Some(_), Some(budget)) => budget,
        _ => return operation(),
    };
    let mut result = None;
    let handoff = budget.run_stage(*execution, || result = Some(operation()));
    match (handoff, result) {
        (Ok(()), Some(result)) => result,
        (Err(cause), Some(Err(inner)))
            if crate::execution::RelationalExecutionDenialCause::from(cause)
                == crate::execution::RelationalExecutionDenialCause::NestedStopped =>
        {
            Err(inner)
        }
        (Err(cause), _) => Err(TransactionCommitError::execution(cause.into())),
        (Ok(()), None) => unreachable!(
            /* Proof: run_stage executes its callback synchronously whenever it succeeds; the callback sets result before returning. */
            "index preparation executed"
        ),
    }
}

impl super::PreparedCommitPublicationExecution {
    pub(crate) fn prepared_index_maintenance_work(&self) -> DerivedIndexMaintenanceWork {
        // Every candidate reaches prepare_publication. None is that owner's
        // verified no-definitions result, so its actual work is empty.
        self.prepared_indexes.as_ref().map_or_else(
            DerivedIndexMaintenanceWork::default,
            PreparedCandidateIndexPublication::work,
        )
    }
}
