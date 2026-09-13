#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::c8_recovery_observation) struct RecoveryObserverManifestMembershipEvidence {
    manifest_count: u64,
    member_count: u64,
    digest: [u8; 32],
}

impl RecoveryObserverManifestMembershipEvidence {
    pub(in crate::c8_recovery_observation) const fn manifest_count(self) -> u64 {
        self.manifest_count
    }

    pub(in crate::c8_recovery_observation) const fn member_count(self) -> u64 {
        self.member_count
    }

    pub(in crate::c8_recovery_observation) const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub(in crate::c8_recovery_observation) const fn from_parts(
        manifest_count: u64,
        member_count: u64,
        digest: [u8; 32],
    ) -> Self {
        Self {
            manifest_count,
            member_count,
            digest,
        }
    }
}
