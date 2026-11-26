//! Validate the selected publication and capture the exact paid successor.
use std::sync::Arc;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelope, CanonicalCommitEnvelopeBodyFieldsPort as _,
};
use worth_relational_contracts::internal_port as contracts;

use crate::mvcc::publication::authority::{
    assemble_versioned_publication, PreparedRelationalPublication,
    VersionedPublicationOriginalInputs,
};
use crate::mvcc::publication::validation::{
    validate_prepared_publication, PublicationRequest, PublicationSequence,
};
use crate::mvcc::publication::PublicationPreparationDenial;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use contracts::config::data::sections::StorageConfig;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::coordinates::CommitId;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use worth_relational_branch::internal_port::branch::root::PreparedRelationalBranchRootCapture;
use worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

pub struct PublicationRootCapture<'a, Attach> {
    pub history: &'a RelationalPreparationHistory,
    pub runtime_identity: u64,
    pub symbols: &'a RuntimeSymbolTable,
    pub planned_schema: Option<&'a Arc<RelationalBranchRootSchemaAuthority>>,
    pub storage: &'a StorageConfig,
    pub attach_execution_inventory: Attach,
}

impl<Attach: FnOnce(&PreparedRelationalBranchRootCapture)> PublicationRootCapture<'_, Attach> {
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_commit_publication(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: CommitId,
        commit_reference: RelationalCommitReceipt,
        binding: &AdmittedRelationalBranchBasis,
        selected_branch_state: &SelectedRelationalBranchState,
        published_partition_delta: worth_relational_storage::internal_port::storage::overlay::published_delta::RelationalPublishedPartitionDelta,
        envelope: Arc<CanonicalCommitEnvelope>,
        schema_registry: &contracts::schema::data::RelationalSchemaRegistry,
        output_positions: Option<worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputPublicationPositions>,
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
            binding.residency(),
            Arc::clone(&envelope),
            envelope.owner_content(),
            schema_registry,
            self.symbols,
            self.planned_schema,
        )?;
        (self.attach_execution_inventory)(&root);
        let mutation = contracts::config::data::policies::MutationConfig {
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
