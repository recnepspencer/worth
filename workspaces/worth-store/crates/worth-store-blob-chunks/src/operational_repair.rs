use sha2::{Digest, Sha256};
use worth_store_physical_backend::{
    ClosedNonCurrentStagingMedia, ClosedStagingArtifactVerificationDenial,
    ClosedStagingArtifactVerificationRequest, NonCurrentStagingPlanBinding,
};
use worth_store_physical_format::BackupBundleArtifactFamily;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobRepairRegionObservation {
    identity: [u8; 32],
    quarantine_required: bool,
}

impl BlobRepairRegionObservation {
    pub fn new(identity: [u8; 32], quarantine_required: bool) -> Option<Self> {
        if identity == [0; 32] {
            return None;
        }
        Some(Self {
            identity,
            quarantine_required,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobRepairConsequencePlan {
    fingerprint: [u8; 32],
    staging_plan_fingerprint: [u8; 32],
    region_identities: Vec<[u8; 32]>,
    maximum_bytes: u64,
    consequence: BlobRepairConsequence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobRepairConsequence {
    RestoreDamagedArtifact,
    ReplaceQuarantinedArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobRepairConsequenceReceipt {
    plan_fingerprint: [u8; 32],
    verified_artifacts: u64,
    verified_bytes: u64,
    consequence: BlobRepairConsequence,
}

#[derive(Debug)]
pub enum BlobRepairConsequenceDenial {
    AllocationFailed,
    StagingPlanMismatch,
    MissingBlobArtifact,
    Backend(ClosedStagingArtifactVerificationDenial),
}

pub struct BlobRepairConsequenceOwner;

impl BlobRepairConsequenceOwner {
    pub fn lower(
        regions: &[BlobRepairRegionObservation],
        staging: &NonCurrentStagingPlanBinding,
    ) -> Result<Option<BlobRepairConsequencePlan>, BlobRepairConsequenceDenial> {
        let mut identities = Vec::new();
        identities
            .try_reserve(regions.len())
            .map_err(|_| BlobRepairConsequenceDenial::AllocationFailed)?;
        identities.extend(regions.iter().map(|region| region.identity));
        if identities.is_empty() {
            return Ok(None);
        }
        identities.sort();
        identities.dedup();
        let consequence = if regions.iter().any(|region| region.quarantine_required) {
            BlobRepairConsequence::ReplaceQuarantinedArtifact
        } else {
            BlobRepairConsequence::RestoreDamagedArtifact
        };
        let mut digest = Sha256::new();
        digest.update(b"worth-store-blob-repair-consequence-plan-v1");
        digest.update(staging.fingerprint());
        digest.update([match consequence {
            BlobRepairConsequence::RestoreDamagedArtifact => 1,
            BlobRepairConsequence::ReplaceQuarantinedArtifact => 2,
        }]);
        for identity in &identities {
            digest.update(identity);
        }
        Ok(Some(BlobRepairConsequencePlan {
            fingerprint: digest.finalize().into(),
            staging_plan_fingerprint: staging.fingerprint(),
            region_identities: identities,
            maximum_bytes: staging.expected_bytes(),
            consequence,
        }))
    }

    pub fn execute(
        plan: &BlobRepairConsequencePlan,
        media: &ClosedNonCurrentStagingMedia,
    ) -> Result<BlobRepairConsequenceReceipt, BlobRepairConsequenceDenial> {
        if media.plan_fingerprint() != plan.staging_plan_fingerprint {
            return Err(BlobRepairConsequenceDenial::StagingPlanMismatch);
        }
        if plan.region_identities.is_empty() {
            return Err(BlobRepairConsequenceDenial::MissingBlobArtifact);
        }
        let verification = worth_store_physical_backend::PhysicalRecoveryStagingOwner::
            verify_closed_artifact_family(ClosedStagingArtifactVerificationRequest::new(
                media,
                BackupBundleArtifactFamily::BlobChunk,
                plan.maximum_bytes,
            ))
            .map_err(BlobRepairConsequenceDenial::Backend)?;
        Ok(BlobRepairConsequenceReceipt {
            plan_fingerprint: plan.fingerprint,
            verified_artifacts: verification.verified_artifacts(),
            verified_bytes: verification.verified_bytes(),
            consequence: plan.consequence,
        })
    }
}

impl BlobRepairConsequencePlan {
    pub const fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }
    pub const fn consequence(&self) -> BlobRepairConsequence {
        self.consequence
    }
}
impl BlobRepairConsequenceReceipt {
    pub const fn plan_fingerprint(self) -> [u8; 32] {
        self.plan_fingerprint
    }
    pub const fn verified_artifacts(self) -> u64 {
        self.verified_artifacts
    }
    pub const fn verified_bytes(self) -> u64 {
        self.verified_bytes
    }
    pub const fn consequence(self) -> BlobRepairConsequence {
        self.consequence
    }
}
