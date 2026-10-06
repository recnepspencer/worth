mod market_world;
mod portfolio_world;
mod risk_world;

use std::sync::Arc;

use crate::facade::{
    LineageRecord, NodeId, NodeState, ReplaySlice, RuntimeMetrics, SignalBranchHandle, SignalError,
    SignalGraph, SignalRuntime, SignalRuntimePolicy, SignalSnapshotId, SignalSnapshotV1,
};

use super::branch_checkpoint::BranchCheckpoint;
use super::evaluation::FintechEvaluationShape;
use super::node_families::{AggregateSourceNodes, FintechRuntime, FxNodes, InstrumentNodes};
use super::scales::FintechScale;
use super::world::FinancialWorldDefinition;
use super::world_handles::FintechWorldHandles;

#[derive(Debug)]
pub(super) struct InstrumentFixture {
    pub instrument_index: usize,
    pub book_index: usize,
    pub core: InstrumentNodes,
    pub buckets: Vec<NodeId>,
    pub scenarios: Vec<NodeId>,
}

pub(crate) struct FintechWorld {
    pub(super) runtime: FintechRuntime,
    pub(super) evaluation: Arc<FintechEvaluationShape>,
    pub(super) handles: FintechWorldHandles,
    pub(super) fx: FxNodes,
    pub(super) aggregate_sources: Vec<AggregateSourceNodes>,
    pub(super) curve_buckets: Vec<NodeId>,
    pub(super) vol_surface_buckets: Vec<NodeId>,
    pub(super) scenario_sources: Vec<NodeId>,
    pub(super) instruments: Vec<InstrumentFixture>,
    pub(super) book_aggregates: Vec<NodeId>,
    pub(super) desk_aggregates: Vec<NodeId>,
    pub(super) scenario_aggregates: Vec<NodeId>,
    pub(super) bucket_aggregates: Vec<NodeId>,
    pub(super) financial_definition: FinancialWorldDefinition,
    pub(super) market_revision: u64,
}

pub(super) type FintechDomainFixture = FintechWorld;

impl FintechWorld {
    pub(crate) fn set_runtime_policy(&mut self, policy: SignalRuntimePolicy) {
        self.runtime.set_runtime_policy(policy);
    }

    pub(crate) fn runtime_metrics(&self) -> RuntimeMetrics {
        self.runtime.observe().metrics()
    }

    pub(super) fn live_node_count(&self) -> usize {
        self.runtime.graph().live_node_ids().len()
    }

    pub(super) fn open_branch(&mut self, name: &str) -> Result<SignalBranchHandle, SignalError> {
        super::branch_history::create_branch(self, name)
    }

    pub(super) fn current_branch(&self) -> SignalBranchHandle {
        self.runtime.observe().current_branch()
    }

    pub(super) fn switch_branch(&mut self, branch: SignalBranchHandle) -> Result<(), SignalError> {
        self.runtime.switch_branch(branch)
    }

    pub(super) fn branch_head_snapshot_id(
        &self,
        branch: SignalBranchHandle,
    ) -> Option<SignalSnapshotId> {
        self.runtime.observe().branch_head_snapshot_id(branch.id)
    }

    pub(super) fn capture_world_snapshot(&mut self) -> SignalSnapshotV1 {
        super::branch_history::capture_active_snapshot(self)
    }

    pub(super) fn capture_branch_snapshot(
        &mut self,
        branch: SignalBranchHandle,
    ) -> Result<SignalSnapshotV1, SignalError> {
        super::branch_history::capture_branch_snapshot(self, branch)
    }

    pub(super) fn restore_saved_snapshot(
        &mut self,
        branch: SignalBranchHandle,
        snapshot: &SignalSnapshotV1,
    ) -> Result<(), SignalError> {
        super::branch_history::restore_branch_snapshot(self, branch, snapshot)
    }

    pub(super) fn replay_for_branch(&self, branch: SignalBranchHandle) -> ReplaySlice {
        super::branch_history::replay_for_branch(self, branch)
    }

    pub(super) fn replay_around_saved_snapshot(&self, snapshot: &SignalSnapshotV1) -> ReplaySlice {
        super::branch_history::replay_around_snapshot(self, snapshot)
    }

    pub(super) fn main_risk_lineage(&self) -> Vec<LineageRecord> {
        super::branch_history::lineage_for_main_risk(self)
    }

    pub(super) fn assert_shape(&self, scale: FintechScale) {
        super::world_shape::assert_world_shape(self, scale)
    }

    pub(super) fn evaluation_shape(&self) -> Arc<FintechEvaluationShape> {
        Arc::clone(&self.evaluation)
    }

    pub(super) fn node_state(&self, node: NodeId) -> Result<NodeState, SignalError> {
        self.runtime.graph().get_state(node)
    }

    pub(super) fn capture_active_checkpoint(&mut self) -> Result<BranchCheckpoint, SignalError> {
        let branch = self.current_branch();
        let audit = self.read_primary_audit_surface()?;
        let snapshot = self.capture_branch_snapshot(branch.clone())?;
        Ok(BranchCheckpoint::new(branch, snapshot, audit))
    }

    pub(super) fn capture_active_checkpoint_with_workers(
        &mut self,
        workers: usize,
    ) -> Result<BranchCheckpoint, SignalError> {
        let branch = self.current_branch();
        let audit = self.read_primary_audit_surface_with_workers(workers)?;
        let snapshot = self.capture_branch_snapshot(branch.clone())?;
        Ok(BranchCheckpoint::new(branch, snapshot, audit))
    }

    pub(super) fn restore_checkpoint(
        &mut self,
        checkpoint: &BranchCheckpoint,
    ) -> Result<(), SignalError> {
        self.restore_saved_snapshot(checkpoint.branch.clone(), &checkpoint.snapshot)
    }

    pub(super) fn attempt_cross_branch_restore(
        &mut self,
        branch: SignalBranchHandle,
        snapshot: &SignalSnapshotV1,
    ) -> Result<(), SignalError> {
        self.restore_saved_snapshot(branch, snapshot)
    }

    pub(super) fn attempt_incompatible_profile_restore(
        &mut self,
        branch: SignalBranchHandle,
        snapshot: &SignalSnapshotV1,
    ) -> Result<(), SignalError> {
        let mut incompatible = snapshot.clone();
        incompatible.meta.core_storage_profile = "incompatible-fintech-test-profile".to_string();
        self.restore_saved_snapshot(branch, &incompatible)
    }
}

pub(super) fn build_fixture(definition: FinancialWorldDefinition) -> FintechWorld {
    let scale = definition.fixture_scale();
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .with_tiers::<super::execution_tier::FintechTier>()
        .build();
    runtime.set_runtime_policy(
        SignalRuntimePolicy::fintech()
            .with_history_limit(8)
            .with_detail_limit(4),
    );

    let market = market_world::build_market_world(&mut runtime, scale);
    let portfolio = portfolio_world::build_portfolio_world(&mut runtime, scale, &market);
    let risk = risk_world::build_risk_world(&mut runtime, scale, &portfolio);
    market_world::seed_partition_baseline(&mut runtime, market.partition.market_regions);

    let handles = FintechWorldHandles::new(
        portfolio.instruments[0].core.market,
        portfolio.instruments[0].core.threshold,
        portfolio.instruments[0].core.risk,
        portfolio.desk_aggregates[0],
        risk.scenario_aggregates[0],
        market.partition,
    );

    let evaluation = Arc::new(FintechEvaluationShape::from_parts(
        market.fx,
        portfolio.aggregate_sources.as_slice(),
        market.curve_buckets.as_slice(),
        market.vol_surface_buckets.as_slice(),
        market.scenario_sources.as_slice(),
        portfolio.instruments.as_slice(),
        portfolio.book_aggregates.as_slice(),
        portfolio.desk_aggregates.as_slice(),
        risk.scenario_aggregates.as_slice(),
        risk.bucket_aggregates.as_slice(),
        handles.partition,
    ));

    FintechWorld {
        runtime,
        evaluation,
        handles,
        fx: market.fx,
        aggregate_sources: portfolio.aggregate_sources,
        curve_buckets: market.curve_buckets,
        vol_surface_buckets: market.vol_surface_buckets,
        scenario_sources: market.scenario_sources,
        instruments: portfolio.instruments,
        book_aggregates: portfolio.book_aggregates,
        desk_aggregates: portfolio.desk_aggregates,
        scenario_aggregates: risk.scenario_aggregates,
        bucket_aggregates: risk.bucket_aggregates,
        financial_definition: definition,
        market_revision: 0,
    }
}
