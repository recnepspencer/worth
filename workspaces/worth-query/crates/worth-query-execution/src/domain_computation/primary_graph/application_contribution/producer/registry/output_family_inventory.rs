use std::any::TypeId;
use std::collections::BTreeMap;

use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryInstalledApplicationProducerRegistry;

impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn output_family_bindings(
        &self,
    ) -> BTreeMap<String, Vec<TypeId>> {
        let mut families = BTreeMap::<String, Vec<TypeId>>::new();
        for entry in self.entries.values() {
            families
                .entry(entry.declaration.output_family.clone())
                .or_default()
                .push(entry.declaration.operation_binding_type);
        }
        for bindings in families.values_mut() {
            bindings.sort_unstable();
            bindings.dedup();
        }
        families
    }
}
