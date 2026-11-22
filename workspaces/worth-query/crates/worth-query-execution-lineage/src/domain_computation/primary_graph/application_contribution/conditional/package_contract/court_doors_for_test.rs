//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationConditionalPackageContract {
    #[doc(hidden)]
    pub fn operation_binding_for_test(
        &self,
    ) -> &WorthQueryPortableApplicationConditionalOperationBinding {
        Self::operation_binding(self)
    }
}

impl WorthQueryApplicationConditionalPackageContract {
    #[doc(hidden)]
    pub fn target_for_test(&self) -> &WorthQueryConditionalNodeLocation {
        Self::target(self)
    }
}
