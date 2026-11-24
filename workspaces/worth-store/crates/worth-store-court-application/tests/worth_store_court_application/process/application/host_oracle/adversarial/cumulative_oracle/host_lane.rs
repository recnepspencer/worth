//! Seeded cumulative Core Query Host lane (Native + World + Query + Idempotency
//! + IntentDrift + Unpublished Custody Continue & Release + Checkpoints + D14
//! Discard + Alternating Builds + Store Mutate/Checkpoint/Close Seam Kills +
//! Recovery-Stage Kills) verified against ONE run-wide MemoryOnly OracleHome &
//! HostOracleComparison.
mod ops;
mod release;

use self::ops::{
    commit_fresh, run_checkpoint_or_close_seam_kill, run_mid_op_seam_kill, verify_retry_and_drift,
};
use self::release::verify_custody_release_tail;
use super::super::super::{
    opening,
    protocol::{Disposition, Request},
    session::Session,
};
use super::super::dimensions_matrix::verify::{
    checkpoint_and_discard, close_expecting_discarded, close_normal, inspect_cadence,
};
use super::super::dimensions_matrix::{assert_cadence_checkpointed, assert_cadence_discarded};
use super::super::recovery_fixtures::discover_mutate_ordinals;
use super::super::{build_id, BUILD_PRIMARY};
use super::{SplitMix64, StepCtx};
use std::path::PathBuf;
use std::sync::{atomic::AtomicUsize, Arc};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::WorthQueryApplicationHomeLocation as Location;
use worth_store_testkit_application_oracle::facade::host_observation::{
    comparison::{self, HostOracleComparison},
    opening::{open_with_placement, PlacementOpenConfig},
    projection::{self, StateProjection},
    receipts::MutationProjection,
    OracleHome,
};

pub(self) struct HostOracleState {
    pub(self) seed: u64,
    pub(self) home_dir: PathBuf,
    pub(self) ch_root: PathBuf,
    pub(self) ch_seq: usize,
    pub(self) build: &'static str,
    pub(self) derived_discarded: bool,
    pub(self) discard_gen: Option<u64>,
    pub(self) ref_home: OracleHome,
    pub(self) map: HostOracleComparison,
    pub(self) committed: Vec<(u64, u64, MutationProjection)>,
    pub(self) next_key: u64,
}

impl HostOracleState {
    pub(self) fn next_ch(&mut self, tag: &str) -> PathBuf {
        self.ch_seq += 1;
        self.ch_root.join(format!("{}-{tag}", self.ch_seq))
    }

    pub(self) fn ctx(&self, step: usize, op: impl Into<String>, cut: impl Into<String>) -> StepCtx {
        StepCtx {
            seed: self.seed,
            step,
            family: "core-host",
            op: op.into(),
            cut: cut.into(),
            build: self.build,
            derived: if self.derived_discarded {
                "d14-discarded"
            } else {
                "retained"
            },
        }
    }

    pub(self) fn ref_state(&self, exec: &ExecutionRequest<'_, '_>) -> StateProjection {
        projection::observe(self.ref_home.runtime(), exec)
    }

    pub(self) fn alloc_kv(&mut self, rng: &mut SplitMix64) -> (u64, u64) {
        self.next_key += 1;
        let k = 0xC4B4_0000_0000_0000 | (self.seed << 24) | self.next_key;
        let v = 1000 + (rng.next_u64() % 100_000);
        (k, v)
    }

    pub(self) fn compare_state(
        &mut self,
        observed: &StateProjection,
        expected: &StateProjection,
        ctx: &StepCtx,
    ) {
        comparison::compare(&mut self.map, observed, expected)
            .unwrap_or_else(|e| panic!("{ctx}: MemoryOnly divergence: {e:?}"));
    }
}

pub(super) struct HostStepper {
    _root: tempfile::TempDir,
    state: HostOracleState,
    rng: SplitMix64,
    ordinals: super::super::recovery_fixtures::MutateSeamOrdinals,
    session: Option<Session>,
    plan: Vec<usize>,
    cursor: usize,
    finished: bool,
}

impl HostStepper {
    pub(super) fn new(seed: u64) -> Self {
        let mut rng = SplitMix64::new(seed ^ 0x484F_5354);
        let req = opening::serial();
        let exec = ExecutionRequest::serial(&req);
        let ordinals = discover_mutate_ordinals(&exec);
        let root = tempfile::tempdir().unwrap();
        let home_dir = root.path().join("home");
        let ch_root = root.path().join("ch");
        std::fs::create_dir_all(&ch_root).unwrap();

        let (ref_home, _) = open_with_placement(
            PlacementOpenConfig {
                location: Location::MemoryOnly,
                build: build_id(BUILD_PRIMARY),
                initialized: Arc::new(AtomicUsize::new(0)),
                contacts: Arc::new(AtomicUsize::new(0)),
                placement: None,
                limits: Default::default(),
            },
            &exec,
        );
        let mut state = HostOracleState {
            seed,
            home_dir,
            ch_root,
            ch_seq: 0,
            build: BUILD_PRIMARY,
            derived_discarded: false,
            discard_gen: None,
            ref_home,
            map: HostOracleComparison::default(),
            committed: Vec::new(),
            next_key: 0,
        };

        let ch0 = state.next_ch("init");
        let (mut session, opened0) =
            Session::start_with_build(&state.home_dir, &ch0, 0, Some(state.build));
        assert_eq!(opened0.initialized, 1, "seed={seed}: init once");
        let ref0 = state.ref_state(&exec);
        let init_ctx = state.ctx(0, "bootstrap", "none");
        state.compare_state(&opened0.state.unwrap(), &ref0, &init_ctx);

        let (k0, v0) = state.alloc_kv(&mut rng);
        commit_fresh(&mut state, &mut session, (k0, v0), 0, &exec);

        let mut plan: Vec<usize> = (0..9).collect();
        for _ in 0..3 {
            plan.push(rng.range(9));
        }
        for i in (1..plan.len()).rev() {
            let j = rng.range(i + 1);
            plan.swap(i, j);
        }
        Self {
            _root: root,
            state,
            rng,
            ordinals,
            session: Some(session),
            plan,
            cursor: 0,
            finished: false,
        }
    }

    pub(super) fn is_done(&self) -> bool {
        self.finished
    }

    pub(super) fn step(&mut self) {
        if self.finished {
            return;
        }
        let req = opening::serial();
        let exec = ExecutionRequest::serial(&req);
        if self.cursor < self.plan.len() {
            let op_kind = self.plan[self.cursor];
            let step = self.cursor + 1;
            self.cursor += 1;
            let mut session = self.session.take().unwrap();
            match op_kind {
                0 => {
                    let kv = self.state.alloc_kv(&mut self.rng);
                    commit_fresh(&mut self.state, &mut session, kv, step, &exec);
                }
                1 => {
                    verify_retry_and_drift(
                        &mut self.state,
                        &mut session,
                        &mut self.rng,
                        step,
                        &exec,
                    );
                }
                2 => {
                    let ctx = self.state.ctx(step, "checkpoint", "none");
                    let resp = session.send(Request::Checkpoint);
                    assert!(
                        matches!(resp.disposition, Some(Disposition::Checkpointed { generation }) if generation > 0),
                        "{ctx}: explicit checkpoint must publish"
                    );
                    self.state.derived_discarded = false;
                    self.state.discard_gen = None;
                    let expected = self.state.ref_state(&exec);
                    self.state
                        .compare_state(&resp.state.unwrap(), &expected, &ctx);
                }
                3 => {
                    let kill_after_discard = self.rng.range(2) == 0;
                    let cut_tag = if kill_after_discard {
                        "kill_live"
                    } else {
                        "close_expecting_discarded"
                    };
                    let dgen = checkpoint_and_discard(&mut session);
                    let pre_ctx = self.state.ctx(step, "d14-discard-reopen", cut_tag);
                    if kill_after_discard {
                        session.kill_live_with_ctx(&pre_ctx.to_string());
                    } else {
                        close_expecting_discarded(&mut session, &pre_ctx.to_string());
                    }
                    self.state.derived_discarded = true;
                    self.state.discard_gen = Some(dgen);
                    self.state.build = self.rng.next_other_build(self.state.build);
                    let ctx = self.state.ctx(step, "d14-discard-reopen", cut_tag);
                    let ch = self.state.next_ch("d14");
                    let (mut s2, o2) = Session::start_with_build(
                        &self.state.home_dir,
                        &ch,
                        0,
                        Some(self.state.build),
                    );
                    assert_eq!((o2.initialized, o2.contacts), (0, 0), "{ctx}: reopen");
                    assert_cadence_discarded(&inspect_cadence(&mut s2), &ctx.to_string());
                    let expected = self.state.ref_state(&exec);
                    self.state
                        .compare_state(&o2.state.unwrap(), &expected, &ctx);
                    session = s2;
                }
                4 => {
                    let ctx = self.state.ctx(step, "graceful-close", "none");
                    close_normal(&mut session, self.state.discard_gen, &ctx.to_string());
                    self.state.derived_discarded = false;
                    self.state.discard_gen = None;
                    self.state.build = self.rng.next_other_build(self.state.build);
                    let ch = self.state.next_ch("close-reopen");
                    let (mut s2, o2) = Session::start_with_build(
                        &self.state.home_dir,
                        &ch,
                        0,
                        Some(self.state.build),
                    );
                    assert_eq!((o2.initialized, o2.contacts), (0, 0), "{ctx}: reopen");
                    assert_cadence_checkpointed(&inspect_cadence(&mut s2), &ctx.to_string());
                    let expected = self.state.ref_state(&exec);
                    self.state
                        .compare_state(&o2.state.unwrap(), &expected, &ctx);
                    session = s2;
                }
                5 => {
                    let kill_ctx = self.state.ctx(step, "quiescent-kill", "kill_live");
                    session.kill_live_with_ctx(&kill_ctx.to_string());
                    self.state.build = self.rng.next_other_build(self.state.build);
                    let ctx = self.state.ctx(step, "quiescent-kill", "kill_live");
                    let ch = self.state.next_ch("qkill");
                    let (s2, o2) = Session::start_with_build(
                        &self.state.home_dir,
                        &ch,
                        0,
                        Some(self.state.build),
                    );
                    assert_eq!((o2.initialized, o2.contacts), (0, 0), "{ctx}: reopen");
                    let expected = self.state.ref_state(&exec);
                    self.state
                        .compare_state(&o2.state.unwrap(), &expected, &ctx);
                    session = s2;
                }
                6 | 7 => {
                    session = run_mid_op_seam_kill(
                        &mut self.state,
                        session,
                        &mut self.rng,
                        step,
                        self.ordinals,
                        op_kind == 7,
                        &exec,
                    );
                }
                8 => {
                    session = run_checkpoint_or_close_seam_kill(
                        &mut self.state,
                        session,
                        &mut self.rng,
                        step,
                        &exec,
                    );
                }
                _ => unreachable!(),
            }
            self.session = Some(session);
            return;
        }

        let mut session = self.session.take().unwrap();
        let fin_ctx = self.state.ctx(99, "final-close", "none");
        close_normal(&mut session, self.state.discard_gen, &fin_ctx.to_string());
        self.state.derived_discarded = false;
        self.state.discard_gen = None;
        self.state.build = self.rng.next_other_build(self.state.build);
        let ch_fin = self.state.next_ch("final");
        let (mut fin, o_fin) =
            Session::start_with_build(&self.state.home_dir, &ch_fin, 0, Some(self.state.build));
        assert_eq!((o_fin.initialized, o_fin.contacts), (0, 0), "{fin_ctx}");
        let expected_fin = self.state.ref_state(&exec);
        self.state
            .compare_state(&o_fin.state.unwrap(), &expected_fin, &fin_ctx);
        close_normal(&mut fin, None, &format!("{fin_ctx}/close"));

        verify_custody_release_tail(
            &mut self.state,
            &mut self.rng,
            self.ordinals,
            &expected_fin,
            &exec,
        );
        self.finished = true;
    }
}

pub(super) fn run_host_cumulative_seed(seed: u64) {
    let mut stepper = HostStepper::new(seed);
    while !stepper.is_done() {
        stepper.step();
    }
    let req = opening::serial();
    let exec = ExecutionRequest::serial(&req);
    drop(
        stepper
            .state
            .ref_home
            .close(&exec)
            .err()
            .expect("MemoryOnly close refusal")
            .resume()
            .unwrap(),
    );
}
