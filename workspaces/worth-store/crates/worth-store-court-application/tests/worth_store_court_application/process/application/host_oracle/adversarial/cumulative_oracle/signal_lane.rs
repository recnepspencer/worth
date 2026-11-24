//! Seeded cumulative Signal + Bridge + Authored Product Branches lane.
//!
//! Drives ONE live `MemoryOnly` `SignalHome` in lockstep with fresh OS child
//! processes across alternating builds, D14 derived-state discards (verifying
//! all 5 cadence journals report `covered == 0`), authored branch forks, mixed
//! commits, Bridge definition updates, exact full-receipt retries (even after
//! branch head advancement), pre-publish recovered Bridge definition
//! verification, quiescent `kill_live`, and mid-op `SynchronizeFileState`
//! barrier kills on BOTH root AND forked authored Signal branches.
mod barrier_verify;
mod child;
pub(super) mod fork_sweep;
mod ops;

use self::barrier_verify::{verify_signal_barrier_and_finalize, SignalBarrierCheck};
use self::child::CumSignalSession;
use self::ops::{
    assert_multi_parity, assert_receipt_bound, fork_on_home, kill_cum_signal_barrier,
    observe_multi_branch, sorted_branches, CumSignalCmd, CumSignalReply,
};
use super::super::super::opening;
use super::super::dimensions_matrix::assert_cadence_discarded;
use super::super::dimensions_signal::wire::WireReceipt;
use super::super::signal_bridge_io_kills::{
    commit_mixed, open_signal_home, publish_definition, verify_recovered_bridge_definition,
    SignalHome, SignalOpen,
};
use super::super::BUILD_PRIMARY;
use super::{SplitMix64, StepCtx};
use std::path::PathBuf;
use std::sync::{atomic::AtomicUsize, Arc};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::WorthQueryApplicationHomeLocation as Location;
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::ContactCounters;

pub(super) struct SignalStepper {
    seed: u64,
    rng: SplitMix64,
    _root: tempfile::TempDir,
    home_dir: PathBuf,
    ch_root: PathBuf,
    ch_seq: usize,
    ref_home: SignalHome,
    expected_def_id: [u8; 32],
    build: &'static str,
    discarded: bool,
    next_ord: u8,
    committed_main: Vec<(u8, WireReceipt)>,
    session: Option<CumSignalSession>,
    phase: usize,
    target_branch: usize,
    target_ord: u8,
    barrier_nth: u64,
}

impl SignalStepper {
    pub(super) fn new(seed: u64) -> Self {
        let req = opening::serial();
        let exec = ExecutionRequest::serial(&req);
        let root = tempfile::tempdir().unwrap();
        let home_dir = root.path().join("home");
        let ch_root = root.path().join("ch");
        std::fs::create_dir_all(&ch_root).unwrap();
        let ref_home = open_signal_home(
            SignalOpen {
                location: Location::MemoryOnly,
                build: BUILD_PRIMARY,
                initialized: Arc::new(AtomicUsize::new(0)),
                contacts: ContactCounters::default(),
                placement: None,
            },
            &exec,
        );
        let expected_def_id =
            verify_recovered_bridge_definition(&ref_home, ref_home.runtime().current_world());
        Self {
            seed,
            rng: SplitMix64::new(seed ^ 0x5349_474E),
            _root: root,
            home_dir,
            ch_root,
            ch_seq: 0,
            ref_home,
            expected_def_id,
            build: BUILD_PRIMARY,
            discarded: false,
            next_ord: 10 + ((seed as u8 % 5) * 20),
            committed_main: Vec::new(),
            session: None,
            phase: 0,
            target_branch: (seed as usize) % 3,
            target_ord: 0,
            barrier_nth: 0,
        }
    }

    pub(super) fn is_done(&self) -> bool {
        self.phase >= 9
    }

    fn next_ch(&mut self, tag: &str) -> PathBuf {
        self.ch_seq += 1;
        self.ch_root.join(format!("{}-{tag}", self.ch_seq))
    }

    fn ctx(&self, step: usize, op: impl Into<String>, cut: impl Into<String>) -> StepCtx {
        StepCtx {
            seed: self.seed,
            step,
            family: "signal+bridge",
            op: op.into(),
            cut: cut.into(),
            build: self.build,
            derived: if self.discarded {
                "d14-discarded"
            } else {
                "retained"
            },
        }
    }

    pub(super) fn step(&mut self) {
        match self.phase {
            0 => {
                let ch = self.next_ch("init");
                let (mut session, o0) = CumSignalSession::start(&self.home_dir, &ch, self.build);
                assert_eq!(o0.initialized, 1, "seed={}: init once", self.seed);
                let init_ctx = self.ctx(0, "init", "none");
                assert_multi_parity(
                    o0.snap.as_ref().unwrap(),
                    &observe_multi_branch(&self.ref_home),
                    &init_ctx,
                );
                for b in 0..2 {
                    fork_on_home(&self.ref_home);
                    let r = session.send(CumSignalCmd::ForkBranch);
                    let fctx = self.ctx(b + 1, format!("fork#{b}"), "none");
                    assert_multi_parity(
                        r.snap.as_ref().unwrap(),
                        &observe_multi_branch(&self.ref_home),
                        &fctx,
                    );
                }
                self.session = Some(session);
                self.phase = 1;
            }
            1..=4 => {
                let step = self.phase + 2;
                let branch_idx = self.rng.range(3);
                let ctx = self.ctx(step, format!("mixed-or-def(branch={branch_idx})"), "none");
                let session = self.session.as_mut().unwrap();
                if step % 2 == 1 {
                    let ord = self.next_ord;
                    self.next_ord += 1;
                    let ref_br = sorted_branches(&self.ref_home)[branch_idx];
                    assert!(commit_mixed(&self.ref_home, ref_br, ord)
                        .require_committed()
                        .is_ok());
                    let r = session.send(CumSignalCmd::CommitMixed {
                        branch_idx,
                        ordinal: ord,
                    });
                    assert_eq!(r.commit_tag.as_deref(), Some("committed"), "{ctx}: live");
                    let snap = r.snap.as_ref().unwrap();
                    let rec = r.receipt.expect("live receipt");
                    assert_receipt_bound(&rec, ord, &snap.branches[branch_idx], &ctx);
                    assert_multi_parity(snap, &observe_multi_branch(&self.ref_home), &ctx);
                    if branch_idx == 0 {
                        self.committed_main.push((ord, rec));
                    }
                } else {
                    let ref_br = sorted_branches(&self.ref_home)[branch_idx];
                    publish_definition(&self.ref_home, ref_br);
                    let r = session.send(CumSignalCmd::PublishDef { branch_idx });
                    assert_multi_parity(
                        r.snap.as_ref().unwrap(),
                        &observe_multi_branch(&self.ref_home),
                        &ctx,
                    );
                }
                self.phase += 1;
            }
            5 => {
                let session = self.session.as_mut().unwrap();
                while self.committed_main.len() < 2 {
                    let ord = self.next_ord;
                    self.next_ord += 1;
                    let ctx = StepCtx {
                        seed: self.seed,
                        step: 6,
                        family: "signal+bridge",
                        op: format!("main-commit(ord={ord})"),
                        cut: "none".into(),
                        build: self.build,
                        derived: "retained",
                    };
                    assert!(commit_mixed(
                        &self.ref_home,
                        self.ref_home.runtime().current_world(),
                        ord
                    )
                    .require_committed()
                    .is_ok());
                    let r = session.send(CumSignalCmd::CommitMixed {
                        branch_idx: 0,
                        ordinal: ord,
                    });
                    assert_eq!(r.commit_tag.as_deref(), Some("committed"), "{ctx}");
                    let snap = r.snap.as_ref().unwrap();
                    let rec = r.receipt.expect("main live receipt");
                    assert_receipt_bound(&rec, ord, &snap.branches[0], &ctx);
                    assert_multi_parity(snap, &observe_multi_branch(&self.ref_home), &ctx);
                    self.committed_main.push((ord, rec));
                }
                self.phase = 6;
            }
            6 => {
                let mut session = self.session.take().unwrap();
                let dgen = session
                    .send(CumSignalCmd::CheckpointAndDiscard)
                    .discard_gen
                    .unwrap();
                assert!(dgen > 0);
                self.discarded = true;
                self.build = self.rng.next_other_build(self.build);
                let ctx_d14 = self.ctx(7, "d14-discard-kill", "kill_live");
                session.kill_live(&ctx_d14.to_string());
                let ch = self.next_ch("d14");
                let (mut s2, o2) = CumSignalSession::start(&self.home_dir, &ch, self.build);
                assert_eq!(
                    (o2.initialized, o2.contacts),
                    (0, (0, 0, 0, 0)),
                    "{ctx_d14}: reopen"
                );
                assert_cadence_discarded(&o2.cadence, &ctx_d14.to_string());
                assert_multi_parity(
                    o2.snap.as_ref().unwrap(),
                    &observe_multi_branch(&self.ref_home),
                    &ctx_d14,
                );
                let vb = s2.send(CumSignalCmd::VerifyRecoveredBridge {
                    branch_idx: self.target_branch,
                });
                assert_eq!(
                    vb.definition_identity,
                    Some(self.expected_def_id),
                    "{ctx_d14}"
                );
                assert_eq!(vb.contacts, (0, 0, 0, 0), "{ctx_d14}: verify 0 contacts");
                assert_eq!(vb.snap, o2.snap, "{ctx_d14}: verify snapshot invariance");
                let (retry_ord, ref_orig_rec) = self.committed_main[0].clone();
                let retry_r = s2.send(CumSignalCmd::CommitMixed {
                    branch_idx: 0,
                    ordinal: retry_ord,
                });
                assert_eq!(
                    retry_r.commit_tag.as_deref(),
                    Some("recovered_committed"),
                    "{ctx_d14}: retry"
                );
                assert_eq!(
                    retry_r.contacts,
                    (0, 0, 0, 0),
                    "{ctx_d14}: retry 0 contacts"
                );
                assert_eq!(
                    retry_r.receipt.expect("d14 retry receipt"),
                    ref_orig_rec,
                    "{ctx_d14}: full retry receipt must match earlier original after head advanced"
                );
                self.session = Some(s2);
                self.phase = 7;
            }
            7 => {
                let mut s2 = self.session.take().unwrap();
                let target_ord = self.next_ord;
                self.next_ord += 1;
                let barrier_nth = 2 + (self.rng.range(45) as u64);
                let kill_build = self.rng.next_other_build(self.build);
                let ctx_kill = StepCtx {
                    seed: self.seed,
                    step: 8,
                    family: "signal+bridge",
                    op: format!(
                        "barrier-kill-mixed(branch={},ord={target_ord})",
                        self.target_branch
                    ),
                    cut: format!("SynchronizeFileState#{barrier_nth}"),
                    build: kill_build,
                    derived: "d14-discarded",
                };
                s2.kill_live(&ctx_kill.to_string());
                kill_cum_signal_barrier(
                    &self.home_dir,
                    self.target_branch,
                    target_ord,
                    kill_build,
                    barrier_nth,
                    &ctx_kill.to_string(),
                );
                self.build = self.rng.next_other_build(kill_build);
                self.target_ord = target_ord;
                self.barrier_nth = barrier_nth;
                self.phase = 8;
            }
            8 => {
                let tb = self.target_branch;
                let target_ord = self.target_ord;
                let ctx_bar = self.ctx(
                    8,
                    format!("barrier-verify-mixed(branch={tb},ord={target_ord})"),
                    format!("SynchronizeFileState#{}", self.barrier_nth),
                );
                let ch_post = self.next_ch("post-bar");
                let ch_fin = self.next_ch("final");
                verify_signal_barrier_and_finalize(
                    SignalBarrierCheck {
                        home_dir: &self.home_dir,
                        ch_post: &ch_post,
                        ch_fin: &ch_fin,
                        branch_idx: tb,
                        target_ord,
                        expected_def_id: self.expected_def_id,
                        build: self.build,
                        ctx: &ctx_bar,
                    },
                    &self.ref_home,
                    &mut self.rng,
                );
                self.phase = 9;
            }
            _ => {}
        }
    }
}

pub(super) fn run_signal_cumulative_seed(seed: u64) {
    let mut stepper = SignalStepper::new(seed);
    while !stepper.is_done() {
        stepper.step();
    }
    let req = opening::serial();
    let exec = ExecutionRequest::serial(&req);
    drop(
        stepper
            .ref_home
            .close(&exec)
            .err()
            .unwrap()
            .resume()
            .unwrap(),
    );
    let _ = CumSignalReply::default;
}
