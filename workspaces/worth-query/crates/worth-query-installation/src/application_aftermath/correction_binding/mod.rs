mod validation;
pub(crate) use validation::resolve_correction_binding;

use worth_query_declaration::facade::application_aftermath::DeclaredCorrectionBinding;

/// Callback-free corrective association resolved against the native mutation inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledCorrectionBinding {
    declaration: DeclaredCorrectionBinding,
    source_type: std::any::TypeId,
    inverse_type: std::any::TypeId,
    disclosure_query_type: std::any::TypeId,
}

impl InstalledCorrectionBinding {
    pub fn declaration(&self) -> &DeclaredCorrectionBinding {
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
