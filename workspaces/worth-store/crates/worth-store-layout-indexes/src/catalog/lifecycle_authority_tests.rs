use super::{
    ArtifactFamilyAccessLane, ArtifactFamilyAuthorityClass, ArtifactFamilyDenial,
    ArtifactFamilyStrategyLane, PhysicalArtifactFamilyDeclaration,
};
use crate::layout_declarations;
use worth_store_contracts::{
    ArtifactFamilyLifecycleClass, DurableArtifactFamilyId, DurableArtifactMigrationPosture,
    DurableArtifactOwningBoundary, DurableArtifactRebuildPosture,
};

#[test]
fn authority_and_strategy_lifecycle_are_proof_bearing() {
    let facade = layout_declarations();
    let hot_classification = facade.classify_family(
        facade
            .declaration(DurableArtifactFamilyId::PhysicalRootManifest)
            .unwrap(),
    );
    let maintenance_classification = facade.classify_family(
        facade
            .declaration(DurableArtifactFamilyId::SnapshotArtifact)
            .unwrap(),
    );
    let hot_authority = facade
        .require_production_authority(hot_classification)
        .unwrap();
    let maintenance_authority = facade
        .require_production_authority(maintenance_classification)
        .unwrap();
    assert_eq!(
        hot_authority.family_id(),
        DurableArtifactFamilyId::PhysicalRootManifest
    );
    assert_eq!(
        hot_authority.classification().lifecycle(),
        super::ArtifactFamilyLifecycleDisposition::StrategyHotPath
    );
    assert_eq!(
        facade
            .require_strategy_lifecycle(hot_authority)
            .unwrap()
            .admitted_lane(),
        ArtifactFamilyStrategyLane::HotPath
    );
    assert_eq!(
        maintenance_authority.classification().lifecycle(),
        super::ArtifactFamilyLifecycleDisposition::StrategyMaintenancePath
    );
    assert_eq!(
        facade
            .require_strategy_lifecycle(maintenance_authority)
            .unwrap()
            .admitted_lane(),
        ArtifactFamilyStrategyLane::MaintenancePath
    );
}

#[test]
fn weaker_authority_and_lifecycle_paths_are_denied() {
    let facade = layout_declarations();
    assert_eq!(
        facade
            .require_production_authority(
                facade.classify_family(
                    facade
                        .declaration(DurableArtifactFamilyId::DedupeIndex)
                        .unwrap()
                )
            )
            .unwrap_err(),
        ArtifactFamilyDenial::DerivedFamilyCannotMintProductionAuthority
    );
    assert_eq!(
        facade
            .require_production_authority(
                facade.classify_family(
                    facade
                        .declaration(DurableArtifactFamilyId::OfflineVerificationRecord)
                        .unwrap()
                )
            )
            .unwrap_err(),
        ArtifactFamilyDenial::DiagnosticFamilyCannotMintProductionAuthority
    );
    assert_eq!(
        facade
            .require_production_authority(
                facade.classify_family(
                    facade
                        .declaration(DurableArtifactFamilyId::ExportBundle)
                        .unwrap()
                )
            )
            .unwrap_err(),
        ArtifactFamilyDenial::TerminalProjectionCannotMintAuthority
    );
    assert_eq!(
        facade
            .classify_family(
                facade
                    .declaration(DurableArtifactFamilyId::ReadmissionRecord)
                    .unwrap()
            )
            .lifecycle(),
        super::ArtifactFamilyLifecycleDisposition::ReadmissionRequired
    );
    assert_eq!(
        facade
            .require_strategy_lifecycle(
                facade
                    .require_production_authority(
                        facade.classify_family(
                            facade
                                .declaration(DurableArtifactFamilyId::ReadmissionRecord)
                                .unwrap()
                        )
                    )
                    .unwrap()
            )
            .unwrap_err(),
        ArtifactFamilyDenial::ReadmissionFamilyCannotEnterStrategyAdmission
    );
    assert_eq!(
        facade
            .classify_family(
                facade
                    .declaration(DurableArtifactFamilyId::ImportBundle)
                    .unwrap()
            )
            .lifecycle(),
        super::ArtifactFamilyLifecycleDisposition::OfflineImportOnly
    );
    assert_eq!(
        facade
            .require_strategy_lifecycle(
                facade
                    .require_production_authority(
                        facade.classify_family(
                            facade
                                .declaration(DurableArtifactFamilyId::ImportBundle)
                                .unwrap()
                        )
                    )
                    .unwrap()
            )
            .unwrap_err(),
        ArtifactFamilyDenial::OfflineImportOnlyFamilyCannotEnterStrategyAdmission
    );
}

#[test]
fn certification_and_verifier_paths_cannot_bypass_admission() {
    let certification = PhysicalArtifactFamilyDeclaration::declare(
        DurableArtifactFamilyId::CorruptionRecord,
        ArtifactFamilyAuthorityClass::CertificationEvidence,
        ArtifactFamilyLifecycleClass::EvidenceOnly,
        ArtifactFamilyAccessLane::VerifierPath,
        DurableArtifactOwningBoundary::WorthStorePhysicalIntegrity,
        DurableArtifactRebuildPosture::NoRebuild,
        DurableArtifactMigrationPosture::StableNoMigration,
        &[],
    );
    let verifier_only = PhysicalArtifactFamilyDeclaration::declare(
        DurableArtifactFamilyId::WalRecoveryDecision,
        ArtifactFamilyAuthorityClass::Authoritative,
        ArtifactFamilyLifecycleClass::RecoveryState,
        ArtifactFamilyAccessLane::VerifierPath,
        DurableArtifactOwningBoundary::WorthStoreRecoveryPhysics,
        DurableArtifactRebuildPosture::NoRebuild,
        DurableArtifactMigrationPosture::StableNoMigration,
        &[],
    );
    let facade = layout_declarations();
    assert_eq!(
        facade.require_production_authority(
            facade.classify_family(Box::leak(Box::new(certification)))
        ),
        Err(ArtifactFamilyDenial::CourtroomCannotMintAuthority)
    );
    assert_eq!(
        facade
            .classify_family(Box::leak(Box::new(verifier_only)))
            .lifecycle(),
        super::ArtifactFamilyLifecycleDisposition::VerifierOnly
    );
    assert_eq!(
        facade
            .require_strategy_lifecycle(
                facade
                    .require_production_authority(
                        facade.classify_family(Box::leak(Box::new(verifier_only)))
                    )
                    .unwrap()
            )
            .unwrap_err(),
        ArtifactFamilyDenial::VerifierLaneCannotEnterStrategyAdmission
    );
}
