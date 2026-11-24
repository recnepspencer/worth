//! Attach the index owner's funded result to the exact prepared publication.
use crate::branch::root::RelationalBranchRoot;
use crate::indexes::authority::PreparedCandidateIndexPublication;
use crate::indexes::data::{DerivedIndexMaintenanceDenialKind, DerivedIndexMaintenanceWork};
use crate::publication::{bundle::PublicationStage, data::PublicationError};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::sync::Arc;

pub(super) fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &crate::runtime::state::subsystems::indexing::IndexDefinitionReadBinding,
    budget: Option<&crate::execution::RequestWorkBudget>,
    predecessor: &Arc<RelationalBranchRoot>,
    successor: &Arc<RelationalBranchRoot>,
    definitions: Option<
        &crate::runtime::state::subsystems::indexing::PreparedRelationalIndexDefinitions,
    >,
    maintenance_budget: Option<crate::indexes::data::DerivedIndexMaintenanceBudget>,
) -> Result<Option<PreparedCandidateIndexPublication>, TransactionCommitError> {
    let change = successor.commit_change().ok_or_else(|| {
        TransactionCommitError::publication(PublicationError::new(
            PublicationStage::BundleAssembly,
            "prepared root lacks its maintenance change",
        ))
    })?;
    let operation = || {
        indexes
            .prepare_publication(
                execution,
                predecessor,
                successor,
                change,
                definitions,
                maintenance_budget,
            )
            .map_err(|denial| match denial.kind {
                DerivedIndexMaintenanceDenialKind::Execution(cause) => {
                    TransactionCommitError::execution(
                        crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                            kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                                cause,
                            ),
                            partition_identity: None,
                        },
                    )
                }
                kind => {
                    TransactionCommitError::publication(PublicationError::from_index_maintenance(
                        crate::indexes::data::DerivedIndexMaintenanceDenial {
                            kind,
                            work: denial.work,
                        },
                    ))
                }
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
