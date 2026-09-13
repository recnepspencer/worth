use super::*;
use crate::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, OperationalOperationId,
    OperationalTransitionId, PointInTimeRecoveryIntent,
};
use crate::{
    PitrCandidatePosture, PitrRoundingPolicy, RecoveryTimelineAdmission, RecoveryTimelineOwner,
};
use worth_store_physical_isolation::RecoverySourceLeaseRegistry;
#[test]
fn exact_frontier_pitr_executes_only_after_a_durable_source_lease() {
    let world = restore_world("phase-9-exact-pitr");
    let target = world.restore_directory.path().join("pitr-target");
    let leases = RecoverySourceLeaseRegistry::open(
        world.restore_directory.path().join("pitr-source-leases"),
    )
    .unwrap();
    std::fs::create_dir_all(&target).unwrap();
    let materialized = world.admissible.custody().structural().materialized();
    let manifest = materialized.manifest();
    let wal_end = manifest.wal_half_open_interval().1;
    let selected_client_ack = manifest.acknowledged_frontier();
    let observation = RecoveryTimelineOwner::admit_observation(RecoveryTimelineAdmission {
        observed_time: 100,
        uncertainty_before: 0,
        uncertainty_after: 0,
        checkpoint_durability: manifest.durable_checkpoint_lsn(),
        wal_structural: wal_end,
        local_durable_commit: wal_end,
        client_acknowledged: selected_client_ack,
        replication_acknowledged: manifest.acknowledged_frontier(),
        authority_identity: world.admissible.admission().admitting_authority(),
        source_lineage: [0x91; 32],
        source_identity: materialized.manifest_digest(),
        posture: PitrCandidatePosture::Available,
    })
    .unwrap();
    let candidates = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::ExactOnly,
        vec![observation],
    )
    .unwrap();
    let security_scope = recovery_security_scope(&world.admissible);
    let lowered = PointInTimeRecoveryIntent::near(
        OperationalOperationId::new("pitr-exact-frontier").unwrap(),
        world.admissible,
        candidates,
        &target,
        security_scope,
        u64::MAX,
        31,
    )
    .resolve()
    .expect("select exact frontier")
    .admit_source_cut(&leases)
    .expect("durably admit exact source closure")
    .lease()
    .lower()
    .expect("lower leased PITR owner DAG");
    assert_recovery_lifecycle_dag(lowered.explanation());
    let expected = lowered
        .authorize(
            &ExactAuthorizationPort {
                substitute_plan: None,
            },
            &operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .unwrap()
        .ready(
            &world.control,
            OperationalTransitionId::new("consume-pitr-staging").unwrap(),
            &world.authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .unwrap()
        .execute(&CurrentStagingAuthorizationPort)
        .expect("execute exact-frontier PITR");
    assert_eq!(
        expected
            .receipt()
            .recovery()
            .exact_frontier()
            .wal_structural(),
        wal_end
    );
    assert_eq!(
        expected
            .receipt()
            .recovery()
            .exact_frontier()
            .client_acknowledged(),
        selected_client_ack
    );
    assert!(expected
        .staged_media()
        .root()
        .starts_with(std::fs::canonicalize(&target).unwrap()));
    assert_eq!(
        selected_staging_kind(&world.authority, &world.control),
        crate::RecoveryStagingOperationKind::PointInTimeRecovery
    );
    assert_eq!(leases.recover_active().unwrap().len(), 1);
}

#[test]
fn pitr_rejects_timeline_evidence_from_a_different_store_authority() {
    let world = restore_world("phase-9-foreign-authority");
    let materialized = world.admissible.custody().structural().materialized();
    let manifest = materialized.manifest();
    let wal_end = manifest.wal_half_open_interval().1;
    let foreign = worth_store_authority::StoreCurrentAuthorityIdentity::from_persisted_fingerprint(
        [0xe1; 32],
    );
    let observation = RecoveryTimelineOwner::admit_observation(RecoveryTimelineAdmission {
        observed_time: 100,
        uncertainty_before: 0,
        uncertainty_after: 0,
        checkpoint_durability: manifest.durable_checkpoint_lsn(),
        wal_structural: wal_end,
        local_durable_commit: wal_end,
        client_acknowledged: manifest.acknowledged_frontier(),
        replication_acknowledged: manifest.acknowledged_frontier(),
        authority_identity: foreign,
        source_lineage: [0x91; 32],
        source_identity: materialized.manifest_digest(),
        posture: PitrCandidatePosture::Available,
    })
    .unwrap();
    let candidates = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::ExactOnly,
        vec![observation],
    )
    .unwrap();
    let target = world
        .restore_directory
        .path()
        .join("foreign-authority-target");
    let security_scope = recovery_security_scope(&world.admissible);
    let denial = PointInTimeRecoveryIntent::near(
        OperationalOperationId::new("pitr-foreign-authority").unwrap(),
        world.admissible,
        candidates,
        target,
        security_scope,
        u64::MAX,
        31,
    )
    .resolve()
    .expect_err("foreign timeline authority cannot select this backup");
    assert_eq!(denial, crate::PitrResolutionDenial::SourceAuthorityMismatch);
}

#[test]
fn pitr_rejects_an_acknowledgement_frontier_not_represented_by_the_source_cut() {
    let world = restore_world("phase-9-impossible-ack-frontier");
    let materialized = world.admissible.custody().structural().materialized();
    let manifest = materialized.manifest();
    let wal_end = manifest.wal_half_open_interval().1;
    let observation = RecoveryTimelineOwner::admit_observation(RecoveryTimelineAdmission {
        observed_time: 100,
        uncertainty_before: 0,
        uncertainty_after: 0,
        checkpoint_durability: manifest.durable_checkpoint_lsn(),
        wal_structural: wal_end,
        local_durable_commit: wal_end,
        client_acknowledged: wal_end.saturating_sub(1),
        replication_acknowledged: wal_end.saturating_sub(1),
        authority_identity: world.admissible.admission().admitting_authority(),
        source_lineage: [0x91; 32],
        source_identity: materialized.manifest_digest(),
        posture: PitrCandidatePosture::Available,
    })
    .unwrap();
    let candidates = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::ExactOnly,
        vec![observation],
    )
    .unwrap();
    let target = world.restore_directory.path().join("impossible-ack-target");
    let security_scope = recovery_security_scope(&world.admissible);
    let denial = PointInTimeRecoveryIntent::near(
        OperationalOperationId::new("pitr-impossible-ack").unwrap(),
        world.admissible,
        candidates,
        target,
        security_scope,
        u64::MAX,
        31,
    )
    .resolve()
    .expect_err("the source cut cannot promise acknowledgements outside staged WAL truth");
    assert_eq!(denial, crate::PitrResolutionDenial::FrontierOutsideSource);
}
