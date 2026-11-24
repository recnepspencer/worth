//! Fresh OS child runner and barrier/stage kill helpers for C4b.2/C4b.3 recovery interruption.
use super::super::super::opening;
use super::super::oracle_verifier::{open_dir, open_dir_configured, DirOpen};
use super::super::{assert_barrier_killed, role_barrier_placement, RestorableHome, BUILD_PRIMARY};
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
use worth_store_testkit_application_oracle::facade::host_interruption::opening::AdversarialPlacement;

const RECOVERY_CHILD: &str =
    "process::application::host_oracle::adversarial::recovery_interruption_oracle::child::host_recovery_kill_child";
const ENV_HOME: &str = "WORTH_ADV_RECKILL_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_RECKILL_MARKERS";
const ENV_SPEC: &str = "WORTH_ADV_RECKILL_SPEC";
const ENV_WAL_BYTES: &str = "WORTH_ADV_RECKILL_WAL_BYTES";
const ENV_BUILD: &str = "WORTH_ADV_RECKILL_BUILD";

pub(in super::super) fn kill_recovery_with_build_and_ctx(
    location: &Path,
    spec: &str,
    wal_bytes: Option<u64>,
    build: Option<&str>,
    ctx: &str,
) {
    let markers = tempfile::tempdir().unwrap();
    let (spec_path, wal_path, build_path) = (
        PathBuf::from(spec),
        wal_bytes.map(|b| PathBuf::from(b.to_string())),
        build.map(PathBuf::from),
    );
    let mut env: Vec<(&str, &Path)> = vec![
        (ENV_HOME, location),
        (ENV_MARKERS, markers.path()),
        (ENV_SPEC, &spec_path),
    ];
    if let Some(ref w) = wal_path {
        env.push((ENV_WAL_BYTES, w));
    }
    if let Some(ref b) = build_path {
        env.push((ENV_BUILD, b));
    }
    let (ready, expected) = match spec.strip_prefix("phys:") {
        Some(stage_label) => (markers.path().join("reached"), stage_label.to_owned()),
        None => (markers.path().join("ready"), spec.to_owned()),
    };
    let ready_check = ready.clone();
    let ctx_owned = ctx.to_owned();
    let end = crate::killed_child::run_and_kill_observing_with_ctx(
        RECOVERY_CHILD,
        &env,
        &ready,
        &markers.path().join("finished"),
        move || {
            assert_eq!(
                std::fs::read_to_string(&ready_check).unwrap_or_default(),
                expected,
                "{ctx_owned}: stage marker mismatch"
            )
        },
        ctx,
    );
    assert_eq!(
        end,
        crate::killed_child::ChildEnd::KilledAtPark,
        "{ctx}: recovery kill {spec} must park"
    );
}

pub(super) fn kill_recovery(location: &Path, spec: &str, wal_bytes: Option<u64>, ctx: &str) {
    kill_recovery_with_build_and_ctx(location, spec, wal_bytes, None, ctx);
}

pub(super) fn measure_recovery_role(
    fixture: &RestorableHome,
    role_tag: &str,
    tag: &str,
) -> (u64, u64) {
    fixture.restore();
    let markers = tempfile::tempdir().unwrap();
    let spec = PathBuf::from(format!("role:{role_tag}:count"));
    let env = [
        (ENV_HOME, fixture.location()),
        (ENV_MARKERS, markers.path()),
        (ENV_SPEC, &spec),
    ];
    let ctx = format!("{tag}/measure-recovery-role/{role_tag}");
    let end = crate::killed_child::run_and_kill_with_ctx(
        RECOVERY_CHILD,
        &env,
        &markers.path().join("ready"),
        &markers.path().join("finished"),
        &ctx,
    );
    assert_eq!(
        end,
        crate::killed_child::ChildEnd::Finished,
        "{ctx}: calibration child must finish"
    );
    let raw = std::fs::read_to_string(markers.path().join("finished"))
        .unwrap_or_else(|e| panic!("{ctx}: read finished marker: {e:?}"));
    let (phys, total) = raw
        .trim()
        .split_once(':')
        .unwrap_or_else(|| panic!("{ctx}: malformed finished marker {raw:?}"));
    (
        phys.parse()
            .unwrap_or_else(|e| panic!("{ctx}: parse physical count from {raw:?}: {e:?}")),
        total
            .parse()
            .unwrap_or_else(|e| panic!("{ctx}: parse total count from {raw:?}: {e:?}")),
    )
}

pub(super) fn kill_recovery_role(
    fixture: &RestorableHome,
    kind: (&str, MediaOperationRole),
    bounds: (u64, u64),
    tag: &str,
) {
    let ((role_tag, role), (ordinal, total)) = (kind, bounds);
    fixture.restore();
    let markers = tempfile::tempdir().unwrap();
    let (spec, ready) = (
        PathBuf::from(format!("role:{role_tag}:{ordinal}")),
        markers.path().join("ready"),
    );
    let ctx = format!("{tag} recovery {role_tag}#{ordinal}/{total}");
    let end = crate::killed_child::run_and_kill_with_ctx(
        RECOVERY_CHILD,
        &[
            (ENV_HOME, fixture.location()),
            (ENV_MARKERS, markers.path()),
            (ENV_SPEC, &spec),
        ],
        &ready,
        &markers.path().join("finished"),
        &ctx,
    );
    assert_barrier_killed(end, &ready, role, ordinal, &ctx);
}

#[test]
fn host_recovery_kill_child() {
    let Some(location) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let (markers, spec) = (
        PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap()),
        std::env::var(ENV_SPEC).unwrap(),
    );
    let wal_profile = std::env::var_os(ENV_WAL_BYTES).map(|raw| {
        ApplicationHomeCertificationProfile::with_wal_segment_bytes(
            raw.to_str().unwrap().parse().unwrap(),
        )
    });
    let run = || run_recovery_child(&location, &markers, &spec);
    match wal_profile {
        Some(p) => p.run(run),
        None => run(),
    }
}

fn run_recovery_child(location: &Path, markers: &Path, spec: &str) {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let build = std::env::var(ENV_BUILD).unwrap_or_else(|_| BUILD_PRIMARY.to_owned());
    let open_with = |placement: AdversarialPlacement| {
        open_dir_configured(
            DirOpen {
                location,
                build: &build,
                placement: Some(placement),
                contacts: Arc::new(AtomicUsize::new(0)),
            },
            &execution,
        )
    };
    if let Some(stage_label) = spec.strip_prefix("phys:") {
        let stage = PhysStage::from_label(stage_label).expect("valid physical recovery stage");
        let yieldpoint = PhysicalRecoveryProcessYieldpoint::new(
            stage,
            markers.join("reached"),
            markers.join("release"),
            markers.join("cancel"),
        );
        let (home, _) = open_with(AdversarialPlacement::new().with_recovery_yieldpoint(yieldpoint));
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
        let placement = AdversarialPlacement::new().with_stage_observer(
            move |arrival: ApplicationHomeOpeningArrival| {
                if arrival.stage == stage && arrival.edge == edge {
                    crate::killed_child::announce(&ready, spec_owned.as_bytes());
                    crate::killed_child::park();
                }
            },
        );
        let (home, _) = open_with(placement);
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
            role_barrier_placement(
                role,
                target.parse().unwrap(),
                Some(markers.join("ready")),
                Some(done.clone()),
            )
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
        let (home, _) = open_dir(
            location,
            Some(placement),
            Arc::new(AtomicUsize::new(0)),
            &execution,
        );
        done.store(true, Ordering::SeqCst);
        if let Some(h) = watcher {
            let _ = h.join();
            let actual = activation.matching_operation_count();
            crate::killed_child::announce(
                &markers.join("finished"),
                format!("missed:actual={actual}").as_bytes(),
            );
            return;
        }
        let (phys, count) = (
            physical_ops.load(Ordering::SeqCst),
            activation.matching_operation_count(),
        );
        home.close(&execution).unwrap().reclaim();
        crate::killed_child::announce(
            &markers.join("finished"),
            format!("{phys}:{count}").as_bytes(),
        );
        return;
    }
    panic!("unknown recovery kill spec {spec}");
}
