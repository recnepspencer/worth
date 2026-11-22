//! Sealed test doors over this module's private members.
use super::*;

impl<'a, Schema> WorthQueryApplicationQueryControls<'a, Schema> {
    #[doc(hidden)]
    pub fn product_one_shot_for_test(
        product: crate::basis::WorthQueryProductBranchLease,
        application_basis: super::super::resource_lifecycle::WorthQueryApplicationBasisLease,
        maximum_result_count: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        Self::product_one_shot(
            product,
            application_basis,
            maximum_result_count,
            request_scope,
        )
    }
}
