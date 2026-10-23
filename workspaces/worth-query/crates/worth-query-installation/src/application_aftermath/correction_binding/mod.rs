mod validation;
pub(crate) use validation::resolve_inverse_binding;

use worth_query_declaration::facade::application_aftermath::DeclaredInverseBinding;

/// Callback-free inverse association resolved against the native mutation inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledInverseBinding {
    declaration: DeclaredInverseBinding,
    source_type: std::any::TypeId,
    inverse_type: std::any::TypeId,
    disclosure_query_type: std::any::TypeId,
}

impl InstalledInverseBinding {
    pub fn declaration(&self) -> &DeclaredInverseBinding {
        &self.declaration
    }
    pub fn source_type(&self) -> std::any::TypeId {
        self.source_type
    }
    pub fn inverse_type(&self) -> std::any::TypeId {
        self.inverse_type
    }
    pub fn disclosure_query_type(&self) -> std::any::TypeId {
        self.disclosure_query_type
    }
}
