//! Pure one-shot read stop enum.

use super::WorthQueryApplicationOneShotDenial;
use worth_relational::facade::mvcc::CompanionPreflightStop;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryAdmittedOneShotStop {
    Admission(CompanionPreflightStop),
    WorkCounterOverflow,
    WorkAccountingMismatch,
    Execution(WorthQueryApplicationOneShotDenial),
}
