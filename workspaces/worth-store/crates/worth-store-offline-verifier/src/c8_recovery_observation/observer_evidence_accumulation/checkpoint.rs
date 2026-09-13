#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecoveryObserverCheckpointObservation {
    pub(crate) page_count: u64,
    pub(crate) covered_lsn: (u64, u64),
    pub(crate) redo_lsn: u64,
    pub(crate) durable_checkpoint_lsn: u64,
    pub(crate) digest: [u8; 32],
}
