//! Validate the selected publication and capture the exact paid successor.
use std::sync::Arc;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;

use crate::branch::{
    basis::AdmittedRelationalBranchBasis,
    root::{PreparedRelationalBranchRootCapture, RelationalBranchRootSchemaAuthority},
    root_selection::SelectedRelationalBranchState,
};
use crate::config::data::StorageConfig;
use crate::history::data::{CanonicalCommitEnvelope, CommitId, RelationalCommitReceipt};
use crate::mvcc::publication::authority::{
    assemble_versioned_publication, PreparedRelationalPublication,
    VersionedPublicationOriginalInputs,
};
use crate::mvcc::publication::validation::{
    validate_prepared_publication, PublicationRequest, PublicationSequence,
};
use crate::mvcc::publication::PublicationPreparationDenial;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

pub(crate) struct PublicationRootCapture<'a, Attach> {
    pub(crate) history: &'a RelationalPreparationHistory,
    pub(crate) runtime_identity: u64,
    pub(crate) symbols: &'a RuntimeSymbolTable,
    pub(crate) planned_schema: Option<&'a Arc<RelationalBranchRootSchemaAuthority>>,
    pub(crate) storage: &'a StorageConfig,
    pub(crate) attach_execution_inventory: Attach,
}

impl<Attach: FnOnce(&PreparedRelationalBranchRootCapture)> PublicationRootCapture<'_, Attach> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_commit_publication(
        self,
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
            self.history,
            self.runtime_identity,
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
            &(*envelope.owner_content()),
            schema_registry,
            self.symbols,
            self.planned_schema,
        )?;
        (self.attach_execution_inventory)(&root);
        let mutation = crate::config::data::MutationConfig {
            cascade_delete_policy: self.storage.cascade_delete_policy,
            adjacency_policy: self.storage.adjacency_policy.clone(),
            cross_context_policy: self.storage.cross_context_policy,
        };
        assemble_versioned_publication(
            execution,
            commit_reference,
            validated,
            root,
            envelope,
            self.runtime_identity,
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
