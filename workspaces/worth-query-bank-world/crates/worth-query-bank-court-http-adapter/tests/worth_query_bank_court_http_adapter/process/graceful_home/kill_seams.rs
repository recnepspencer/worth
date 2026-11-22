//! C3.5 real process-kill courts against the separate-process Bank external rail:
//! all six in-flight dispatch fault cases (attempt durable, observation not yet
//! recorded), inbound callback & duplicate deduplication across kills, and R14
//! rail restart with lost state.
mod dispatch_faults;

use super::{fixture::*, operation::FAMILY, rejection::Reject};
use bank_external_rail::RailCorrelation;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{atomic::AtomicUsize, Arc};
use std::time::{Duration, Instant};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

const ENV_MODE: &str = "WORTH_BANK_KILL_MODE";
const ENV_HOME: &str = "WORTH_BANK_KILL_HOME";
const ENV_RAIL: &str = "WORTH_BANK_KILL_RAIL";
const ENV_CTRL: &str = "WORTH_BANK_KILL_CTRL";
const ENV_KEY: &str = "WORTH_BANK_KILL_KEY";
const ENV_READY: &str = "WORTH_BANK_KILL_READY";
const ENV_CB: &str = "WORTH_BANK_KILL_CB";
const CHILD_TEST: &str = "process::graceful_home::kill_seams::bank_kill_seam_child";

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct ChildDispatchInfo {
    pub(super) correlation: [u8; 32],
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ChildInboundInfo {
    message_identity: [u8; 32],
    original_ordinal: u64,
    completion_ordinal: u64,
}

pub(super) fn rail_correlation(bytes: &[u8; 32]) -> RailCorrelation {
    RailCorrelation::new(FAMILY, bytes.to_vec())
}

struct ParkedChildGuard {
    child: Option<std::process::Child>,
    marker_dir: PathBuf,
}

impl ParkedChildGuard {
    fn child_mut(&mut self) -> &mut std::process::Child {
        self.child.as_mut().expect("bank kill child still owned")
    }

    fn kill_and_reap(mut self) -> std::process::ExitStatus {
        let mut child = self.child.take().expect("bank kill child still owned");
        child.kill().expect("kill parked bank child without close");
        child.wait().expect("reap killed bank child")
    }
}

impl Drop for ParkedChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = std::fs::remove_dir_all(&self.marker_dir);
    }
}

fn write_ready_atomic(ready: &Path, payload: &[u8]) {
    let tmp = ready.with_extension("json.tmp");
    std::fs::write(&tmp, payload).expect("write temporary kill readiness marker");
    std::fs::rename(&tmp, ready).expect("atomically publish kill readiness marker");
}

pub(super) fn spawn_and_kill_child(
    world: &World,
    mode: &str,
    key: u64,
    callback_file: Option<&Path>,
) -> Vec<u8> {
    let marker_dir = std::env::temp_dir().join(format!(
        "worth-bank-kill-{}-{}-{}",
        std::process::id(),
        key,
        mode.replace(':', "_")
    ));
    let _ = std::fs::create_dir_all(&marker_dir);
    let ready = marker_dir.join("ready.json");
    let _ = std::fs::remove_file(&ready);
    let _ = std::fs::remove_file(ready.with_extension("json.tmp"));
    let mut cmd = Command::new(std::env::current_exe().unwrap());
    cmd.args(["--exact", CHILD_TEST, "--nocapture"])
        .env(ENV_MODE, mode)
        .env(ENV_HOME, &world.location)
        .env(ENV_RAIL, world.rail.local_addr().to_string())
        .env(ENV_CTRL, world.rail.test_control_addr().to_string())
        .env(ENV_KEY, key.to_string())
        .env(ENV_READY, &ready)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    if let Some(cb) = callback_file {
        cmd.env(ENV_CB, cb);
    }
    let mut guard = ParkedChildGuard {
        child: Some(cmd.spawn().expect("bank kill child starts")),
        marker_dir,
    };
    let deadline = Instant::now() + Duration::from_secs(60);
    while !ready.is_file() {
        if let Some(status) = guard.child_mut().try_wait().unwrap() {
            panic!("bank kill child ({mode}) exited early with {status}");
        }
        assert!(
            Instant::now() < deadline,
            "bank kill child ({mode}) timed out before parking"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let payload = std::fs::read(&ready).unwrap();
    let status = guard.kill_and_reap();
    assert!(!status.success(), "killed bank child must not exit cleanly");
    payload
}

pub(super) fn assert_unresolved_unknown(opened: &Opened, correlation: &[u8; 32], label: &str) {
    let custody = opened
        .home
        .runtime()
        .runtime()
        .durable_dispatch_custody(correlation)
        .unwrap_or_else(|| panic!("{label}: custody must exist after reopen"));
    assert_eq!(
        custody.observation(),
        Some(QueryDispatchRecordedObservation::Unresolved(
            QueryDispatchUnresolvedCause::UnknownProviderOutcome
        )),
        "{label}: killed in-flight attempt must reopen as Unresolved(UnknownProviderOutcome)"
    );
}

#[test]
fn authentic_callback_completes_killed_dispatch_and_duplicate_delivery_across_kill_deduplicates() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = checkpoint_capture::ExecutionRequest::leased(&lease);
    let mut world = World::new();
    let key = 9511_u64;
    let info: ChildDispatchInfo = serde_json::from_slice(&spawn_and_kill_child(
        &world,
        "dispatch:lost-response",
        key,
        None,
    ))
    .unwrap();
    let body = world.callback.receive(&world.capture_loop.blocking);
    let cb_file = std::env::temp_dir().join(format!("worth-bank-cb-{}.bin", std::process::id()));
    std::fs::write(&cb_file, &body).unwrap();

    // Child 2 reopens after dispatch kill, delivers authentic callback, settles
    // completion, then is OS-killed without closing.
    let inbound: ChildInboundInfo = serde_json::from_slice(&spawn_and_kill_child(
        &world,
        "inbound:deliver-and-kill",
        key,
        Some(&cb_file),
    ))
    .unwrap();
    let _ = std::fs::remove_file(&cb_file);

    // Fresh reopen delivers the exact same signed callback bytes across the kill:
    // deduplicated as AlreadyCompleted with identical durable terminal coordinates.
    let reopened = world.open(execution, false);
    let dup = reopened
        .home
        .runtime()
        .runtime()
        .receive_inbound_occurrence(&reopened.verifier, &body, &request_scope())
        .unwrap();
    assert_eq!(dup.posture(), WorthQueryInboundReceiptPosture::AlreadyCompleted);
    assert_eq!(dup.message_identity(), &inbound.message_identity);
    drop(dup);
    reopened.maintain();
    let terminal = reopened
        .home
        .runtime()
        .runtime()
        .observe_inbound_terminal(&reopened.verifier, info.correlation)
        .unwrap();
    assert_eq!(terminal.original_world_commit().ordinal(), inbound.original_ordinal);
    assert_eq!(terminal.completion_world_commit().ordinal(), inbound.completion_ordinal);
    assert!(reopened.transport.production_dispatches().is_empty());
    assert_eq!(reopened.transport.admission_count(), 1);
    assert_eq!(reopened.transport.completed_effect_count(), 1);
    reopened.close(&execution);
    world.finish();
}

#[test]
fn r14_kill_then_rail_restart_with_lost_state_never_claims_completion() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = checkpoint_capture::ExecutionRequest::leased(&lease);
    let mut world = World::new();
    let key = 9521_u64;
    let info: ChildDispatchInfo =
        serde_json::from_slice(&spawn_and_kill_child(&world, "dispatch:succeed", key, None))
            .unwrap();
    let rc = rail_correlation(&info.correlation);
    // Prove the original rail completed this exact correlation before losing state.
    assert_eq!(world.capture_loop.ledger_status(&rc), LedgerStatus::Completed);
    let prior_notice = world
        .capture_loop
        .completed_notice(&rc)
        .expect("old rail retains completed notice before restart");
    assert_eq!(prior_notice.estate(), notice(key).estate().get());
    assert_eq!(prior_notice.notice(), notice(key).notice().get());
    assert_eq!(prior_notice.subject(), notice(key).subject().get());
    assert_eq!(world.capture_loop.admission_count(), 1);
    assert_eq!(world.capture_loop.completed_effect_count(), 1);
    // Restart the external rail with all state lost (R14).
    world.restart_rail_with_lost_state();

    let reopened = world.open(execution, false);
    assert_unresolved_unknown(&reopened, &info.correlation, "r14");
    assert!(reopened
        .home
        .runtime()
        .runtime()
        .observe_inbound_terminal(&reopened.verifier, info.correlation)
        .is_none());
    let handle = reopened.recover(notice(key), key);
    let disposition = reopened.admitted(notice(key), key, |phase, admission, _, _| {
        let rt = reopened.home.runtime().runtime();
        let err = rt.settle_recorded_recovery_completion(phase, handle, admission).unwrap_err();
        assert_eq!(err.kind(), WorthQueryRecoveryHandleDenialKind::UnresolvedExternalPosture);
        let retained = err.into_parts().1.expect("claim retained");
        let auth = rt.admit_recovery_effect_authority(&retained, admission).unwrap();
        let reconcile = reconcile_recovery_handle(phase, retained, &auth).unwrap();
        reopened.transport.reconcile_admitted_effect(&reconcile)
    });
    assert_eq!(
        disposition,
        BankRailReconciliationDisposition::UnresolvedNoRecord {
            correlation: rc.clone(),
        }
    );
    assert_eq!(
        reopened.admitted(notice(key), key, |phase, admission, binding, _| {
            reopened
                .home
                .runtime()
                .runtime()
                .resolve_admitted_recovery_claim_status(phase, admission, binding)
                .unwrap()
                .unwrap()
        }),
        WorthQueryRecoveryClaimStatus::Consumed
    );
    assert_unresolved_unknown(&reopened, &info.correlation, "r14-after-reconcile");
    assert!(reopened.transport.production_dispatches().is_empty());
    assert_eq!(reopened.transport.admission_count(), 0);
    assert_eq!(reopened.transport.completed_effect_count(), 0);
    reopened.close(&execution);

    // Second reopen still never claims completion.
    let again = world.open(execution, false);
    assert_unresolved_unknown(&again, &info.correlation, "r14-again");
    again.close(&execution);
    world.finish();
}

#[test]
fn bank_kill_seam_child() {
    let Ok(mode) = std::env::var(ENV_MODE) else { return; };
    worth_execution::run_with_execution_stack(move || {
        let location = PathBuf::from(std::env::var_os(ENV_HOME).unwrap());
        let rail_addr = std::env::var(ENV_RAIL).unwrap().parse().unwrap();
        let ctrl_addr = std::env::var(ENV_CTRL).unwrap().parse().unwrap();
        let key: u64 = std::env::var(ENV_KEY).unwrap().parse().unwrap();
        let ready = PathBuf::from(std::env::var_os(ENV_READY).unwrap());
        let lease = checkpoint_capture::lease(256 << 20);
        let execution = checkpoint_capture::ExecutionRequest::leased(&lease);
        let handlers = Arc::new(AtomicUsize::new(0));

        if mode == "inbound:deliver-and-kill" {
            let body = std::fs::read(std::env::var_os(ENV_CB).unwrap()).unwrap();
            let opened = open_at(location, rail_addr, ctrl_addr, handlers, execution, false);
            let rx = opened
                .home
                .runtime()
                .runtime()
                .receive_inbound_occurrence(&opened.verifier, &body, &request_scope())
                .unwrap();
            assert_eq!(rx.posture(), WorthQueryInboundReceiptPosture::Performed);
            let msg = *rx.message_identity();
            drop(rx);
            opened.maintain();
            let handle = opened.recover(notice(key), key);
            let corr = *handle.binding().dispatch_outbox().unwrap().correlation().bytes();
            let terminal = opened
                .home
                .runtime()
                .runtime()
                .observe_inbound_terminal(&opened.verifier, corr)
                .unwrap();
            let info = ChildInboundInfo {
                message_identity: msg,
                original_ordinal: terminal.original_world_commit().ordinal(),
                completion_ordinal: terminal.completion_world_commit().ordinal(),
            };
            opened.admitted(notice(key), key, |phase, admission, _, _| {
                opened
                    .home
                    .runtime()
                    .runtime()
                    .settle_recorded_recovery_completion(phase, handle, admission)
                    .unwrap();
            });
            write_ready_atomic(&ready, &serde_json::to_vec(&info).unwrap());
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
        }

        let opened = open_at(location, rail_addr, ctrl_addr, handlers, execution, true);
        let (script, timeout, is_reject) = match mode.as_str() {
            "dispatch:succeed" => (FaultScript::Succeed, Duration::from_secs(5), false),
            "dispatch:lost-response" => (FaultScript::CommitThenLoseResponse, Duration::from_secs(5), false),
            "dispatch:dup-ack" => (FaultScript::DuplicateAcknowledgement, Duration::from_secs(5), false),
            "dispatch:timeout" => (FaultScript::CompleteAfterDelay { delay_millis: 250 }, Duration::from_millis(10), false),
            "dispatch:disappear" => (FaultScript::DisappearMidDispatch, Duration::from_secs(5), false),
            "dispatch:reject" => (FaultScript::Succeed, Duration::from_secs(5), true),
            other => panic!("unknown bank kill mode: {other}"),
        };
        opened.transport.under(script, timeout);
        let transport_ref = Arc::clone(&opened.transport);
        let ready_path = ready.clone();
        opened.transport.after_next_dispatch(move || {
            let corr_vec = transport_ref.attempts()[0].token().to_vec();
            let correlation: [u8; 32] = corr_vec.try_into().expect("32-byte correlation");
            let info = ChildDispatchInfo { correlation };
            write_ready_atomic(&ready_path, &serde_json::to_vec(&info).unwrap());
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
        });
        if is_reject {
            let scope = request_scope();
            let rt = opened.home.runtime();
            let principal = identity::inspection_authentication::authenticate(rt.runtime().installed_schema(), &scope);
            let _ = rt
                .request(&principal, &scope)
                .advanced()
                .mutate(Reject(action(notice(key))))
                .without_source()
                .idempotency(&key)
                .execute_in_program(rt, ExecutionAllocationPolicy::SystemAllocation);
        } else {
            let _ = opened.emit(notice(key), key);
        }
        unreachable!("bank child must be killed while parked before observation recording");
    });
}
