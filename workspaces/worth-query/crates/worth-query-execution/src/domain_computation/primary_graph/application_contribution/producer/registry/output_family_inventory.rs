use crate::domain_computation::primary_graph::output_family_identity::OutputFamilyIdentity;
use std::any::TypeId;
use std::collections::{BTreeMap, HashSet};

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
    ) -> BTreeMap<OutputFamilyIdentity, Vec<(TypeId, String)>> {
        declared_output_family_bindings(self.entries.values().map(|entry| &entry.declaration))
    }
}

pub(super) fn declared_output_family_bindings<'a>(
    declarations: impl Iterator<Item = &'a super::DeclaredProducerBinding>,
) -> BTreeMap<OutputFamilyIdentity, Vec<(TypeId, String)>> {
    let mut families = BTreeMap::<OutputFamilyIdentity, Vec<(TypeId, String)>>::new();
    for declaration in declarations {
        families
            .entry(declaration.output_family.clone().into())
            .or_default()
            .push((
                declaration.operation_binding_type,
                declaration.output_role.clone(),
            ));
    }
    for bindings in families.values_mut() {
        let mut seen = HashSet::new();
        bindings.retain(|binding| seen.insert(binding.clone()));
    }
    families
}
