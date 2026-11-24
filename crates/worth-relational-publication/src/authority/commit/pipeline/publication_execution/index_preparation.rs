//! Attach the index owner's funded result to the exact prepared publication.
use worth_relational_contracts::internal_port as contracts;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_indexes::internal_port::indexes::authority::maintenance::candidate_publication::PreparedCandidateIndexPublication;
use contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind;
use contracts::indexes::data::maintenance::DerivedIndexMaintenanceWork;
use contracts::publication::bundle::publication_status::PublicationStage;
use contracts::publication::data::publication_error::PublicationError;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::sync::Arc;

pub(super) fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::IndexDefinitionReadBinding,
    budget: Option<&contracts::execution::request_work::RequestWorkBudget>,
    predecessor: &Arc<RelationalBranchRoot>,
    successor: &Arc<RelationalBranchRoot>,
    definitions: Option<
        &worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::definition_preparation::PreparedRelationalIndexDefinitions,
    >,
    maintenance_budget: Option<
        contracts::indexes::data::maintenance::DerivedIndexMaintenanceBudget,
    >,
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
                        contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                            kind: contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                                cause,
                            ),
                            partition_identity: None,
                        },
                    )
                }
                kind => {
                    TransactionCommitError::publication(PublicationError::from_index_maintenance(
                        contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenial {
                            kind,
                            work: denial.work,
                        },
                    ))
                }
            })
    };
    let budget = match (
        contracts::execution::request_custody::request_lease(execution),
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
            if contracts::execution::denial_cause::RelationalExecutionDenialCause::from(cause)
                == contracts::execution::denial_cause::RelationalExecutionDenialCause::NestedStopped =>
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
