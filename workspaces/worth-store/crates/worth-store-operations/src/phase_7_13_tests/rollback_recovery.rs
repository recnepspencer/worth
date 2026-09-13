use super::*;
use crate::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, OperationalOperationId,
    OperationalTransitionId, RollbackIntent,
};
use crate::{
    PitrCandidatePosture, PitrRoundingPolicy, RecoveryTimelineAdmission, RecoveryTimelineOwner,
    RollbackReplayOwner,
};
use worth_store_authority::report_retained_store_authority_evidence;
use worth_store_physical_isolation::RecoverySourceLeaseRegistry;
#[test]
fn retained_authority_rollback_stages_forward_without_mutating_current_media() {
    let world = restore_world("phase-10-retained-rollback");
    let target = world.restore_directory.path().join("rollback-target");
    let leases = RecoverySourceLeaseRegistry::open(
        world
            .restore_directory
            .path()
            .join("rollback-source-leases"),
    )
    .unwrap();
    std::fs::create_dir_all(&target).unwrap();
    let retained = report_retained_store_authority_evidence(&world.authority);
    let materialized = world.admissible.custody().structural().materialized();
    let manifest = materialized.manifest();
    let wal_end = manifest.wal_half_open_interval().1;
    let lineage = RollbackReplayOwner::source_lineage(&retained, manifest);
    let frontier = RecoveryTimelineOwner::admit_observation(RecoveryTimelineAdmission {
        observed_time: 100,
        uncertainty_before: 0,
        uncertainty_after: 0,
        checkpoint_durability: manifest.durable_checkpoint_lsn(),
        wal_structural: wal_end,
        local_durable_commit: wal_end,
        client_acknowledged: manifest.acknowledged_frontier(),
        replication_acknowledged: manifest.acknowledged_frontier(),
        authority_identity: world.admissible.admission().admitting_authority(),
        source_lineage: lineage,
        source_identity: materialized.manifest_digest(),
        posture: PitrCandidatePosture::Available,
    })
    .unwrap();
    let frontier = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::ExactOnly,
        vec![frontier],
    )
    .unwrap()
    .select()
    .unwrap()
    .exact_frontier();
    let current_before = media_snapshot(world.scenario.source_root());
    let security_scope = recovery_security_scope(&world.admissible);
    let lowered = RollbackIntent::from_retained_authority(
        OperationalOperationId::new("rollback-retained-authority").unwrap(),
        retained,
        world.admissible,
        frontier,
        &target,
        security_scope,
        u64::MAX,
        31,
    )
    .resolve()
    .expect("resolve retained source")
    .admit_source_cut(&leases)
    .expect("durably lease retained closure")
    .lease()
    .lower()
    .expect("lower rollback owner DAG");
    assert_recovery_lifecycle_dag(lowered.explanation());
    let executed = lowered
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
            OperationalTransitionId::new("consume-rollback-staging").unwrap(),
            &world.authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .unwrap()
        .execute(&CurrentStagingAuthorizationPort)
        .expect("stage retained authority as a new root");
    assert_eq!(executed.receipt().recovery().frontier(), frontier);
    assert_eq!(media_snapshot(world.scenario.source_root()), current_before);
    assert_eq!(
        selected_staging_kind(&world.authority, &world.control),
        crate::RecoveryStagingOperationKind::Rollback
    );

    assert_eq!(leases.recover_active().unwrap().len(), 1);
}
