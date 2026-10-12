//! Basis-retention stops are execution outcomes, never authorization denials.
use crate::basis::WorthQueryProductBranchAdmissionDenial as Basis;
use crate::domain_computation::primary_graph::WorthQueryApplicationLiveOutcome as Outcome;
use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
pub(super) fn outcome<Query, QueryResult>(cause: Basis) -> Option<Outcome<Query, QueryResult>> {
    match cause {
        Basis::ReadExecution(Ok(kind)) => Some(Outcome::ExecutionRequest(kind)),
        Basis::ReadExecution(Err(Control::Cancelled)) | Basis::ObservationCancelled => {
            Some(Outcome::Cancelled)
        }
        Basis::ReadExecution(Err(Control::TimedOut)) | Basis::ObservationDeadlineExceeded => {
            Some(Outcome::DeadlineExceeded)
        }
        _ => None,
    }
}
