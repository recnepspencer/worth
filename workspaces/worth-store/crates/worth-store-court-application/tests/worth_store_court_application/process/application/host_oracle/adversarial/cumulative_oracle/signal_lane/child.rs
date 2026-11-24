//! Fresh OS child process session & barrier-kill runner for C4b.4 cumulative
//! Signal + Bridge + Authored Product Branches lane.
use super::super::super::super::graceful::signal::derived_discard::host::checkpoint_and_discard_signal;
use super::super::super::super::opening;
use super::super::super::dimensions_signal::wire::{WireCustody, WireReceipt};
use super::super::super::signal_bridge_io_kills::{
    commit_mixed, open_signal_home, publish_definition,
    verify_recovered_bridge_definition_with_graphs, SignalHome, SignalOpen,
};
use super::super::super::sync_barrier_placement;
use super::ops::{
    fork_on_home, observe_multi_branch, sorted_branches, CumSignalCmd, CumSignalReply,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
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
    "process::application::host_oracle::adversarial::cumulative_oracle::signal_lane::child::c4b4_signal_cum_child";
pub(super) const BARRIER_CHILD: &str =
    "process::application::host_oracle::adversarial::cumulative_oracle::signal_lane::child::c4b4_signal_barrier_child";
pub(super) const ENV_HOME: &str = "WORTH_C4B4_SIG_HOME";
pub(super) const ENV_CH: &str = "WORTH_C4B4_SIG_CH";
pub(super) const ENV_BUILD: &str = "WORTH_C4B4_SIG_BUILD";
pub(super) const ENV_BSPEC: &str = "WORTH_C4B4_SIG_BSPEC";

pub(super) struct CumSignalSession {
    child: Child,
    channel: PathBuf,
}

impl CumSignalSession {
    pub(super) fn start(home: &Path, channel: &Path, build: &str) -> (Self, CumSignalReply) {
        std::fs::create_dir_all(channel).unwrap();
        let stdout = std::fs::File::create(channel.join("stdout.log")).unwrap();
        let stderr = std::fs::File::create(channel.join("stderr.log")).unwrap();
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--nocapture"])
            .env(ENV_HOME, home)
            .env(ENV_CH, channel)
            .env(ENV_BUILD, build)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .unwrap();
        let mut s = Self {
            child,
            channel: channel.to_path_buf(),
        };
        let open = s.recv();
        (s, open)
    }

    pub(super) fn send(&mut self, cmd: CumSignalCmd) -> CumSignalReply {
        crate::killed_child::announce(
            &self.channel.join("request"),
            &serde_json::to_vec(&cmd).unwrap(),
        );
        self.recv()
    }

    fn recv(&mut self) -> CumSignalReply {
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
                "c4b4_signal_cum_child stopped/timed out: {ended:?}\n{}\n{}",
                std::fs::read_to_string(self.channel.join("stdout.log")).unwrap_or_default(),
                std::fs::read_to_string(self.channel.join("stderr.log")).unwrap_or_default()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    pub(super) fn kill_live(&mut self, ctx: &str) {
        let exited = self
            .child
            .try_wait()
            .expect("poll sig child before OS kill");
        assert!(
            exited.is_none(),
            "{ctx}: c4b4_signal_cum_child exited before parent-induced kill: {exited:?}"
        );
        self.child.kill().expect("OS kill sig child");
        let status = self.child.wait().expect("reap killed sig child");
        crate::killed_child::assert_os_killed(status, ctx);
    }

    pub(super) fn finish(mut self) {
        let status = self.child.wait().unwrap();
        assert!(status.success(), "sig child exit: {status:?}");
    }
}

impl Drop for CumSignalSession {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

fn inspect_custody(home: &SignalHome) -> Vec<WireCustody> {
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
                owner_debug: format!("{:?}", id.owner_identity().durable().unwrap()),
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

fn inspect_cadence(
    home: &SignalHome,
    exec: &ExecutionRequest<'_, '_>,
) -> BTreeMap<String, (u64, u64)> {
    home.inspect_checkpoint(exec)
        .unwrap()
        .journals
        .into_iter()
        .map(|j| (j.name.to_owned(), (j.covered, j.head)))
        .collect()
}

#[test]
fn c4b4_signal_cum_child() {
    let Some(home_path) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let channel = PathBuf::from(std::env::var_os(ENV_CH).unwrap());
    let build = std::env::var(ENV_BUILD).unwrap();
    let req = opening::serial();
    let exec = ExecutionRequest::serial(&req);
    let init = Arc::new(AtomicUsize::new(0));
    let contacts = ContactCounters::default();
    let mut home = Some(open_signal_home(
        SignalOpen {
            location: Location::Directory(home_path),
            build: &build,
            initialized: init.clone(),
            contacts: contacts.clone(),
            placement: None,
        },
        &exec,
    ));
    let send = |h: Option<&SignalHome>, tag, rec, def_id, dgen, cgen, closed| {
        let (snap, custody, cadence) = h.map_or((None, Vec::new(), BTreeMap::new()), |s| {
            (
                Some(observe_multi_branch(s)),
                inspect_custody(s),
                inspect_cadence(s, &exec),
            )
        });
        let body = CumSignalReply {
            snap,
            custody,
            receipt: rec,
            initialized: init.load(Ordering::SeqCst),
            contacts: contacts.snapshot(),
            cadence,
            commit_tag: tag,
            definition_identity: def_id,
            discard_gen: dgen,
            close_gen: cgen,
            closed,
        };
        crate::killed_child::announce(
            &channel.join("response"),
            &serde_json::to_vec(&body).unwrap(),
        );
    };
    send(home.as_ref(), None, None, None, None, None, false);
    loop {
        let req_path = channel.join("request");
        if !req_path.is_file() {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        let cmd: CumSignalCmd = serde_json::from_slice(&std::fs::read(&req_path).unwrap()).unwrap();
        std::fs::remove_file(req_path).unwrap();
        let (mut tag, mut rec, mut def_id, mut dgen) = (None, None, None, None);
        match cmd {
            CumSignalCmd::ForkBranch => fork_on_home(home.as_ref().unwrap()),
            CumSignalCmd::CommitMixed {
                branch_idx,
                ordinal,
            } => {
                let h = home.as_ref().unwrap();
                let br = sorted_branches(h)[branch_idx];
                let (t, r) = match commit_mixed(h, br, ordinal) {
                    CommitOutcome::Committed(r) => ("committed", Some(WireReceipt::from_live(&r))),
                    CommitOutcome::RecoveredCommitted(r) => {
                        ("recovered_committed", Some(WireReceipt::from_recovered(&r)))
                    }
                    CommitOutcome::ProductUnpublished(_) => ("product_unpublished", None),
                    other => panic!("unexpected Signal outcome: {other:?}"),
                };
                tag = Some(t.to_owned());
                rec = r;
            }
            CumSignalCmd::PublishDef { branch_idx } => {
                let h = home.as_ref().unwrap();
                let br = sorted_branches(h)[branch_idx];
                publish_definition(h, br);
            }
            CumSignalCmd::ContinueCustody => {
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
                    match recovery.continue_original_in_request(&exec).unwrap() {
                        WorldRecoveredContinuationOutcome::Pending(c) => drop(c),
                        WorldRecoveredContinuationOutcome::Published { .. } => break,
                        other => panic!("Signal continuation failed: {other:?}"),
                    }
                }
            }
            CumSignalCmd::VerifyRecoveredBridge { branch_idx } => {
                let h = home.as_ref().unwrap();
                let branches = sorted_branches(h);
                def_id = Some(verify_recovered_bridge_definition_with_graphs(
                    h,
                    branches[branch_idx],
                    branches.len(),
                ));
            }
            CumSignalCmd::CheckpointAndDiscard => {
                dgen = Some(checkpoint_and_discard_signal(home.as_ref().unwrap(), &exec).1);
            }
            CumSignalCmd::Close { expect_discarded } => {
                let closed = home.take().unwrap().close(&exec).unwrap();
                assert!(closed.is_closed());
                let cgen = match closed.outcome() {
                    CloseOutcome::Closed {
                        checkpoint: Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded),
                        ..
                    } if expect_discarded => None,
                    CloseOutcome::Closed {
                        checkpoint: Ok(cp), ..
                    } if !expect_discarded => Some(cp.generation()),
                    _ => panic!("unexpected Signal close (expect_discarded={expect_discarded})"),
                };
                closed.reclaim();
                send(None, None, None, None, None, cgen, true);
                return;
            }
        }
        send(home.as_ref(), tag, rec, def_id, dgen, None, false);
    }
}

#[test]
fn c4b4_signal_barrier_child() {
    let Some(home_path) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os(ENV_CH).unwrap());
    let build = std::env::var(ENV_BUILD).unwrap();
    let spec = std::env::var(ENV_BSPEC).unwrap();
    let mut parts = spec.split('|');
    let branch_idx: usize = parts.next().unwrap().parse().unwrap();
    let ordinal: u8 = parts.next().unwrap().parse().unwrap();
    let nth: u64 = parts.next().unwrap().parse().unwrap();
    let req = opening::serial();
    let exec = ExecutionRequest::serial(&req);
    let done = Arc::new(AtomicBool::new(false));
    let (placement, activation, watcher) = if nth == u64::MAX {
        sync_barrier_placement(u64::MAX, None, None)
    } else {
        sync_barrier_placement(nth, Some(markers.join("ready")), Some(done.clone()))
    };
    let home = open_signal_home(
        SignalOpen {
            location: Location::Directory(home_path),
            build: &build,
            initialized: Arc::new(AtomicUsize::new(0)),
            contacts: ContactCounters::default(),
            placement: Some(placement),
        },
        &exec,
    );
    activation.arm().unwrap();
    let target = sorted_branches(&home)[branch_idx];
    let outcome_tag = match commit_mixed(&home, target, ordinal) {
        CommitOutcome::Committed(_) => "committed",
        CommitOutcome::RecoveredCommitted(_) => "recovered_committed",
        CommitOutcome::ProductUnpublished(_) => "product_unpublished",
        _ => "other",
    };
    done.store(true, Ordering::SeqCst);
    if let Some(h) = watcher {
        let _ = h.join();
    }
    let actual = activation.matching_operation_count();
    crate::killed_child::announce(
        &markers.join("finished"),
        format!("missed:actual={actual}:outcome={outcome_tag}").as_bytes(),
    );
}
