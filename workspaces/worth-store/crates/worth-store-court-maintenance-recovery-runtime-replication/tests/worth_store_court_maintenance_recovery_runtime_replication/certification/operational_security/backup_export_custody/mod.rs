#[allow(unused_imports)] use crate::backup_export_custody::*;
use {worth_foundational::aspects, worth_foundational::AspectContract, worth_foundational::AspectKey, worth_foundational::AspectValue, worth_foundational::InternedString, worth_foundational::ScalarAspectType};
use {worth_proof::TransitionOutcome};
use {worth_store_aspect_native::StoreAspectAuthorityInput, worth_store_aspect_native::StoreAspectBoundaryFact, worth_store_aspect_native::StoreAspectIdentity, worth_store_aspect_native::StorePhysicalBoundaryWitness};
use {worth_store_authority::require_current_store_authority, worth_store_authority::StoreCurrentAuthorityWitness};
use {worth_store_contracts::StorePhysicalAuthorityWitness, worth_store_contracts::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE};
use {worth_store_offline_verifier::OfflineCustodyCapsuleObservation};
use {worth_store_operations::BackupExportCapsuleEmission, worth_store_operations::BackupExportCustodyDeclaration, worth_store_operations::BackupExportCustodyMode, worth_store_operations::BackupExportCustodyReadiness, worth_store_operations::BackupImportCustodyReadmission};
use {worth_store_security::store_backup_restore_boundary_fact, worth_store_security::StoreAuthenticityRequirement, worth_store_security::StoreAuthenticityRequirementClass, worth_store_security::StoreBackupRestoreAfterKeyRotationBoundaryEvidence, worth_store_security::StoreBackupRestoreAfterKeyRotationBoundaryFact, worth_store_security::StoreCustodyPosture, worth_store_security::StoreKeyScope, worth_store_security::StoreKeyVersionPosture, worth_store_security::StoreRawSecurityScopeDeclaration, worth_store_security::StoreSecurityScopeAdmissionExpectation, worth_store_security::StoreTenantScope, worth_store_security::StoreTrustBoundaryReadmissionTrigger};

#[test]
fn certification_courtroom_exercises_backup_export_import_custody_readiness() {
    let authority = current_authority("cert.phase7.backup-export");
    let declaration = worth_store_operations::BackupExportCustodyDeclaration::native(
        &authority,
        worth_store_operations::BackupExportCustodyMode::Export,
        worth_store_security::StoreKeyVersionPosture::Current,
    )
    .expect("current export declaration should build");

    assert_eq!(
        declaration.raw_declaration().custody_posture(),
        Some(worth_store_security::StoreCustodyPosture::ExportPrepared)
    );

    let admission = declaration
        .admit_with_current_authority(&authority)
        .expect("export declaration should admit through custody path");
    let custody = worth_store_operations::BackupExportCustodyReadiness::from_admitted_custody(admission)
        .expect("backup custody readiness should admit");
    let emission = worth_store_operations::BackupExportCapsuleEmission::prepare(custody);

    assert_eq!(
        emission.security_scope().tenant_scope(),
        worth_store_security::StoreTenantScope::BackupRestoreBoundary
    );
    assert_eq!(
        emission.security_scope().custody_posture(),
        worth_store_security::StoreCustodyPosture::ExportPrepared
    );

    let raw = worth_store_security::StoreRawSecurityScopeDeclaration::deserialized_unadmitted(
        authority.physical_witness(),
        worth_store_security::StoreKeyScope::BackupExportEnvelope,
        worth_store_security::StoreKeyVersionPosture::Current,
        worth_store_security::StoreTenantScope::ImportReadmissionBoundary,
        Some(backup_authenticity()),
        Some(worth_store_security::StoreCustodyPosture::ImportedUnreadmitted),
    );
    let observation = worth_store_offline_verifier::OfflineCustodyCapsuleObservation::from_deserialized_capsule(
        raw,
        backup_restore_trigger(raw, &authority),
    )
    .expect("offline verifier should observe raw capsule declaration only");
    let readmitted = worth_store_operations::BackupImportCustodyReadmission::new(observation)
        .readmit_with_current_authority(&authority)
        .expect("restored bundle should regain current security scope by readmission");
    let custody = worth_store_operations::BackupExportCustodyReadiness::from_admitted_custody(readmitted)
        .expect("readmitted import custody should satisfy backup/export readiness");

    assert_eq!(
        custody.identity().tenant_scope(),
        worth_store_security::StoreTenantScope::ImportReadmissionBoundary
    );
    assert_eq!(custody.custody_posture(), worth_store_security::StoreCustodyPosture::Readmitted);
}
