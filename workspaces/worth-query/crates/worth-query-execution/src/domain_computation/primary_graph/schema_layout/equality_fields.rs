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
            .filter(|layout| layout.equality_queryable)
    }

    pub(in crate::domain_computation) fn equality_selector_index(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<DerivedIndexId> {
        self.equality_field(entity, aspect, field)
            .and_then(|layout| layout.equality_index_id)
    }

    pub(in crate::domain_computation::primary_graph) fn equality_fields_mut(
        &mut self,
    ) -> impl Iterator<Item = (&(String, String, String), &mut WorthQueryPrimaryFieldLayout)> {
        self.fields
            .iter_mut()
            .filter(|(_, layout)| layout.equality_queryable)
    }

    pub(in crate::domain_computation::primary_graph) fn equality_index_ids(
        &self,
    ) -> impl Iterator<Item = DerivedIndexId> + '_ {
        self.fields
            .values()
            .filter_map(|field| field.equality_index_id)
    }
}
