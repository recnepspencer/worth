//! The existing schema installation supplies binding names without registration.
use super::super::output_binding_identity::OutputBindingIdentity;
use super::WorthQueryApplicationOutputLineage;
use crate::equality_type_id::EqualityTypeId as TypeId;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn install_binding_identities<'a>(
        &mut self,
        bindings: impl Iterator<Item = (TypeId, &'a str)>,
    ) {
        for (binding, identity) in bindings {
            self.binding_identities.insert(binding, identity);
        }
    }
    pub fn binding_identity(&self, binding: TypeId) -> Option<OutputBindingIdentity> {
        self.binding_identities.identity(binding)
    }
    pub fn binding_type(&self, identity: &OutputBindingIdentity) -> TypeId {
        self.binding_identities
            .binding_type(identity)
            .expect("lineage binding is installed")
    }
    pub fn fixture_binding(&mut self, binding: TypeId, name: &str) -> OutputBindingIdentity {
        self.binding_identities.insert(binding, name);
        self.binding_identity(binding).unwrap()
    }
}
