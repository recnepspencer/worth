#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::c8_recovery_observation) struct RecoveryObserverSelectorEvidence {
    selector_count: u64,
    linked_selector_count: u64,
    unpaired_link_count: u64,
    store_identity: Option<[u8; 16]>,
    current_root_generation: Option<u64>,
    digest: [u8; 32],
}

impl RecoveryObserverSelectorEvidence {
    pub(in crate::c8_recovery_observation) const fn selector_count(self) -> u64 {
        self.selector_count
    }

    pub(in crate::c8_recovery_observation) const fn linked_selector_count(self) -> u64 {
        self.linked_selector_count
    }

    pub(in crate::c8_recovery_observation) const fn unpaired_link_count(self) -> u64 {
        self.unpaired_link_count
    }

    pub(in crate::c8_recovery_observation) const fn store_identity(self) -> Option<[u8; 16]> {
        self.store_identity
    }

    pub(in crate::c8_recovery_observation) const fn current_root_generation(self) -> Option<u64> {
        self.current_root_generation
    }

    pub(in crate::c8_recovery_observation) const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub(in crate::c8_recovery_observation) const fn from_parts(
        selector_count: u64,
        linked_selector_count: u64,
        unpaired_link_count: u64,
        store_identity: Option<[u8; 16]>,
        current_root_generation: Option<u64>,
        digest: [u8; 32],
    ) -> Self {
        Self {
            selector_count,
            linked_selector_count,
            unpaired_link_count,
            store_identity,
            current_root_generation,
            digest,
        }
    }
}
