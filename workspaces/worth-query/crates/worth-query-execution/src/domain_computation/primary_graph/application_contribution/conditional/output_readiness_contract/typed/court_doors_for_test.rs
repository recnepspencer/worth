//! Sealed test doors over this module's private members.
use super::*;

impl<D, O, F> WorthQueryOutputReadinessContract<D, O, F> {
    #[doc(hidden)]
    pub fn delivery_fields_for_test(
        &self,
    ) -> &std::collections::BTreeSet<schema::ApplicationFieldBindingLocus> {
        &self.delivery_fields
    }
}

impl<D, O, F> WorthQueryOutputReadinessContract<D, O, F> {
    #[doc(hidden)]
    pub fn fields_for_test(&self) -> &Vec<FieldExpectation> {
        &self.fields
    }
}

impl FieldExpectation {
    #[doc(hidden)]
    pub fn dependency_for_test(
        &self,
    ) -> Result<domain::WorthQuerySemanticTruthDependency, WorthQueryOutputReadinessContractDenial>
    {
        Self::dependency(self)
    }
}

impl FieldExpectation {
    #[doc(hidden)]
    pub fn mask_for_test(&self) -> domain::AspectMask<domain::ProjectionMask> {
        Self::mask(self)
    }
}
