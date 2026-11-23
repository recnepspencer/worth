//! C4b.1: Real OS kills at every durable `SynchronizeFileState` I/O barrier of
//! a mixed Native + Signal + Bridge + World publication, verifying fresh-process
//! reopen, custody continuation, admitted-key retry, and MemoryOnly parity.
mod fixture;
pub(super) use fixture::*;

use super::super::opening;
use super::{
    assert_all_reopen_phases_killed, assert_barrier_killed, run_parallel_calibrated_barriers,
    sync_barrier_placement, ReopenPhase, RestorableHome, BUILD_PRIMARY,
};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    primary_graph::WorthQueryApplicationCommitOutcome as CommitOutcome,
    product::WorthQueryProductBranch, WorthQueryApplicationHomeLocation as Location,
};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::ContactCounters;
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome;
use worth_store::physical_runtime::certification::MediaOperationRole;

const CHILD: &str =
    "process::application::host_oracle::adversarial::signal_bridge_io_kills::signal_bridge_io_kill_child";
const ENV_HOME: &str = "WORTH_ADV_SIGNAL_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_SIGNAL_MARKERS";
const ENV_ORDINAL: &str = "WORTH_ADV_SIGNAL_ORDINAL";

fn prepare_signal_home_dir(loc: &Path, execution: &ExecutionRequest<'_, '_>) {
    let home = open_signal_home(
        SignalOpen { location: Location::Directory(loc.to_path_buf()), build: BUILD_PRIMARY, initialized: Arc::new(AtomicUsize::new(0)), contacts: ContactCounters::default(), placement: None },
        execution,
    );
    let main = home.runtime().current_world();
    publish_definition(&home, main);
    home.close(execution).unwrap().reclaim();
}

#[test]
fn signal_bridge_mixed_publication_io_boundary_kill_sweep() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let (expected_before, expected_after, expected_key, expected_def_id) =
        build_signal_reference(&execution);
    let outcomes = run_parallel_calibrated_barriers(
        "Signal/Bridge",
        || {
            let w_req = opening::serial();
            let home = RestorableHome::prepare(|loc| prepare_signal_home_dir(loc, &ExecutionRequest::serial(&w_req)));
            let (m1, m2) = (measure_signal_syncs(&home), measure_signal_syncs(&home));
            assert_eq!(m1, m2, "stabilized Signal/Bridge layout must calibrate identically");
            (home, m1)
        },
        |worker_home, ordinal, worker_total| {
            let w_req = opening::serial();
            worker_home.restore();
            kill_signal_exact(worker_home.location(), ordinal, worker_total);
            verify_signal_reopen(
                worker_home.location(),
                (&expected_before, &expected_after, expected_key, expected_def_id),
                ordinal,
                &ExecutionRequest::serial(&w_req),
            )
        },
    );
    assert_all_reopen_phases_killed("Signal/Bridge", &outcomes);
}

pub(super) fn build_signal_reference(execution: &ExecutionRequest<'_, '_>) -> (SignalSnapshot, SignalSnapshot, [u8; 32], [u8; 32]) {
    let target_key = admitted_signal_key_identity(2);
    let ref_home = open_signal_home(
        SignalOpen { location: Location::MemoryOnly, build: BUILD_PRIMARY, initialized: Arc::new(AtomicUsize::new(0)), contacts: ContactCounters::default(), placement: None },
        execution,
    );
    let ref_main = ref_home.runtime().current_world();
    publish_definition(&ref_home, ref_main);
    let expected_before = observe_signal(&ref_home);
    let target_receipt = commit_mixed(&ref_home, ref_main, 2).require_committed().unwrap();
    let expected_after = observe_signal(&ref_home);
    assert_live_receipt_bound(&target_receipt, &expected_after, &expected_after, target_key, "MemoryOnly");
    drop(target_receipt);
    assert_ne!(expected_before.intent, expected_after.intent);
    assert_ne!(expected_before.native_basis, expected_after.native_basis);
    assert_eq!(expected_after.details.signal_generation, expected_before.details.signal_generation + 1);
    assert_eq!(expected_after.details.world_reference_generation, expected_before.details.world_reference_generation + 1);
    assert_eq!(expected_after.world_commit_ordinal, expected_before.world_commit_ordinal + 1);
    let def_id = verify_recovered_bridge_definition(&ref_home, ref_main);
    assert_eq!(observe_signal(&ref_home), expected_after, "MemoryOnly: Bridge def check must not mutate snapshot");
    assert_bridge_publication_healthy(&ref_home, ref_main, expected_after.world_commit_ordinal, expected_after.details.signal_generation);
    drop(ref_home.close(execution).err().unwrap().resume().unwrap());
    (expected_before, expected_after, target_key, def_id)
}

pub(super) fn measure_signal_syncs(saved: &RestorableHome) -> u64 {
    saved.restore();
    let markers = tempfile::tempdir().unwrap();
    let env = [(ENV_HOME, saved.location()), (ENV_MARKERS, markers.path()), (ENV_ORDINAL, Path::new("count"))];
    let end = crate::killed_child::run_and_kill(CHILD, &env, &markers.path().join("ready"), &markers.path().join("finished"));
    assert_eq!(end, crate::killed_child::ChildEnd::Finished);
    std::fs::read_to_string(markers.path().join("finished")).unwrap().trim().parse().unwrap()
}

pub(super) fn kill_signal_exact(location: &Path, ordinal: u64, total_syncs: u64) {
    let markers = tempfile::tempdir().unwrap();
    let ord_path = PathBuf::from(ordinal.to_string());
    let ready = markers.path().join("ready");
    let env = [(ENV_HOME, location), (ENV_MARKERS, markers.path()), (ENV_ORDINAL, &ord_path)];
    let end = crate::killed_child::run_and_kill(CHILD, &env, &ready, &markers.path().join("finished"));
    assert_barrier_killed(end, &ready, MediaOperationRole::SynchronizeFileState, ordinal, &format!("Signal/Bridge sync#{ordinal}/{total_syncs}"));
}

fn verify_signal_reopen(
    location: &Path,
    expected: (&SignalSnapshot, &SignalSnapshot, [u8; 32], [u8; 32]),
    ordinal: u64,
    execution: &ExecutionRequest<'_, '_>,
) -> ReopenPhase {
    let (expected_before, expected_after, expected_key, expected_def_id) = expected;
    let init = Arc::new(AtomicUsize::new(0));
    let contacts = ContactCounters::default();
    let home = open_signal_home(
        SignalOpen { location: Location::Directory(location.to_path_buf()), build: BUILD_PRIMARY, initialized: init.clone(), contacts: contacts.clone(), placement: None },
        execution,
    );
    assert_eq!(init.load(Ordering::SeqCst), 0, "sync#{ordinal}: no re-init");
    assert_eq!(contacts.snapshot(), (0, 0, 0, 0), "sync#{ordinal}: no reopen provider calls");
    let main = home.runtime().current_world();
    let handles: Vec<_> = home
        .runtime().product_publication_recovery_page(None, std::num::NonZeroUsize::new(16).unwrap()).unwrap()
        .rows().iter().map(|r| r.handle().clone()).collect();
    let state = observe_signal(&home);
    assert_eq!(
        (state.details.installed_conditional_definition_count, state.details.installed_conditional_target_ref_count),
        (1, 1),
        "sync#{ordinal}: cold reopen reconstitutes exact 1 active Bridge definition"
    );
    let phase = if !handles.is_empty() {
        verify_signal_custody((&home, main), &handles, (&state, expected_before, expected_after), (ordinal, expected_key, &contacts, execution));
        ReopenPhase::Custody
    } else if state.intent == expected_before.intent {
        assert_signal_predecessor(&state, expected_before, ordinal);
        let live = commit_mixed(&home, main, 2).require_committed().expect("PreOp commit");
        let after = observe_signal(&home);
        assert_signal_advanced(&after, expected_before, expected_after, (ordinal, true));
        assert_live_receipt_bound(&live, &after, expected_after, expected_key, &format!("sync#{ordinal}/preop"));
        drop(live);
        match commit_mixed(&home, main, 2) {
            CommitOutcome::Committed(rec) => assert_live_receipt_bound(&rec, &after, expected_after, expected_key, &format!("sync#{ordinal}/dup-live")),
            CommitOutcome::RecoveredCommitted(rec) => assert_recovered_receipt_bound(&rec, &after, expected_after, expected_key, &format!("sync#{ordinal}/dup-rec")),
            other => panic!("sync#{ordinal}: duplicate retry unexpected: {other:?}"),
        }
        assert_eq!(observe_signal(&home), after, "sync#{ordinal}: duplicate retry must not advance");
        ReopenPhase::PreOp
    } else {
        assert_signal_advanced(&state, expected_before, expected_after, (ordinal, false));
        let CommitOutcome::RecoveredCommitted(rec) = commit_mixed(&home, main, 2) else {
            panic!("sync#{ordinal}: committed retry must return RecoveredCommitted");
        };
        assert_recovered_receipt_bound(&rec, &state, expected_after, expected_key, &format!("sync#{ordinal}/committed"));
        assert_eq!(observe_signal(&home), state, "sync#{ordinal}: retry must not advance");
        assert_eq!(contacts.snapshot(), (0, 0, 0, 0), "sync#{ordinal}: zero replay on committed");
        ReopenPhase::Committed
    };
    let settled = observe_signal(&home);
    assert_eq!(settled.intent, expected_after.intent, "sync#{ordinal}: settled intent parity");
    assert_eq!(settled.native_basis, expected_after.native_basis, "sync#{ordinal}: settled native basis parity");
    assert_eq!(settled.details.signal_generation, expected_after.details.signal_generation, "sync#{ordinal}: signal_generation parity");
    assert_eq!(settled.details.world_reference_generation, expected_after.details.world_reference_generation, "sync#{ordinal}: world_reference_generation parity");
    assert_eq!(verify_recovered_bridge_definition(&home, main), expected_def_id, "sync#{ordinal}: recovered Bridge definition identity parity");
    assert_eq!(observe_signal(&home), settled, "sync#{ordinal}: verifying recovered Bridge def must not mutate snapshot");
    assert_bridge_publication_healthy(&home, main, settled.world_commit_ordinal, settled.details.signal_generation);
    home.close(execution).unwrap_or_else(|e| panic!("sync#{ordinal}: Signal close: {e:?}")).reclaim();
    phase
}

fn assert_signal_predecessor(actual: &SignalSnapshot, before: &SignalSnapshot, ordinal: u64) {
    assert_eq!(actual.intent, before.intent, "sync#{ordinal}: pre-op intent parity");
    assert_eq!(actual.world_commit_ordinal, before.world_commit_ordinal, "sync#{ordinal}: pre-op ordinal parity");
    assert_eq!(actual.native_basis, before.native_basis, "sync#{ordinal}: pre-op Native basis parity");
    assert_eq!(actual.details.signal_generation, before.details.signal_generation);
    assert_eq!(actual.details.world_reference_generation, before.details.world_reference_generation);
    assert_eq!(actual.details.owned_signal_active_node_count, before.details.owned_signal_active_node_count);
    assert_eq!(actual.details.head_signal_movement, before.details.head_signal_movement);
}

fn assert_signal_advanced(actual: &SignalSnapshot, before: &SignalSnapshot, after: &SignalSnapshot, opts: (u64, bool)) {
    let (ordinal, allow_released_gap) = opts;
    assert_eq!(actual.intent, after.intent, "sync#{ordinal}: intent parity");
    assert_eq!(actual.native_basis, after.native_basis, "sync#{ordinal}: stable Native basis parity");
    assert_eq!(actual.details.signal_generation, after.details.signal_generation, "sync#{ordinal}: signal_generation parity");
    assert_eq!(actual.details.world_reference_generation, after.details.world_reference_generation, "sync#{ordinal}: world_reference_generation parity");
    assert!(actual.details.installed_conditional_definition_count >= 1);
    assert_eq!(actual.details.installed_conditional_definition_count, actual.details.installed_conditional_target_ref_count);
    assert_eq!(actual.details.owned_signal_active_node_count, after.details.owned_signal_active_node_count);
    assert_eq!(actual.details.head_signal_movement, after.details.head_signal_movement);
    let expected_next = before.world_commit_ordinal + 1;
    if allow_released_gap {
        assert!(
            actual.world_commit_ordinal == expected_next || actual.world_commit_ordinal == expected_next + 1,
            "sync#{ordinal}: post-retry ordinal {} must be {expected_next} or {}", actual.world_commit_ordinal, expected_next + 1
        );
    } else {
        assert_eq!(actual.world_commit_ordinal, expected_next, "sync#{ordinal}: committed/continued ordinal must be exact predecessor + 1");
    }
}

fn verify_signal_custody(
    target: (&SignalHome, WorthQueryProductBranch),
    handles: &[worth_runtime_world::facade::ProductUnpublishedRecoveryHandle],
    states: (&SignalSnapshot, &SignalSnapshot, &SignalSnapshot),
    run: (u64, [u8; 32], &ContactCounters, &ExecutionRequest<'_, '_>),
) {
    let (home, main) = target;
    let (ordinal, expected_key, contacts, execution) = run;
    let (state, expected_before, expected_after) = states;
    assert_eq!(handles.len(), 1, "sync#{ordinal}: at most one unpublished custody");
    assert_signal_predecessor(state, expected_before, ordinal);
    assert!(matches!(commit_mixed(home, main, 2), CommitOutcome::ProductUnpublished(_)));
    let recovery = home.runtime().readmit_product_publication_recovery(&handles[0]).unwrap();
    let effect = recovery.inspect().unwrap();
    let custody_attempt = effect.attempt_identity().ordinal();
    let native = effect.native_semantic_result().map(|n| n.receipt());
    let (custody_commit, custody_ver) = (native.map(|r| r.commit_id.0), native.map(|r| r.version_id.as_u64()));
    drop(effect);
    loop {
        match recovery.continue_original_in_request(execution).unwrap() {
            WorldRecoveredContinuationOutcome::Pending(c) => drop(c),
            WorldRecoveredContinuationOutcome::Published { .. } => break,
            other => panic!("sync#{ordinal}: Signal continuation failed: {other:?}"),
        }
    }
    let continued = observe_signal(home);
    assert_signal_advanced(&continued, expected_before, expected_after, (ordinal, false));
    assert_eq!(contacts.snapshot(), (0, 0, 0, 0), "sync#{ordinal}: zero replay on continuation");
    let CommitOutcome::RecoveredCommitted(rec) = commit_mixed(home, main, 2) else {
        panic!("sync#{ordinal}: post-continuation retry must return RecoveredCommitted");
    };
    assert_eq!(rec.attempt_identity().ordinal(), custody_attempt);
    assert_eq!(Some(rec.commit_reference().commit_id.0), custody_commit);
    assert_eq!(Some(rec.commit_reference().version_id.as_u64()), custody_ver);
    assert_recovered_receipt_bound(&rec, &continued, expected_after, expected_key, &format!("sync#{ordinal}/custody"));
    assert_eq!(observe_signal(home), continued, "sync#{ordinal}: retry must not advance");
}

#[test]
fn signal_bridge_io_kill_child() {
    let Some(location) = std::env::var_os(ENV_HOME).map(PathBuf::from) else { return; };
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let ord_str = std::env::var(ENV_ORDINAL).unwrap();
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let done = Arc::new(AtomicBool::new(false));
    let (placement, activation, watcher) = if ord_str == "count" {
        sync_barrier_placement(u64::MAX, None, None)
    } else {
        sync_barrier_placement(ord_str.parse().unwrap(), Some(markers.join("ready")), Some(done.clone()))
    };
    let home = open_signal_home(
        SignalOpen { location: Location::Directory(location), build: BUILD_PRIMARY, initialized: Arc::new(AtomicUsize::new(0)), contacts: ContactCounters::default(), placement: Some(placement) },
        &execution,
    );
    activation.arm().unwrap();
    let main = home.runtime().current_world();
    let _ = commit_mixed(&home, main, 2);
    done.store(true, Ordering::SeqCst);
    if let Some(handle) = watcher {
        let _ = handle.join();
        let actual = activation.matching_operation_count();
        crate::killed_child::announce(&markers.join("finished"), format!("missed:actual={actual}").as_bytes());
        return;
    }
    let count = activation.matching_operation_count();
    home.close(&execution).unwrap().reclaim();
    crate::killed_child::announce(&markers.join("finished"), count.to_string().as_bytes());
}
