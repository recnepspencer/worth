use super::WorthQueryPrimaryGraphLayout;
use worth_foundational::facade::{AspectKey, FieldKey};
impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn supports_relation(
        &self,
        relation: &str,
    ) -> bool {
        self.relation(relation).is_some()
    }
    pub(in crate::domain_computation::primary_graph) fn supports_equality_field(
        &self,
        aspect: &AspectKey,
        field: &FieldKey,
    ) -> bool {
        self.equality_field_keys
            .get(aspect)
            .is_some_and(|fields| fields.contains(field))
    }
    pub(in crate::domain_computation::primary_graph) fn supports_projection_field(
        &self,
        aspect: &AspectKey,
        field: &FieldKey,
    ) -> bool {
        self.projection_field_keys
            .get(aspect)
            .is_some_and(|fields| fields.contains(field))
    }
}
