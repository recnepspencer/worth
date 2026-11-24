//! Recovery-only original catalog payload, issued after actual source admission.
use branch::history::data::canonical_commit_envelope::index_catalog::CanonicalIndexDefinitionDelta;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::IndexingSubsystem;

#[derive(Debug, Clone)]
pub struct RecoveredIndexCatalog {
    runtime_instance_id: u64,
    pub delta: CanonicalIndexDefinitionDelta,
    authored: branch::history::data::canonical_commit_envelope::authored_index_writes::CanonicalAuthoredIndexWrites,
}
impl RecoveredIndexCatalog {
    pub fn original_for(
        &self,
        runtime: u64,
    ) -> Result<
        (
            CanonicalIndexDefinitionDelta,
            branch::history::data::canonical_commit_envelope::authored_index_writes::CanonicalAuthoredIndexWrites,
        ),
        contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    >{
        self.require_runtime(runtime)?;
        Ok((self.delta.clone(), self.authored.clone()))
    }

    pub fn require_runtime(
        &self,
        runtime: u64,
    ) -> Result<
        (),
        contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    > {
        if runtime != self.runtime_instance_id {
            return Err(contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::conflict(
                contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                    contracts::transactions::data::outcomes::conflict_class::ConflictClass::ForeignRuntime {
                        expected_runtime_instance_id: runtime,
                        actual_runtime_instance_id: self.runtime_instance_id,
                    },
                ),
            ));
        }
        Ok(())
    }
}
pub fn readmit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    indexes: &IndexingSubsystem,
    runtime_instance_id: u64,
    original: &crate::durability::migration::ReadmittedCanonicalCommit,
    recover_index_routing: impl FnOnce() -> Result<
        (),
        contracts::execution::denial_cause::RelationalExecutionDenialCause,
    >,
) -> Result<RecoveredIndexCatalog, RelationalRecoveryStop> {
    let delta = original.envelope().owner_index_definition_delta();
    for _ in delta.definitions() {
        contracts::execution::request_custody::check_request(execution)
            .map_err(RelationalRecoveryStop::Request)?;
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
