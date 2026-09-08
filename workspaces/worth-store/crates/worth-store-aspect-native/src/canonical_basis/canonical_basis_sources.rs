use crate::{
    StoreCanonicalBasisFamily, StoreCanonicalBasisFieldRole, StoreCanonicalBasisLane,
    StoreCanonicalBasisSourceDenial, StoreCanonicalBasisSourceKind, StoreCanonicalBasisSourceOwner,
};

const FORBIDDEN_TEXT_SOURCES: &[StoreCanonicalBasisFieldRole] = &[
    StoreCanonicalBasisFieldRole::TerminalProjection,
    StoreCanonicalBasisFieldRole::OperatorDisplay,
    StoreCanonicalBasisFieldRole::DocumentChecksum,
    StoreCanonicalBasisFieldRole::CompatibilityText,
    StoreCanonicalBasisFieldRole::DigestText,
    StoreCanonicalBasisFieldRole::RawJsonPayload,
];

const ASPECT_STATE: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::FoundationalAspectState];
const ASPECT_PATCH: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::FoundationalAspectPatch];
const RECEIPT: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::FoundationalReceipt];
const DIAGNOSTIC: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::FoundationalDiagnostic];
const PERFORMANCE: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::FoundationalPerformanceEvidence];
const HANDOFF: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::StoreReadinessHandoff];
const SOURCE_MANIFEST: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::StoreSourceManifest];
const PAGE_HEADER: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::StorePageHeader];
const PHYSICAL_WITNESS: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::StorePhysicalWitness];
const PHYSICAL_RECORD: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::StorePhysicalFormatRecord];
const ARTIFACT_IDENTITY: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::FoundationalPhysicalArtifactIdentity];
const ADAPTER_EVIDENCE: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::FoundationalPhysicalAdapterEvidence];
const FORMAT_DECLARATION: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::StorePhysicalFormatDeclaration];
const QUARANTINE_OBSERVATION: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::StorePhysicalQuarantineObservation];
const SCRUB_PROGRESS: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::StorePhysicalIntegrityScrubProgress];
const WAL: &[StoreCanonicalBasisSourceKind] = &[StoreCanonicalBasisSourceKind::StoreWalRecord];
const PHYSICAL_MUTATION_REQUEST: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::StorePhysicalMutationRequest];
const RECOVERY_RECEIPT: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::StoreRecoveryReceipt];
const RECOVERY_PERFORMANCE: &[StoreCanonicalBasisSourceKind] =
    &[StoreCanonicalBasisSourceKind::StoreRecoveryPerformanceEvidence];

pub const STORE_CANONICAL_BASIS_SOURCE_OWNERS: &[StoreCanonicalBasisSourceOwner] = &[
    owner(
        StoreCanonicalBasisFamily::AspectBoundaryFact,
        "worth-store-aspect-native",
        "aspect-native authority",
        ASPECT_STATE,
        StoreCanonicalBasisLane::AspectValueState,
    ),
    owner(
        StoreCanonicalBasisFamily::AspectPatchBoundaryFact,
        "worth-store-aspect-native",
        "aspect-native patch authority",
        ASPECT_PATCH,
        StoreCanonicalBasisLane::AspectPatch,
    ),
    owner(
        StoreCanonicalBasisFamily::BoundaryReceiptEvidence,
        "worth-store-aspect-native",
        "aspect-native receipts",
        RECEIPT,
        StoreCanonicalBasisLane::Receipt,
    ),
    owner(
        StoreCanonicalBasisFamily::DiagnosticEvidence,
        "worth-store-aspect-native",
        "aspect-native diagnostics",
        DIAGNOSTIC,
        StoreCanonicalBasisLane::Diagnostic,
    ),
    owner(
        StoreCanonicalBasisFamily::PerformanceReceiptEvidence,
        "worth-store-aspect-native",
        "aspect-native performance",
        PERFORMANCE,
        StoreCanonicalBasisLane::PerformanceEvidence,
    ),
    owner(
        StoreCanonicalBasisFamily::ReadinessHandoff,
        "worth-store-readiness",
        "readiness handoff",
        HANDOFF,
        StoreCanonicalBasisLane::Handoff,
    ),
    owner(
        StoreCanonicalBasisFamily::S2EntryBoundaryEvidence,
        "worth-store-readiness",
        "S2 boundary readiness",
        PHYSICAL_WITNESS,
        StoreCanonicalBasisLane::Handoff,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalSourceManifest,
        "worth-store-physical-format",
        "source manifest",
        SOURCE_MANIFEST,
        StoreCanonicalBasisLane::PhysicalSourceManifest,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalPageHeader,
        "worth-store-physical-format",
        "page header",
        PAGE_HEADER,
        StoreCanonicalBasisLane::PhysicalRecord,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalPageRecord,
        "worth-store-physical-format",
        "page record",
        PHYSICAL_RECORD,
        StoreCanonicalBasisLane::PhysicalRecord,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalExtentRecord,
        "worth-store-physical-format",
        "extent record",
        PHYSICAL_RECORD,
        StoreCanonicalBasisLane::PhysicalRecord,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalReference,
        "worth-store-physical-format",
        "physical reference",
        PHYSICAL_RECORD,
        StoreCanonicalBasisLane::PhysicalRecord,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalOfflineVerifierEvidence,
        "worth-store-physical-format",
        "offline verifier",
        PHYSICAL_RECORD,
        StoreCanonicalBasisLane::PhysicalRecord,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalHeaderDecodeEvidence,
        "worth-store-physical-format",
        "header decoder",
        PAGE_HEADER,
        StoreCanonicalBasisLane::PhysicalRecord,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalFormatEvidence,
        "worth-store-physical-format",
        "physical format evidence",
        PHYSICAL_RECORD,
        StoreCanonicalBasisLane::PhysicalRecord,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalManifestDiscoveryEvidence,
        "worth-store-physical-format",
        "manifest discovery",
        SOURCE_MANIFEST,
        StoreCanonicalBasisLane::PhysicalSourceManifest,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalArtifactIdentity,
        "worth-foundational",
        "physical artifact identity",
        ARTIFACT_IDENTITY,
        StoreCanonicalBasisLane::PhysicalIntegrity,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalAdapterEvidence,
        "worth-foundational",
        "descriptive physical adapter evidence",
        ADAPTER_EVIDENCE,
        StoreCanonicalBasisLane::PhysicalIntegrity,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalIntegrityChecksumCoverage,
        "worth-store-physical-format",
        "integrity checksum declaration",
        FORMAT_DECLARATION,
        StoreCanonicalBasisLane::PhysicalRecord,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalQuarantineObservation,
        "worth-store",
        "managed scrub quarantine observation",
        QUARANTINE_OBSERVATION,
        StoreCanonicalBasisLane::PhysicalIntegrity,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalIntegrityScrubProgress,
        "worth-store",
        "managed physical integrity scrub progress",
        SCRUB_PROGRESS,
        StoreCanonicalBasisLane::PhysicalIntegrity,
    ),
    owner(
        StoreCanonicalBasisFamily::WalFrameIntegrityEvidence,
        "worth-store-wal",
        "WAL frame integrity",
        WAL,
        StoreCanonicalBasisLane::Wal,
    ),
    owner(
        StoreCanonicalBasisFamily::WalRecord,
        "worth-store-wal",
        "WAL record",
        WAL,
        StoreCanonicalBasisLane::Wal,
    ),
    owner(
        StoreCanonicalBasisFamily::PhysicalMutationRequestFingerprint,
        "worth-store",
        "physical mutation request equivalence",
        PHYSICAL_MUTATION_REQUEST,
        StoreCanonicalBasisLane::PhysicalMutation,
    ),
    owner(
        StoreCanonicalBasisFamily::RecoveryIntegrityHandoff,
        "worth-store-recovery-physics",
        "recovery integrity handoff",
        RECOVERY_RECEIPT,
        StoreCanonicalBasisLane::Recovery,
    ),
    owner(
        StoreCanonicalBasisFamily::RecoveryWalReplayReceipt,
        "worth-store-recovery-physics",
        "WAL replay receipt",
        RECOVERY_RECEIPT,
        StoreCanonicalBasisLane::Recovery,
    ),
    owner(
        StoreCanonicalBasisFamily::RecoveryCheckpointValidityReceipt,
        "worth-store-recovery-physics",
        "checkpoint validity receipt",
        RECOVERY_RECEIPT,
        StoreCanonicalBasisLane::Recovery,
    ),
    owner(
        StoreCanonicalBasisFamily::RecoveryVettedRecordReceipt,
        "worth-store-recovery-physics",
        "vetted record receipt",
        RECOVERY_RECEIPT,
        StoreCanonicalBasisLane::Recovery,
    ),
    owner(
        StoreCanonicalBasisFamily::RecoveryPerformanceReport,
        "worth-store-recovery-physics",
        "recovery performance",
        RECOVERY_PERFORMANCE,
        StoreCanonicalBasisLane::Recovery,
    ),
];

pub fn canonical_basis_source_owner_for_family(
    family: StoreCanonicalBasisFamily,
) -> Result<&'static StoreCanonicalBasisSourceOwner, StoreCanonicalBasisSourceDenial> {
    STORE_CANONICAL_BASIS_SOURCE_OWNERS
        .iter()
        .find(|owner| owner.family() == family)
        .ok_or(StoreCanonicalBasisSourceDenial::MissingSourceOwner {
            family,
            classifying_subsystem: "Store canonical-basis source ownership",
        })
}

pub fn certify_canonical_basis_source(
    family: StoreCanonicalBasisFamily,
    source: StoreCanonicalBasisSourceKind,
) -> Result<(), StoreCanonicalBasisSourceDenial> {
    let owner = canonical_basis_source_owner_for_family(family)?;
    if owner.allows_source(source) {
        Ok(())
    } else {
        Err(StoreCanonicalBasisSourceDenial::WrongNativeSourceKind { family, source })
    }
}

pub fn certify_canonical_basis_field_role(
    field_role: StoreCanonicalBasisFieldRole,
) -> Result<(), StoreCanonicalBasisSourceDenial> {
    if FORBIDDEN_TEXT_SOURCES.contains(&field_role) {
        Err(StoreCanonicalBasisSourceDenial::ForbiddenFieldRole { field_role })
    } else {
        Ok(())
    }
}

const fn owner(
    family: StoreCanonicalBasisFamily,
    owner_crate: &'static str,
    classifying_subsystem: &'static str,
    allowed_sources: &'static [StoreCanonicalBasisSourceKind],
    foundational_lane: StoreCanonicalBasisLane,
) -> StoreCanonicalBasisSourceOwner {
    StoreCanonicalBasisSourceOwner::new(
        family,
        owner_crate,
        classifying_subsystem,
        allowed_sources,
        FORBIDDEN_TEXT_SOURCES,
        foundational_lane,
    )
}
