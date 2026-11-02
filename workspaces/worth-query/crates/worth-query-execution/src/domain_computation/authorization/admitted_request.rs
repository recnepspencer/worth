//! Original authentication lifetime carried into a split history-output read.
use super::WorthQueryAdmissionLapse;
use std::time::Instant;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};

/// This retains the actual admitted request, not a permission decision. Only
/// the operation admission owner constructs it from its authentication facts.
pub(in crate::domain_computation) struct AdmittedRequestCurrentness {
    pub(super) request: WorthQueryRequestScope,
    pub(super) authentication_valid_until: Instant,
}

impl AdmittedRequestCurrentness {
    pub(in crate::domain_computation) fn authentication_deadline(&self) -> Instant {
        self.authentication_valid_until
    }

    pub(in crate::domain_computation) fn request(&self) -> &WorthQueryRequestScope {
        &self.request
    }
    pub(in crate::domain_computation) fn same_request(
        &self,
        request: &WorthQueryRequestScope,
    ) -> bool {
        self.request.same_request(request)
    }

    pub(in crate::domain_computation) fn lapse(&self) -> Option<WorthQueryAdmissionLapse> {
        request_lapse(&self.request, self.authentication_valid_until)
    }
}

pub(super) fn request_lapse(
    request: &WorthQueryRequestScope,
    authentication_valid_until: Instant,
) -> Option<WorthQueryAdmissionLapse> {
    if let Some(interruption) = request.interruption() {
        return Some(match interruption {
            WorthQueryRequestInterruption::Cancelled => WorthQueryAdmissionLapse::Cancelled,
            WorthQueryRequestInterruption::DeadlineExceeded => {
                WorthQueryAdmissionLapse::DeadlineExceeded
            }
        });
    }
    (Instant::now() >= authentication_valid_until)
        .then_some(WorthQueryAdmissionLapse::AuthenticationExpired)
}
