use std::any::TypeId;
use std::collections::BTreeMap;

use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryInstalledApplicationProducerRegistry;

impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    /// Each output family's operation bindings, with the output role each
    /// binding's producer reads as the family's current output.
    pub(in crate::domain_computation::primary_graph) fn output_family_bindings(
        &self,
    ) -> BTreeMap<String, Vec<(TypeId, String)>> {
        let mut families = BTreeMap::<String, Vec<(TypeId, String)>>::new();
        for entry in self.entries.values() {
            families
                .entry(entry.declaration.output_family.clone())
                .or_default()
                .push((
                    entry.declaration.operation_binding_type,
                    entry.declaration.output_role.clone(),
                ));
        }
        for bindings in families.values_mut() {
            bindings.sort_unstable();
            bindings.dedup();
        }
        families
    }
}
