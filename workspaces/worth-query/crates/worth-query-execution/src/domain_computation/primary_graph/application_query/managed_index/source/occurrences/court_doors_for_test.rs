//! Sealed test doors over `PersistentSourceRows` for external execution courts.
use super::*;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::runtime::{
    ManagedIndexSourceChanges, PreparedManagedIndexSourceOccurrence,
};

impl PersistentSourceRows {
    #[doc(hidden)]
    pub fn acquire_for_test(
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        graph: &WorthQueryInstalledGraphReadContract,
        maximum: usize,
    ) -> Result<Self, ManagedIndexSourcePreparationDenial> {
        let owner = WorthQueryGraphWorkSessionIdentity::for_native_source(scope)?;
        Self::acquire(execution, &scope.proposed(), owner, graph, maximum)
            .map_err(MaterialDenial::into_native)
    }

    #[doc(hidden)]
    pub fn update_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        changed: &[(
            Option<ManagedIndexSourceOccurrence>,
            Option<PreparedManagedIndexSourceOccurrence>,
        )],
        graph: &WorthQueryInstalledGraphReadContract,
        inherited: bool,
    ) -> Result<(Self, ManagedIndexSourceChanges), ManagedIndexSourcePreparationDenial> {
        Self::update(self, execution, scope, changed, graph, inherited)
            .map_err(MaterialDenial::into_native)
    }

    #[doc(hidden)]
    pub fn contains_for_test(&self, entity: EntityId) -> bool {
        Self::contains(self, entity)
    }

    #[doc(hidden)]
    pub fn child_string_fields_for_test(
        &self,
        row: EntityId,
        relation_slot: &str,
        field_slot: &str,
    ) -> Vec<String> {
        let node = self.get(row).expect("retained row present");
        let relation = node.relation(relation_slot).expect("projected relation");
        relation
            .rows()
            .iter()
            .map(
                |child| match child.field(field_slot).expect("projected field").value() {
                    AspectValue::String(InternedString::Raw(text)) => text.clone(),
                    AspectValue::String(InternedString::Stored(text)) => text.as_str().to_owned(),
                    other => panic!("expected String field, got {other:?}"),
                },
            )
            .collect()
    }
}
