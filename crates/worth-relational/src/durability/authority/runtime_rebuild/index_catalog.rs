//! Recovery-only original catalog payload, issued after actual source admission.
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::history::data::CanonicalIndexDefinitionDelta;
use crate::runtime::state::subsystems::indexing::IndexingSubsystem;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;

pub(super) mod predecessor;

#[derive(Debug, Clone)]
pub(crate) struct RecoveredIndexCatalog {
    runtime_instance_id: u64,
    delta: CanonicalIndexDefinitionDelta,
    authored: crate::history::data::CanonicalAuthoredIndexWrites,
}
impl RecoveredIndexCatalog {
    pub(crate) fn original_for(
        &self,
        runtime: u64,
    ) -> Result<
        (
            CanonicalIndexDefinitionDelta,
            crate::history::data::CanonicalAuthoredIndexWrites,
        ),
        crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    > {
        self.require_runtime(runtime)?;
        Ok((self.delta.clone(), self.authored.clone()))
    }

    fn require_runtime(
        &self,
        runtime: u64,
    ) -> Result<
        (),
        crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    > {
        if runtime != self.runtime_instance_id {
            return Err(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::conflict(
                crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                    crate::transactions::data::outcomes::conflict_class::ConflictClass::ForeignRuntime {
                        expected_runtime_instance_id: runtime,
                        actual_runtime_instance_id: self.runtime_instance_id,
                    },
                ),
            ));
        }
        Ok(())
    }
}
pub(super) fn readmit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &IndexingSubsystem,
    runtime_instance_id: u64,
    original: &crate::durability::migration::ReadmittedCanonicalCommit,
    recover_index_routing: impl FnOnce() -> Result<
        (),
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    >,
) -> Result<RecoveredIndexCatalog, RelationalRecoveryStop> {
    let delta = &(*original.envelope().owner_index_definition_delta());
    for _ in delta.definitions() {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
    }
    indexes.readmit_catalog_delta(delta).map_err(|detail| {
        DurabilityError::new(RecoveryFailureClass::CorruptCanonicalArtifact, detail)
    })?;
    if !delta.definitions().is_empty() {
        // A definition insert retires the installed routing, exactly as on the
        // live registration path; replay must derive index writes under the
        // same routing the original commit used.
        recover_index_routing().map_err(RelationalRecoveryStop::Request)?;
    }
    Ok(RecoveredIndexCatalog {
        runtime_instance_id,
        delta: delta.clone(),
        authored: (*original.envelope().owner_authored_index_writes()).clone(),
    })
}
