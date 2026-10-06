use std::collections::BTreeSet;
use std::sync::Arc;

use crate::transactions::data::MergedCommitPlan;

pub(super) fn packet_partition_scope(
    merged_plan: Option<&MergedCommitPlan>,
) -> Arc<[crate::identity::data::PartitionId]> {
    let mut touched = BTreeSet::new();
    if let Some(plan) = merged_plan {
        for intent in &plan.merged_intents {
            intent.seed_touched_partitions(&mut touched);
        }
    }
    touched.into_iter().collect::<Vec<_>>().into()
}

pub(super) fn packet_partition_scope_checked(
    merged_plan: Option<&MergedCommitPlan>,
    budget: &crate::validation::custom_rule::CustomPreparationBudget,
) -> Arc<[crate::identity::data::PartitionId]> {
    let mut touched = BTreeSet::new();
    if let Some(plan) = merged_plan {
        for intent in &plan.merged_intents {
            if !budget.try_plan_item(0) {
                break;
            }
            let _ = intent.try_visit_touched_partitions(&mut |partition| {
                if !budget.try_plan_item(if touched.contains(&partition) {
                    0
                } else {
                    (std::mem::size_of::<crate::identity::data::PartitionId>()
                        + 4 * std::mem::size_of::<usize>()) as u64
                }) {
                    return Err(());
                }
                touched.insert(partition);
                Ok(())
            });
            if budget.stop().is_some() {
                break;
            }
        }
    }
    let bytes = (touched.len() as u64)
        .saturating_mul(std::mem::size_of::<crate::identity::data::PartitionId>() as u64);
    if !budget.try_plan_item(bytes) {
        return Arc::from([]);
    }
    touched.into_iter().collect::<Vec<_>>().into()
}
