//! Optional-field classification from the installed aspect contract.

use worth_foundational::facade::{AspectFieldLocator, AspectShape, FieldRequirement};

use super::WorthQueryPrimaryGraphLayout;

impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation) fn field_is_optional(
        &self,
        entity: &str,
        locator: &AspectFieldLocator,
    ) -> bool {
        let Some(field) = locator.field_path().fields().first() else {
            return false;
        };
        let Some(contract) = self.aspect_contract(entity, locator.aspect().aspect_key()) else {
            return false;
        };
        let AspectShape::Struct(shape) = contract.shape() else {
            return false;
        };
        shape
            .field(field)
            .is_some_and(|field| field.requirement() == FieldRequirement::Optional)
    }
}
