//! Sealed test doors over this module's private members.
use super::*;

impl<'request, Schema> WorthQueryApplicationQueryControls<'request, Schema> {
    #[doc(hidden)]
    pub fn selected_read_one_shot_for_test(
        product: WorthQueryProductObservationLease,
        application_basis: WorthQueryApplicationBasisLease,
        maximum_result_count: NonZeroUsize,
        request_scope: &'request WorthQueryRequestScope,
    ) -> Result<Self, CompanionPreflightStop> {
        Self::selected_read_one_shot(
            product,
            application_basis,
            maximum_result_count,
            request_scope,
        )
    }
}
