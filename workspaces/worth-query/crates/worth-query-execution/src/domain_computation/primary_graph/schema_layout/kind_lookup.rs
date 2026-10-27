//! Exact application entity-kind lookup at the installed layout owner.
use worth_relational::facade::identity::KindId;
impl super::WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation) fn entity_kind(&self, entity: &str) -> Option<KindId> {
        self.entity_kinds.get(entity).copied()
    }

    pub(in crate::domain_computation::primary_graph) fn entity_kind_lookup_work(
        &self,
        entity: &str,
    ) -> Option<u64> {
        let entries = self.entity_kinds.len();
        let levels = usize::BITS as usize - entries.max(1).leading_zeros() as usize;
        let comparisons = entries.min(11).checked_mul(levels)?.checked_add(1)?;
        let bytes_per_comparison = entity.len().checked_add(1)?;
        u64::try_from(comparisons.checked_mul(bytes_per_comparison)?).ok()
    }

    pub(in crate::domain_computation::primary_graph) fn entity_name(
        &self,
        kind: KindId,
    ) -> Option<&str> {
        self.entity_kinds
            .iter()
            .find_map(|(name, candidate)| (*candidate == kind).then_some(name.as_str()))
    }
}
