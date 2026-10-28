//! Unit courts inspect resident index state without altering its admission.
use super::*;
use crate::indexes::data::DerivedIndexGeneration;
impl IndexAccess<'_> {
    pub(crate) fn generations_snapshot(&self) -> Vec<DerivedIndexGeneration> {
        self.runtime
            .indexes
            .all_generations()
            .into_iter()
            .map(|generation| generation.as_ref().clone())
            .collect()
    }

    pub(crate) fn entity_unique_field_entries(
        &self,
        field_locator: &worth_foundational::facade::AspectFieldLocator,
    ) -> Option<
        std::collections::BTreeMap<
            crate::storage::data::AuthoritativeFieldComparisonKey,
            std::collections::BTreeSet<crate::identity::data::EntityId>,
        >,
    > {
        self.runtime
            .indexes
            .with_unique_index(|index| index.get(field_locator).cloned())
    }
}
