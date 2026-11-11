/// Closed Signal publication streams. Store treats their payloads as opaque.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalDurableJournal {
    Definitions,
    Advances,
}
impl SignalDurableJournal {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Definitions => "signal.definitions",
            Self::Advances => "signal.advances",
        }
    }
}
