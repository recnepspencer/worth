use serde::Serialize;
use std::collections::BTreeMap;

use crate::facade::{
    SignalBranchId, SignalRuntime, SignalRuntimePolicy, SignalSnapshotId, SignalSnapshotV1,
};

use worth_dst::{Seed, Trace};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) enum WorkflowDomain {
    GeometryKernel,
    Fintech,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) enum AdversarialWorkflow {
    FeatureEditRewireRestoreChurn,
    PartitionScopeCliffSession,
    LateTickCorrectionWithBranchReplay,
    RiskAlertFlapUnderMemoChurn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) enum FailureInjectionPoint {
    DuringEvaluation,
    DuringBranchLocalWork,
    DuringSnapshotRestoreChurn,
    DuringEventFlush,
}

#[derive(Clone, Debug, Serialize)]
struct WorkflowStep {
    index: usize,
    label: String,
    active_branch: String,
    failure_injection: Option<FailureInjectionPoint>,
}

#[derive(Clone, Debug, Serialize)]
struct DifferentialFailureBundle {
    domain: WorkflowDomain,
    workflow: AdversarialWorkflow,
    seed: u64,
    policy: String,
    executor: String,
    steps: Vec<WorkflowStep>,
    note: String,
    replay_diff_count: usize,
    lineage_diff_count: usize,
}

#[derive(Clone)]
pub(super) struct BranchTruth {
    pub(super) a: u64,
    pub(super) b: u64,
    pub(super) head_snapshot: Option<SignalSnapshotId>,
}

#[derive(Default, Clone)]
pub(super) struct ReferenceModel {
    pub(super) active: SignalBranchId,
    pub(super) branches: BTreeMap<SignalBranchId, BranchTruth>,
    pub(super) snapshots: BTreeMap<SignalSnapshotId, BranchTruth>,
}

impl ReferenceModel {
    pub(super) fn branch(&self, branch_id: SignalBranchId) -> &BranchTruth {
        self.branches.get(&branch_id).unwrap()
    }

    pub(super) fn branch_mut(&mut self, branch_id: SignalBranchId) -> &mut BranchTruth {
        self.branches.get_mut(&branch_id).unwrap()
    }
}

pub(super) fn capture_active_branch_snapshot<D, I, E, Ctx, T>(
    runtime: &mut SignalRuntime<D, I, E, Ctx, T>,
    model: &mut ReferenceModel,
    snapshots: &mut BTreeMap<SignalSnapshotId, SignalSnapshotV1>,
    history: &mut BTreeMap<SignalBranchId, Vec<SignalSnapshotId>>,
) -> SignalSnapshotV1
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    let branch = runtime.observe().current_branch();
    let snapshot = runtime.capture_branch_snapshot(branch.clone()).unwrap();
    let mut truth = model.branch(branch.id).clone();
    truth.head_snapshot = Some(snapshot.meta.snapshot_id);
    snapshots.insert(snapshot.meta.snapshot_id, snapshot.clone());
    history
        .entry(branch.id)
        .or_default()
        .push(snapshot.meta.snapshot_id);
    model.snapshots.insert(snapshot.meta.snapshot_id, truth);
    model.branch_mut(branch.id).head_snapshot = Some(snapshot.meta.snapshot_id);
    snapshot
}

pub(super) struct SignalAdversarialHarness {
    seed: Seed,
    domain: WorkflowDomain,
    workflow: AdversarialWorkflow,
    steps: Vec<WorkflowStep>,
    schedule: Trace,
}

pub(super) fn trace_adv(message: impl AsRef<str>) {
    if std::env::var_os("WORTH_SIGNAL_TRACE_ADV").is_some() {
        eprintln!("{}", message.as_ref());
    }
}

impl SignalAdversarialHarness {
    pub(super) fn new(seed: Seed, domain: WorkflowDomain, workflow: AdversarialWorkflow) -> Self {
        Self {
            seed,
            domain,
            workflow,
            steps: Vec::new(),
            schedule: Trace::default(),
        }
    }

    pub(super) fn record<D, I, E, Ctx, T>(
        &mut self,
        runtime: &SignalRuntime<D, I, E, Ctx, T>,
        index: usize,
        label: impl Into<String>,
        failure_injection: Option<FailureInjectionPoint>,
    ) where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        T: Copy + Ord,
    {
        self.steps.push(WorkflowStep {
            index,
            label: label.into(),
            active_branch: runtime.observe().current_branch().name,
            failure_injection,
        });
    }

    pub(super) fn set_schedule(&mut self, trace: Trace) {
        self.schedule = trace;
    }

    pub(super) fn schedule(&self) -> &[u8] {
        self.schedule.as_bytes()
    }

    pub(super) fn panic_invariant(
        &self,
        policy: &SignalRuntimePolicy,
        executor: &str,
        note: impl Into<String>,
    ) -> ! {
        let bundle = DifferentialFailureBundle {
            domain: self.domain,
            workflow: self.workflow,
            seed: self.seed.value(),
            policy: format!("{:?}", policy.tier),
            executor: executor.to_string(),
            steps: self.steps.clone(),
            note: note.into(),
            replay_diff_count: 0,
            lineage_diff_count: 0,
        };
        panic!("{}", serde_json::to_string_pretty(&bundle).unwrap());
    }

    pub(super) fn panic_diff(
        &self,
        policy: &SignalRuntimePolicy,
        executor: &str,
        note: impl Into<String>,
        replay_diff_count: usize,
        lineage_diff_count: usize,
    ) -> ! {
        let bundle = DifferentialFailureBundle {
            domain: self.domain,
            workflow: self.workflow,
            seed: self.seed.value(),
            policy: format!("{:?}", policy.tier),
            executor: executor.to_string(),
            steps: self.steps.clone(),
            note: note.into(),
            replay_diff_count,
            lineage_diff_count,
        };
        panic!("{}", serde_json::to_string_pretty(&bundle).unwrap());
    }
}
