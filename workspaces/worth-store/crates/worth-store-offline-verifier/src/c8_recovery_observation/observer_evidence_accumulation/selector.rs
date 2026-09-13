#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecoveryObserverSelectorObservation {
    pub(crate) identity: u64,
    pub(crate) linked_identity: Option<u64>,
    pub(crate) store_identity: [u8; 16],
    pub(crate) role: u8,
    pub(crate) root_generation: u64,
}
