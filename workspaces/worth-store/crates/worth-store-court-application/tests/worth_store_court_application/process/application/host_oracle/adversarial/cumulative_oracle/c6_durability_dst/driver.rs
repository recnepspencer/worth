//! Main seeded scheduler and exact branch/payload/retention parity assertions
//! for the C6 Store Durability DST.
use super::super::SplitMix64;
use super::faults::{open_world_home_with_faults, DstFaultHandles};
use super::observe::{
    observe_ref_entity_payloads, observe_root_entity_payloads, verify_branches_and_ancestry,
    ExpectedBranch,
};
use super::tla_oracle::{BuildTag, DerivedState, ForkPhase, TlaAction, TlaOracle, MAX_COMMITS};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use worth_execution::ExecutionRequest;
use worth_query_testkit_host::facade::host::checkpoint_capture;
use worth_relational::facade::{history::BranchId, runtime::RelationalRuntime};
use worth_runtime_world::facade::CompositeCommitIdentity;
use worth_store_testkit_application_oracle::world_home::WorldHomeFixture;

const STEPS_PER_SEED: usize = 16;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct DstObservedStats {
    pub steps: usize,
    pub fault_write_fired: usize,
    pub fault_indeterminate_fired: usize,
    pub fault_checkpoint_fired: usize,
    pub fault_fork_fired: usize,
    pub fault_recovery_staging_fired: usize,
    pub commits_retired: usize,
    pub d14_blocks_verified: usize,
}

impl DstObservedStats {
    pub(super) fn merge(&mut self, other: Self) {
        self.steps += other.steps;
        self.fault_write_fired += other.fault_write_fired;
        self.fault_indeterminate_fired += other.fault_indeterminate_fired;
        self.fault_checkpoint_fired += other.fault_checkpoint_fired;
        self.fault_fork_fired += other.fault_fork_fired;
        self.fault_recovery_staging_fired += other.fault_recovery_staging_fired;
        self.commits_retired += other.commits_retired;
        self.d14_blocks_verified += other.d14_blocks_verified;
    }
}

pub(super) struct StoreDurabilityDst {
    pub seed: u64,
    pub rng: SplitMix64,
    _root: tempfile::TempDir,
    pub home_dir: PathBuf,
    pub fixture: Option<WorldHomeFixture>,
    pub faults: Option<DstFaultHandles>,
    pub oracle: TlaOracle,
    pub ref_runtime: RelationalRuntime,
    pub last_root_ordinal: u64,
    pub epoch_ordinals: BTreeMap<u64, u64>,
    pub retired_ordinals: BTreeSet<u64>,
    pub live_branches: BTreeMap<String, ExpectedBranch>,
    pub pending_fork: Option<(String, String, bool, bool, ExpectedBranch)>,
    pub next_round: usize,
    pub next_fork_seq: usize,
    pub step_idx: usize,
    pub stats: DstObservedStats,
}

impl StoreDurabilityDst {
    pub(super) fn new(seed: u64, record_trace: bool) -> Self {
        let mut rng = SplitMix64::new(seed ^ 0xC6D5_7001);
        let root = tempfile::tempdir().expect("create dst tempdir");
        let home_dir = root.path().join("home");
        let lease = checkpoint_capture::lease(128 << 20);
        let exec = ExecutionRequest::leased(&lease);
        let sync_nth = 1 + (rng.range(2) as u64);
        let (fixture, faults) =
            open_world_home_with_faults(&home_dir, BuildTag::B1, sync_nth, &exec);
        let boot = fixture.bootstrap_root(&exec);
        let boot_ord = boot.product_branch().selected_commit().ordinal();
        drop(boot);
        let ref_runtime = crate::workload::runtime_builder(crate::schema_transition::SOURCE)
            .build()
            .expect("valid MemoryOnly Relational reference runtime");
        let boot_payloads = observe_ref_entity_payloads(&ref_runtime, &exec);
        let mut epoch_ordinals = BTreeMap::new();
        epoch_ordinals.insert(0, boot_ord);
        let mut live_branches = BTreeMap::new();
        live_branches.insert(
            "Original World Root".to_owned(),
            ExpectedBranch {
                ordinal: boot_ord,
                payloads: boot_payloads,
            },
        );
        Self {
            seed,
            rng,
            _root: root,
            home_dir,
            fixture: Some(fixture),
            faults: Some(faults),
            oracle: TlaOracle::new(record_trace),
            ref_runtime,
            last_root_ordinal: boot_ord,
            epoch_ordinals,
            retired_ordinals: BTreeSet::new(),
            live_branches,
            pending_fork: None,
            next_round: 0,
            next_fork_seq: 0,
            step_idx: 0,
            stats: DstObservedStats::default(),
        }
    }

    pub(super) fn mirror_ref_commit(&self, round: usize, exec: &ExecutionRequest<'_, '_>) {
        let main = BranchId("main".to_owned());
        crate::workload::commit(
            &self.ref_runtime,
            exec,
            &main,
            crate::batches::create(round),
        )
        .expect("MemoryOnly reference commit must succeed");
    }

    pub(super) fn ctx(&self, op: &str) -> String {
        format!(
            "seed={},step={},op={},build={}",
            self.seed,
            self.step_idx,
            op,
            self.oracle.state.build_id.as_tla()
        )
    }

    pub(super) fn next_other_build(&mut self) -> BuildTag {
        let others = self.oracle.state.build_id.others();
        *self.rng.choose(&others)
    }

    pub(super) fn expected_root_retained_ordinals(&self) -> Vec<u64> {
        self.epoch_ordinals
            .values()
            .copied()
            .filter(|ord| !self.retired_ordinals.contains(ord))
            .collect()
    }

    /// Observe the real Store retention boundary after history retirement and
    /// advance the oracle `retainedFloor` strictly from Store's observed state.
    /// Scans across ALL tracked epochs (`0..=durable_head`) BEFORE oracle bound
    /// enforcement so any unsafe retirement of the sealed checkpoint epoch fails
    /// the TLA+ oracle immediately.
    pub(super) fn observe_and_record_retirement(
        &mut self,
        reclaimed: Vec<CompositeCommitIdentity>,
        actual_retained_ords: Vec<u64>,
        ctx: &str,
    ) {
        let live_head_ords: BTreeSet<u64> =
            self.live_branches.values().map(|b| b.ordinal).collect();
        for c in &reclaimed {
            let ord = c.ordinal();
            assert!(
                !live_head_ords.contains(&ord),
                "{ctx}: retire_changed_history reclaimed a live branch head ordinal {ord}"
            );
            self.retired_ordinals.insert(ord);
        }
        self.stats.commits_retired += reclaimed.len();

        let retained_set: BTreeSet<u64> = actual_retained_ords.iter().copied().collect();
        for c in &reclaimed {
            assert!(
                !retained_set.contains(&c.ordinal()),
                "{ctx}: reclaimed commit ordinal {} still reported in root ancestry",
                c.ordinal()
            );
        }
        assert_eq!(
            actual_retained_ords,
            self.expected_root_retained_ordinals(),
            "{ctx}: Store retained root ordinals disagree with observed reclaimed history"
        );

        let mut observed_prefix_floor = 0u64;
        for ep in 0..=self.oracle.state.durable_head {
            let Some(&ord) = self.epoch_ordinals.get(&ep) else {
                break;
            };
            if !retained_set.contains(&ord) {
                observed_prefix_floor = ep + 1;
            } else {
                break;
            }
        }
        if observed_prefix_floor > self.oracle.state.retained_floor {
            self.oracle.apply_and_verify(
                TlaAction::RetireHistoryPrefix {
                    new_floor: observed_prefix_floor,
                },
                ctx,
            );
        }
    }

    pub(super) fn verify_live_parity(&self, exec: &ExecutionRequest<'_, '_>, ctx: &str) {
        let fx = self.fixture.as_ref().expect("live fixture");
        let actual_payloads = observe_root_entity_payloads(fx, exec, ctx);
        let expected_payloads = observe_ref_entity_payloads(&self.ref_runtime, exec);
        assert_eq!(
            actual_payloads, expected_payloads,
            "{ctx}: Relational root entity payload parity mismatch vs MemoryOnly reference"
        );
        let expected_retained_ords = self.expected_root_retained_ordinals();
        verify_branches_and_ancestry(
            fx,
            exec,
            &expected_retained_ords,
            !self.retired_ordinals.is_empty(),
            &self.live_branches,
            ctx,
        );
    }

    fn run(&mut self) {
        let lease = checkpoint_capture::lease(128 << 20);
        let exec = ExecutionRequest::leased(&lease);

        self.op_clean_commit(&exec);

        for idx in 1..=STEPS_PER_SEED {
            self.step_idx = idx;
            let s = self.oracle.state;
            let mut choices: Vec<u8> = Vec::with_capacity(8);
            if s.durable_head + 1 < MAX_COMMITS {
                choices.push(0);
                choices.push(1);
            }
            let can_cp = s.derived_state != DerivedState::DiscardedBlocked
                && (s.checkpoint_epoch < s.durable_head
                    || (s.derived_state == DerivedState::DiscardedReopened
                        && s.checkpoint_epoch > 0));
            if can_cp {
                choices.push(2);
            }
            if s.retained_floor < s.checkpoint_epoch && s.checkpoint_epoch == s.durable_head {
                choices.push(3);
            }
            if s.derived_state == DerivedState::Covered && s.checkpoint_epoch >= 1 {
                choices.push(4);
            }
            if s.durable_head >= 1 && matches!(s.fork_phase, ForkPhase::Idle | ForkPhase::Published)
            {
                choices.push(5);
            }
            choices.push(6);
            choices.push(7);

            let choice = *self.rng.choose(&choices);
            match choice {
                0 => self.op_clean_commit(&exec),
                1 => self.op_fault_commit_and_recover(&exec),
                2 => self.op_checkpoint_sealed_or_crashed(&exec),
                3 => self.op_standalone_retire(&exec),
                4 => self.op_d14_discard(&exec),
                5 => self.op_durable_fork(&exec),
                6 => self.op_graceful_reopen(&exec),
                7 => self.op_quiescent_crash_and_recover(&exec),
                _ => unreachable!(),
            }
        }

        self.step_idx = STEPS_PER_SEED + 1;
        self.op_graceful_reopen(&exec);
        if let Some(fx) = self.fixture.take() {
            let _ = fx.home.close(&exec);
        }
    }
}

pub(super) fn run_durability_dst_seed(seed: u64, trace_dir: Option<&Path>) -> DstObservedStats {
    let mut dst = StoreDurabilityDst::new(seed, trace_dir.is_some());
    dst.run();
    if let Some(dir) = trace_dir {
        dst.oracle
            .write_ndjson(&dir.join(format!("seed_{seed}.ndjson")));
    }
    dst.stats.steps = dst.oracle.step_count();
    dst.stats
}
