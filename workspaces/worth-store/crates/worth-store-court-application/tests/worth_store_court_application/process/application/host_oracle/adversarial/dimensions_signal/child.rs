//! Fresh-process child harness and strict live OS kill for C4b.3 Signal+Bridge.
use super::super::super::graceful::signal::derived_discard::host::checkpoint_and_discard_signal;
use super::super::super::opening;
use super::super::signal_bridge_io_kills::{
    assert_bridge_publication_healthy, commit_mixed, observe_signal, open_signal_home,
    verify_recovered_bridge_definition, SignalHome, SignalOpen,
};
use super::wire::{SignalCmd, SignalReply, WireCustody, WireReceipt, WireSnapshot};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    primary_graph::WorthQueryApplicationCommitOutcome as CommitOutcome,
    WorthQueryApplicationHomeCloseOutcome as CloseOutcome,
    WorthQueryApplicationHomeLocation as Location,
};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::ContactCounters;
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome;
use worth_store_application::ApplicationHomeCheckpointDenial;

const CHILD: &str =
    "process::application::host_oracle::adversarial::dimensions_signal::child::signal_dim_child";
const ENV_CHANNEL: &str = "WORTH_ADV_SIGDIM_CHANNEL";
const ENV_HOME: &str = "WORTH_ADV_SIGDIM_HOME";
const ENV_BUILD: &str = "WORTH_ADV_SIGDIM_BUILD";

pub(super) struct SignalSession {
    child: Child,
    channel: PathBuf,
}

impl SignalSession {
    pub(super) fn start(home: &Path, channel: &Path, build: &str) -> (Self, SignalReply) {
        std::fs::create_dir_all(channel).unwrap();
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--nocapture"])
            .env(ENV_CHANNEL, channel)
            .env(ENV_HOME, home)
            .env(ENV_BUILD, build)
            .stdout(Stdio::from(
                std::fs::File::create(channel.join("stdout.log")).unwrap(),
            ))
            .stderr(Stdio::from(
                std::fs::File::create(channel.join("stderr.log")).unwrap(),
            ))
            .spawn()
            .unwrap();
        let mut s = Self {
            child,
            channel: channel.to_path_buf(),
        };
        let open = s.recv();
        (s, open)
    }

    pub(super) fn send(&mut self, cmd: SignalCmd) -> SignalReply {
        crate::killed_child::announce(
            &self.channel.join("request"),
            &serde_json::to_vec(&cmd).unwrap(),
        );
        self.recv()
    }

    fn recv(&mut self) -> SignalReply {
        let deadline = Instant::now() + Duration::from_secs(180);
        let file = self.channel.join("response");
        loop {
            if file.is_file() {
                let bytes = std::fs::read(&file).unwrap();
                std::fs::remove_file(&file).unwrap();
                return serde_json::from_slice(&bytes).unwrap();
            }
            let ended = self.child.try_wait().unwrap();
            if ended.is_some() && file.is_file() {
                continue;
            }
            assert!(
                ended.is_none() && Instant::now() < deadline,
                "signal_dim_child stopped or timed out: {ended:?}\n{}\n{}",
                std::fs::read_to_string(self.channel.join("stdout.log")).unwrap_or_default(),
                std::fs::read_to_string(self.channel.join("stderr.log")).unwrap_or_default()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Explicitly terminate a STILL-LIVE child process with a real OS kill.
    pub(super) fn kill_live(&mut self) {
        let exited = self.child.try_wait().expect("poll child before OS kill");
        assert!(
            exited.is_none(),
            "signal_dim_child exited before parent-induced kill: {exited:?}"
        );
        self.child
            .kill()
            .expect("parent OS kill of live signal_dim_child must succeed");
        let status = self.child.wait().expect("reap killed signal_dim_child");
        assert!(
            !status.success(),
            "OS-killed signal_dim_child must not report success: {status:?}"
        );
    }

    pub(super) fn finish(&mut self) {
        assert!(
            self.child.wait().unwrap().success(),
            "signal_dim_child clean exit"
        );
    }
}

impl Drop for SignalSession {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

fn inspect_signal_custody(home: &SignalHome) -> Vec<WireCustody> {
    let page = home
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(16).unwrap())
        .unwrap();
    page.rows()
        .iter()
        .map(|row| {
            let id = row.handle().identity();
            let rec = home
                .runtime()
                .readmit_product_publication_recovery(row.handle())
                .unwrap();
            let effect = rec.inspect().unwrap();
            let native = effect.native_semantic_result().map(|n| n.receipt());
            WireCustody {
                owner_debug: format!(
                    "{:?}",
                    id.owner_identity()
                        .durable()
                        .expect("directory home custody has durable owner")
                ),
                custody_ordinal: id.ordinal(),
                attempt_ordinal: effect.attempt_identity().ordinal(),
                native_commit_id: native.map(|r| r.commit_id.0),
                native_version_id: native.map(|r| r.version_id.as_u64()),
                expected_commit: effect.expected_product().selected_commit().ordinal(),
                owner_effect_count: effect.owner_effect_count(),
            }
        })
        .collect()
}

fn inspect_signal_cadence(
    home: &SignalHome,
    execution: &ExecutionRequest<'_, '_>,
) -> BTreeMap<String, (u64, u64)> {
    home.inspect_checkpoint(execution)
        .unwrap()
        .journals
        .into_iter()
        .map(|j| (j.name.to_owned(), (j.covered, j.head)))
        .collect()
}

#[test]
fn signal_dim_child() {
    let Some(channel) = std::env::var_os(ENV_CHANNEL).map(PathBuf::from) else {
        return;
    };
    let location = PathBuf::from(std::env::var_os(ENV_HOME).unwrap());
    let build = std::env::var(ENV_BUILD).unwrap();
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let init = Arc::new(AtomicUsize::new(0));
    let contacts = ContactCounters::default();
    let mut home = Some(open_signal_home(
        SignalOpen {
            location: Location::Directory(location),
            build: &build,
            initialized: init.clone(),
            contacts: contacts.clone(),
            placement: None,
        },
        &execution,
    ));
    let send_reply = |home: Option<&SignalHome>,
                      commit_tag,
                      receipt,
                      definition_identity,
                      discard_gen,
                      close_gen,
                      closed| {
        let (snapshot, custody, cadence) = home.map_or((None, Vec::new(), BTreeMap::new()), |h| {
            (
                Some(WireSnapshot::from_snapshot(&observe_signal(h))),
                inspect_signal_custody(h),
                inspect_signal_cadence(h, &execution),
            )
        });
        let body = SignalReply {
            snapshot,
            custody,
            receipt,
            initialized: init.load(Ordering::SeqCst),
            contacts: contacts.snapshot(),
            cadence,
            commit_tag,
            definition_identity,
            discard_gen,
            close_gen,
            closed,
        };
        crate::killed_child::announce(
            &channel.join("response"),
            &serde_json::to_vec(&body).unwrap(),
        );
    };
    send_reply(home.as_ref(), None, None, None, None, None, false);
    loop {
        let req_path = channel.join("request");
        if !req_path.is_file() {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        let cmd: SignalCmd = serde_json::from_slice(&std::fs::read(&req_path).unwrap()).unwrap();
        std::fs::remove_file(req_path).unwrap();
        let (mut tag, mut receipt, mut def_id, mut dgen) = (None, None, None, None);
        match cmd {
            SignalCmd::CheckpointAndDiscard => {
                dgen = Some(checkpoint_and_discard_signal(home.as_ref().unwrap(), &execution).1);
            }
            SignalCmd::CommitMixed(ord) => {
                let h = home.as_ref().unwrap();
                let (t, r) = match commit_mixed(h, h.runtime().current_world(), ord) {
                    CommitOutcome::Committed(rec) => {
                        ("committed", Some(WireReceipt::from_live(&rec)))
                    }
                    CommitOutcome::RecoveredCommitted(rec) => (
                        "recovered_committed",
                        Some(WireReceipt::from_recovered(&rec)),
                    ),
                    CommitOutcome::ProductUnpublished(_) => ("product_unpublished", None),
                    other => panic!("unexpected Signal CommitOutcome: {other:?}"),
                };
                tag = Some(t.to_owned());
                receipt = r;
            }
            SignalCmd::ContinueCustody => {
                let h = home.as_ref().unwrap();
                let rows = h
                    .runtime()
                    .product_publication_recovery_page(
                        None,
                        std::num::NonZeroUsize::new(16).unwrap(),
                    )
                    .unwrap();
                let recovery = h
                    .runtime()
                    .readmit_product_publication_recovery(rows.rows()[0].handle())
                    .unwrap();
                loop {
                    match recovery.continue_original_in_request(&execution).unwrap() {
                        WorldRecoveredContinuationOutcome::Pending(c) => drop(c),
                        WorldRecoveredContinuationOutcome::Published { .. } => break,
                        other => panic!("Signal continuation failed: {other:?}"),
                    }
                }
            }
            SignalCmd::VerifyRecoveredBridge => {
                let h = home.as_ref().unwrap();
                def_id = Some(verify_recovered_bridge_definition(
                    h,
                    h.runtime().current_world(),
                ));
            }
            SignalCmd::VerifyBridgeHealthy {
                before_ord,
                before_sig_gen,
            } => {
                let h = home.as_ref().unwrap();
                assert_bridge_publication_healthy(
                    h,
                    h.runtime().current_world(),
                    before_ord,
                    before_sig_gen,
                );
            }
            SignalCmd::Close { expect_discarded } => {
                let closed = home.take().unwrap().close(&execution).unwrap();
                assert!(closed.is_closed());
                let cgen = match closed.outcome() {
                    CloseOutcome::Closed {
                        checkpoint: Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded),
                        ..
                    } if expect_discarded => None,
                    CloseOutcome::Closed {
                        checkpoint: Ok(cp), ..
                    } if !expect_discarded => Some(cp.generation()),
                    CloseOutcome::Closed { checkpoint, .. } => {
                        panic!(
                            "unexpected Signal close (expect_discarded={expect_discarded}): {checkpoint:?}"
                        )
                    }
                    CloseOutcome::CloseRefused { reason, .. } => {
                        panic!("unexpected Signal CloseRefused: {reason:?}")
                    }
                };
                closed.reclaim();
                send_reply(None, None, None, None, None, cgen, true);
                return;
            }
        }
        send_reply(home.as_ref(), tag, receipt, def_id, dgen, None, false);
    }
}
