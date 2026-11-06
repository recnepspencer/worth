//! Sealed test doors over this module's private members.
use super::*;

impl FieldExpectation {
    #[doc(hidden)]
    pub fn aspect_for_test(&self) -> &domain::AspectContract {
        &self.aspect
    }
}

impl FieldExpectation {
    #[doc(hidden)]
    pub fn native_binding_for_test(&self) -> &domain::AspectBinding {
        &self.native_binding
    }
}

impl FieldExpectation {
    #[doc(hidden)]
    pub fn declared_for_test<F: schema::fragment::RecordSelection>(
        declaration: &schema::ErasedApplicationSchemaDeclaration,
        field: F,
    ) -> Result<Self, NativeFieldContractDenial> {
        Self::declared(declaration, field)
    }
}

impl NativeFieldContractDenial {
    #[doc(hidden)]
    pub fn field_locus_for_test(&self) -> Option<&schema::ApplicationFieldBindingLocus> {
        Self::field_locus(self)
    }
}

impl FieldExpectation {
    #[doc(hidden)]
    pub fn seal_for_test<S: schema::ApplicationSchema>(
        &self,
        installed: &domain::WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), NativeFieldContractDenial> {
        Self::seal::<S>(self, installed)
    }
}

impl NativeFieldContractDenial {
    #[doc(hidden)]
    pub fn detail_for_test(&self) -> &str {
        Self::detail(self)
    }
}
