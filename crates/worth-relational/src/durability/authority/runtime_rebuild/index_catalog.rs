//! Recovery-only original catalog payload, issued after actual source admission.
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::history::data::CanonicalIndexDefinitionDelta;
use crate::runtime::RelationalRuntime;

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
        crate::transactions::data::TransactionCommitError,
    > {
        if runtime != self.runtime_instance_id {
            return Err(crate::transactions::data::TransactionCommitError::conflict(
                crate::transactions::data::CommitConflict::new(
                    crate::transactions::data::ConflictClass::ForeignRuntime {
                        expected_runtime_instance_id: runtime,
                        actual_runtime_instance_id: self.runtime_instance_id,
                    },
                ),
            ));
        }
        Ok((self.delta.clone(), self.authored.clone()))
    }
}
pub(super) fn readmit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    original: &crate::durability::migration::ReadmittedCanonicalCommit,
) -> Result<RecoveredIndexCatalog, RelationalRecoveryStop> {
    let delta = &original.envelope().index_definition_delta;
    for _ in delta.definitions() {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
    }
    runtime
        .indexes
        .readmit_catalog_delta(delta)
        .map_err(|detail| {
            DurabilityError::new(RecoveryFailureClass::CorruptCanonicalArtifact, detail)
        })?;
    if !delta.definitions().is_empty() {
        // A definition insert retires the installed routing, exactly as on the
        // live registration path; replay must derive index writes under the
        // same routing the original commit used.
        runtime
            .index_authority()
            .recover_installed_index_routing(execution)
            .map_err(RelationalRecoveryStop::Request)?;
    }
    Ok(RecoveredIndexCatalog {
        runtime_instance_id: runtime.runtime_instance_id(),
        delta: delta.clone(),
        authored: original.envelope().authored_index_writes.clone(),
    })
}
