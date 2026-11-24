//! Capture the installed lineage basis and install entity-slot bindings in the staged commit.

use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use crate::history::data::RelationalCommitReceipt;
use crate::lineage::authority::{prepare_posture_finalization, resolve_commit_naming_plan};
use crate::lineage::data::{CommitNamingPlan, LineagePostureClaim, PreparedLineageFinalization};
use crate::storage::overlay::WorkingState;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::outcomes::{
    created_entity_bindings::CommitCreatedEntityBindings,
    created_relation_bindings::CommitCreatedRelationBindings,
    transaction_commit_error::TransactionCommitError,
};
use crate::transactions::data::primitives::RecordRef;

use crate::runtime::state::subsystems::{
    lineage::LineageNamingBasisBinding, lineage_identity::LineageIdentityAllocator,
    services::instrumentation::RuntimeInstrumentation,
};

pub(crate) struct LineagePreparationAuthority<'a> {
    pub(crate) schema: &'a crate::schema::data::RelationalSchemaRegistry,
    pub(crate) naming: &'a LineageNamingBasisBinding,
    pub(crate) identities: &'a LineageIdentityAllocator,
    pub(crate) instrumentation: &'a RuntimeInstrumentation,
}

impl<'runtime> LineagePreparationAuthority<'runtime> {
    pub(crate) fn ensure_lineage_for_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        staged: &mut WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
        created_entities: &CommitCreatedEntityBindings,
        created_relations: &CommitCreatedRelationBindings,
        naming_plan: Option<&CommitNamingPlan>,
        graph_selection: Option<&crate::history::data::CanonicalOwnedGraphSelection>,
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        let postures = match naming_plan {
            Some(plan) => resolve_commit_naming_plan(plan, created_entities, created_relations)
                .map_err(TransactionCommitError::lineage_name_denial)?,
            None => Vec::new(),
        };
        self.finalize_commit_with_installed_schema(
            execution,
            staged,
            commit,
            merged_plan,
            changed_records,
            &postures,
            graph_selection,
        )
    }
}

impl<'runtime> LineagePreparationAuthority<'runtime> {
    /// Production entry over Relational's own installed schema (fail-closed
    /// `MissingBirth`) and branch naming state (zero-clone read guard).
    fn finalize_commit_with_installed_schema(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        staged: &mut WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
        postures: &[LineagePostureClaim],
        graph_selection: Option<&crate::history::data::CanonicalOwnedGraphSelection>,
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        let schema = self.schema;
        let basis = self.naming.read_guard(commit.branch_id.clone());
        let mut prepared = prepare_posture_finalization(
            self.identities,
            staged,
            commit,
            merged_plan,
            changed_records,
            schema.persistent_name_schemes(),
            Some(schema),
            &basis,
            postures,
            graph_selection,
        )?;
        install_lineage_bindings(execution, staged, &mut prepared)?;
        drop(basis);
        self.instrumentation.count(|counters| {
            counters.count_lineage_finalization(
                prepared.artifact().event_batch().events().len(),
                prepared.artifact().decision_log().decisions().len(),
            );
        });
        Ok(prepared)
    }
}

#[cfg(test)]
impl<'runtime> LineagePreparationAuthority<'runtime> {
    /// Test-only production-authority entry verifying Relational ignores
    /// caller-supplied live/retired maps and derives them from its own
    /// `LineageSubsystem`.
    pub(crate) fn ensure_lineage_for_commit_with_postures(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        staged: &mut WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
        input: &crate::lineage::authority::posture_fixture::LineageCommitPostureInput,
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        let basis = self.naming.read_guard(commit.branch_id.clone());
        let mut prepared = prepare_posture_finalization(
            self.identities,
            staged,
            commit,
            merged_plan,
            changed_records,
            &input.scheme_registry,
            None,
            &basis,
            &input.postures,
            None,
        )?;
        install_lineage_bindings(execution, staged, &mut prepared)?;
        drop(basis);
        self.instrumentation.count(|counters| {
            counters.count_lineage_finalization(
                prepared.artifact().event_batch().events().len(),
                prepared.artifact().decision_log().decisions().len(),
            );
        });
        Ok(prepared)
    }
}

fn install_lineage_bindings(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    staged: &mut WorkingState,
    prepared: &mut PreparedLineageFinalization,
) -> Result<(), TransactionCommitError> {
    for (entity_id, lineage_id) in prepared.take_entity_bindings() {
        install_lineage_id(execution, staged, entity_id, lineage_id)?;
    }
    Ok(())
}

fn install_lineage_id(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    staged: &mut WorkingState,
    entity_id: crate::identity::data::EntityId,
    lineage_id: crate::identity::data::LineageId,
) -> Result<(), TransactionCommitError> {
    staged
        .get_partition_mut(entity_id.partition_id)
        .entity_arena
        .install_lineage_id(execution, entity_id.slot_index(), lineage_id)
        .map_err(|denial| match denial {
            crate::storage::substrate::ExtraLineageDenial::Request(cause) => {
                TransactionCommitError::execution(cause.into())
            }
            crate::storage::substrate::ExtraLineageDenial::Missing(detail) => {
                TransactionCommitError::publication(
                    crate::publication::data::PublicationError::new(
                        crate::publication::bundle::PublicationStage::BundleAssembly,
                        detail,
                    ),
                )
            }
        })
}
