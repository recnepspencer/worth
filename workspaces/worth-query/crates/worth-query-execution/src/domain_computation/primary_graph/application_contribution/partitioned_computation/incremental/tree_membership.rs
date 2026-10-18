//! Current routing delta versus an explicitly supplied full tree plan.
use super::super::routing::ComputationPartitionRouting;
use super::retained::RetainedPartitions;
use std::sync::Arc;
use worth_execution::ReductionPlan;
use worth_foundational::facade::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;

pub(super) enum TreeTarget {
    Current {
        routing: Arc<ComputationPartitionRouting>,
        removed: CanonicalUniqueVec<PartitionIdentity>,
        work: Option<u64>,
        full: Option<ReductionPlan>,
    },
    Declared(ReductionPlan),
}
impl From<ReductionPlan> for TreeTarget {
    fn from(plan: ReductionPlan) -> Self {
        Self::Declared(plan)
    }
}
impl TreeTarget {
    pub(super) fn new<Key, Item, Reduced>(
        retained: &RetainedPartitions<Key, Item, Reduced>,
        routing: Arc<ComputationPartitionRouting>,
        removed: CanonicalUniqueVec<PartitionIdentity>,
        identities_unchanged: bool,
    ) -> Self {
        // This condition is derived from actual removals and changed native
        // slot lookups. It establishes only the tree's shape, never freshness.
        let full = (!identities_unchanged).then(|| plan(&routing));
        let work = match &full {
            Some(plan) => plan.checked_build_work(),
            None => retained.tree.checked_full_build_cost().map(|cost| cost.0),
        };
        Self::Current {
            routing,
            removed,
            work,
            full,
        }
    }
    pub(super) fn full_work(&self) -> Option<u64> {
        match self {
            Self::Current { work, .. } => *work,
            Self::Declared(plan) => plan.checked_build_work(),
        }
    }
    pub(super) fn removed<Key, Item, Reduced>(
        &self,
        retained: &RetainedPartitions<Key, Item, Reduced>,
    ) -> CanonicalUniqueVec<PartitionIdentity> {
        match self {
            Self::Current { removed, .. } => removed.clone(),
            // Explicit arbitrary plans remain honestly full. The ordinary
            // routing-delta path never discovers deletion through this walk.
            Self::Declared(plan) => CanonicalUniqueVec::try_from_sorted_unique(
                retained
                    .partitions
                    .keys()
                    .filter(|id| plan.identities().binary_search(id).is_err())
                    .copied()
                    .collect(),
            )
            .expect("retained keys are canonical"),
        }
    }
    pub(super) fn into_plan(self) -> ReductionPlan {
        match self {
            Self::Declared(plan)
            | Self::Current {
                full: Some(plan), ..
            } => plan,
            Self::Current { routing, .. } => plan(&routing),
        }
    }
}
fn plan(routing: &ComputationPartitionRouting) -> ReductionPlan {
    ReductionPlan::from_canonical(CanonicalUniqueVec::try_from_sorted_unique(routing.partitions().inspect(|_| super::super::super::super::fact_verification_observation::computation_tree_plan()).collect()).expect("native routing is canonical"))
}
