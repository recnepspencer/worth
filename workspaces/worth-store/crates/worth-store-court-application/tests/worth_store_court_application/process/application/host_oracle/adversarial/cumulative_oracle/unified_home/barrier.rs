//! Mid-operation `SynchronizeFileState` barrier kill child for the C4b.4 Single-Home
//! Unified 4-Family Oracle.
use super::super::super::{assert_barrier_killed, sync_barrier_placement};
use super::ops::{
    bind_unified_wf, exec_set_retention, exec_wf_correctable, open_unified_raw,
    run_unified_adopt_p1,
};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::WorthQueryApplicationHomeLocation as Location;
use worth_query_testkit_host::facade::host::checkpoint_capture;
use worth_store::physical_runtime::certification::MediaOperationRole;

const BARRIER_CHILD: &str =
    "process::application::host_oracle::adversarial::cumulative_oracle::unified_home::barrier::c4b4_unified_barrier_child";
const ENV_HOME: &str = "WORTH_C4B4_UNIB_HOME";
const ENV_CH: &str = "WORTH_C4B4_UNIB_CH";
const ENV_BUILD: &str = "WORTH_C4B4_UNIB_BUILD";
const ENV_BSPEC: &str = "WORTH_C4B4_UNIB_BSPEC";
const ENV_COORDS: &str = "WORTH_C4B4_UNIB_COORDS";

pub(super) fn kill_unified_barrier(
    home: &Path,
    op_spec: &str,
    build: &str,
    nth: u64,
    coords_file: &Path,
    ctx: &str,
) {
    let markers = tempfile::tempdir().unwrap();
    let spec = PathBuf::from(format!("{op_spec}|{nth}"));
    let ready = markers.path().join("ready");
    let end = crate::killed_child::run_and_kill_with_ctx(
        BARRIER_CHILD,
        &[
            (ENV_HOME, home),
            (ENV_CH, markers.path()),
            (ENV_BUILD, Path::new(build)),
            (ENV_BSPEC, &spec),
            (ENV_COORDS, coords_file),
        ],
        &ready,
        &markers.path().join("finished"),
        ctx,
    );
    assert_barrier_killed(
        end,
        &ready,
        MediaOperationRole::SynchronizeFileState,
        nth,
        ctx,
    );
}

#[test]
fn c4b4_unified_barrier_child() {
    let Some(home_path) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os(ENV_CH).unwrap());
    let build = std::env::var(ENV_BUILD).unwrap();
    let spec = std::env::var(ENV_BSPEC).unwrap();
    let (op_spec, nth_s) = spec.rsplit_once('|').unwrap();
    let nth: u64 = nth_s.parse().unwrap();
    let lease = checkpoint_capture::lease(256 << 20);
    let exec = ExecutionRequest::leased(&lease);
    let done = Arc::new(AtomicBool::new(false));
    let (placement, activation, watcher) =
        sync_barrier_placement(nth, Some(markers.join("ready")), Some(done.clone()));
    let raw = open_unified_raw(
        &Location::Directory(home_path),
        &build,
        Some(placement),
        exec,
    );
    if op_spec == "wf" {
        let (wf, _auth) = bind_unified_wf(raw);
        activation.arm().unwrap();
        let _ = exec_wf_correctable(&wf);
    } else if let Some(rest) = op_spec.strip_prefix("set:") {
        let mut p = rest.split(':');
        let b: usize = p.next().unwrap().parse().unwrap();
        let d: u64 = p.next().unwrap().parse().unwrap();
        let k: u64 = p.next().unwrap().parse().unwrap();
        activation.arm().unwrap();
        let _ = exec_set_retention(raw.runtime(), b, d, k);
    } else if op_spec == "adopt" {
        activation.arm().unwrap();
        run_unified_adopt_p1(raw.runtime());
    } else {
        panic!("unknown unified barrier op {op_spec}");
    }
    done.store(true, Ordering::SeqCst);
    if let Some(h) = watcher {
        let _ = h.join();
    }
    let actual = activation.matching_operation_count();
    crate::killed_child::announce(
        &markers.join("finished"),
        format!("missed:actual={actual}").as_bytes(),
    );
}
