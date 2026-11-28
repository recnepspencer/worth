//! Recheck Native's carried unique-key claims against the selected C head.
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_branch::internal_port::indexes::data::BoundedEntityFieldLookupDenialKind;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::IndexDefinitionReadBinding;
use worth_relational_mutation::internal_port::validation::unique_field_probes::UniqueFieldProbeClaims;

use contracts::history::data::coordinates::BranchId;
use contracts::mvcc::publication::outcome::refusals::RelationalPublicationDenial;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
use contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};

pub enum UniqueFieldAdmissionDenial {
    Conflict(CommitConflict),
    Unavailable(RelationalPublicationDenial),
    Request(contracts::execution::denial_cause::RelationalExecutionDenialCause),
}

impl From<UniqueFieldAdmissionDenial> for TransactionCommitError {
    fn from(denial: UniqueFieldAdmissionDenial) -> Self {
        match denial {
            UniqueFieldAdmissionDenial::Conflict(conflict) => conflict.into(),
            UniqueFieldAdmissionDenial::Unavailable(cause) => Self::publication_denied(cause),
            UniqueFieldAdmissionDenial::Request(cause) => Self::execution(
                contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                    kind: contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause),
                    partition_identity: None,
                }
            ),
        }
    }
}

pub fn admit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &IndexDefinitionReadBinding,
    branch: &BranchId,
    root: &RelationalBranchRoot,
    claims: &[UniqueFieldProbeClaims],
) -> Result<(), UniqueFieldAdmissionDenial> {
    for field in claims {
        let scope = field.scope();
        for (canonical, writer, value, retiring) in field.claims() {
            let occupied = indexes
                .unique_field_has_survivor(
                    execution,
                    branch,
                    root,
                    (
                        scope.entity_kind_id,
                        scope.backing_index_id,
                        scope.field_locator,
                    ),
                    canonical,
                    writer,
                    retiring,
                )
                .map_err(|error| index_denial(scope, error))?;
            if occupied {
                return Err(UniqueFieldAdmissionDenial::Conflict(
                    scope.duplicate_conflict(value.clone()),
                ));
            }
        }
    }
    Ok(())
}

impl UniqueFieldAdmissionDenial {
    pub(super) fn into_outcome(self) -> super::RelationalPublicationOutcome {
        match self {
            Self::Conflict(conflict) => super::RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::UniqueFieldConflict(conflict),
            ),
            Self::Unavailable(cause) => super::RelationalPublicationOutcome::denied(cause),
            Self::Request(cause) => super::RelationalPublicationOutcome::RequestStopped(
                cause
                    .as_request_denial()
                    .unwrap_or(worth_execution::WorkCeilingDenial::Panicked),
            ),
        }
    }
}

pub fn index_denial(
    scope: worth_relational_mutation::internal_port::validation::unique_field_probes::UniqueFieldScope<'_>,
    error: worth_relational_branch::internal_port::indexes::data::BoundedEntityFieldLookupDenial,
) -> UniqueFieldAdmissionDenial {
    match error.kind() {
        BoundedEntityFieldLookupDenialKind::Execution(cause) => {
            UniqueFieldAdmissionDenial::Request(cause)
        }
        _ => UniqueFieldAdmissionDenial::Unavailable(
            RelationalPublicationDenial::UniqueFieldIndexUnavailable {
                index_id: scope.backing_index_id,
                entity_kind_id: scope.entity_kind_id,
            },
        ),
    }
}
