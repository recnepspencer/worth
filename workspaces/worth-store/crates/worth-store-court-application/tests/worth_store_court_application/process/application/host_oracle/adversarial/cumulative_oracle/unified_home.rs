//! Single-Home Unified Multi-Family Cumulative Oracle (`DocumentRetentionSchema`):
//!
//! Runs ONE single durable Home (`Location::Directory`) against ONE live
//! `Location::MemoryOnly` reference with ONE run-wide
//! `DurableIdentityCorrespondence`, mixing 4 owner families in one cumulative
//! sequence where a kill in ANY family proves ALL other families' acknowledged
//! state on that SAME Home survives that exact same reopen:
//!   1. Core Query Host ordinary mutations + Idempotency + IntentDrift
//!   2. Authored Product Branches (fork + independent mutations + fork kill)
//!   3. Governed Workflow (`GovernedDocumentWorkflow` + `CorrectableRetentionBinding`
//!      across two instances, barrier kill, D14 discard, and `Historical`)
//!   4. Branch Program Adoption (`P0 -> P1` with workflow disposition)
//! across alternating builds and D14 derived-state discards (`covered == 0`).
mod barrier;
mod child;
mod late_phases;
mod ops;
mod second_wf;
mod wire;

use self::barrier::kill_unified_barrier;
use self::child::UnifiedSession;
use self::ops::{
    bind_unified_wf, exec_set_retention, fork_doc_branch, observe_doc_wf_locators, observe_unified,
    open_unified_raw, DocRuntime, RawDocHome,
};
use self::wire::{assert_unified_parity, UnifiedCmd, UnifiedSnap};
use super::super::workflow_correction_fixture::WorkflowHome;
use super::super::BUILD_PRIMARY;
use super::{SplitMix64, StepCtx};
use std::collections::BTreeMap;
use std::path::PathBuf;
use worth_execution::ExecutionRequest;
use worth_query_host::facade::WorthQueryApplicationHomeLocation as Location;
use worth_query_testkit_document_retention::facade::document_retention_model::workflow::CertificationAuthenticationOwner;
use worth_query_testkit_host::facade::host::checkpoint_capture;
use worth_store_testkit_application_oracle::facade::DurableIdentityCorrespondence;

const KEY_CORE_MAIN: u64 = 1001;
const KEY_FORK_MUT: u64 = 2001;

pub(self) enum RefHomeSlot {
    Raw(RawDocHome),
    Wf(WorkflowHome, CertificationAuthenticationOwner),
    Closed,
}

impl RefHomeSlot {
    pub(self) fn runtime(&self) -> &DocRuntime {
        match self {
            Self::Raw(h) => h.runtime(),
            Self::Wf(w, _) => w.runtime(),
            Self::Closed => panic!("ref home closed"),
        }
    }

    pub(self) fn bind_wf(&mut self) -> (&WorkflowHome, &CertificationAuthenticationOwner) {
        if matches!(self, Self::Raw(_)) {
            let Self::Raw(h) = std::mem::replace(self, Self::Closed) else {
                unreachable!();
            };
            let (w, auth) = bind_unified_wf(h);
            *self = Self::Wf(w, auth);
        }
        let Self::Wf(w, auth) = self else {
            unreachable!();
        };
        (w, auth)
    }

    pub(self) fn observe(&self, wf_locators: &[Vec<u8>]) -> UnifiedSnap {
        let wf_states = match self {
            Self::Wf(w, _) if !wf_locators.is_empty() => observe_doc_wf_locators(w, wf_locators),
            _ => Vec::new(),
        };
        observe_unified(self.runtime(), wf_states, BTreeMap::new())
    }
}

pub(super) struct UnifiedHomeStepper {
    pub(self) seed: u64,
    pub(self) rng: SplitMix64,
    _root: tempfile::TempDir,
    pub(self) home_dir: PathBuf,
    ch_root: PathBuf,
    pub(self) coords_file: PathBuf,
    ch_seq: usize,
    pub(self) ref_slot: RefHomeSlot,
    pub(self) ref_wf_locators: Vec<Vec<u8>>,
    pub(self) durable_wf_locators: Vec<Vec<u8>>,
    pub(self) map: DurableIdentityCorrespondence,
    pub(self) build: &'static str,
    pub(self) discarded: bool,
    pub(self) phase: usize,
    pub(self) main_days: u64,
    pub(self) fork_days: u64,
    pub(self) session: Option<UnifiedSession>,
}

impl UnifiedHomeStepper {
    pub(super) fn new(seed: u64) -> Self {
        let lease = checkpoint_capture::lease(256 << 20);
        let exec = ExecutionRequest::leased(&lease);
        let root = tempfile::tempdir().unwrap();
        let home_dir = root.path().join("home");
        let ch_root = root.path().join("ch");
        std::fs::create_dir_all(&ch_root).unwrap();
        let coords_file = root.path().join("coords.json");
        let ref_raw = open_unified_raw(&Location::MemoryOnly, BUILD_PRIMARY, None, exec);
        Self {
            seed,
            rng: SplitMix64::new(seed ^ 0x554E_4946),
            _root: root,
            home_dir,
            ch_root,
            coords_file,
            ch_seq: 0,
            ref_slot: RefHomeSlot::Raw(ref_raw),
            ref_wf_locators: Vec::new(),
            durable_wf_locators: Vec::new(),
            map: DurableIdentityCorrespondence::default(),
            build: BUILD_PRIMARY,
            discarded: false,
            phase: 0,
            // Avoid 7 (seeded initial), 8 (workflow UPDATED_RETENTION), and 6 (failing-assessment sentinel).
            main_days: if seed.is_multiple_of(2) { 5 } else { 9 },
            fork_days: if seed.is_multiple_of(2) { 10 } else { 5 },
            session: None,
        }
    }

    pub(super) fn is_done(&self) -> bool {
        self.phase >= 5
    }

    pub(self) fn next_ch(&mut self, tag: &str) -> PathBuf {
        self.ch_seq += 1;
        self.ch_root.join(format!("{}-{tag}", self.ch_seq))
    }

    pub(self) fn ctx(&self, step: usize, op: impl Into<String>, cut: impl Into<String>) -> StepCtx {
        StepCtx {
            seed: self.seed,
            step,
            family: "unified-single-home",
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
            0 => self.step_core_and_fork(),
            1 => self.step_fork_barrier_kill(),
            2 => self.step_workflow_and_d14(),
            3 => self.step_second_workflow_and_historical(),
            4 => self.step_adoption_and_final(),
            _ => {}
        }
    }

    fn step_core_and_fork(&mut self) {
        let ctx = self.ctx(0, "core-mutate+drift+fork", "none");
        let ch = self.next_ch("init");
        let (mut s, o0) =
            UnifiedSession::start(&self.home_dir, &ch, self.build, &self.durable_wf_locators);
        assert_unified_parity(
            &mut self.map,
            o0.snap.as_ref().unwrap(),
            &self.ref_slot.observe(&self.ref_wf_locators),
            &ctx,
        );
        let _ = exec_set_retention(self.ref_slot.runtime(), 0, self.main_days, KEY_CORE_MAIN);
        let r1 = s.send(UnifiedCmd::SetRetention {
            branch_idx: 0,
            days: self.main_days,
            key: KEY_CORE_MAIN,
        });
        assert_eq!(r1.fate_tag.as_deref(), Some("committed"), "{ctx}");
        let orig_rcpt = r1.receipt_id.clone().expect("core receipt");
        let retry = s.send(UnifiedCmd::SetRetention {
            branch_idx: 0,
            days: self.main_days,
            key: KEY_CORE_MAIN,
        });
        assert!(
            matches!(
                retry.fate_tag.as_deref(),
                Some("already_committed" | "recovered_committed")
            ),
            "{ctx}: retry fate {:?}",
            retry.fate_tag
        );
        assert_eq!(retry.receipt_id, Some(orig_rcpt), "{ctx}: retry receipt");
        let drift = s.send(UnifiedCmd::SetRetention {
            branch_idx: 0,
            days: self.main_days + 1,
            key: KEY_CORE_MAIN,
        });
        assert_eq!(
            drift.fate_tag.as_deref(),
            Some("intent_drift"),
            "{ctx}: drift"
        );
        fork_doc_branch(self.ref_slot.runtime());
        let forked = s.send(UnifiedCmd::ForkBranch);
        assert_unified_parity(
            &mut self.map,
            forked.snap.as_ref().unwrap(),
            &self.ref_slot.observe(&self.ref_wf_locators),
            &ctx,
        );
        self.session = Some(s);
        self.phase = 1;
    }

    fn step_fork_barrier_kill(&mut self) {
        let mut s = self.session.take().unwrap();
        let pre_ctx = self.ctx(1, "pre-fork-kill", "kill_live");
        s.kill_live(&pre_ctx.to_string());
        let nth = 1 + (self.rng.range(12) as u64);
        let kill_build = self.rng.next_other_build(self.build);
        let op = format!("set:1:{}:{}", self.fork_days, KEY_FORK_MUT);
        let kill_ctx = StepCtx {
            build: kill_build,
            ..self.ctx(1, "fork-mutate-kill", format!("SynchronizeFileState#{nth}"))
        };
        kill_unified_barrier(
            &self.home_dir,
            &op,
            kill_build,
            nth,
            &self.coords_file,
            &kill_ctx.to_string(),
        );
        self.build = self.rng.next_other_build(kill_build);
        let ctx = self.ctx(
            1,
            "fork-mutate-reopen",
            format!("SynchronizeFileState#{nth}"),
        );
        let ch = self.next_ch("post-fork");
        let (mut s2, o2) =
            UnifiedSession::start(&self.home_dir, &ch, self.build, &self.durable_wf_locators);
        let snap2 = o2.snap.unwrap();
        let ref_before = self.ref_slot.observe(&self.ref_wf_locators);
        assert_eq!(
            snap2.branches[0].retention, self.main_days,
            "{ctx}: main survived"
        );
        if snap2.custody_count > 0 {
            assert_unified_parity(&mut self.map, &snap2, &ref_before, &ctx);
            let u = s2.send(UnifiedCmd::SetRetention {
                branch_idx: 1,
                days: self.fork_days,
                key: KEY_FORK_MUT,
            });
            assert_eq!(u.fate_tag.as_deref(), Some("product_unpublished"), "{ctx}");
            let cont = s2.send(UnifiedCmd::ContinueCustody);
            let _ = exec_set_retention(self.ref_slot.runtime(), 1, self.fork_days, KEY_FORK_MUT);
            assert_unified_parity(
                &mut self.map,
                cont.snap.as_ref().unwrap(),
                &self.ref_slot.observe(&self.ref_wf_locators),
                &ctx,
            );
        } else if snap2.branches[1].retention == ref_before.branches[1].retention {
            assert_unified_parity(&mut self.map, &snap2, &ref_before, &ctx);
            let _ = exec_set_retention(self.ref_slot.runtime(), 1, self.fork_days, KEY_FORK_MUT);
            let r = s2.send(UnifiedCmd::SetRetention {
                branch_idx: 1,
                days: self.fork_days,
                key: KEY_FORK_MUT,
            });
            assert_eq!(r.fate_tag.as_deref(), Some("committed"), "{ctx}");
            assert_unified_parity(
                &mut self.map,
                r.snap.as_ref().unwrap(),
                &self.ref_slot.observe(&self.ref_wf_locators),
                &ctx,
            );
        } else {
            let _ = exec_set_retention(self.ref_slot.runtime(), 1, self.fork_days, KEY_FORK_MUT);
            assert_unified_parity(
                &mut self.map,
                &snap2,
                &self.ref_slot.observe(&self.ref_wf_locators),
                &ctx,
            );
        }
        self.session = Some(s2);
        self.phase = 2;
    }
}

pub(super) fn run_unified_single_home_seed(seed: u64) {
    let mut stepper = UnifiedHomeStepper::new(seed);
    while !stepper.is_done() {
        stepper.step();
    }
}
