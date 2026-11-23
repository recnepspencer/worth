//! Capture the selected root with the exact admitted execution contracts for this round.
use std::sync::Arc;

use super::RelationalPreparationRuntime;
use crate::branch::{
    basis::AdmittedRelationalBranchBasis, root_selection::SelectedRelationalBranchState,
};
use crate::history::data::{CanonicalCommitEnvelope, CommitId, RelationalCommitReceipt};
use crate::mvcc::publication::authority::{
    assemble_versioned_publication, PreparedRelationalPublication,
    VersionedPublicationOriginalInputs,
};
use crate::mvcc::publication::validation::{
    validate_prepared_publication, PublicationRequest, PublicationSequence,
};
use crate::mvcc::publication::PublicationPreparationDenial;

impl RelationalPreparationRuntime {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_commit_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: CommitId,
        commit_reference: RelationalCommitReceipt,
        binding: &AdmittedRelationalBranchBasis,
        selected_branch_state: &SelectedRelationalBranchState,
        published_partition_delta: crate::storage::RelationalPublishedPartitionDelta,
        envelope: Arc<CanonicalCommitEnvelope>,
        schema_registry: &crate::schema::data::RelationalSchemaRegistry,
        output_positions: Option<crate::history::data::RequiredOutputPublicationPositions>,
    ) -> Result<PreparedRelationalPublication, PublicationPreparationDenial> {
        let validated = validate_prepared_publication(
            &self.history,
            self.runtime_instance_id(),
            PublicationRequest {
                commit_id,
                commit_reference: &commit_reference,
                binding,
                envelope: &envelope,
                sequence: PublicationSequence::Truth,
            },
        )?;
        let root = self.history.prepare_branch_root_capture(
            execution,
            selected_branch_state.state(),
            &published_partition_delta,
            selected_branch_state.root(),
            Arc::clone(&envelope),
            &envelope.content,
            schema_registry,
            &self.services.symbols,
            self.execution_contracts
                .as_ref()
                .map(|contracts| &contracts.schema),
        )?;
        if let Some(contracts) = &self.execution_contracts {
            root.root()
                .execution_contracts
                .set(Arc::clone(contracts))
                .expect("prepared Home root has not bound any executor");
        }
        let mutation = crate::config::data::MutationConfig {
            cascade_delete_policy: self.config.storage.cascade_delete_policy,
            adjacency_policy: self.config.storage.adjacency_policy.clone(),
            cross_context_policy: self.config.storage.cross_context_policy,
        };
        assemble_versioned_publication(
            execution,
            commit_reference,
            validated,
            root,
            envelope,
            self.runtime_instance_id(),
            Some(VersionedPublicationOriginalInputs {
                prestate_schema: selected_branch_state
                    .state()
                    .root()
                    .retained_schema_authority(),
                mutation,
            }),
            output_positions,
            None,
        )
        .map(PreparedRelationalPublication::from_versioned)
    }
}

impl<'a> From<&'a super::RelationalRuntime>
    for crate::mvcc::publication::authority::RecoveredPublicationInstall<'a>
{
    fn from(runtime: &'a super::RelationalRuntime) -> Self {
        Self {
            history: &runtime.history,
        }
    }
}
