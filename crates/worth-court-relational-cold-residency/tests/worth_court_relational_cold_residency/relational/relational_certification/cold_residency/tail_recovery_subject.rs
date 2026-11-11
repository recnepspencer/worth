//! Closed physical inputs to staged tail recovery.
#[derive(Clone, Copy, Debug)]
pub(super) enum Input {
    Head,
    History,
    Symbols,
    Buckets,
    Nodes,
    Events,
    Index,
    Envelope,
    TailEnvelope,
}
impl Input {
    pub(super) const ALL: [Self; 9] = [
        Self::Head,
        Self::History,
        Self::Symbols,
        Self::Buckets,
        Self::Nodes,
        Self::Events,
        Self::Index,
        Self::Envelope,
        Self::TailEnvelope,
    ];
}
