use super::super::retained_charge::SignalExecutionBasisChargeDenial;
use super::SignalEvaluationStorage;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation,
};

impl SignalEvaluationStorage {
    /// Only captured immutable seed storage uses the diagnostic carrier lane.
    pub(in crate::data::graph::storage::execution_basis) fn measure_seed_heap_charge(
        &self,
        work: &mut Preparation,
    ) -> Result<Charge, SignalExecutionBasisChargeDenial> {
        work.visit()?;
        self.topology.require_retained_indexes()?;
        let Self {
            hot,
            warm,
            cold,
            retained_node_ledger: _, // Shared accounting service, not retained payload.
            retained_node_custody,
            retained_seed_custody,
            fork_custody,
            topology,
            // Inline state; newly added fields must cross this boundary too.
            compaction:
                crate::data::graph::compaction::CompactionState {
                    tombstone_count: _,
                    gc_threshold: _,
                    debt: _,
                    cursor: _,
                },
            causes,
            cause_readmission_required: _,
            conditional_versions,
            conditional_versions_custody,
            repeated_admissions,
            partitions,
            diagnostics,
        } = self;
        Ok(hot
            .retained_heap_charge(work)?
            .checked_add(warm.retained_heap_charge(work)?)?
            .checked_add(cold.retained_heap_charge(work)?)?
            .checked_add(retained_node_custody.retained_heap_charge(work)?)?
            .checked_add(retained_seed_custody.retained_heap_charge(work)?)?
            .checked_add(fork_custody.retained_heap_charge(work)?)?
            .checked_add(topology.retained_heap_charge(work)?)?
            .checked_add(causes.retained_heap_charge(work)?)?
            .checked_add(conditional_versions.retained_heap_charge(work)?)?
            .checked_add(conditional_versions_custody.retained_heap_charge(work)?)?
            .checked_add(repeated_admissions.retained_heap_charge(work)?)?
            .checked_add(partitions.retained_heap_charge(work)?)?
            .checked_add(diagnostics.retained_branch_carrier_charge(work)?)?)
    }
}
