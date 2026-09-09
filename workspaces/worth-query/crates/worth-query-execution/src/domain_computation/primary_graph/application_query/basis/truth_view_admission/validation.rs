use crate::domain_computation::primary_graph::application_query::{
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};

pub(super) fn validate_truth_view_request(
    request: &WorthQueryRequestScope,
) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
    match request.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => Err(denial(
            WorthQueryApplicationQueryAdmissionDenialKind::Cancelled,
            "truth-view basis",
        )),
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => Err(denial(
            WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded,
            "truth-view basis",
        )),
        None => Ok(()),
    }
}

pub(super) fn denial(
    kind: WorthQueryApplicationQueryAdmissionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(kind, subject)
}
