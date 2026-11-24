//! Seeded cumulative C2.8 Durable World Branch Creation, Retirement, History
//! Disposal, and Retired-Name Reuse oracle across fresh OS child processes,
//! alternating builds, D14 derived-state discards, quiescent kills, and mid-op
//! durable `SynchronizeFileState` barrier kills, verified against a pure
//! `MemoryOnly` Native reference and pure topology-derived exact reclaim sets.
mod memory_ref;
mod setup;
mod verify;

use self::setup::{run_topology_setup, TopologySetup, ROOT_NAME};
use self::verify::{verify_barrier_and_finalize, BarrierTarget};
use super::super::dimensions_matrix::assert_cadence_discarded;
use super::super::BUILD_PRIMARY;
use super::world_branch_child::{kill_wb_publish_barrier, WbCmd, WorldBranchSession};
use super::{SplitMix64, StepCtx};
use std::collections::BTreeSet;
use std::path::PathBuf;
use worth_execution::ExecutionRequest;

pub(super) struct WorldBranchStepper {
    seed: u64,
    rng: SplitMix64,
    _root: tempfile::TempDir,
    home_dir: PathBuf,
    ch_root: PathBuf,
    ch_seq: usize,
    build: &'static str,
    discarded: bool,
    phase: usize,
    session: Option<WorldBranchSession>,
    topo: Option<TopologySetup>,
    retired_head: u64,
    target_branch: String,
    barrier_round: usize,
    barrier_nth: u64,
}

impl WorldBranchStepper {
    pub(super) fn new(seed: u64) -> Self {
        let root = tempfile::tempdir().unwrap();
        let home_dir = root.path().join("home");
        let ch_root = root.path().join("ch");
        std::fs::create_dir_all(&ch_root).unwrap();
        Self {
            seed,
            rng: SplitMix64::new(seed ^ 0x4332_3857),
            _root: root,
            home_dir,
            ch_root,
            ch_seq: 0,
            build: BUILD_PRIMARY,
            discarded: false,
            phase: 0,
            session: None,
            topo: None,
            retired_head: 0,
            target_branch: String::new(),
            barrier_round: 0,
            barrier_nth: 0,
        }
    }

    pub(super) fn is_done(&self) -> bool {
        self.phase >= 6
    }

    fn next_ch(&mut self, tag: &str) -> PathBuf {
        self.ch_seq += 1;
        self.ch_root.join(format!("{}-{tag}", self.ch_seq))
    }

    fn ctx(&self, step: usize, op: impl Into<String>, cut: impl Into<String>) -> StepCtx {
        StepCtx {
            seed: self.seed,
            step,
            family: "c2.8-world-branches",
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
        let req = crate::workload::serial_request();
        let exec = ExecutionRequest::serial(&req);
        match self.phase {
            0 => {
                let ch = self.next_ch("init");
                let (mut session, _) = WorldBranchSession::start(&self.home_dir, &ch, self.build);
                let topo = run_topology_setup(&mut session, &mut self.rng, self.seed, self.build);
                self.session = Some(session);
                self.topo = Some(topo);
                self.phase = 1;
            }
            1 => {
                let mut session = self.session.take().unwrap();
                let dgen = session
                    .send(WbCmd::CheckpointAndDiscard)
                    .discard_gen
                    .unwrap();
                assert!(dgen > 0);
                self.discarded = true;
                let kill_ctx = self.ctx(10, "d14-discard-kill", "kill_live");
                session.kill_live(&kill_ctx.to_string());
                self.build = self.rng.next_other_build(self.build);
                let ctx_d14 = self.ctx(10, "d14-discard-reopen", "kill_live");
                let ch = self.next_ch("d14");
                let (s2, o2) = WorldBranchSession::start(&self.home_dir, &ch, self.build);
                let snap2 = o2.snap.unwrap();
                assert_cadence_discarded(&snap2.cadence, &ctx_d14.to_string());
                let topo = self.topo.as_ref().unwrap();
                topo.model.assert_matches(&snap2, &ctx_d14);
                topo.mem_ref.assert_payload_parity(&snap2, &exec, &ctx_d14);
                self.session = Some(s2);
                self.phase = 2;
            }
            2 => {
                let mut s2 = self.session.take().unwrap();
                let topo = self.topo.as_mut().unwrap();
                let ret_reply = s2.send(WbCmd::RetireBranch {
                    name: topo.parent_to_retire.clone(),
                });
                let retired_head = ret_reply.retired_commit.unwrap();
                topo.mem_ref.retire_branch(&topo.parent_to_retire);
                self.retired_head = retired_head;
                assert_eq!(retired_head, topo.sib_retired_head, "retired head");
                topo.model.live.remove(&topo.parent_to_retire);
                topo.model.retired_readable.insert(retired_head);
                let op_name = format!("retire({})", topo.parent_to_retire);
                let ctx_ret = StepCtx {
                    seed: self.seed,
                    step: 11,
                    family: "c2.8-world-branches",
                    op: op_name,
                    cut: "none".into(),
                    build: self.build,
                    derived: "d14-discarded",
                };
                topo.model
                    .assert_matches(ret_reply.snap.as_ref().unwrap(), &ctx_ret);
                let kill_ctx = StepCtx {
                    seed: self.seed,
                    step: 12,
                    family: "c2.8-world-branches",
                    op: "post-retire-kill".into(),
                    cut: "kill_live".into(),
                    build: self.build,
                    derived: "d14-discarded",
                };
                s2.kill_live(&kill_ctx.to_string());
                self.build = self.rng.next_other_build(self.build);
                let ctx_rq = self.ctx(12, "post-retire-reopen", "kill_live");
                let ch = self.next_ch("post-ret");
                let (mut s3, o3) = WorldBranchSession::start(&self.home_dir, &ch, self.build);
                let topo = self.topo.as_ref().unwrap();
                topo.model
                    .assert_matches(o3.snap.as_ref().unwrap(), &ctx_rq);
                s3.send(WbCmd::AssertReadable {
                    ordinals: topo.model.retired_readable.iter().copied().collect(),
                });
                self.session = Some(s3);
                self.phase = 3;
            }
            3 => {
                let mut s3 = self.session.take().unwrap();
                let ctx_disp = self.ctx(13, "exact-history-disposal", "none");
                let topo = self.topo.as_mut().unwrap();
                let before_disp_commits = topo.model.installed_commits;
                let disp = s3.send(WbCmd::DisposeRetiredHistory {
                    retired_commit: self.retired_head,
                });
                let disp_set: BTreeSet<u64> = disp.reclaimed_ordinals.iter().copied().collect();
                assert_eq!(
                    disp_set, topo.sib_exclusive,
                    "{ctx_disp}: retired disposal must reclaim EXACT topology exclusive set (no extra/missing)"
                );
                assert_eq!(
                    disp.snap.as_ref().unwrap().installed_commits,
                    before_disp_commits - topo.sib_exclusive.len(),
                    "{ctx_disp}: exact installed_commits drop"
                );
                for ord in &disp_set {
                    topo.model.retired_readable.remove(ord);
                    topo.model.disposed.insert(*ord);
                }
                topo.model.installed_commits = disp.snap.as_ref().unwrap().installed_commits;

                let before_chg_commits = topo.model.installed_commits;
                let changed = s3.send(WbCmd::RetireChangedHistory);
                let chg_set: BTreeSet<u64> = changed.reclaimed_ordinals.iter().copied().collect();
                assert_eq!(
                    chg_set, topo.expected_changed_set,
                    "{ctx_disp}: changed-history disposal must reclaim EXACT topology interior set (no extra/missing reclamation)"
                );
                assert_eq!(
                    changed.snap.as_ref().unwrap().installed_commits,
                    before_chg_commits - topo.expected_changed_set.len(),
                    "{ctx_disp}: exact changed installed_commits drop"
                );
                for ord in &chg_set {
                    assert!(
                        !topo.protected_ancestors.contains(ord),
                        "{ctx_disp}: protected commit #{ord} must NEVER be reclaimed"
                    );
                    topo.model.disposed.insert(*ord);
                }
                topo.model.installed_commits = changed.snap.as_ref().unwrap().installed_commits;
                s3.send(WbCmd::AssertReadable {
                    ordinals: topo.protected_live_heads.clone(),
                });
                s3.send(WbCmd::AssertProtectedInAncestry {
                    ordinals: topo.protected_ancestors.clone(),
                });
                s3.send(WbCmd::AssertDisposed {
                    ordinals: topo.model.disposed.iter().copied().collect(),
                });

                // Reuse retired name under strictly new incarnation!
                topo.branch_seq += 1;
                let reuse_prefix = format!("s{}-reuse-{}", self.seed, topo.branch_seq);
                let reused = s3
                    .send(WbCmd::CreateBranch {
                        parent: ROOT_NAME.to_owned(),
                        name: topo.parent_to_retire.clone(),
                        fork_prefix: reuse_prefix.clone(),
                        native_fork: true,
                        signal_fork: true,
                    })
                    .snap
                    .unwrap();
                topo.mem_ref.create_branch(
                    ROOT_NAME,
                    &topo.parent_to_retire,
                    &format!("{reuse_prefix}-native"),
                    true,
                    &exec,
                );
                topo.model.record_branch(&reused, &topo.parent_to_retire);
                assert!(
                    topo.model.live[&topo.parent_to_retire].incarnation > 1,
                    "{ctx_disp}: retired name reuse must advance incarnation"
                );
                topo.mem_ref
                    .assert_payload_parity(&reused, &exec, &ctx_disp);
                self.session = Some(s3);
                self.phase = 4;
            }
            4 => {
                let mut s3 = self.session.take().unwrap();
                let topo = self.topo.as_mut().unwrap();
                let live_names: Vec<String> = topo.model.live.keys().cloned().collect();
                let target_branch = self.rng.choose(&live_names).clone();
                let barrier_nth = 1 + (self.rng.range(12) as u64);
                let barrier_round = topo.round;
                topo.round += 1;
                let live_kill_ctx = StepCtx {
                    seed: self.seed,
                    step: 14,
                    family: "c2.8-world-branches",
                    op: format!("pre-barrier-kill({target_branch})"),
                    cut: "kill_live".into(),
                    build: self.build,
                    derived: "d14-discarded",
                };
                s3.kill_live(&live_kill_ctx.to_string());
                let kill_build = self.rng.next_other_build(self.build);
                let ctx_kill = StepCtx {
                    seed: self.seed,
                    step: 14,
                    family: "c2.8-world-branches",
                    op: format!("barrier-kill-publish({target_branch})"),
                    cut: format!("SynchronizeFileState#{barrier_nth}"),
                    build: kill_build,
                    derived: "d14-discarded",
                };
                kill_wb_publish_barrier(
                    &self.home_dir,
                    &target_branch,
                    barrier_round,
                    kill_build,
                    barrier_nth,
                    &ctx_kill.to_string(),
                );
                self.build = self.rng.next_other_build(kill_build);
                self.target_branch = target_branch;
                self.barrier_round = barrier_round;
                self.barrier_nth = barrier_nth;
                self.phase = 5;
            }
            5 => {
                let ctx_bar = self.ctx(
                    15,
                    format!("barrier-verify-publish({})", self.target_branch),
                    format!("SynchronizeFileState#{}", self.barrier_nth),
                );
                let ch_post = self.next_ch("post-bar");
                let ch_fin = self.next_ch("final");
                let mut topo_owned = self.topo.take().unwrap();
                verify_barrier_and_finalize(
                    BarrierTarget {
                        home_dir: &self.home_dir,
                        ch_post: &ch_post,
                        ch_fin: &ch_fin,
                        branch: &self.target_branch,
                        round: self.barrier_round,
                        build: self.build,
                        ctx: &ctx_bar,
                    },
                    &mut topo_owned,
                    &mut self.rng,
                    &exec,
                );
                self.phase = 6;
            }
            _ => {}
        }
    }
}

pub(super) fn run_world_branch_cumulative_seed(seed: u64) {
    let mut stepper = WorldBranchStepper::new(seed);
    while !stepper.is_done() {
        stepper.step();
    }
}
