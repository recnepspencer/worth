#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecoveryObserverManifestMembershipObservation {
    pub(crate) manifest_count: u64,
    pub(crate) member_count: u64,
    pub(crate) digest: [u8; 32],
}

impl RecoveryObserverManifestMembershipObservation {
    pub(crate) const fn empty() -> Self {
        Self {
            manifest_count: 0,
            member_count: 0,
            digest: [0; 32],
        }
    }
}
