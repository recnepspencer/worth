//! C4b.2: Recovery interruption at every named physical recovery yieldpoint
//! stage (all 13), every ApplicationHomeOpeningStage (all 6, Before & After),
//! recovery-time SynchronizeFileState barriers, and chained consecutive
//! recovery kills — converging idempotently to MemoryOnly parity.
mod child;

pub(super) use self::child::kill_recovery_with_build_and_ctx;
use self::child::{kill_recovery, kill_recovery_role, measure_recovery_role};
use super::super::opening;
use super::oracle_verifier::{build_reference, verify_reopen_and_continue};
use super::recovery_fixtures::{
    discover_mutate_ordinals, prepare_cleanup_wal_fixture, prepare_committed_in_flight,
    prepare_guaranteed_split_committed, prepare_indeterminate_replacement,
    prepare_non_splitting_posture, prepare_pre_operation, prepare_unpublished_custody,
    CLEANUP_WAL_BYTES,
};
use std::num::NonZeroU64;
use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::{
    certification::MediaOperationRole, PhysicalRecoveryYieldpointStage as PhysStage,
};
use worth_store_application::{
    ApplicationHomeCertificationProfile, ApplicationHomeOpeningEdge as OpenEdge,
    ApplicationHomeOpeningStage as OpenStage,
};
use worth_store_testkit_application_oracle::facade::host_observation::adversarial_verdict::RecoveredPrefix;

pub(super) const PHYSICAL_STAGES: [PhysStage; 13] = [
    PhysStage::StagingMaterialization,
    PhysStage::StagingSynchronization,
    PhysStage::CandidateMaterialization,
    PhysStage::CandidateSynchronization,
    PhysStage::RootProtocolReplacement,
    PhysStage::RecordNamespaceSynchronization,
    PhysStage::FreshReopenCurrentSelector,
    PhysStage::FreshReopenRootManifest,
    PhysStage::FreshReopenExactBinding,
    PhysStage::CleanupFreshnessRead,
    PhysStage::CleanupRemoval,
    PhysStage::ObligationDischarge,
    PhysStage::ServingHandoff,
];

// No wildcard: any new Store physical recovery stage must extend this roster.
const fn physical_stage_index(stage: PhysStage) -> usize {
    match stage {
        PhysStage::StagingMaterialization => 0,
        PhysStage::StagingSynchronization => 1,
        PhysStage::CandidateMaterialization => 2,
        PhysStage::CandidateSynchronization => 3,
        PhysStage::RootProtocolReplacement => 4,
        PhysStage::RecordNamespaceSynchronization => 5,
        PhysStage::FreshReopenCurrentSelector => 6,
        PhysStage::FreshReopenRootManifest => 7,
        PhysStage::FreshReopenExactBinding => 8,
        PhysStage::CleanupFreshnessRead => 9,
        PhysStage::CleanupRemoval => 10,
        PhysStage::ObligationDischarge => 11,
        PhysStage::ServingHandoff => 12,
    }
}

// No wildcard: any new ApplicationHomeOpeningStage must extend this roster.
pub(super) const fn opening_stage_index(stage: OpenStage) -> usize {
    match stage {
        OpenStage::DescriptorIncarnation => 0,
        OpenStage::NativeReadmission => 1,
        OpenStage::WorldReconciliation => 2,
        OpenStage::QueryReadmission => 3,
        OpenStage::Custody => 4,
        OpenStage::ServiceHandoff => 5,
    }
}

#[test]
fn host_recovery_killed_at_every_physical_recovery_stage() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_reference(&execution);
    let ordinals = discover_mutate_ordinals(&execution);
    let committed = prepare_committed_in_flight(ordinals, &execution);
    let custody = prepare_unpublished_custody(ordinals, &execution);
    let pre_op = prepare_pre_operation(ordinals, &execution);
    let indeterminate = prepare_indeterminate_replacement(ordinals, &execution);
    let (cleanup, cleanup_ref) = prepare_cleanup_wal_fixture(&execution);
    let cleanup_profile = ApplicationHomeCertificationProfile::with_wal_segment_bytes(
        NonZeroU64::new(CLEANUP_WAL_BYTES).unwrap(),
    );

    for stage in PHYSICAL_STAGES {
        assert_eq!(PHYSICAL_STAGES[physical_stage_index(stage)], stage);
        let spec = format!("phys:{}", stage.label());
        match stage {
            PhysStage::CleanupFreshnessRead | PhysStage::CleanupRemoval => {
                let ctx = format!("cleanup/{spec}");
                cleanup.restore();
                kill_recovery(cleanup.location(), &spec, Some(CLEANUP_WAL_BYTES), &ctx);
                cleanup_profile.run(|| {
                    let v = verify_reopen_and_continue(
                        cleanup.location(),
                        &cleanup_ref,
                        &execution,
                        &ctx,
                    );
                    assert_eq!(v, RecoveredPrefix::PreOperation, "{ctx}: cleanup prefix");
                });
            }
            PhysStage::ObligationDischarge => {
                let ctx = format!("discharge/{spec}");
                indeterminate.restore();
                kill_recovery(indeterminate.location(), &spec, None, &ctx);
                let v = verify_reopen_and_continue(
                    indeterminate.location(),
                    &reference,
                    &execution,
                    &ctx,
                );
                assert_eq!(
                    v,
                    RecoveredPrefix::CommittedInFlight,
                    "{ctx}: discharge prefix"
                );
            }
            _ => {
                let ctx = format!("committed/{spec}");
                committed.restore();
                kill_recovery(committed.location(), &spec, None, &ctx);
                let v =
                    verify_reopen_and_continue(committed.location(), &reference, &execution, &ctx);
                assert_eq!(
                    v,
                    RecoveredPrefix::CommittedInFlight,
                    "{ctx}: committed prefix"
                );
            }
        }
    }

    // Also prove physical recovery kills across UnpublishedCustody and PreOperation postures.
    for stage in [
        PhysStage::StagingMaterialization,
        PhysStage::RootProtocolReplacement,
        PhysStage::ServingHandoff,
    ] {
        let spec = format!("phys:{}", stage.label());
        let custody_ctx = format!("custody/{spec}");
        custody.restore();
        kill_recovery(custody.location(), &spec, None, &custody_ctx);
        let v =
            verify_reopen_and_continue(custody.location(), &reference, &execution, &custody_ctx);
        assert!(
            matches!(v, RecoveredPrefix::UnpublishedCustody { .. }),
            "{custody_ctx}: {v:?}"
        );
        let preop_ctx = format!("preop/{spec}");
        pre_op.restore();
        kill_recovery(pre_op.location(), &spec, None, &preop_ctx);
        let v = verify_reopen_and_continue(pre_op.location(), &reference, &execution, &preop_ctx);
        assert_eq!(v, RecoveredPrefix::PreOperation, "{preop_ctx}");
    }
}

#[test]
fn host_recovery_killed_at_every_opening_stage_before_and_after() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_reference(&execution);
    let ordinals = discover_mutate_ordinals(&execution);
    let postures = [
        (
            "committed",
            prepare_committed_in_flight(ordinals, &execution),
        ),
        ("custody", prepare_unpublished_custody(ordinals, &execution)),
        ("preop", prepare_pre_operation(ordinals, &execution)),
    ];
    for stage in OpenStage::ALL {
        assert_eq!(OpenStage::ALL[opening_stage_index(stage)], stage);
    }
    for (posture, fixture) in &postures {
        for stage in OpenStage::ALL {
            for edge in [OpenEdge::Before, OpenEdge::After] {
                fixture.restore();
                let edge_tag = if edge == OpenEdge::Before {
                    "before"
                } else {
                    "after"
                };
                let spec = format!("open:{}:{edge_tag}", opening_stage_index(stage));
                let label = format!("{posture}/{stage:?}:{edge:?}");
                kill_recovery(fixture.location(), &spec, None, &label);
                let v =
                    verify_reopen_and_continue(fixture.location(), &reference, &execution, &label);
                assert_expected_posture(posture, &v, &label);
            }
        }
    }
}

#[test]
fn host_recovery_killed_at_every_recovery_sync_barrier_and_chained_kills() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_reference(&execution);
    let ordinals = discover_mutate_ordinals(&execution);
    let committed = prepare_committed_in_flight(ordinals, &execution);
    let custody = prepare_unpublished_custody(ordinals, &execution);
    let pre_op = prepare_pre_operation(ordinals, &execution);

    let committed_short = prepare_non_splitting_posture("committed", ordinals, &execution);
    let custody_short = prepare_non_splitting_posture("custody", ordinals, &execution);
    let pre_op_short = prepare_non_splitting_posture("preop", ordinals, &execution);
    let (committed_split, split_ref) = prepare_guaranteed_split_committed(ordinals, &execution);

    // Part A: Every recovery-time AtomicReplace (partial root-protocol replacement),
    // SynchronizeDirectoryPublication, and SynchronizeFileState barrier across
    // deterministic non-splitting layouts (all 3 postures) AND the deterministic
    // full-leaf splitting layout (`committed_split`), with exact `t1 == t2` and
    // zero retries — every single measured barrier MUST confirm `assert_barrier_killed`.
    let sync_totals = std::sync::Mutex::new(std::collections::BTreeMap::<&str, u64>::new());
    let sync_totals_ref = &sync_totals;
    std::thread::scope(|scope| {
        for (tag, posture, fixture, target_ref) in [
            ("committed_short", "committed", &committed_short, &reference),
            ("custody_short", "custody", &custody_short, &reference),
            ("preop_short", "preop", &pre_op_short, &reference),
            ("committed_split", "committed", &committed_split, &split_ref),
        ] {
            scope.spawn(move || {
                let w_req = opening::serial();
                let w_exec = ExecutionRequest::serial(&w_req);
                for (role_tag, role, min_phys) in [
                    ("replace", MediaOperationRole::AtomicReplace, 3u64),
                    (
                        "dir_sync",
                        MediaOperationRole::SynchronizeDirectoryPublication,
                        1u64,
                    ),
                    ("sync", MediaOperationRole::SynchronizeFileState, 1u64),
                ] {
                    let (p1, t1) = measure_recovery_role(fixture, role_tag, tag);
                    let (p2, t2) = measure_recovery_role(fixture, role_tag, tag);
                    assert_eq!(
                        (p1, t1),
                        (p2, t2),
                        "{tag}/{role_tag}: deterministic layout must have exact calibration"
                    );
                    assert!(
                        p1 >= min_phys && t1 >= p1,
                        "{tag}/{role_tag}: expected >= {min_phys} physical ops, got ({p1}, {t1})"
                    );
                    if role_tag == "sync" {
                        sync_totals_ref.lock().unwrap().insert(tag, t1);
                    }
                    for ordinal in 1..=t1 {
                        kill_recovery_role(fixture, (role_tag, role), (ordinal, t1), tag);
                        let label = format!("{tag}/recovery_{role_tag}#{ordinal}_of_{t1}");
                        let v = verify_reopen_and_continue(
                            fixture.location(),
                            target_ref,
                            &w_exec,
                            &label,
                        );
                        assert_expected_posture(posture, &v, &label);
                    }
                }
            });
        }
    });
    let totals = sync_totals.into_inner().unwrap();
    assert!(
        totals["committed_split"] > totals["committed_short"],
        "split layout ({}) must exceed non-split ({})",
        totals["committed_split"],
        totals["committed_short"]
    );
    let chain = [
        format!("phys:{}", PhysStage::CandidateSynchronization.label()),
        format!(
            "open:{}:after",
            opening_stage_index(OpenStage::WorldReconciliation)
        ),
        format!("open:{}:after", opening_stage_index(OpenStage::Custody)),
    ];
    for (posture, fixture) in [
        ("committed", &committed),
        ("custody", &custody),
        ("preop", &pre_op),
    ] {
        fixture.restore();
        for spec in &chain {
            kill_recovery(
                fixture.location(),
                spec,
                None,
                &format!("chained/{posture}/{spec}"),
            );
        }
        let label = format!("chained/{posture}");
        let v = verify_reopen_and_continue(fixture.location(), &reference, &execution, &label);
        assert_expected_posture(posture, &v, &label);
    }
}

fn assert_expected_posture(posture: &str, verdict: &RecoveredPrefix, label: &str) {
    match posture {
        "committed" => assert_eq!(*verdict, RecoveredPrefix::CommittedInFlight, "{label}"),
        "custody" => assert!(
            matches!(verdict, RecoveredPrefix::UnpublishedCustody { .. }),
            "{label}: {verdict:?}"
        ),
        "preop" => assert_eq!(*verdict, RecoveredPrefix::PreOperation, "{label}"),
        other => panic!("unknown posture {other}"),
    }
}
