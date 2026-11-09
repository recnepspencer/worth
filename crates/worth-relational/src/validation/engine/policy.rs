use crate::validation::data::{InvariantCostClass, InvariantExecutionPoint, InvariantGroupSet};
use crate::validation::engine::InvariantRuntimeView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InvariantScale {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InvariantContext {
    pub scale: InvariantScale,
    pub version_depth: usize,
    pub snapshot_pressure: bool,
}

pub(crate) fn derive_invariant_context(runtime: &InvariantRuntimeView) -> InvariantContext {
    let (entity_count, relation_count, version_depth, snapshot_pressure) =
        runtime.invariant_scale_inputs();
    let total_records = entity_count + relation_count;

    let scale = match total_records {
        0..=1_000 => InvariantScale::Small,
        1_001..=100_000 => InvariantScale::Medium,
        _ => InvariantScale::Large,
    };

    InvariantContext {
        scale,
        version_depth,
        snapshot_pressure,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RelationalInvariantRuntime {
    skip_mask: u32,
    deferred_mask: u32,
    run_at: [u32; InvariantExecutionPoint::COUNT],
    max_cost: [InvariantCostClass; InvariantExecutionPoint::COUNT],
}

impl RelationalInvariantRuntime {
    pub(crate) fn resolve(
        profile: super::profile::InvariantRequestProfile,
        context: InvariantContext,
    ) -> Self {
        let mut run_at = [0u32; InvariantExecutionPoint::COUNT];
        let mut max_cost = [InvariantCostClass::Global; InvariantExecutionPoint::COUNT];
        let graph_composition_execution_point =
            super::profile::InvariantRequestProfile::GraphComposition.execution_point();

        run_at[InvariantExecutionPoint::ProposedState as usize] = profile.consumed_groups().mask();
        run_at[InvariantExecutionPoint::CommitBoundary as usize] = profile.consumed_groups().mask();
        run_at[InvariantExecutionPoint::MutationSensitive as usize] =
            profile.consumed_groups().mask();
        run_at[graph_composition_execution_point as usize] = profile.consumed_groups().mask();
        run_at[InvariantExecutionPoint::SnapshotPublication as usize] =
            profile.consumed_groups().mask();
        run_at[InvariantExecutionPoint::CertificationBoundary as usize] =
            profile.consumed_groups().mask();
        run_at[InvariantExecutionPoint::HarnessAudit as usize] = profile.consumed_groups().mask();

        // Commit-boundary cost is not a legality budget.  A blocking
        // registration may be Global even when the runtime is large; request
        // admission treats blocking registrations as required and never
        // silently filters them by this ceiling.
        max_cost[InvariantExecutionPoint::CommitBoundary as usize] = InvariantCostClass::Global;
        // Global uniqueness remains a correctness check until a
        // branch-qualified authoritative index exists. Mutation-sensitive
        // validation therefore admits its selected-state scan explicitly.
        max_cost[InvariantExecutionPoint::MutationSensitive as usize] = InvariantCostClass::Global;
        max_cost[graph_composition_execution_point as usize] = InvariantCostClass::Touched;
        max_cost[InvariantExecutionPoint::SnapshotPublication as usize] =
            if context.snapshot_pressure || matches!(context.scale, InvariantScale::Large) {
                InvariantCostClass::Partition
            } else {
                InvariantCostClass::Global
            };
        max_cost[InvariantExecutionPoint::CertificationBoundary as usize] =
            InvariantCostClass::Global;
        max_cost[InvariantExecutionPoint::HarnessAudit as usize] = InvariantCostClass::Global;

        Self {
            skip_mask: 0,
            deferred_mask: 0,
            run_at,
            max_cost,
        }
    }

    #[inline]
    pub(crate) fn should_run(
        &self,
        groups: InvariantGroupSet,
        checkpoint: InvariantExecutionPoint,
    ) -> bool {
        let allowed = self.run_at[checkpoint as usize] & !self.skip_mask & !self.deferred_mask;
        InvariantGroupSet::from_mask(allowed).intersects(groups)
    }

    #[inline]
    pub(crate) fn max_cost_at(&self, checkpoint: InvariantExecutionPoint) -> InvariantCostClass {
        self.max_cost[checkpoint as usize]
    }
}

pub(crate) const fn cost_rank(cost: InvariantCostClass) -> u8 {
    match cost {
        InvariantCostClass::Touched => 0,
        InvariantCostClass::Partition => 1,
        InvariantCostClass::Global => 2,
    }
}

pub(crate) const fn cost_allowed(limit: InvariantCostClass, cost: InvariantCostClass) -> bool {
    cost_rank(cost) <= cost_rank(limit)
}

#[cfg(test)]
mod tests {
    use super::{cost_allowed, InvariantContext, InvariantScale, RelationalInvariantRuntime};
    use crate::validation::data::{InvariantCostClass, InvariantExecutionPoint};
    use crate::validation::engine::InvariantRequestProfile;

    #[test]
    fn large_runtime_context_preserves_global_commit_boundary_ceiling() {
        let runtime = RelationalInvariantRuntime::resolve(
            InvariantRequestProfile::CommitBoundary,
            InvariantContext {
                scale: InvariantScale::Large,
                version_depth: 2_000,
                snapshot_pressure: true,
            },
        );

        assert_eq!(
            runtime.max_cost_at(InvariantExecutionPoint::CommitBoundary),
            InvariantCostClass::Global
        );
    }

    #[test]
    fn graph_composition_profile_is_touched_cost_and_topology_scoped() {
        let runtime = RelationalInvariantRuntime::resolve(
            InvariantRequestProfile::GraphComposition,
            InvariantContext {
                scale: InvariantScale::Large,
                version_depth: 2_000,
                snapshot_pressure: true,
            },
        );

        assert_eq!(
            runtime.max_cost_at(InvariantExecutionPoint::GraphComposition),
            InvariantCostClass::Touched
        );
    }

    #[test]
    fn cost_ordering_is_touched_then_partition_then_global() {
        assert!(cost_allowed(
            InvariantCostClass::Partition,
            InvariantCostClass::Touched
        ));
        assert!(cost_allowed(
            InvariantCostClass::Global,
            InvariantCostClass::Partition
        ));
        assert!(!cost_allowed(
            InvariantCostClass::Touched,
            InvariantCostClass::Global
        ));
    }
}
