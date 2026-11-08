use crate::branch::{AdmittedRelationalBranchBasis, RelationalBranchLifecyclePosture};
use crate::identity::data::EntityId;
use crate::mvcc::{BranchBoundRelationalTransaction, RelationalPreparationPort};
use crate::mvcc::{
    RelationalRetainedEntitySelection, RelationalRetainedEntitySelectionDenial as Denial,
};
use crate::runtime::RelationalPreparationRuntime;

impl RelationalPreparationPort {
    /// Transfer the original current transaction and an actual retained source
    /// into the dedicated selection preparation receiver. The target must have
    /// no staged reads, commands, schema/recovery work or active savepoints.
    pub fn admit_retained_entity_selection(
        &self,
        transaction: BranchBoundRelationalTransaction,
        historical: &AdmittedRelationalBranchBasis,
        entity: EntityId,
    ) -> Result<RelationalRetainedEntitySelection, Denial> {
        let runtime = self.binding.runtime_snapshot();
        let _operation = runtime.admit_operation().ok_or(Denial::OwnerUnavailable)?;
        runtime.require_retained_selection_profile()?;
        require_pristine(&transaction)?;
        runtime.validate_retained_selection_sources(&transaction.basis, historical)?;
        if entity.generation.is_zero() {
            return Err(Denial::IdentityChanged);
        }
        Ok(RelationalRetainedEntitySelection {
            transaction,
            historical: historical.clone(),
            entity,
        })
    }
}

pub(super) fn require_pristine(
    transaction: &BranchBoundRelationalTransaction,
) -> Result<(), Denial> {
    let intent = &transaction.intent;
    if !transaction.batches().is_empty()
        || !transaction.read_leases.is_empty()
        || !transaction.merge_parent_bases.is_empty()
        || transaction.recovered_codec_publication_custody.is_some()
        || transaction.schema_authority_input.is_some()
        || intent.proposed_schema_transition().is_some()
        || intent.schema_reconciliation_policy().is_some()
        || intent.materialization_mode().is_some()
        || !transaction.savepoints.is_empty()
        || transaction.next_savepoint_ordinal != 1
        || transaction.last_merged_plan.is_some()
        || transaction.pending_symbols.is_some()
        || transaction.footprint.reads().len() != 0
        || transaction.footprint.writes().len() != 0
        || transaction.footprint.write_partitions().len() != 0
    {
        return Err(Denial::TargetNotPristine);
    }
    Ok(())
}

impl RelationalPreparationRuntime {
    pub(crate) fn require_retained_selection_profile(&self) -> Result<(), Denial> {
        if self.durability_admission.has_attached_medium() {
            return Err(Denial::DurabilityUnsupported);
        }
        Ok(())
    }

    pub(crate) fn validate_retained_selection_sources(
        &self,
        current: &AdmittedRelationalBranchBasis,
        historical: &AdmittedRelationalBranchBasis,
    ) -> Result<(), Denial> {
        if current.identity().runtime_instance_id() != self.runtime_instance_id()
            || historical.identity().runtime_instance_id() != self.runtime_instance_id()
        {
            return Err(Denial::ForeignRuntime);
        }
        if !current.is_current() {
            return Err(Denial::TargetNotCurrent);
        }
        let source_cell = self
            .history
            .branch_cell(historical.identity().branch_id())
            .filter(|cell| cell.identity() == historical.identity())
            .ok_or(Denial::HistoricalSourceRetired)?;
        if source_cell.lifecycle_posture() != RelationalBranchLifecyclePosture::Live {
            return Err(Denial::HistoricalSourceRetired);
        }
        if historical.commit_identity().is_none() {
            return Err(Denial::HistoricalSourceUnavailable);
        }
        let target_schema = current.inner.root.retained_schema_authority();
        let source_schema = historical.inner.root.retained_schema_authority();
        let same_authority = std::sync::Arc::ptr_eq(&source_schema, &target_schema);
        if !same_authority
            && (!source_schema.matches(target_schema.snapshot())
                || source_schema.descriptor_semantics_version()
                    != target_schema.descriptor_semantics_version())
        {
            return Err(Denial::SchemaChanged);
        }
        Ok(())
    }
}
