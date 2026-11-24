//! Fresh OS child process entrypoints for C4b.4 remaining families:
//! - Shared `DocumentRetentionSchema` Home (Governed Workflow -> D14 -> Historical
//!   -> Branch Program Adoption P0->P1 -> D14 on ONE cumulative durable Home),
//! - Authored Correction (`reverse(&source)` with exact original-key fate & receipt parity),
//! - External Dispatch Custody (`NotifyAccountBalance` with exact receipt & <=1/1 dispatch).
mod corr_disp;
pub(super) mod ops;
pub(super) mod wf_adopt;

use self::corr_disp::{run_correction_phase, run_dispatch_phase};
use self::ops::{adopt_p1, notify_dispatch, open_dispatch, reverse_attempt};
use self::wf_adopt::{bind_wf_home, open_raw_wf_home, run_wf_adopt_phase};
use super::super::super::opening;
use super::super::correction_fixture::open_label_home_with_build;
use super::super::sync_barrier_placement;
use super::super::workflow_correction_fixture::{discover_original_start, run_correctable_step};
use super::StepCtx;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    primary_graph::WorthQueryCorrectionSourceCoordinates,
    WorthQueryApplicationHomeLocation as Location,
};
use worth_query_testkit_host::facade::host::checkpoint_capture;

pub(super) const KILL_CHILD: &str =
    "process::application::host_oracle::adversarial::cumulative_oracle::families_child::c4b4_family_kill_child";
const REOPEN_CHILD: &str =
    "process::application::host_oracle::adversarial::cumulative_oracle::families_child::c4b4_family_reopen_child";
pub(super) const ENV_FAMILY: &str = "WORTH_C4B4_FAM_KIND";
pub(super) const ENV_PHASE: &str = "WORTH_C4B4_FAM_PHASE";
pub(super) const ENV_HOME: &str = "WORTH_C4B4_FAM_HOME";
pub(super) const ENV_MARKERS: &str = "WORTH_C4B4_FAM_MARKERS";
pub(super) const ENV_BUILD: &str = "WORTH_C4B4_FAM_BUILD";
pub(super) const ENV_CUT: &str = "WORTH_C4B4_FAM_CUT";
pub(super) const ENV_EXTRA: &str = "WORTH_C4B4_FAM_EXTRA";

pub(self) const REVERSE_KEY: &str = "c4b4-reverse-label";
pub(self) const DISPATCH_KEY: u64 = 0xC4B4_D150;
pub(self) const DISPATCH_BAL: u64 = 777;
pub(super) const KEY_WF_DONE: u64 = 6013;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FamilyKind {
    WorkflowAdoption,
    Correction,
    Dispatch,
}

impl FamilyKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::WorkflowAdoption => "workflow+adoption",
            Self::Correction => "correction",
            Self::Dispatch => "dispatch",
        }
    }

    fn parse(s: &str) -> Self {
        match s {
            "workflow+adoption" => Self::WorkflowAdoption,
            "correction" => Self::Correction,
            "dispatch" => Self::Dispatch,
            other => panic!("unknown family {other}"),
        }
    }
}

pub(super) fn run_reopen_phase(
    family: FamilyKind,
    phase: &str,
    home: &Path,
    build: &str,
    extra: &Path,
    ctx: &StepCtx,
) {
    let markers = tempfile::tempdir().unwrap();
    let fam_s = PathBuf::from(family.as_str());
    let phase_s = PathBuf::from(phase);
    let park_kill = phase.ends_with("_discard_kill");
    if park_kill {
        let ready = markers.path().join("ready");
        let end = crate::killed_child::run_and_kill_with_ctx(
            REOPEN_CHILD,
            &[
                (ENV_FAMILY, &fam_s),
                (ENV_PHASE, &phase_s),
                (ENV_HOME, home),
                (ENV_MARKERS, markers.path()),
                (ENV_BUILD, Path::new(build)),
                ("WORTH_ADV_WF_BUILD", Path::new(build)),
                (ENV_EXTRA, extra),
            ],
            &ready,
            &markers.path().join("finished"),
            &ctx.to_string(),
        );
        assert_eq!(
            end,
            crate::killed_child::ChildEnd::KilledAtPark,
            "{ctx}: phase {phase} must park & be OS killed"
        );
        return;
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", REOPEN_CHILD, "--nocapture"])
        .env(ENV_FAMILY, &fam_s)
        .env(ENV_PHASE, &phase_s)
        .env(ENV_HOME, home)
        .env(ENV_MARKERS, markers.path())
        .env(ENV_BUILD, build)
        .env("WORTH_ADV_WF_BUILD", build)
        .env(ENV_EXTRA, extra)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{ctx}: child phase {phase} failed ({:?})\n{}\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn c4b4_family_kill_child() {
    let Some(fam_s) = std::env::var_os(ENV_FAMILY).and_then(|s| s.into_string().ok()) else {
        return;
    };
    let family = FamilyKind::parse(&fam_s);
    let phase = std::env::var(ENV_PHASE).unwrap_or_default();
    let loc = PathBuf::from(std::env::var_os(ENV_HOME).unwrap());
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let build = std::env::var(ENV_BUILD).unwrap();
    let nth: u64 = std::env::var(ENV_CUT).unwrap().parse().unwrap();
    let extra = PathBuf::from(std::env::var_os(ENV_EXTRA).unwrap());
    let done = Arc::new(AtomicBool::new(false));
    let (placement, activation, watcher) =
        sync_barrier_placement(nth, Some(markers.join("ready")), Some(done.clone()));
    match (family, phase.as_str()) {
        (FamilyKind::WorkflowAdoption, "kill_wf") => {
            let lease = checkpoint_capture::lease(256 << 20);
            let raw = open_raw_wf_home(
                &Location::Directory(loc),
                &build,
                Some(placement),
                ExecutionRequest::leased(&lease),
            );
            let (h, _auth) = bind_wf_home(raw);
            let orig = discover_original_start(&h);
            activation.arm().unwrap();
            let _ = run_correctable_step(&h, &orig);
        }
        (FamilyKind::WorkflowAdoption, "kill_adopt") => {
            let lease = checkpoint_capture::lease(256 << 20);
            let h = open_raw_wf_home(
                &Location::Directory(loc),
                &build,
                Some(placement),
                ExecutionRequest::leased(&lease),
            );
            activation.arm().unwrap();
            adopt_p1(&h);
        }
        (FamilyKind::Correction, _) => {
            let lease = checkpoint_capture::lease(256 << 20);
            let coords: WorthQueryCorrectionSourceCoordinates =
                serde_json::from_slice(&std::fs::read(&extra).unwrap()).unwrap();
            let h = open_label_home_with_build(
                Location::Directory(loc),
                &build,
                Some(placement),
                ExecutionRequest::leased(&lease),
            );
            activation.arm().unwrap();
            let _ = reverse_attempt(&h, &coords, REVERSE_KEY);
        }
        (FamilyKind::Dispatch, _) => {
            let req = opening::serial();
            let h = open_dispatch(
                Location::Directory(loc),
                &build,
                false,
                Some(placement),
                Some(extra),
                &ExecutionRequest::serial(&req),
            );
            activation.arm().unwrap();
            let _ = notify_dispatch(&h, DISPATCH_KEY, DISPATCH_BAL);
        }
        other => panic!("unexpected kill child spec: {other:?}"),
    }
    done.store(true, Ordering::SeqCst);
    if let Some(w) = watcher {
        let _ = w.join();
    }
    let actual = activation.matching_operation_count();
    crate::killed_child::announce(
        &markers.join("finished"),
        format!("missed:actual={actual}").as_bytes(),
    );
}

#[test]
fn c4b4_family_reopen_child() {
    let Some(fam_s) = std::env::var_os(ENV_FAMILY).and_then(|s| s.into_string().ok()) else {
        return;
    };
    let family = FamilyKind::parse(&fam_s);
    let phase = std::env::var(ENV_PHASE).unwrap();
    let loc = PathBuf::from(std::env::var_os(ENV_HOME).unwrap());
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let build = std::env::var(ENV_BUILD).unwrap();
    let extra = PathBuf::from(std::env::var_os(ENV_EXTRA).unwrap());
    match family {
        FamilyKind::WorkflowAdoption => run_wf_adopt_phase(&phase, loc, &markers, &build),
        FamilyKind::Correction => run_correction_phase(&phase, loc, &markers, &build, &extra),
        FamilyKind::Dispatch => run_dispatch_phase(&phase, loc, &markers, &build, &extra),
    }
}
