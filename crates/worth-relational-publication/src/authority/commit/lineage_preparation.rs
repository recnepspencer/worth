//! Capture the installed lineage basis and install entity-slot bindings in the staged commit.

use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalization;
use contracts::lineage::data::postures::CommitNamingPlan;
use contracts::lineage::data::postures::{GroupPartitionClaim, LineagePostureClaim};
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::outcomes::commit_preparation::{
    CommitPreparationError, CommitPreparationErrorOwnerPort as _,
};
use contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use contracts::transactions::data::primitives::RecordRef;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::lineage::authority::commit_finalization::resolve_commit_naming_plan;
use worth_relational_lineage::internal_port::lineage::authority::posture_finalization::prepare_posture_finalization;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::NamingBasisUnavailable;
use worth_relational_mutation::internal_port::aggregate_identity::preparation::TouchedAggregatePartitions;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::naming_basis_binding::LineageNamingBasisBinding;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage_identity::LineageIdentityAllocator;
use branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;

pub struct LineagePreparationAuthority<'a> {
    pub schema: &'a contracts::schema::data::RelationalSchemaRegistry,
    pub naming: &'a LineageNamingBasisBinding,
    pub identities: &'a LineageIdentityAllocator,
    pub instrumentation: &'a RuntimeInstrumentation,
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
        aggregate_partitions: &TouchedAggregatePartitions,
        graph_selection: Option<&branch::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelection>,
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        let postures = match naming_plan {
            Some(plan) => resolve_commit_naming_plan(plan, created_entities, created_relations)
                .map_err(TransactionCommitError::lineage_name_denial)?,
            None => Vec::new(),
        };
        let groups = naming_plan.map_or(&[][..], |plan| plan.groups.as_slice());
        self.finalize_commit_with_installed_schema(
            execution,
            staged,
            commit,
            merged_plan,
            changed_records,
            (&postures, groups),
            aggregate_partitions,
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
        (postures, groups): (&[LineagePostureClaim], &[GroupPartitionClaim]),
        aggregate_partitions: &TouchedAggregatePartitions,
        graph_selection: Option<&branch::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelection>,
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        let schema = self.schema;
        let basis = self.naming.read_guard(commit.branch_id.clone());
        let prepared = prepare_posture_finalization(
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
        );
        basis
            .settle()
            .map_err(|denial| naming_basis_commit_error(commit, denial))?;
        let mut prepared = prepared?;
        let groups_result = super::group_finalization::finalize_commit_groups(
            commit,
            groups,
            aggregate_partitions,
            graph_selection,
            &basis,
            &mut prepared,
        );
        basis
            .settle()
            .map_err(|denial| naming_basis_commit_error(commit, denial))?;
        let transitions = groups_result?;
        let relation_bindings: Vec<_> = prepared
            .relation_lineages()
            .iter()
            .map(|(&rel, &lin)| (rel, lin))
            .collect();
        basis.preflight_commit_settlement(
            prepared.artifact().event_batch().events(),
            prepared.entity_bindings(),
            &relation_bindings,
            &transitions,
        );
        basis
            .settle()
            .map_err(|denial| naming_basis_commit_error(commit, denial))?;
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

impl<'runtime> LineagePreparationAuthority<'runtime> {
    /// Test-only production-authority entry verifying Relational ignores
    /// caller-supplied live/retired maps and derives them from its own
    /// `LineageSubsystem`.
    #[doc(hidden)]
    pub fn ensure_lineage_for_commit_with_postures_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        staged: &mut WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
        input: &worth_relational_lineage::internal_port::lineage::authority::posture_fixture::LineageCommitPostureInput,
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        let basis = self.naming.read_guard(commit.branch_id.clone());
        let prepared = prepare_posture_finalization(
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
        );
        basis
            .settle()
            .map_err(|denial| naming_basis_commit_error(commit, denial))?;
        let mut prepared = prepared?;
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

pub(super) fn naming_basis_commit_error(
    commit: &RelationalCommitReceipt,
    denial: NamingBasisUnavailable,
) -> TransactionCommitError {
    match denial {
        NamingBasisUnavailable::Storage(cause) => {
            TransactionCommitError::preparation(CommitPreparationError::storage_unavailable(
                commit.branch_id.clone(),
                Some(commit.commit_id.0),
                commit.version_id,
                contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
                    subject: contracts::storage::data::storage_unavailable::RelationalStorageSubject::Lineage {
                        first: contracts::identity::data::primitives::LineageId(0),
                        last: contracts::identity::data::primitives::LineageId(0),
                    },
                    cause,
                },
            ))
        }
        NamingBasisUnavailable::Unrecoverable => TransactionCommitError::publication(
            contracts::publication::data::publication_error::PublicationError::new(
                contracts::publication::bundle::publication_status::PublicationStage::BundleAssembly,
                "branch naming basis is unavailable",
            ),
        ),
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
    entity_id: contracts::identity::data::record_ids::EntityId,
    lineage_id: contracts::identity::data::primitives::LineageId,
) -> Result<(), TransactionCommitError> {
    staged
        .get_partition_mut(entity_id.partition_id)
        .entity_arena
        .install_lineage_id(execution, entity_id.slot_index(), lineage_id)
        .map_err(|denial| match denial {
            worth_relational_storage::internal_port::storage::substrate::record_arena::arena::entity_lineage::ExtraLineageDenial::Request(cause) => {
                TransactionCommitError::execution(cause.into())
            }
            worth_relational_storage::internal_port::storage::substrate::record_arena::arena::entity_lineage::ExtraLineageDenial::Missing(detail) => {
                TransactionCommitError::publication(
                    contracts::publication::data::publication_error::PublicationError::new(
                        contracts::publication::bundle::publication_status::PublicationStage::BundleAssembly,
                        detail,
                    ),
                )
            }
        })
}
