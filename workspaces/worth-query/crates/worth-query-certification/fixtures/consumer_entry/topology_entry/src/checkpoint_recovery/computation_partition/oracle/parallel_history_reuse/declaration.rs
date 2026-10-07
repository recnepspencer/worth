//! Independently declared full-run compute and tree work. Input/item identity
//! encoding belongs to preparation and is not separately observable here.
use super::*;
use worth_query_decl::facade::application_operation::application_computation_partition_identity;

pub(super) fn computed_work(model: &ReuseFacts) -> u64 {
    let identities = model
        .members()
        .map(|item| {
            application_computation_partition_identity(&RegionKey(item.key), &mut |_| {
                Ok::<_, ()>(())
            })
            .unwrap()
            .partition()
        })
        .collect::<std::collections::BTreeSet<_>>();
    let tree =
        worth_execution::ReductionPlan::try_from_sorted_unique(identities.into_iter().collect())
            .unwrap()
            .checked_build_work()
            .unwrap();
    model.expected_compute_charge(tree)
}
