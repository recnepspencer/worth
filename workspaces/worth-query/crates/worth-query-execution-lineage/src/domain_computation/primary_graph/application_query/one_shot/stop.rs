//! Pure one-shot read stop enum.

use worth_relational::facade::mvcc::CompanionPreflightStop;
use super::WorthQueryApplicationOneShotDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryAdmittedOneShotStop {
    Admission(CompanionPreflightStop),
    WorkCounterOverflow,
    WorkAccountingMismatch,
    Execution(WorthQueryApplicationOneShotDenial),
}
