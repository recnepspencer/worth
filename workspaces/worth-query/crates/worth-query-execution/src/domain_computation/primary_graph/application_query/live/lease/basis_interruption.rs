//! Basis-retention stops are execution outcomes, never authorization denials.
use crate::basis::WorthQueryProductBranchAdmissionDenial as Basis;
use crate::domain_computation::primary_graph::WorthQueryApplicationLiveOutcome as Outcome;
use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
pub(super) fn outcome<Query, QueryResult>(cause: Basis) -> Option<Outcome<Query, QueryResult>> {
    match cause {
        Basis::Stopped(Ok(kind)) => Some(Outcome::ExecutionRequest(kind)),
        Basis::Stopped(Err(Control::Cancelled)) => Some(Outcome::Cancelled),
        Basis::Stopped(Err(Control::TimedOut)) => Some(Outcome::DeadlineExceeded),
        Basis::Refused(_) => None,
    }
}
