//! Stable kind of the Signal effect associated with a composite original.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalBranchEffectKind {
    Advanced,
    Forked,
}
