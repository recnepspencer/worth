#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecoveryObserverResidueObservation {
    pub(crate) bytes: u64,
    pub(crate) digest: [u8; 32],
}

impl RecoveryObserverResidueObservation {
    pub(crate) const fn empty() -> Self {
        Self {
            bytes: 0,
            digest: [0; 32],
        }
    }
}
