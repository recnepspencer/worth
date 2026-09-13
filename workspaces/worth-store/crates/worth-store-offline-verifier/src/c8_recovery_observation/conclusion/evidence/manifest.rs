use super::super::super::observer_evidence::RecoveryObserverManifestMembershipEvidence;
use super::super::super::observer_evidence_accumulation::{
    EvidenceDigestBuilder, RecoveryObserverManifestMembershipObservation,
};

pub(crate) struct ManifestEvidenceAccumulator {
    manifests: u64,
    members: u64,
    digest: EvidenceDigestBuilder,
}

impl ManifestEvidenceAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            manifests: 0,
            members: 0,
            digest: EvidenceDigestBuilder::new(
                b"worth.store.recovery-observer.manifest-membership.v1",
            ),
        }
    }

    pub(crate) fn observe(&mut self, membership: RecoveryObserverManifestMembershipObservation) {
        self.manifests = self.manifests.saturating_add(membership.manifest_count);
        self.members = self.members.saturating_add(membership.member_count);
        if membership.manifest_count > 0 {
            self.digest.record(&membership.digest);
        }
    }

    pub(crate) fn finish(self) -> RecoveryObserverManifestMembershipEvidence {
        RecoveryObserverManifestMembershipEvidence::from_parts(
            self.manifests,
            self.members,
            self.digest.finish().digest(),
        )
    }
}
