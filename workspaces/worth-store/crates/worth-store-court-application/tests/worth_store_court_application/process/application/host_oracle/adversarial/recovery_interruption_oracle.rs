//! C4b.2: Recovery interruption at every named physical recovery yieldpoint
//! stage (all 13), every ApplicationHomeOpeningStage (all 6, Before & After),
//! recovery-time SynchronizeFileState barriers, and chained consecutive
//! recovery kills — converging idempotently to MemoryOnly parity.
use super::super::opening;
use super::oracle_verifier::{build_reference, open_dir, verify_reopen_and_continue};
use super::recovery_fixtures::{
    discover_mutate_ordinals, prepare_cleanup_wal_fixture, prepare_committed_in_flight,
    prepare_guaranteed_split_committed, prepare_indeterminate_replacement,
    prepare_non_splitting_posture, prepare_pre_operation, prepare_unpublished_custody,
    CLEANUP_WAL_BYTES,
};
use super::{assert_barrier_killed, role_barrier_placement, RestorableHome};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::{
    certification::MediaOperationRole, PhysicalRecoveryProcessYieldpoint,
    PhysicalRecoveryYieldpointStage as PhysStage,
};
use worth_store_application::{
    ApplicationHomeCertificationProfile, ApplicationHomeOpeningArrival,
    ApplicationHomeOpeningEdge as OpenEdge, ApplicationHomeOpeningStage as OpenStage,
};
use worth_store_testkit_application_oracle::facade::{
    host_interruption::opening::AdversarialPlacement,
    host_observation::adversarial_verdict::RecoveredPrefix,
};

const RECOVERY_CHILD: &str =
    "process::application::host_oracle::adversarial::recovery_interruption_oracle::host_recovery_kill_child";
const ENV_HOME: &str = "WORTH_ADV_RECKILL_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_RECKILL_MARKERS";
const ENV_SPEC: &str = "WORTH_ADV_RECKILL_SPEC";
const ENV_WAL_BYTES: &str = "WORTH_ADV_RECKILL_WAL_BYTES";

const PHYSICAL_STAGES: [PhysStage; 13] = [
    PhysStage::StagingMaterialization, PhysStage::StagingSynchronization,
    PhysStage::CandidateMaterialization, PhysStage::CandidateSynchronization,
    PhysStage::RootProtocolReplacement, PhysStage::RecordNamespaceSynchronization,
    PhysStage::FreshReopenCurrentSelector, PhysStage::FreshReopenRootManifest,
    PhysStage::FreshReopenExactBinding, PhysStage::CleanupFreshnessRead,
    PhysStage::CleanupRemoval, PhysStage::ObligationDischarge, PhysStage::ServingHandoff,
];

// No wildcard: any new Store physical recovery stage must extend this roster.
const fn physical_stage_index(stage: PhysStage) -> usize {
    match stage {
        PhysStage::StagingMaterialization => 0, PhysStage::StagingSynchronization => 1,
        PhysStage::CandidateMaterialization => 2, PhysStage::CandidateSynchronization => 3,
        PhysStage::RootProtocolReplacement => 4, PhysStage::RecordNamespaceSynchronization => 5,
        PhysStage::FreshReopenCurrentSelector => 6, PhysStage::FreshReopenRootManifest => 7,
        PhysStage::FreshReopenExactBinding => 8, PhysStage::CleanupFreshnessRead => 9,
        PhysStage::CleanupRemoval => 10, PhysStage::ObligationDischarge => 11,
        PhysStage::ServingHandoff => 12,
    }
}

// No wildcard: any new ApplicationHomeOpeningStage must extend this roster.
const fn opening_stage_index(stage: OpenStage) -> usize {
    match stage {
        OpenStage::DescriptorIncarnation => 0, OpenStage::NativeReadmission => 1,
        OpenStage::WorldReconciliation => 2, OpenStage::QueryReadmission => 3,
        OpenStage::Custody => 4, OpenStage::ServiceHandoff => 5,
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
                cleanup.restore();
                kill_recovery(cleanup.location(), &spec, Some(CLEANUP_WAL_BYTES));
                cleanup_profile.run(|| {
                    let v = verify_reopen_and_continue(cleanup.location(), &cleanup_ref, &execution, &spec);
                    assert_eq!(v, RecoveredPrefix::PreOperation, "{spec}: cleanup prefix");
                });
            }
            PhysStage::ObligationDischarge => {
                indeterminate.restore();
                kill_recovery(indeterminate.location(), &spec, None);
                let v = verify_reopen_and_continue(indeterminate.location(), &reference, &execution, &spec);
                assert_eq!(v, RecoveredPrefix::CommittedInFlight, "{spec}: discharge prefix");
            }
            _ => {
                committed.restore();
                kill_recovery(committed.location(), &spec, None);
                let v = verify_reopen_and_continue(committed.location(), &reference, &execution, &spec);
                assert_eq!(v, RecoveredPrefix::CommittedInFlight, "{spec}: committed prefix");
            }
        }
    }

    // Also prove physical recovery kills across UnpublishedCustody and PreOperation postures.
    for stage in [PhysStage::StagingMaterialization, PhysStage::RootProtocolReplacement, PhysStage::ServingHandoff] {
        let spec = format!("phys:{}", stage.label());
        custody.restore();
        kill_recovery(custody.location(), &spec, None);
        let v = verify_reopen_and_continue(custody.location(), &reference, &execution, &format!("custody/{spec}"));
        assert!(matches!(v, RecoveredPrefix::UnpublishedCustody { .. }), "custody/{spec}: {v:?}");
        pre_op.restore();
        kill_recovery(pre_op.location(), &spec, None);
        let v = verify_reopen_and_continue(pre_op.location(), &reference, &execution, &format!("preop/{spec}"));
        assert_eq!(v, RecoveredPrefix::PreOperation, "preop/{spec}");
    }
}

#[test]
fn host_recovery_killed_at_every_opening_stage_before_and_after() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_reference(&execution);
    let ordinals = discover_mutate_ordinals(&execution);
    let postures = [
        ("committed", prepare_committed_in_flight(ordinals, &execution)),
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
                let edge_tag = if edge == OpenEdge::Before { "before" } else { "after" };
                let spec = format!("open:{}:{edge_tag}", opening_stage_index(stage));
                kill_recovery(fixture.location(), &spec, None);
                let label = format!("{posture}/{stage:?}:{edge:?}");
                let v = verify_reopen_and_continue(fixture.location(), &reference, &execution, &label);
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
                    ("dir_sync", MediaOperationRole::SynchronizeDirectoryPublication, 1u64),
                    ("sync", MediaOperationRole::SynchronizeFileState, 1u64),
                ] {
                    let (p1, t1) = measure_recovery_role(fixture, role_tag);
                    let (p2, t2) = measure_recovery_role(fixture, role_tag);
                    assert_eq!((p1, t1), (p2, t2), "{tag}/{role_tag}: deterministic layout must have exact calibration");
                    assert!(p1 >= min_phys && t1 >= p1, "{tag}/{role_tag}: expected >= {min_phys} physical ops, got ({p1}, {t1})");
                    if role_tag == "sync" {
                        sync_totals_ref.lock().unwrap().insert(tag, t1);
                    }
                    for ordinal in 1..=t1 {
                        kill_recovery_role(fixture, (role_tag, role), (ordinal, t1), tag);
                        let label = format!("{tag}/recovery_{role_tag}#{ordinal}_of_{t1}");
                        let v = verify_reopen_and_continue(fixture.location(), target_ref, &w_exec, &label);
                        assert_expected_posture(posture, &v, &label);
                    }
                }
            });
        }
    });
    let totals = sync_totals.into_inner().unwrap();
    assert!(
        totals["committed_split"] > totals["committed_short"],
        "guaranteed-split layout ({}) must execute strictly more syncs than non-splitting layout ({})",
        totals["committed_split"],
        totals["committed_short"]
    );

    // Part B: 3-deep chained consecutive recovery kills across all 3 postures.
    let chain = [
        format!("phys:{}", PhysStage::CandidateSynchronization.label()),
        format!("open:{}:after", opening_stage_index(OpenStage::WorldReconciliation)),
        format!("open:{}:after", opening_stage_index(OpenStage::Custody)),
    ];
    for (posture, fixture) in [("committed", &committed), ("custody", &custody), ("preop", &pre_op)] {
        fixture.restore();
        for (step, spec) in chain.iter().enumerate() {
            kill_recovery(fixture.location(), spec, None);
            eprintln!("chained recovery kill {posture} step#{step}: {spec}");
        }
        let label = format!("chained/{posture}");
        let v = verify_reopen_and_continue(fixture.location(), &reference, &execution, &label);
        assert_expected_posture(posture, &v, &label);
    }
}

fn assert_expected_posture(posture: &str, verdict: &RecoveredPrefix, label: &str) {
    match posture {
        "committed" => assert_eq!(*verdict, RecoveredPrefix::CommittedInFlight, "{label}"),
        "custody" => assert!(matches!(verdict, RecoveredPrefix::UnpublishedCustody { .. }), "{label}: {verdict:?}"),
        "preop" => assert_eq!(*verdict, RecoveredPrefix::PreOperation, "{label}"),
        other => panic!("unknown posture {other}"),
    }
}

fn kill_recovery(location: &Path, spec: &str, wal_bytes: Option<u64>) {
    let markers = tempfile::tempdir().unwrap();
    let spec_path = PathBuf::from(spec);
    let wal_path = wal_bytes.map(|b| PathBuf::from(b.to_string()));
    let mut env: Vec<(&str, &Path)> = vec![(ENV_HOME, location), (ENV_MARKERS, markers.path()), (ENV_SPEC, &spec_path)];
    if let Some(ref w) = wal_path { env.push((ENV_WAL_BYTES, w)); }
    let (ready, expected) = match spec.strip_prefix("phys:") {
        Some(stage_label) => (markers.path().join("reached"), stage_label.to_owned()),
        None => (markers.path().join("ready"), spec.to_owned()),
    };
    let ready_check = ready.clone();
    let end = crate::killed_child::run_and_kill_observing(
        RECOVERY_CHILD, &env, &ready, &markers.path().join("finished"),
        move || assert_eq!(std::fs::read_to_string(&ready_check).unwrap_or_default(), expected, "recovery stage marker mismatch before kill"),
    );
    assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark, "recovery kill spec {spec} must park at target stage");
}

fn measure_recovery_role(fixture: &RestorableHome, role_tag: &str) -> (u64, u64) {
    fixture.restore();
    let markers = tempfile::tempdir().unwrap();
    let spec = PathBuf::from(format!("role:{role_tag}:count"));
    let env = [(ENV_HOME, fixture.location()), (ENV_MARKERS, markers.path()), (ENV_SPEC, &spec)];
    let end = crate::killed_child::run_and_kill(RECOVERY_CHILD, &env, &markers.path().join("ready"), &markers.path().join("finished"));
    assert_eq!(end, crate::killed_child::ChildEnd::Finished);
    let raw = std::fs::read_to_string(markers.path().join("finished")).unwrap();
    let mut parts = raw.trim().split(':');
    (parts.next().unwrap().parse().unwrap(), parts.next().unwrap().parse().unwrap())
}

fn kill_recovery_role(
    fixture: &RestorableHome,
    kind: (&str, MediaOperationRole),
    bounds: (u64, u64),
    tag: &str,
) {
    let (role_tag, role) = kind;
    let (ordinal, total) = bounds;
    fixture.restore();
    let markers = tempfile::tempdir().unwrap();
    let spec = PathBuf::from(format!("role:{role_tag}:{ordinal}"));
    let ready = markers.path().join("ready");
    let end = crate::killed_child::run_and_kill(
        RECOVERY_CHILD,
        &[(ENV_HOME, fixture.location()), (ENV_MARKERS, markers.path()), (ENV_SPEC, &spec)],
        &ready,
        &markers.path().join("finished"),
    );
    assert_barrier_killed(end, &ready, role, ordinal, &format!("{tag} recovery {role_tag}#{ordinal} of {total}"));
}

#[test]
fn host_recovery_kill_child() {
    let Some(location) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let spec = std::env::var(ENV_SPEC).unwrap();
    let wal_profile = std::env::var_os(ENV_WAL_BYTES).map(|raw| {
        let bytes = raw.to_str().unwrap().parse::<NonZeroU64>().unwrap();
        ApplicationHomeCertificationProfile::with_wal_segment_bytes(bytes)
    });
    let run = || run_recovery_child(&location, &markers, &spec);
    match wal_profile {
        Some(profile) => profile.run(run),
        None => run(),
    }
}

fn run_recovery_child(location: &Path, markers: &Path, spec: &str) {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    if let Some(stage_label) = spec.strip_prefix("phys:") {
        let stage = PhysStage::from_label(stage_label).expect("valid physical recovery stage");
        let yieldpoint = PhysicalRecoveryProcessYieldpoint::new(
            stage,
            markers.join("reached"),
            markers.join("release"),
            markers.join("cancel"),
        );
        let (home, _) = open_dir(
            location,
            Some(AdversarialPlacement::new().with_recovery_yieldpoint(yieldpoint)),
            Arc::new(AtomicUsize::new(0)),
            &execution,
        );
        home.close(&execution).unwrap().reclaim();
        crate::killed_child::announce(&markers.join("finished"), b"missed_phys");
        return;
    }
    if let Some(rest) = spec.strip_prefix("open:") {
        let mut parts = rest.split(':');
        let stage = OpenStage::ALL[parts.next().unwrap().parse::<usize>().unwrap()];
        let edge = match parts.next().unwrap() {
            "before" => OpenEdge::Before,
            "after" => OpenEdge::After,
            other => panic!("unknown edge {other}"),
        };
        let ready = markers.join("ready");
        let spec_owned = spec.to_owned();
        let placement = AdversarialPlacement::new().with_stage_observer(move |arrival: ApplicationHomeOpeningArrival| {
            if arrival.stage == stage && arrival.edge == edge {
                crate::killed_child::announce(&ready, spec_owned.as_bytes());
                crate::killed_child::park();
            }
        });
        let (home, _) = open_dir(location, Some(placement), Arc::new(AtomicUsize::new(0)), &execution);
        home.close(&execution).unwrap().reclaim();
        crate::killed_child::announce(&markers.join("finished"), b"missed_open");
        return;
    }
    if let Some(rest) = spec.strip_prefix("role:") {
        let mut parts = rest.split(':');
        let role = match parts.next().unwrap() {
            "sync" => MediaOperationRole::SynchronizeFileState,
            "replace" => MediaOperationRole::AtomicReplace,
            "dir_sync" => MediaOperationRole::SynchronizeDirectoryPublication,
            other => panic!("unknown recovery media role {other}"),
        };
        let target = parts.next().unwrap();
        let done = Arc::new(AtomicBool::new(false));
        let (mut placement, activation, watcher) = if target == "count" {
            role_barrier_placement(role, u64::MAX, None, None)
        } else {
            role_barrier_placement(role, target.parse().unwrap(), Some(markers.join("ready")), Some(done.clone()))
        };
        let physical_ops = Arc::new(std::sync::atomic::AtomicU64::new(0));
        if target == "count" {
            let (act, phys) = (activation.clone(), physical_ops.clone());
            placement = placement.with_stage_observer(move |a: ApplicationHomeOpeningArrival| {
                if a.stage == OpenStage::DescriptorIncarnation && a.edge == OpenEdge::Before {
                    phys.store(act.matching_operation_count(), Ordering::SeqCst);
                }
            });
        }
        activation.arm().unwrap();
        let (home, _) = open_dir(location, Some(placement), Arc::new(AtomicUsize::new(0)), &execution);
        done.store(true, Ordering::SeqCst);
        if let Some(h) = watcher {
            let _ = h.join();
            let actual = activation.matching_operation_count();
            crate::killed_child::announce(&markers.join("finished"), format!("missed:actual={actual}").as_bytes());
            return;
        }
        let (phys, count) = (physical_ops.load(Ordering::SeqCst), activation.matching_operation_count());
        home.close(&execution).unwrap().reclaim();
        crate::killed_child::announce(&markers.join("finished"), format!("{phys}:{count}").as_bytes());
        return;
    }
    panic!("unknown recovery kill spec {spec}");
}
