//! Declared binding names order lineage; Rust type identity only locates metadata.
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::{collections::HashMap, sync::Arc};
use crate::domain_computation::primary_graph::vocabulary::output_binding_identity::OutputBindingIdentity;



#[derive(Default)]
pub struct OutputBindingInventory {
    by_type: HashMap<TypeId, OutputBindingIdentity>,
    by_identity: HashMap<OutputBindingIdentity, TypeId>,
}

impl OutputBindingInventory {
    pub(super) fn insert(&mut self, binding: TypeId, name: &str) {
        let identity = OutputBindingIdentity::declared(name);
        if let Some(old) = self.by_type.insert(binding, identity.clone()) {
            assert_eq!(old, identity);
        }
        if let Some(old) = self.by_identity.insert(identity, binding) {
            assert_eq!(old, binding);
        }
    }
    pub(super) fn identity(&self, binding: TypeId) -> Option<OutputBindingIdentity> {
        self.by_type.get(&binding).cloned()
    }
    pub(super) fn binding_type(&self, identity: &OutputBindingIdentity) -> Option<TypeId> {
        self.by_identity.get(identity).copied()
    }
}

mod court_doors_for_test;
