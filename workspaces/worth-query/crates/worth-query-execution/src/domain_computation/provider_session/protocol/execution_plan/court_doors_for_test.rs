//! Sealed test doors over this module's private members.
use super::*;

impl<'run> WorthQueryAdmittedProviderExecutionPlan<'run> {
    #[doc(hidden)]
    pub fn bind_application_product_for_test(
        self,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> Result<Self, WorthQueryProviderSessionFailure> {
        Self::bind_application_product(self, product)
    }
}
