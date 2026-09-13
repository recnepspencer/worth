use std::path::Path;

use crate::OfflinePhysicalArtifactFamily;
use worth_store_physical_backend::OfflineMediaFileIdentity;
use worth_store_physical_format::PhysicalGenerationOwner;
use worth_store_security::{StoreSecurityScopeAdmissionReceiptId, StoreSecurityScopeIdentity};

use super::{
    OfflineFileTruthEvidence, OfflineIntegrityPosture, OfflineRecoveryAvailability,
    OfflineSecurityEvidencePosture,
};
use crate::OfflineStructuralIdentification;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineAuthorityClass {
    Authoritative,
    Derived,
    ContentAuthority,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfflineTruthEvidenceReferences {
    media_source_index: usize,
    observed_content_digest: [u8; 32],
    declared_expected_digest: Option<[u8; 32]>,
    security_scope_receipt: Option<StoreSecurityScopeAdmissionReceiptId>,
}

impl OfflineTruthEvidenceReferences {
    pub const fn media_source_index(self) -> usize {
        self.media_source_index
    }
    pub const fn observed_content_digest(self) -> [u8; 32] {
        self.observed_content_digest
    }
    pub const fn declared_expected_digest(self) -> Option<[u8; 32]> {
        self.declared_expected_digest
    }
    pub const fn security_scope_receipt(self) -> Option<StoreSecurityScopeAdmissionReceiptId> {
        self.security_scope_receipt
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceBoundTruthRegion {
    media_identity: OfflineMediaFileIdentity,
    start: u64,
    end: u64,
    family: OfflinePhysicalArtifactFamily,
    generation: Option<u64>,
    physical_owner: Option<PhysicalGenerationOwner>,
    structural_identification: OfflineStructuralIdentification,
    authority_class: OfflineAuthorityClass,
    integrity: OfflineIntegrityPosture,
    authenticity: OfflineSecurityEvidencePosture,
    custody: OfflineSecurityEvidencePosture,
    security_scope: Option<StoreSecurityScopeIdentity>,
    recovery_availability: OfflineRecoveryAvailability,
    content_digest: [u8; 32],
    evidence_references: OfflineTruthEvidenceReferences,
}

impl EvidenceBoundTruthRegion {
    pub(crate) fn from_walked_file(
        file: &crate::inspection::OfflineWalkedFile,
        evidence: Option<&OfflineFileTruthEvidence>,
        integrity: OfflineIntegrityPosture,
    ) -> Self {
        let authenticity = evidence.map_or(OfflineSecurityEvidencePosture::Unavailable, |value| {
            value.authenticity()
        });
        let custody = evidence.map_or(OfflineSecurityEvidencePosture::Unavailable, |value| {
            value.custody()
        });
        Self {
            media_identity: file.source().clone(),
            start: 0,
            end: file.length(),
            family: file.family(),
            generation: file.generation(),
            physical_owner: file.physical_owner(),
            structural_identification: file.structural_identification(),
            authority_class: authority_class(file.family(), file.structural_identification()),
            integrity,
            authenticity,
            custody,
            security_scope: evidence.and_then(OfflineFileTruthEvidence::security_scope),
            recovery_availability: evidence.map_or(OfflineRecoveryAvailability::Unknown, |value| {
                value.recovery_availability()
            }),
            content_digest: file.content_digest(),
            evidence_references: OfflineTruthEvidenceReferences {
                media_source_index: file.source_index(),
                observed_content_digest: file.content_digest(),
                declared_expected_digest: evidence
                    .and_then(OfflineFileTruthEvidence::expected_digest),
                security_scope_receipt: evidence
                    .and_then(OfflineFileTruthEvidence::security_scope_receipt),
            },
        }
    }
    pub fn source(&self) -> &Path {
        self.media_identity.path()
    }
    pub const fn media_identity(&self) -> &OfflineMediaFileIdentity {
        &self.media_identity
    }
    pub const fn range(&self) -> (u64, u64) {
        (self.start, self.end)
    }
    pub const fn family(&self) -> OfflinePhysicalArtifactFamily {
        self.family
    }
    pub const fn generation(&self) -> Option<u64> {
        self.generation
    }
    pub const fn physical_owner(&self) -> Option<PhysicalGenerationOwner> {
        self.physical_owner
    }
    pub const fn structural_identification(&self) -> OfflineStructuralIdentification {
        self.structural_identification
    }
    pub const fn authority_class(&self) -> OfflineAuthorityClass {
        self.authority_class
    }
    pub const fn integrity(&self) -> OfflineIntegrityPosture {
        self.integrity
    }
    pub const fn authenticity(&self) -> OfflineSecurityEvidencePosture {
        self.authenticity
    }
    pub const fn custody(&self) -> OfflineSecurityEvidencePosture {
        self.custody
    }
    pub const fn security_scope(&self) -> Option<StoreSecurityScopeIdentity> {
        self.security_scope
    }
    pub const fn recovery_availability(&self) -> OfflineRecoveryAvailability {
        self.recovery_availability
    }
    pub const fn content_digest(&self) -> [u8; 32] {
        self.content_digest
    }
    pub const fn evidence_references(&self) -> OfflineTruthEvidenceReferences {
        self.evidence_references
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationalTruthRegion {
    TrustedAuthorityRegion(EvidenceBoundTruthRegion),
    DegradedDerivedRegion(EvidenceBoundTruthRegion),
    RebuildableRegion(EvidenceBoundTruthRegion),
    QuarantinedRegion(EvidenceBoundTruthRegion),
    UnrecoverableAuthorityRegion(EvidenceBoundTruthRegion),
    IndeterminateTruthRegion(EvidenceBoundTruthRegion),
    AliasGroup {
        region: EvidenceBoundTruthRegion,
        claimants: Vec<std::path::PathBuf>,
    },
    OverlapConflict {
        representative: EvidenceBoundTruthRegion,
        additional_claims: Vec<EvidenceBoundTruthRegion>,
        claimants: Vec<std::path::PathBuf>,
    },
}

impl OperationalTruthRegion {
    pub const fn evidence(&self) -> &EvidenceBoundTruthRegion {
        match self {
            Self::TrustedAuthorityRegion(region)
            | Self::DegradedDerivedRegion(region)
            | Self::RebuildableRegion(region)
            | Self::QuarantinedRegion(region)
            | Self::UnrecoverableAuthorityRegion(region)
            | Self::IndeterminateTruthRegion(region)
            | Self::AliasGroup { region, .. } => region,
            Self::OverlapConflict { representative, .. } => representative,
        }
    }
}

const fn authority_class(
    family: OfflinePhysicalArtifactFamily,
    structural_identification: OfflineStructuralIdentification,
) -> OfflineAuthorityClass {
    if matches!(
        structural_identification,
        OfflineStructuralIdentification::FileNameHint
    ) {
        return OfflineAuthorityClass::Unknown;
    }
    match family {
        OfflinePhysicalArtifactFamily::Manifest
        | OfflinePhysicalArtifactFamily::Page
        | OfflinePhysicalArtifactFamily::Extent
        | OfflinePhysicalArtifactFamily::Wal => OfflineAuthorityClass::Authoritative,
        OfflinePhysicalArtifactFamily::Index => OfflineAuthorityClass::Derived,
        OfflinePhysicalArtifactFamily::BlobChunk => OfflineAuthorityClass::ContentAuthority,
        OfflinePhysicalArtifactFamily::Unknown => OfflineAuthorityClass::Unknown,
    }
}
