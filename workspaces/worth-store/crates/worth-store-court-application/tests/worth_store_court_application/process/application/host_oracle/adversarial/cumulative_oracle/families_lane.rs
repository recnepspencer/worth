//! Seeded cumulative adversarial runner for the 4 remaining M3 owner families:
//! 1. Shared `DocumentRetentionSchema` Home mixing BOTH Governed Workflow
//!    (`CorrectableRetentionBinding` -> D14 -> `Historical`) AND Branch Program
//!    Adoption (`P0 -> P1` -> D14) on ONE durable cumulative Home,
//! 2. Authored Correction (`reverse(&source)` with exact original-key fate &
//!    `ReceiptId` parity across D14),
//! 3. External Dispatch Custody (`NotifyAccountBalance`, <=1 / exact 1 dispatch
//!    & exact `ReceiptId` parity across D14).
//!
//! All three durable Homes advance in PRNG-interleaved steps across fresh OS
//! child processes, PRNG-selected `SynchronizeFileState` barrier cuts, all 3
//! alternating build identities, and D14 derived-state discards (`covered == 0`
//! on all 5 cadence journals).
use super::super::{assert_barrier_killed, BUILD_PRIMARY};
use super::families_child::{
    run_reopen_phase, FamilyKind, ENV_BUILD, ENV_CUT, ENV_EXTRA, ENV_FAMILY, ENV_HOME, ENV_MARKERS,
    ENV_PHASE, KILL_CHILD,
};
use super::{SplitMix64, StepCtx};
use std::path::{Path, PathBuf};
use worth_store::physical_runtime::certification::MediaOperationRole;

fn run_kill_barrier(
    family: FamilyKind,
    phase: &str,
    home: &Path,
    build: &str,
    nth: u64,
    extra: &Path,
    ctx: &StepCtx,
) {
    let markers = tempfile::tempdir().unwrap();
    let fam_s = PathBuf::from(family.as_str());
    let phase_s = PathBuf::from(phase);
    let nth_s = PathBuf::from(nth.to_string());
    let ready = markers.path().join("ready");
    let end = crate::killed_child::run_and_kill_with_ctx(
        KILL_CHILD,
        &[
            (ENV_FAMILY, &fam_s),
            (ENV_PHASE, &phase_s),
            (ENV_HOME, home),
            (ENV_MARKERS, markers.path()),
            (ENV_BUILD, Path::new(build)),
            ("WORTH_ADV_WF_BUILD", Path::new(build)),
            (ENV_CUT, &nth_s),
            (ENV_EXTRA, extra),
        ],
        &ready,
        &markers.path().join("finished"),
        &ctx.to_string(),
    );
    assert_barrier_killed(
        end,
        &ready,
        MediaOperationRole::SynchronizeFileState,
        nth,
        &ctx.to_string(),
    );
}

enum FamAction {
    Phase(&'static str, &'static str),
    Kill(&'static str, usize),
}

struct FamilyRunner {
    kind: FamilyKind,
    _root: tempfile::TempDir,
    home: PathBuf,
    extra: PathBuf,
    build: &'static str,
    steps: Vec<FamAction>,
    cursor: usize,
}

impl FamilyRunner {
    fn new(kind: FamilyKind, steps: Vec<FamAction>) -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let extra = root.path().join("extra.dat");
        Self {
            kind,
            _root: root,
            home,
            extra,
            build: BUILD_PRIMARY,
            steps,
            cursor: 0,
        }
    }

    fn done(&self) -> bool {
        self.cursor >= self.steps.len()
    }
}

pub(super) struct FamiliesStepper {
    seed: u64,
    rng: SplitMix64,
    runners: Vec<FamilyRunner>,
    global_step: usize,
}

impl FamiliesStepper {
    pub(super) fn new(seed: u64) -> Self {
        let rng = SplitMix64::new(seed ^ 0x4641_4D53);
        let runners = vec![
            FamilyRunner::new(
                FamilyKind::WorkflowAdoption,
                vec![
                    FamAction::Phase("wf_seed", "retained"),
                    FamAction::Kill("kill_wf", 20),
                    FamAction::Phase("wf_reopen_discard_kill", "d14-discarded"),
                    FamAction::Phase("wf_after_discard_to_historical", "d14-discarded"),
                    FamAction::Kill("kill_adopt", 20),
                    FamAction::Phase("adopt_reopen_discard_kill", "d14-discarded"),
                    FamAction::Phase("adopt_final_verify", "d14-discarded"),
                ],
            ),
            FamilyRunner::new(
                FamilyKind::Correction,
                vec![
                    FamAction::Phase("seed", "retained"),
                    FamAction::Kill("kill_corr", 20),
                    FamAction::Phase("reopen_discard_kill", "d14-discarded"),
                    FamAction::Phase("final_verify", "d14-discarded"),
                ],
            ),
            FamilyRunner::new(
                FamilyKind::Dispatch,
                vec![
                    FamAction::Phase("seed", "retained"),
                    FamAction::Kill("kill_disp", 16),
                    FamAction::Phase("reopen_discard_kill", "d14-discarded"),
                    FamAction::Phase("final_verify", "d14-discarded"),
                ],
            ),
        ];
        Self {
            seed,
            rng,
            runners,
            global_step: 0,
        }
    }

    pub(super) fn is_done(&self) -> bool {
        self.runners.iter().all(FamilyRunner::done)
    }

    pub(super) fn step(&mut self) {
        let active: Vec<usize> = self
            .runners
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.done())
            .map(|(i, _)| i)
            .collect();
        if active.is_empty() {
            return;
        }
        let pick = *self.rng.choose(&active);
        let r = &mut self.runners[pick];
        let step_idx = r.cursor;
        r.cursor += 1;
        if step_idx > 0 {
            r.build = self.rng.next_other_build(r.build);
        }
        self.global_step += 1;
        match r.steps[step_idx] {
            FamAction::Phase(phase, derived) => {
                let cut = if phase.ends_with("_discard_kill") {
                    "kill_live"
                } else {
                    "none"
                };
                let ctx = StepCtx {
                    seed: self.seed,
                    step: self.global_step,
                    family: r.kind.as_str(),
                    op: phase.to_owned(),
                    cut: cut.to_owned(),
                    build: r.build,
                    derived,
                };
                run_reopen_phase(r.kind, phase, &r.home, r.build, &r.extra, &ctx);
            }
            FamAction::Kill(phase, max_cut) => {
                let nth = 1 + (self.rng.range(max_cut) as u64);
                let ctx = StepCtx {
                    seed: self.seed,
                    step: self.global_step,
                    family: r.kind.as_str(),
                    op: phase.to_owned(),
                    cut: format!("SynchronizeFileState#{nth}"),
                    build: r.build,
                    derived: "retained",
                };
                run_kill_barrier(r.kind, phase, &r.home, r.build, nth, &r.extra, &ctx);
            }
        }
    }
}

pub(super) fn run_remaining_families_cumulative_seed(seed: u64) {
    let mut stepper = FamiliesStepper::new(seed);
    while !stepper.is_done() {
        stepper.step();
    }
}
