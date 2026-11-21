//! Pure outstanding in-flight dispatch denial enum.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOutstandingInFlightDenial {
    Missing,
    PendingPublication,
    OriginalMismatch,
    TerminalReached,
    CapacityExhausted,
}
