use std::collections::BTreeMap;

use crate::facade::*;

use super::audit_surface::PrimaryAuditSurface;
use super::fixture::FintechWorld;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FintechTruthSnapshot {
    pub primary_market: AspectVersion,
    pub primary_threshold: AspectVersion,
    pub primary_audit: PrimaryAuditSurface,
    pub rates_partition: AspectVersion,
    pub credit_partition: AspectVersion,
    pub rates_bucket_zero: AspectVersion,
    pub coarse_partition_book: AspectVersion,
    pub bucket_aggregates: Vec<AspectVersion>,
    pub scenario_aggregates: Vec<AspectVersion>,
    pub branch_heads: BTreeMap<String, Option<SignalSnapshotId>>,
    pub replays: BTreeMap<String, ReplaySlice>,
    pub lineages: BTreeMap<String, Vec<LineageRecord>>,
}

impl FintechTruthSnapshot {
    pub(super) fn capture_core(world: &mut FintechWorld) -> Result<Self, SignalError> {
        Self::capture(world, &[], &[], 1, 1)
    }

    pub(super) fn capture(
        world: &mut FintechWorld,
        named_branches: &[(&str, SignalBranchHandle)],
        replay_branches: &[(&str, SignalBranchHandle)],
        bucket_count: usize,
        scenario_count: usize,
    ) -> Result<Self, SignalError> {
        let mut branch_heads = BTreeMap::new();
        for (alias, branch) in named_branches {
            branch_heads.insert(
                (*alias).to_string(),
                world.branch_head_snapshot_id(branch.clone()),
            );
        }

        let mut replays = BTreeMap::new();
        for (alias, branch) in replay_branches {
            replays.insert(
                (*alias).to_string(),
                world.replay_for_branch(branch.clone()),
            );
        }

        let bucket_limit = bucket_count.min(world.bucket_aggregates.len());
        let scenario_limit = scenario_count.min(world.scenario_aggregates.len());
        let mut bucket_aggregates = Vec::with_capacity(bucket_limit);
        for index in 0..bucket_limit {
            bucket_aggregates.push(world.read_bucket_aggregate(index)?);
        }
        let mut scenario_aggregates = Vec::with_capacity(scenario_limit);
        for index in 0..scenario_limit {
            scenario_aggregates.push(world.read_scenario_aggregate(index)?);
        }

        Ok(Self {
            primary_market: world.read_primary_market_source()?,
            primary_threshold: world.read_primary_threshold()?,
            primary_audit: world.read_primary_audit_surface()?,
            rates_partition: world.read_rates_partition()?,
            credit_partition: world.read_credit_partition()?,
            rates_bucket_zero: world.read_rates_bucket_zero()?,
            coarse_partition_book: world.read_coarse_partition_book()?,
            bucket_aggregates,
            scenario_aggregates,
            branch_heads,
            replays,
            lineages: BTreeMap::new(),
        })
    }
}
