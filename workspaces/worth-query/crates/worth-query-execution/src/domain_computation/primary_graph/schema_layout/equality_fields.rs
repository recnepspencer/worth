use worth_relational::facade::indexes::DerivedIndexId;

use super::{WorthQueryPrimaryFieldLayout, WorthQueryPrimaryGraphLayout};

impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation) fn equality_field(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&WorthQueryPrimaryFieldLayout> {
        self.fields
            .get(&(entity.to_string(), aspect.to_string(), field.to_string()))
            .filter(|layout| layout.equality_index_id.is_some())
    }

    /// One owned lookup key and the comparison work of one std B-tree search.
    /// A non-leaf root has at least two children; each lower node has at
    /// least six, yielding minimum key counts 1, 11, 71, ... by level.
    pub(in crate::domain_computation::primary_graph) fn equality_lookup_bound(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<(u64, u64)> {
        let key_bytes = entity
            .len()
            .checked_add(aspect.len())?
            .checked_add(field.len())?;
        let mut levels = usize::from(!self.fields.is_empty());
        let mut minimum_keys = 1usize;
        while let Some(next) = minimum_keys.checked_mul(6).and_then(|n| n.checked_add(5)) {
            if next > self.fields.len() {
                break;
            }
            levels = levels.checked_add(1)?;
            minimum_keys = next;
        }
        let comparisons = levels.checked_mul(11)?;
        let navigation = comparisons
            .checked_mul(key_bytes.checked_add(3)?)?
            .checked_add(1)?;
        let work = key_bytes.checked_add(navigation)?;
        Some((u64::try_from(key_bytes).ok()?, u64::try_from(work).ok()?))
    }

    /// Preparation visits for measuring the selected B-tree lookup. Its
    /// base-six level loop cannot exceed this base-two width.
    pub(in crate::domain_computation::primary_graph) fn equality_lookup_measurement_work_bound(
        &self,
    ) -> Option<u64> {
        let width = usize::BITS.checked_sub(self.fields.len().leading_zeros())?;
        u64::from(width).checked_mul(4)?.checked_add(4)
    }

    pub(in crate::domain_computation::primary_graph) fn equality_fields_mut(
        &mut self,
    ) -> impl Iterator<Item = (&(String, String, String), &mut WorthQueryPrimaryFieldLayout)> {
        self.fields
            .iter_mut()
            .filter(|(_, layout)| layout.equality_index_id.is_some())
    }

    pub(in crate::domain_computation::primary_graph) fn equality_index_ids(
        &self,
    ) -> impl Iterator<Item = DerivedIndexId> + '_ {
        self.fields
            .values()
            .filter_map(|field| field.equality_index_id)
    }
}
