//! Structural faults armed on the actual provider receiver.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Fault {
    #[default]
    None,
    OmitSource,
    DuplicateSource,
    DuplicateChild,
    DisconnectedCycle,
}
