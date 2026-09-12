use worth_store_formal_models::{
    compose_compaction_action, compose_durability_action, compose_import_action,
    compose_lease_action, compose_quarantine_state, compose_replication_action,
    compose_source_precedence_action, CompactionVisibilityAction, DurabilityRecoveryAction,
    ImportPublicationAction, LeaseReclaimAction, QuarantineReadmissionState,
    ReplicationAdmissionAction, SharedAdmissionFrontier, SharedFrontierAction,
    SharedFrontierDenial, SharedFrontierModel, SharedQuarantineFrontier,
    SharedReachabilityFrontier, SharedVisibilityFrontier, SourcePrecedenceAction,
};

#[test]
fn compaction_cutover_with_a_live_lease_reopens_with_a_legal_precedence_path() {
    let mut model = SharedFrontierModel::initial();
    apply(
        &mut model,
        compose_lease_action(LeaseReclaimAction::LeaseAcquired {
            slot: 3,
            generation: 9,
        }),
    );
    apply(
        &mut model,
        compose_source_precedence_action(SourcePrecedenceAction::SourceSelected),
    );
    apply(
        &mut model,
        compose_compaction_action(CompactionVisibilityAction::ValidateReadPlanCutover),
    );
    apply(
        &mut model,
        compose_durability_action(DurabilityRecoveryAction::Crash),
    );
    apply(
        &mut model,
        compose_durability_action(DurabilityRecoveryAction::Reopen),
    );

    assert_eq!(model.visibility(), SharedVisibilityFrontier::Reopened);
    assert_eq!(model.reachability(), SharedReachabilityFrontier::LiveLease);
    assert!(model.recovery_precedence_preserved());
}

#[test]
fn checkpoint_publication_cannot_make_quarantined_truth_current() {
    let mut model = SharedFrontierModel::initial();
    apply(
        &mut model,
        compose_durability_action(DurabilityRecoveryAction::CheckpointDurable),
    );
    apply(
        &mut model,
        compose_quarantine_state(QuarantineReadmissionState::Sealed),
    );

    assert_eq!(
        model.apply(
            compose_durability_action(DurabilityRecoveryAction::CheckpointPublished).unwrap()
        ),
        Err(SharedFrontierDenial::QuarantineBlocksPublication)
    );
}

#[test]
fn quarantine_blocks_reclaim_and_reuse_until_verified_readmission() {
    let mut model = SharedFrontierModel::initial();
    apply(
        &mut model,
        compose_quarantine_state(QuarantineReadmissionState::Sealed),
    );

    assert_eq!(
        model.apply(compose_lease_action(LeaseReclaimAction::ReclaimAdmitted).unwrap()),
        Err(SharedFrontierDenial::QuarantineBlocksRelease)
    );
    assert_eq!(
        model.apply(
            compose_lease_action(LeaseReclaimAction::IdentityReuseAdmitted {
                old_generation: 9,
                new_generation: 10,
            })
            .unwrap()
        ),
        Err(SharedFrontierDenial::QuarantineBlocksReuse)
    );

    apply(
        &mut model,
        compose_quarantine_state(QuarantineReadmissionState::RecoveryVerificationPending),
    );
    apply(
        &mut model,
        compose_quarantine_state(QuarantineReadmissionState::Readmitted),
    );
    apply(
        &mut model,
        compose_lease_action(LeaseReclaimAction::ReclaimAdmitted),
    );
    apply(
        &mut model,
        compose_lease_action(LeaseReclaimAction::IdentityReuseAdmitted {
            old_generation: 9,
            new_generation: 10,
        }),
    );

    assert_eq!(model.quarantine(), SharedQuarantineFrontier::Released);
    assert_eq!(model.reachability(), SharedReachabilityFrontier::Reused);
}

#[test]
fn external_publication_requires_durability_and_divergence_remains_terminal() {
    let mut import = SharedFrontierModel::initial();
    apply(
        &mut import,
        compose_import_action(ImportPublicationAction::CurrentScopeReadmitted),
    );
    assert_eq!(
        import.apply(compose_import_action(ImportPublicationAction::PublicationDurable).unwrap()),
        Err(SharedFrontierDenial::DurabilityAdmissionRequired)
    );
    apply(
        &mut import,
        compose_import_action(ImportPublicationAction::RecoveredArtifactAdmitted),
    );
    apply(
        &mut import,
        compose_import_action(ImportPublicationAction::CrashBeforePublication),
    );
    assert_eq!(
        import.apply(compose_import_action(ImportPublicationAction::PublicationDurable).unwrap()),
        Err(SharedFrontierDenial::ReopenRequiredAfterCrash)
    );
    apply(
        &mut import,
        compose_durability_action(DurabilityRecoveryAction::Reopen),
    );
    apply(
        &mut import,
        compose_import_action(ImportPublicationAction::PublicationDurable),
    );
    assert_eq!(import.admission(), SharedAdmissionFrontier::Published);

    let mut replication = SharedFrontierModel::initial();
    apply(
        &mut replication,
        compose_replication_action(ReplicationAdmissionAction::SourceAdmitted),
    );
    apply(
        &mut replication,
        compose_replication_action(ReplicationAdmissionAction::FreshProgressObserved),
    );
    apply(
        &mut replication,
        compose_replication_action(ReplicationAdmissionAction::LineageDivergenceDetected),
    );
    assert_eq!(
        replication.apply(
            compose_replication_action(ReplicationAdmissionAction::FreshPublicationDurable)
                .unwrap()
        ),
        Err(SharedFrontierDenial::DivergenceBlocksPublication)
    );
    assert_eq!(replication.admission(), SharedAdmissionFrontier::Divergence);
}

#[test]
fn every_shared_action_is_composed_from_a_local_protocol_family() {
    let composed = [
        compose_durability_action(DurabilityRecoveryAction::CheckpointDurable),
        compose_source_precedence_action(SourcePrecedenceAction::SourceSelected),
        compose_lease_action(LeaseReclaimAction::LeaseAcquired {
            slot: 3,
            generation: 9,
        }),
        compose_lease_action(LeaseReclaimAction::LeaseReleased {
            slot: 3,
            generation: 9,
        }),
        compose_compaction_action(CompactionVisibilityAction::ValidateReadPlanCutover),
        compose_durability_action(DurabilityRecoveryAction::Crash),
        compose_durability_action(DurabilityRecoveryAction::Reopen),
        compose_quarantine_state(QuarantineReadmissionState::Sealed),
        compose_quarantine_state(QuarantineReadmissionState::RecoveryVerificationPending),
        compose_quarantine_state(QuarantineReadmissionState::Readmitted),
        compose_compaction_action(CompactionVisibilityAction::DeferReclaim),
        compose_lease_action(LeaseReclaimAction::ReclaimAdmitted),
        compose_lease_action(LeaseReclaimAction::IdentityReuseAdmitted {
            old_generation: 9,
            new_generation: 10,
        }),
        compose_durability_action(DurabilityRecoveryAction::CheckpointPublished),
        compose_import_action(ImportPublicationAction::CurrentScopeReadmitted),
        compose_replication_action(ReplicationAdmissionAction::SourceAdmitted),
        compose_replication_action(ReplicationAdmissionAction::FreshProgressObserved),
        compose_replication_action(ReplicationAdmissionAction::FreshPublicationDurable),
        compose_replication_action(ReplicationAdmissionAction::LineageDivergenceDetected),
    ]
    .into_iter()
    .map(|action| action.expect("representative local action must compose"))
    .collect::<BTreeSet<_>>();

    assert_eq!(composed, BTreeSet::from(SharedFrontierAction::all()));
}

fn apply(model: &mut SharedFrontierModel, action: Option<SharedFrontierAction>) {
    model
        .apply(action.expect("selected local action composes into a shared frontier"))
        .expect("ordinary composed race step is legal");
}
use std::collections::BTreeSet;
