#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecoveryObserverWalObservation {
    pub(crate) valid_prefix_bytes: u64,
    pub(crate) observed_bytes: u64,
    pub(crate) frame_count: u64,
    pub(crate) first_lsn: Option<u64>,
    pub(crate) last_lsn: Option<u64>,
    pub(crate) digest: [u8; 32],
}
