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
    },
    Declared(ReductionPlan),
}
impl From<ReductionPlan> for TreeTarget {
    fn from(plan: ReductionPlan) -> Self {
        Self::Declared(plan)
    }
}
impl TreeTarget {
    pub(super) fn new<Key, Item, Reduced, Stopped>(
        retained: &RetainedPartitions<Key, Item, Reduced>,
        routing: Arc<ComputationPartitionRouting>,
        removed: CanonicalUniqueVec<PartitionIdentity>,
        identities_unchanged: bool,
        results: &std::collections::BTreeMap<PartitionIdentity, Reduced>,
        execution: &super::super::super::request_execution::QueryRequestExecution<'_>,
    ) -> Result<Self, super::super::WorthQueryPartitionedComputationDenial<Stopped>>
    where
        Reduced: Clone + worth_execution::ChargedBytes + worth_execution::CanonicalBits,
    {
        use super::super::WorthQueryPartitionedComputationDenial as D;
        let work = if identities_unchanged {
            retained.tree.checked_full_build_cost().map(|cost| cost.0)
        } else {
            let bytes = results.len().checked_mul(size_of::<PartitionIdentity>())
                .and_then(|n|u64::try_from(n).ok()).ok_or(D::Resource(super::super::super::WorthQueryManagedComputationResourceDenial::CapacityOverflow))?;
            let _memory = execution.reserve(bytes).map_err(D::Resource)?;
            let inserted = CanonicalUniqueVec::try_from_sorted_unique(
                results
                    .keys()
                    .copied()
                    .filter(|id| retained.tree.leaf(*id).is_none())
                    .collect(),
            )
            .expect("changed native results are canonical");
            Some(
                retained
                    .tree
                    .checked_full_build_cost_after_edits(
                        &removed,
                        &inserted,
                        execution.allocation_policy().map_err(D::Resource)?,
                    )
                    .map_err(D::from_shape_cost)?
                    .0,
            )
        };
        Ok(Self::Current {
            routing,
            removed,
            work,
        })
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
            Self::Declared(plan) => plan,
            Self::Current { routing, .. } => plan(&routing),
        }
    }
}
fn plan(routing: &ComputationPartitionRouting) -> ReductionPlan {
    ReductionPlan::from_canonical(CanonicalUniqueVec::try_from_sorted_unique(routing.partitions().inspect(|_| super::super::super::super::fact_verification_observation::computation_tree_plan()).collect()).expect("native routing is canonical"))
}
