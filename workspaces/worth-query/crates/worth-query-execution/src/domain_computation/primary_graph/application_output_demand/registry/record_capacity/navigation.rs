//! Charged navigation and removal movement for the ordered record ledger.

use super::*;

impl DemandRegistryState {
    /// The selected ordered lookup is charged before its first comparison.
    /// A non-root B-tree node has at least five keys and at most eleven keys.
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn charge_record_lookup(
        &self,
        key: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.charge_record_lookup_count(key, self.records.len(), admission)
    }

    /// Every live record occupies at least one byte of this finite installed
    /// ledger. This prices a future selected lookup before a Signal effect.
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn charge_maximum_record_lookup(
        &self,
        key: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let levels = self
            .record_budget_bytes
            .map_or(Ok(usize::BITS as usize), ordered_levels)?;
        self.charge_record_lookup_levels(key, levels, 11, admission)
    }

    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn charge_record_lookup_after_insert(
        &self,
        key: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let count = self.records.len().checked_add(1).ok_or_else(work_denial)?;
        self.charge_record_lookup_count(key, count, admission)
    }

    fn charge_record_lookup_count(
        &self,
        key: &WorthQueryOutputDemandKey,
        count: usize,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.charge_record_lookup_levels(key, ordered_levels(count)?, count.min(11), admission)
    }

    fn charge_record_lookup_levels(
        &self,
        key: &WorthQueryOutputDemandKey,
        levels: usize,
        keys_per_level: usize,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        // A B-tree compares at most eleven resident keys per level. Across an
        // effect, pointer-width bounds every addressable tree without a quota.
        let comparisons = levels.checked_mul(keys_per_level).ok_or_else(work_denial)?;
        let comparison_work = super::super::key_comparison::comparison_work([key])
            .ok_or_else(work_denial)?
            .maximum;
        let work = comparisons
            .checked_mul(comparison_work.saturating_sub(1))
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(u64::try_from(work).map_err(|_| work_denial())?)
            .and_then(|()| admission.charge_ordered_operations(1, comparisons as u64))
            .map_err(admission_denial)
    }

    /// Rust's pinned BTreeMap removal can merge or rotate a parent and two
    /// children at each level of the searched path. Three complete node moves
    /// per level conservatively cover their initialized keys, values, and
    /// sibling shifts before this selected row is removed.
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn charge_record_removal_movement(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let levels = ordered_levels(self.records.len())?;
        let node = ordered_node_bytes().map_err(|_| work_denial())?;
        let work = levels
            .checked_mul(3)
            .and_then(|count| count.checked_mul(node))
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(u64::try_from(work).map_err(|_| work_denial())?)
            .map_err(admission_denial)
    }
}
