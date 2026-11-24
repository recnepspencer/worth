//! Adoption failure carrier retaining reservation and candidate custody.

use super::super::{
    WorthQueryDocumentHistoryDenial as Denial, WorthQueryDocumentHistoryReservation,
};
use crate::domain_computation::primary_graph::WorthQueryExpectedProductBranch;

#[must_use = "retain the original reservation and actual candidate after refusal"]
pub struct WorthQueryDocumentHistoryAdoptionFailure<'runtime, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) reservation:
        WorthQueryDocumentHistoryReservation,
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) candidate:
        WorthQueryExpectedProductBranch<'runtime, S>,
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) denial:
        Denial,
}

impl<'runtime, S> WorthQueryDocumentHistoryAdoptionFailure<'runtime, S> {
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryDocumentHistoryReservation,
        WorthQueryExpectedProductBranch<'runtime, S>,
        Denial,
    ) {
        (self.reservation, self.candidate, self.denial)
    }

    pub fn denial(&self) -> &Denial {
        &self.denial
    }
}

impl<S> std::fmt::Debug for WorthQueryDocumentHistoryAdoptionFailure<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryDocumentHistoryAdoptionFailure")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}
