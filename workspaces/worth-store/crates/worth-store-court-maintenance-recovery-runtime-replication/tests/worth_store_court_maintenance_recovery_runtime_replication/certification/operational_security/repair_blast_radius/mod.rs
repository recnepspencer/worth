#[allow(unused_imports)] use crate::repair_blast_radius::*;
use {worth_foundational::aspects, worth_foundational::AspectContract, worth_foundational::AspectKey, worth_foundational::AspectValue, worth_foundational::InternedString, worth_foundational::ScalarAspectType};
use {worth_proof::TransitionOutcome};
use {worth_store_aspect_native::StoreAspectAuthorityInput, worth_store_aspect_native::StoreAspectBoundaryFact, worth_store_aspect_native::StoreAspectIdentity, worth_store_aspect_native::StorePhysicalBoundaryWitness};
use {worth_store_authority::require_current_store_authority, worth_store_authority::StoreCurrentAuthorityWitness};
use {worth_store_contracts::StorePhysicalAuthorityWitness, worth_store_contracts::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE};
use {worth_store_offline_verifier::OfflineRepairBlastRadiusObservation, worth_store_offline_verifier::OfflineRepairEvidenceKind};
use {worth_store_operations::RepairBlastRadiusDeclaration, worth_store_operations::RepairBlastRadiusDenial, worth_store_operations::RepairBlastRadiusPlan};
use {worth_store_security::classify_audit_record_as_security_scope_source, worth_store_security::classify_iam_role_as_security_scope_source, worth_store_security::classify_offline_verifier_evidence_as_security_scope_source, worth_store_security::classify_operator_identity_as_security_scope_source, worth_store_security::reject_non_store_security_scope_source, worth_store_security::repair_blast_radius_authenticity, worth_store_security::StoreCustodyPosture, worth_store_security::StoreKeyScope, worth_store_security::StoreKeyVersionPosture, worth_store_security::StoreRawSecurityScopeDeclaration, worth_store_security::StoreRepairPhysicalRegionDeclaration, worth_store_security::StoreSecurityScopeAdmissionDenial, worth_store_security::StoreSecurityScopeAdmissionRebindRequired, worth_store_security::StoreSecurityScopeDenialKind, worth_store_security::StoreTenantScope};

#[test]
fn repair_read_plan_requires_admitted_store_blast_radius() {
    let authority = current_authority("cert.phase8.admitted");
    let declaration = worth_store_operations::RepairBlastRadiusDeclaration::native(
        &authority,
        worth_store_security::StoreRepairPhysicalRegionDeclaration::raw("repair-region-a"),
        worth_store_security::StoreKeyVersionPosture::Current,
        worth_store_security::StoreCustodyPosture::InternalStoreCustody,
    )
    .expect("current Store blast radius should declare repair readiness");
    let region = declaration.physical_region().clone();
    let readiness = worth_store_operations::RepairBlastRadiusPlan::declare(declaration)
        .admit_with_store_blast_radius()
        .expect("current Store blast radius should admit repair readiness");
    let read_plan = readiness
        .prepare_repair_read(region)
        .expect("admitted region should prepare repair read");

    assert_eq!(
        read_plan.security_scope().key_scope(),
        worth_store_security::StoreKeyScope::RepairScopeEnvelope
    );
    assert_eq!(
        read_plan.security_scope().tenant_scope(),
        worth_store_security::StoreTenantScope::RepairBlastRadius
    );
    assert_eq!(read_plan.counters().repair_reads_prepared(), 1);
}

#[test]
fn correct_region_id_cannot_override_wrong_tenant_scope() {
    let authority = current_authority("cert.phase8.wrong-tenant");
    let raw = worth_store_security::StoreRawSecurityScopeDeclaration::deserialized_unadmitted(
        authority.physical_witness(),
        worth_store_security::StoreKeyScope::RepairScopeEnvelope,
        worth_store_security::StoreKeyVersionPosture::Current,
        worth_store_security::StoreTenantScope::TenantPhysicalBoundary,
        Some(repair_blast_radius_authenticity()),
        Some(worth_store_security::StoreCustodyPosture::InternalStoreCustody),
    );
    let observation = worth_store_offline_verifier::OfflineRepairBlastRadiusObservation::from_offline_repair_report(
        raw,
        worth_store_security::StoreRepairPhysicalRegionDeclaration::raw("repair-region-a"),
        worth_store_offline_verifier::OfflineRepairEvidenceKind::RepairReadCloseout,
    )
    .expect("offline report should be raw evidence");
    let denial = worth_store_operations::RepairBlastRadiusDeclaration::from_offline_observation(&authority, observation)
        .expect_err("wrong tenant should deny before repair read");

    assert!(matches!(
        denial,
        worth_store_operations::RepairBlastRadiusDenial::SecurityScopeAdmissionDenied {
            source: worth_store_security::StoreSecurityScopeAdmissionDenial::WrongTenantScope,
            ..
        }
    ));
}

#[test]
fn rebind_required_key_posture_preserves_rebind_topology() {
    let authority = current_authority("cert.phase8.rebind-key");
    let denial = worth_store_operations::RepairBlastRadiusDeclaration::native(
        &authority,
        worth_store_security::StoreRepairPhysicalRegionDeclaration::raw("repair-region-a"),
        worth_store_security::StoreKeyVersionPosture::RebindRequired,
        worth_store_security::StoreCustodyPosture::InternalStoreCustody,
    )
    .expect_err("rebind-required key posture should deny before repair read");

    if let worth_store_operations::RepairBlastRadiusDenial::SecurityScopeAdmissionRebindRequired { source, counters } =
        denial
    {
        assert_eq!(
            source,
            worth_store_security::StoreSecurityScopeAdmissionRebindRequired::KeyVersionRebindRequired(
                worth_store_security::StoreKeyVersionPosture::RebindRequired
            )
        );
        assert_eq!(counters.key_rebind_required_rejections(), 1);
        assert_eq!(counters.repair_reads_prepared(), 0);
        assert_eq!(counters.repair_denied(), 1);
    } else {
        panic!("rebind-required repair admission must stay typed");
    }
}

#[test]
fn non_store_sources_cannot_mint_repair_readiness() {
    let operator = reject_non_store_security_scope_source(
        classify_operator_identity_as_security_scope_source(),
    );
    let iam = reject_non_store_security_scope_source(classify_iam_role_as_security_scope_source());
    let audit =
        reject_non_store_security_scope_source(classify_audit_record_as_security_scope_source());
    let offline = reject_non_store_security_scope_source(
        classify_offline_verifier_evidence_as_security_scope_source(),
    );

    assert_eq!(
        operator.kind(),
        worth_store_security::StoreSecurityScopeDenialKind::OperatorIdentityIsNotRepairAuthority
    );
    assert_eq!(
        iam.kind(),
        worth_store_security::StoreSecurityScopeDenialKind::IamRoleIsNotCustodyPosture
    );
    assert_eq!(
        audit.kind(),
        worth_store_security::StoreSecurityScopeDenialKind::AuditRecordIsNotRepairAuthority
    );
    assert_eq!(
        offline.kind(),
        worth_store_security::StoreSecurityScopeDenialKind::OfflineVerifierEvidenceIsNotRepairAuthority
    );
}
