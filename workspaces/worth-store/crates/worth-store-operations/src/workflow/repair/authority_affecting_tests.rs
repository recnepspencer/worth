use super::integrity_classification::{
    IntegrityRepairArtifactFamily, IntegrityRepairOwnerBinding, IntegrityRepairRegion,
    IntegrityRepairRegionClass,
};

use super::intent::physical_target_identity;
use super::RepairCandidateSet;
use crate::phase_1_6_tests::support::backup_custody;
use crate::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, OperationalOperationId,
    OperationalSecurityScope, OperationalTransitionId, OwnerPlanEffect, OwnerPlanExecutionStage,
    StoreOwnerKind,
};

#[test]
fn authority_quarantine_repair_preserves_owner_meaning_across_artifact_consequences() {
    use worth_store_physical_format::BackupBundleArtifactFamily;

    let world = crate::phase_7_13_tests::restore_world("repair-owner-consequences");
    let manifest = world
        .admissible
        .custody()
        .structural()
        .materialized()
        .manifest();
    let mut damaged = Vec::new();
    for (family, repair_family, identity) in [
        (
            BackupBundleArtifactFamily::Index,
            IntegrityRepairArtifactFamily::LayoutIndex,
            [0x41; 32],
        ),
        (
            BackupBundleArtifactFamily::BlobChunk,
            IntegrityRepairArtifactFamily::BlobChunk,
            [0x42; 32],
        ),
    ] {
        let row = manifest
            .artifacts()
            .iter()
            .find(|row| row.family() == family)
            .expect("fixture carries each affected owner family");
        let path = world.backup_root.join(row.output_name());
        damaged.push(
            IntegrityRepairRegion::bounded(
                identity,
                0,
                row.bytes(),
                IntegrityRepairRegionClass::QuarantineRequired,
                row.content_digest(),
                physical_target_identity(&path).expect("canonical owner target"),
                IntegrityRepairOwnerBinding::observed(
                    repair_family,
                    Some(row.generation()),
                    row.reclaim_owner()
                        .generation_owner()
                        .map(|owner| owner.stable_fingerprint()),
                    None,
                ),
            )
            .expect("owner-bound repair region"),
        );
    }
    let candidates = RepairCandidateSet {
        operation_id: OperationalOperationId::new("repair-owner-consequences").unwrap(),
        damaged: damaged
            .into_iter()
            .map(|region| {
                let source = match region.owner_binding().family() {
                    IntegrityRepairArtifactFamily::LayoutIndex => manifest
                        .artifacts()
                        .iter()
                        .find(|row| row.family() == BackupBundleArtifactFamily::Index),
                    IntegrityRepairArtifactFamily::BlobChunk => manifest
                        .artifacts()
                        .iter()
                        .find(|row| row.family() == BackupBundleArtifactFamily::BlobChunk),
                    _ => None,
                }
                .map(|row| world.backup_root.join(row.output_name()))
                .unwrap();
                super::resolved_region::ResolvedRepairRegion::new(region, source)
            })
            .collect(),
        untouched: 6,
        unrecoverable: Vec::new(),
        basis_identity: [0x43; 32],
        authority_identity: world.authority.authority_identity(),
        security_scope: OperationalSecurityScope::from_admission(
            backup_custody(&world.authority).receipt(),
        ),
    };
    let target = world.restore_directory.path().join("repair-target");
    std::fs::create_dir_all(&target).unwrap();
    let lowered = candidates
        .select_authority_affecting_staging(world.admissible, &target, u64::MAX, 4096)
        .expect("trusted source is exact current authority")
        .lower_owners()
        .expect("every affected owner lowers its own consequence");
    let explanation = lowered.explanation();
    assert_eq!(explanation.node_count(), 5);
    assert_eq!(explanation.edge_count(), 4);
    for (owner, effect) in [
        (
            StoreOwnerKind::PhysicalIntegrity,
            OwnerPlanEffect::ClassifyQuarantine,
        ),
        (
            StoreOwnerKind::PhysicalBackend,
            OwnerPlanEffect::CopyBackupComponent,
        ),
        (
            StoreOwnerKind::RecoveryPhysics,
            OwnerPlanEffect::ReplayWalToExactFrontier,
        ),
        (
            StoreOwnerKind::LayoutIndexes,
            OwnerPlanEffect::ReplaceQuarantinedLayout,
        ),
        (
            StoreOwnerKind::BlobChunks,
            OwnerPlanEffect::ReplaceQuarantinedBlob,
        ),
    ] {
        assert!(explanation.nodes().iter().any(|node| {
            node.owner() == owner
                && node.effect() == effect
                && node.stage() == OwnerPlanExecutionStage::Staging
                && node.expected_receipt_fingerprint() != [0; 32]
        }));
    }
    assert!(explanation
        .prerequisites()
        .iter()
        .all(|prerequisite| prerequisite.durability_barrier()));
    let first_irreversible = explanation
        .first_irreversible_node()
        .expect("repair has irreversible staging work");
    assert_eq!(
        explanation
            .nodes()
            .iter()
            .find(|node| node.identity() == first_irreversible)
            .expect("first irreversible identity belongs to the DAG")
            .owner(),
        StoreOwnerKind::PhysicalBackend
    );
    let authorized = lowered
        .authorize(
            &crate::phase_7_13_tests::ExactAuthorizationPort {
                substitute_plan: None,
            },
            &crate::phase_7_13_tests::operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .expect("exact five-owner authorization");
    let executed = authorized
        .ready(
            &world.control,
            OperationalTransitionId::new("repair-owner-ready").unwrap(),
            &world.authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .expect("repair readiness")
        .execute(&crate::phase_7_13_tests::CurrentStagingAuthorizationPort)
        .expect("owner validations execute against closed staged media");
    assert_eq!(
        executed
            .layout()
            .expect("layout owner receipt")
            .consequence(),
        worth_store_layout_indexes::LayoutRepairConsequence::ReplaceQuarantinedArtifact
    );
    assert_eq!(
        executed.blob().expect("blob owner receipt").consequence(),
        worth_store_blob_chunks::BlobRepairConsequence::ReplaceQuarantinedArtifact
    );
}
