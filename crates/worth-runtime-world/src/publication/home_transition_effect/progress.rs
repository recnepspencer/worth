//! Descriptive retained progress, never permission or a physical fate carrier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldHomeTransitionEffectProgress {
    ReadyToSubmit,
    AwaitingAcknowledgement,
    Acknowledged,
    ReopenRequired,
}
