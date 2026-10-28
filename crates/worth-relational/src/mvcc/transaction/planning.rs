//! Bulk planning and admission for one detached transaction.

use crate::transactions::data::{
    CommitConflict, LineageSafeBulkMutationBatch, NamingStableBulkMutationBatch,
    PlannedBulkMutationBatch, ProvenanceCompleteBulkMutationBatch,
};

impl super::BranchBoundRelationalTransaction {
    /// Describe the currently authored keys without issuing text identities.
    /// Call `prepare_merged_plan` first when naming admission requires interned
    /// keys; that fallible preview retains the dictionary in this transaction.
    pub fn plan_bulk_mutation_batch(
        &self,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<Option<PlannedBulkMutationBatch>, CommitConflict> {
        self.ensure_runtime_affinity_for_runtime(runtime)?;
        let intents =
            crate::transactions::planning::bulk::canonical_bulk_mutation_intents(self.batches());
        if intents.is_empty() {
            return Ok(None);
        }

        Ok(Some(PlannedBulkMutationBatch {
            transaction_id: self.transaction_id,
            scope: crate::transactions::planning::bulk::bulk_mutation_scope(&intents),
            locality: crate::transactions::planning::bulk::bulk_mutation_locality(&intents),
            naming: crate::transactions::planning::bulk::bulk_mutation_naming(&intents),
            lineage: crate::transactions::planning::bulk::bulk_mutation_lineage(&intents),
            provenance: crate::transactions::planning::bulk::bulk_mutation_provenance(
                self.transaction_id,
                Some(self.basis.identity().branch_id().clone()),
                self.batches(),
            ),
            intents: intents.into(),
        }))
    }

    pub fn admit_naming_stable_bulk_mutation_batch(
        &self,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<Option<NamingStableBulkMutationBatch>, CommitConflict> {
        let Some(planned) = self.plan_bulk_mutation_batch(runtime)? else {
            return Ok(None);
        };
        crate::transactions::admission::bulk::validate_naming_plan(
            &planned,
            self.client_key_symbol_policy,
        )?;
        Ok(Some(
            crate::transactions::data::naming_stable_bulk_mutation_batch(planned),
        ))
    }

    pub fn admit_lineage_safe_bulk_mutation_batch(
        &self,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<Option<LineageSafeBulkMutationBatch>, CommitConflict> {
        let Some(naming_stable) = self.admit_naming_stable_bulk_mutation_batch(runtime)? else {
            return Ok(None);
        };
        crate::transactions::admission::bulk::validate_lineage_plan(naming_stable.planned())?;
        Ok(Some(
            crate::transactions::data::lineage_safe_bulk_mutation_batch(naming_stable),
        ))
    }

    pub fn admit_provenance_complete_bulk_mutation_batch(
        &self,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<Option<ProvenanceCompleteBulkMutationBatch>, CommitConflict> {
        let Some(lineage_safe) = self.admit_lineage_safe_bulk_mutation_batch(runtime)? else {
            return Ok(None);
        };
        crate::transactions::admission::bulk::validate_provenance_plan(
            lineage_safe.planned(),
            self.batches(),
        )?;
        Ok(Some(
            crate::transactions::data::provenance_complete_bulk_mutation_batch(lineage_safe),
        ))
    }
}
