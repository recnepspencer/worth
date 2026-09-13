use super::OfflineIntegrityPosture;
use super::{
    ObservedRecoveryFrontier, OfflineAuthorityClass, OfflineRecoveryAvailability,
    OfflineSecurityEvidencePosture, OperationalTruthRegion, OperationalTruthReport,
};
use crate::OfflinePhysicalArtifactFamily;
use sha2::{Digest, Sha256};

impl OperationalTruthReport {
    /// Stable identity for canonical semantic truth composition.
    pub fn truth_evidence_identity(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth-store-operational-truth-evidence-v1");
        digest.update(self.source_inspection_identity());
        digest.update(self.coverage().covered_bytes().to_be_bytes());
        digest.update(self.coverage().region_count().to_be_bytes());
        digest.update(self.peak_owned_allocation_bytes().to_be_bytes());
        digest.update((self.regions().len() as u64).to_be_bytes());
        for region in self.regions() {
            update_region(&mut digest, region);
        }
        let candidates = self.recovery_candidates().candidates();
        digest.update((candidates.len() as u64).to_be_bytes());
        for candidate in candidates {
            digest.update([family_tag(candidate.family())]);
            update_frontier(&mut digest, candidate.frontier());
            digest.update(candidate.content_digest());
        }
        digest.finalize().into()
    }
}

fn update_region(digest: &mut Sha256, region: &OperationalTruthRegion) {
    digest.update([region_tag(region)]);
    let evidence = region.evidence();
    let (start, end) = evidence.range();
    digest.update(start.to_be_bytes());
    digest.update(end.to_be_bytes());
    digest.update([family_tag(evidence.family())]);
    update_optional_u64(digest, evidence.generation());
    match evidence.physical_owner() {
        Some(owner) => {
            digest.update([1]);
            digest.update(owner.stable_fingerprint());
        }
        None => digest.update([0]),
    }
    digest.update([
        authority_tag(evidence.authority_class()),
        integrity_tag(evidence.integrity()),
        security_tag(evidence.authenticity()),
        security_tag(evidence.custody()),
        recovery_tag(evidence.recovery_availability()),
    ]);
    match evidence.security_scope() {
        Some(scope) => {
            digest.update([1]);
            digest.update(scope.stable_fingerprint());
        }
        None => digest.update([0]),
    }
    digest.update(evidence.content_digest());
    let references = evidence.evidence_references();
    digest.update((references.media_source_index() as u64).to_be_bytes());
    digest.update(references.observed_content_digest());
    update_optional_digest(digest, references.declared_expected_digest());
    match references.security_scope_receipt() {
        Some(receipt) => {
            digest.update([1]);
            digest.update(receipt.admission_sequence().to_be_bytes());
            digest.update(receipt.security_scope_fingerprint().to_be_bytes());
            digest.update(receipt.proof_progression_fingerprint().to_be_bytes());
        }
        None => digest.update([0]),
    }
    match region {
        OperationalTruthRegion::AliasGroup { claimants, .. } => {
            digest.update((claimants.len() as u64).to_be_bytes());
        }
        OperationalTruthRegion::OverlapConflict {
            additional_claims,
            claimants,
            ..
        } => {
            digest.update((additional_claims.len() as u64).to_be_bytes());
            digest.update((claimants.len() as u64).to_be_bytes());
            for claim in additional_claims {
                digest.update(claim.content_digest());
                let (start, end) = claim.range();
                digest.update(start.to_be_bytes());
                digest.update(end.to_be_bytes());
            }
        }
        _ => {}
    }
}

fn update_frontier(digest: &mut Sha256, frontier: ObservedRecoveryFrontier) {
    match frontier {
        ObservedRecoveryFrontier::RootManifest {
            root_reference,
            generation,
        } => {
            digest.update([1]);
            digest.update(root_reference.to_be_bytes());
            digest.update(generation.to_be_bytes());
        }
        ObservedRecoveryFrontier::Checkpoint {
            checkpoint_identity_digest,
            manifest_generation,
            durable_checkpoint_lsn,
            root_reference,
            root_generation,
            covered_lsn_start,
            covered_lsn_end_exclusive,
            redo_lsn,
        } => {
            digest.update([2]);
            digest.update(checkpoint_identity_digest);
            digest.update(manifest_generation.to_be_bytes());
            digest.update(durable_checkpoint_lsn.to_be_bytes());
            digest.update(root_reference.to_be_bytes());
            digest.update(root_generation.to_be_bytes());
            digest.update(covered_lsn_start.to_be_bytes());
            digest.update(covered_lsn_end_exclusive.to_be_bytes());
            digest.update(redo_lsn.to_be_bytes());
        }
        ObservedRecoveryFrontier::WalSegment {
            segment_id,
            generation,
            start_lsn,
            end_exclusive_lsn,
        } => {
            digest.update([3]);
            digest.update(segment_id.to_be_bytes());
            digest.update(generation.to_be_bytes());
            digest.update(start_lsn.to_be_bytes());
            digest.update(end_exclusive_lsn.to_be_bytes());
        }
    }
}

fn update_optional_u64(digest: &mut Sha256, value: Option<u64>) {
    match value {
        Some(value) => {
            digest.update([1]);
            digest.update(value.to_be_bytes());
        }
        None => digest.update([0]),
    }
}

fn update_optional_digest(digest: &mut Sha256, value: Option<[u8; 32]>) {
    match value {
        Some(value) => {
            digest.update([1]);
            digest.update(value);
        }
        None => digest.update([0]),
    }
}

const fn region_tag(region: &OperationalTruthRegion) -> u8 {
    match region {
        OperationalTruthRegion::TrustedAuthorityRegion(_) => 1,
        OperationalTruthRegion::DegradedDerivedRegion(_) => 2,
        OperationalTruthRegion::RebuildableRegion(_) => 3,
        OperationalTruthRegion::QuarantinedRegion(_) => 4,
        OperationalTruthRegion::UnrecoverableAuthorityRegion(_) => 5,
        OperationalTruthRegion::IndeterminateTruthRegion(_) => 6,
        OperationalTruthRegion::AliasGroup { .. } => 7,
        OperationalTruthRegion::OverlapConflict { .. } => 8,
    }
}

const fn family_tag(family: OfflinePhysicalArtifactFamily) -> u8 {
    match family {
        OfflinePhysicalArtifactFamily::Manifest => 1,
        OfflinePhysicalArtifactFamily::Page => 2,
        OfflinePhysicalArtifactFamily::Extent => 3,
        OfflinePhysicalArtifactFamily::Wal => 4,
        OfflinePhysicalArtifactFamily::Index => 5,
        OfflinePhysicalArtifactFamily::BlobChunk => 6,
        OfflinePhysicalArtifactFamily::Unknown => 7,
    }
}

const fn authority_tag(class: OfflineAuthorityClass) -> u8 {
    match class {
        OfflineAuthorityClass::Authoritative => 1,
        OfflineAuthorityClass::Derived => 2,
        OfflineAuthorityClass::ContentAuthority => 3,
        OfflineAuthorityClass::Unknown => 4,
    }
}

const fn integrity_tag(posture: OfflineIntegrityPosture) -> u8 {
    match posture {
        OfflineIntegrityPosture::Confirmed => 1,
        OfflineIntegrityPosture::DigestMismatch => 2,
        OfflineIntegrityPosture::IntegrityNotDeclared => 3,
    }
}

const fn security_tag(posture: OfflineSecurityEvidencePosture) -> u8 {
    match posture {
        OfflineSecurityEvidencePosture::Confirmed => 1,
        OfflineSecurityEvidencePosture::Unavailable => 2,
        OfflineSecurityEvidencePosture::Unsupported => 3,
        OfflineSecurityEvidencePosture::WrongScope => 4,
        OfflineSecurityEvidencePosture::Failed => 5,
    }
}

const fn recovery_tag(availability: OfflineRecoveryAvailability) -> u8 {
    match availability {
        OfflineRecoveryAvailability::Available => 1,
        OfflineRecoveryAvailability::Unavailable => 2,
        OfflineRecoveryAvailability::Unknown => 3,
    }
}
