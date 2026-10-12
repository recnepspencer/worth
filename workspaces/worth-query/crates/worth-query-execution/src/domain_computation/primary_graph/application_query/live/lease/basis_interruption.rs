//! Live retention's request interruptions are authorization stops.
use crate::basis::WorthQueryProductBranchAdmissionDenial as Basis;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial as Denial,
    WorthQueryOperationAuthorizationDenialKind as Kind,
};
pub(super) fn authorization(cause: Basis) -> Option<Denial> {
    let kind = match cause {
        Basis::ReadExecution(stop) => match stop {
            Ok(kind) => Kind::ExecutionRequest(kind),
            Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled) => {
                Kind::Cancelled
            }
            Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut) => {
                Kind::DeadlineExceeded
            }
        },
        Basis::ObservationCancelled => Kind::Cancelled,
        Basis::ObservationDeadlineExceeded => Kind::DeadlineExceeded,
        _ => return None,
    };
    Some(Denial::new(kind, "live basis retention"))
}
