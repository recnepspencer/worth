#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecoveryObserverPageLsnObservation {
    pub(crate) count: u64,
    pub(crate) minimum: Option<u64>,
    pub(crate) maximum: Option<u64>,
    pub(crate) digest: [u8; 32],
}

impl RecoveryObserverPageLsnObservation {
    pub(crate) const fn empty() -> Self {
        Self {
            count: 0,
            minimum: None,
            maximum: None,
            digest: [0; 32],
        }
    }
}
