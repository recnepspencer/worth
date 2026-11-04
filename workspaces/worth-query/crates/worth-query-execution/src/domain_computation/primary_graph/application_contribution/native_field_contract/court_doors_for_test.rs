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
    pub fn declared_for_test<S, E, A, F, V: 'static, W, Q, U>(
        declaration: &schema::ErasedApplicationSchemaDeclaration,
        field: schema::ApplicationFieldRef<S, E, A, F, V, W, Q, U>,
    ) -> Result<Self, NativeFieldContractDenial>
    where
        F: schema::DeclaredApplicationFieldValue<Value = V>,
        U: schema::ApplicationFieldUnit,
    {
        Self::declared::<S, E, A, F, V, W, Q, U>(declaration, field)
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
