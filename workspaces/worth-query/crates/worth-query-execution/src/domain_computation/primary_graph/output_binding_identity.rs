//! Declared binding names order lineage; Rust type identity only locates metadata.
use std::{any::TypeId, collections::HashMap, sync::Arc};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct OutputBindingIdentity(Arc<str>);

impl OutputBindingIdentity {
    pub(super) fn declared(name: &str) -> Self {
        Self(Arc::from(name))
    }
    #[cfg(test)]
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Default)]
pub(super) struct OutputBindingInventory {
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
