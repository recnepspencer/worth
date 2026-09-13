use std::path::Path;

use crate::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, BackupRestoreIntent,
    ExecutedBackupRestore, ExecutedPointInTimeRecovery, ExecutedRollback, OperationalControlStore,
    OperationalControlStorePort, OperationalOperationId, OperationalSecurityScope,
    OperationalTransitionId, PitrCandidatePosture, PitrRoundingPolicy, PointInTimeRecoveryIntent,
    ProductionRestoreAdmissibleBackupBundle, RecoveryTimelineAdmission, RecoveryTimelineOwner,
    RollbackIntent, RollbackReplayOwner,
};
use worth_store_authority::{
    report_retained_store_authority_evidence, StoreCurrentAuthorityWitness,
};
use worth_store_physical_isolation::RecoverySourceLeaseRegistry;

use super::{
    certification_operator_assertion, CurrentScenarioStagingPort, ExactScenarioAuthorizationPort,
};

pub fn execute_scenario_restore_staging(
    operation_name: &str,
    source: ProductionRestoreAdmissibleBackupBundle,
    target_parent: &Path,
    authority: &StoreCurrentAuthorityWitness,
    control: &OperationalControlStore,
    append: &dyn OperationalControlStorePort,
) -> ExecutedBackupRestore {
    std::fs::create_dir_all(target_parent).expect("restore staging parent");
    let security_scope =
        OperationalSecurityScope::from_admission(source.custody().custody_receipt());
    BackupRestoreIntent::from_verified_backup(
        operation(operation_name),
        source,
        target_parent,
        security_scope,
        u64::MAX,
        31,
    )
    .resolve()
    .lower()
    .expect("canonical restore owner DAG")
    .authorize(
        &ExactScenarioAuthorizationPort,
        &certification_operator_assertion(),
        20,
        80,
        AuthorizationReplayPolicy::SingleUse,
        AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
    )
    .expect("exact restore authorization")
    .ready_with_certification_control_store(
        control,
        append,
        transition(operation_name, "consume-staging-authorization"),
        authority,
        21,
        AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
    )
    .expect("durable restore authorization consumption")
    .execute(&CurrentScenarioStagingPort)
    .expect("owner-backed restore staging")
}

pub fn execute_scenario_pitr_staging(
    operation_name: &str,
    source: ProductionRestoreAdmissibleBackupBundle,
    target_parent: &Path,
    lease_root: &Path,
    authority: &StoreCurrentAuthorityWitness,
    control: &OperationalControlStore,
    append: &dyn OperationalControlStorePort,
) -> ExecutedPointInTimeRecovery {
    std::fs::create_dir_all(target_parent).expect("PITR staging parent");
    let leases = RecoverySourceLeaseRegistry::open(lease_root).expect("PITR source lease registry");
    let materialized = source.custody().structural().materialized();
    let manifest = materialized.manifest();
    let wal_end = manifest.wal_half_open_interval().1;
    let observation = RecoveryTimelineOwner::admit_observation(RecoveryTimelineAdmission {
        observed_time: 100,
        uncertainty_before: 0,
        uncertainty_after: 0,
        checkpoint_durability: manifest.durable_checkpoint_lsn(),
        wal_structural: wal_end,
        local_durable_commit: wal_end,
        client_acknowledged: manifest.acknowledged_frontier(),
        replication_acknowledged: manifest.acknowledged_frontier(),
        authority_identity: source.admission().admitting_authority(),
        source_lineage: [0x91; 32],
        source_identity: materialized.manifest_digest(),
        posture: PitrCandidatePosture::Available,
    })
    .expect("exact PITR observation");
    let candidates = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::ExactOnly,
        vec![observation],
    )
    .expect("exact PITR candidate set");
    let security_scope =
        OperationalSecurityScope::from_admission(source.custody().custody_receipt());
    PointInTimeRecoveryIntent::near(
        operation(operation_name),
        source,
        candidates,
        target_parent,
        security_scope,
        u64::MAX,
        31,
    )
    .resolve()
    .expect("exact PITR frontier")
    .admit_source_cut(&leases)
    .expect("durable PITR source cut")
    .lease()
    .lower()
    .expect("canonical PITR owner DAG")
    .authorize(
        &ExactScenarioAuthorizationPort,
        &certification_operator_assertion(),
        20,
        80,
        AuthorizationReplayPolicy::SingleUse,
        AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
    )
    .expect("exact PITR authorization")
    .ready_with_certification_control_store(
        control,
        append,
        transition(operation_name, "consume-staging-authorization"),
        authority,
        21,
        AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
    )
    .expect("durable PITR authorization consumption")
    .execute(&CurrentScenarioStagingPort)
    .expect("owner-backed PITR staging")
}

pub fn execute_scenario_rollback_staging(
    operation_name: &str,
    source: ProductionRestoreAdmissibleBackupBundle,
    target_parent: &Path,
    lease_root: &Path,
    authority: &StoreCurrentAuthorityWitness,
    control: &OperationalControlStore,
    append: &dyn OperationalControlStorePort,
) -> ExecutedRollback {
    std::fs::create_dir_all(target_parent).expect("rollback staging parent");
    let leases =
        RecoverySourceLeaseRegistry::open(lease_root).expect("rollback source lease registry");
    let retained = report_retained_store_authority_evidence(authority);
    let materialized = source.custody().structural().materialized();
    let manifest = materialized.manifest();
    let wal_end = manifest.wal_half_open_interval().1;
    let lineage = RollbackReplayOwner::source_lineage(&retained, manifest);
    let observation = RecoveryTimelineOwner::admit_observation(RecoveryTimelineAdmission {
        observed_time: 100,
        uncertainty_before: 0,
        uncertainty_after: 0,
        checkpoint_durability: manifest.durable_checkpoint_lsn(),
        wal_structural: wal_end,
        local_durable_commit: wal_end,
        client_acknowledged: manifest.acknowledged_frontier(),
        replication_acknowledged: manifest.acknowledged_frontier(),
        authority_identity: source.admission().admitting_authority(),
        source_lineage: lineage,
        source_identity: materialized.manifest_digest(),
        posture: PitrCandidatePosture::Available,
    })
    .expect("rollback source observation");
    let frontier = RecoveryTimelineOwner::resolve_candidates(
        100,
        PitrRoundingPolicy::ExactOnly,
        vec![observation],
    )
    .expect("rollback candidate set")
    .select()
    .expect("retained rollback source")
    .exact_frontier();
    let security_scope =
        OperationalSecurityScope::from_admission(source.custody().custody_receipt());
    RollbackIntent::from_retained_authority(
        operation(operation_name),
        retained,
        source,
        frontier,
        target_parent,
        security_scope,
        u64::MAX,
        31,
    )
    .resolve()
    .expect("resolve retained rollback source")
    .admit_source_cut(&leases)
    .expect("durable rollback source cut")
    .lease()
    .lower()
    .expect("canonical rollback owner DAG")
    .authorize(
        &ExactScenarioAuthorizationPort,
        &certification_operator_assertion(),
        20,
        80,
        AuthorizationReplayPolicy::SingleUse,
        AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
    )
    .expect("exact rollback authorization")
    .ready_with_certification_control_store(
        control,
        append,
        transition(operation_name, "consume-staging-authorization"),
        authority,
        21,
        AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
    )
    .expect("durable rollback authorization consumption")
    .execute(&CurrentScenarioStagingPort)
    .expect("owner-backed rollback staging")
}

fn operation(name: &str) -> OperationalOperationId {
    OperationalOperationId::new(name).expect("scenario operation identity")
}

fn transition(operation: &str, phase: &str) -> OperationalTransitionId {
    OperationalTransitionId::new(format!("{operation}/{phase}"))
        .expect("scenario transition identity")
}

#[cfg(test)]
mod tests {
    use crate::OperationalCounterReceipt;

    use super::*;
    use crate::certification_scenario::OwnerBackedBackupScenario;

    #[test]
    fn public_scenario_helpers_execute_all_three_non_current_staging_workflows() {
        let scenario = OwnerBackedBackupScenario::materialize("recovery-staging-workflows");
        let control = scenario.control_store();
        let restore_source = scenario
            .execute_named("recovery-staging-workflows", "restore-source", &control)
            .into_restore_source();
        let pitr_source = scenario
            .execute_named("recovery-staging-workflows", "pitr-source", &control)
            .into_restore_source();
        let rollback_source = scenario
            .execute_named("recovery-staging-workflows", "rollback-source", &control)
            .into_restore_source();

        let restore = execute_scenario_restore_staging(
            "recovery-staging-workflows/restore",
            restore_source,
            &scenario.workspace_root().join("restore"),
            scenario.authority(),
            &control,
            &control,
        );
        let pitr = execute_scenario_pitr_staging(
            "recovery-staging-workflows/pitr",
            pitr_source,
            &scenario.workspace_root().join("pitr"),
            &scenario.workspace_root().join("pitr-leases"),
            scenario.authority(),
            &control,
            &control,
        );
        let rollback = execute_scenario_rollback_staging(
            "recovery-staging-workflows/rollback",
            rollback_source,
            &scenario.workspace_root().join("rollback"),
            &scenario.workspace_root().join("rollback-leases"),
            scenario.authority(),
            &control,
            &control,
        );

        for counter in [
            OperationalCounterReceipt::from_backup_restore(&restore),
            OperationalCounterReceipt::from_point_in_time_recovery(&pitr),
            OperationalCounterReceipt::from_rollback(&rollback),
        ] {
            counter.validate_structure().unwrap();
        }
    }
}
