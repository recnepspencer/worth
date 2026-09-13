#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecoveryObserverWalTopologyObservation {
    pub(crate) segment: u64,
    pub(crate) generation: u64,
    pub(crate) first_lsn: u64,
    pub(crate) last_lsn: u64,
    pub(crate) denial: Option<super::super::RecoveryObserverWalTopologyDenial>,
}
