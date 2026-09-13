use super::integrity_classification::{
    IntegrityRepairArtifactFamily, IntegrityRepairOwnerBinding, IntegrityRepairRegion,
    IntegrityRepairRegionClass,
};
use sha2::{Digest, Sha256};
use worth_store_layout_indexes::DerivedIndexRepairRequest;

use super::intent::physical_target_identity;
use super::{RepairCandidateSet, RepairResolutionDenial};
use crate::phase_1_6_tests::support::{backup_custody, BackupScenario};
use crate::{
    AuthorizationProviderDecision, AuthorizationProviderFailure, AuthorizationReplayPolicy,
    AuthorizationRevocationObservation, ExternalOperatorAssertion, OperationalAuthorizationPort,
    OperationalAuthorizationRequest, OperationalOperationId, OperationalSecurityScope,
    OperationalTransitionId,
};

#[test]
fn equal_content_cannot_substitute_a_different_repair_target() {
    let directory = tempfile::tempdir().expect("repair directory");
    let intended = directory.path().join("intended.index");
    let substitute = directory.path().join("substitute.index");
    let replacement = directory.path().join("replacement.index");
    std::fs::write(&intended, b"same damaged bytes").expect("intended target");
    std::fs::write(&substitute, b"same damaged bytes").expect("substitute target");
    std::fs::write(&replacement, b"rebuilt index bytes").expect("replacement source");

    let evidence_digest = digest(b"same damaged bytes");
    let authority = crate::backup::export::current_authority("repair-target-binding");
    let security_scope =
        OperationalSecurityScope::from_admission(backup_custody(&authority).receipt());
    let damaged = IntegrityRepairRegion::bounded(
        [0x21; 32],
        0,
        18,
        IntegrityRepairRegionClass::DerivedRebuildable,
        evidence_digest,
        physical_target_identity(&intended).expect("intended target identity"),
        derived_owner_binding(),
    )
    .expect("repair region");
    let candidates = RepairCandidateSet {
        operation_id: OperationalOperationId::new("repair-target-binding").expect("operation id"),
        damaged: vec![super::resolved_region::ResolvedRepairRegion::new(
            damaged, intended,
        )],
        untouched: 0,
        unrecoverable: Vec::new(),
        basis_identity: [0x22; 32],
        authority_identity: authority.authority_identity(),
        security_scope,
    };
    let substituted_request = DerivedIndexRepairRequest::new(
        [0x23; 32],
        substitute,
        evidence_digest,
        replacement,
        digest(b"rebuilt index bytes"),
        7,
        8,
        4 * 1024,
    );

    assert!(matches!(
        candidates.select_derived_maintenance(vec![substituted_request]),
        Err(RepairResolutionDenial::IncompleteOwnerCoverage)
    ));
}

#[test]
fn closed_owner_receipts_are_the_only_repair_execution_projection_source() {
    let directory = tempfile::tempdir().expect("repair directory");
    let target = directory.path().join("layout.index");
    let replacement = directory.path().join("layout.rebuilt");
    std::fs::write(&target, b"damaged layout").expect("target");
    std::fs::write(&replacement, b"rebuilt layout").expect("replacement");
    let authority = crate::backup::export::current_authority("repair-projection");
    let candidates = RepairCandidateSet {
        operation_id: OperationalOperationId::new("repair-projection").expect("operation id"),
        damaged: vec![super::resolved_region::ResolvedRepairRegion::new(
            IntegrityRepairRegion::bounded(
                [0x31; 32],
                0,
                14,
                IntegrityRepairRegionClass::DerivedRebuildable,
                digest(b"damaged layout"),
                physical_target_identity(&target).expect("target identity"),
                derived_owner_binding(),
            )
            .expect("repair region"),
            target.clone(),
        )],
        untouched: 0,
        unrecoverable: Vec::new(),
        basis_identity: [0x32; 32],
        authority_identity: authority.authority_identity(),
        security_scope: OperationalSecurityScope::from_admission(
            backup_custody(&authority).receipt(),
        ),
    };
    let request = DerivedIndexRepairRequest::new(
        [0x33; 32],
        &target,
        digest(b"damaged layout"),
        &replacement,
        digest(b"rebuilt layout"),
        4,
        5,
        4 * 1024,
    );
    let lowered = candidates
        .select_derived_maintenance(vec![request])
        .expect("exact owner selection")
        .lower_owners()
        .expect("owner lowering");
    let authorized = lowered
        .authorize(
            &ExactRepairAuthorization,
            &operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .expect("repair authorization");
    let control_scenario = BackupScenario::new("repair-projection-control");
    let control = control_scenario.control_store();
    let executed = authorized
        .ready(
            &control,
            OperationalTransitionId::new("repair-projection-consumption").expect("transition"),
            &authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .expect("repair readiness")
        .execute()
        .expect("repair execution");

    assert_eq!(
        std::fs::read(&target).expect("repaired target"),
        b"rebuilt layout"
    );
    let projection = executed
        .project_execution_boundary(&authority)
        .expect("downstream boundary projection");
    assert_eq!(projection.owner_receipt_count(), 2);
    assert_eq!(
        projection.evidence().receipt().execution_posture(),
        worth_foundational::FoundationalBoundaryEvidenceExecutionPosture::Executed
    );
    assert_eq!(
        projection.plan_fingerprint(),
        executed.authorization().plan_fingerprint()
    );
}

struct ExactRepairAuthorization;

impl OperationalAuthorizationPort for ExactRepairAuthorization {
    fn authorize(
        &self,
        request: OperationalAuthorizationRequest<'_>,
        assertion: &ExternalOperatorAssertion,
    ) -> Result<AuthorizationProviderDecision, AuthorizationProviderFailure> {
        Ok(AuthorizationProviderDecision::authorized(
            [0x34; 32],
            request.plan_fingerprint(),
            assertion.proof_of_possession_binding(),
            request.requested_at(),
            request.expires_at(),
        ))
    }
}

fn operator_assertion() -> ExternalOperatorAssertion {
    ExternalOperatorAssertion::admit(
        "repair-test-provider",
        "repair-approval",
        b"signed-repair-approval",
        [0x35; 32],
        10,
        100,
    )
    .expect("operator assertion")
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn derived_owner_binding() -> IntegrityRepairOwnerBinding {
    IntegrityRepairOwnerBinding::observed(
        IntegrityRepairArtifactFamily::LayoutIndex,
        Some(7),
        None,
        None,
    )
}
