use std::collections::BTreeSet;
use std::sync::Arc;

use crate::authority::commit::semantic_plan::RelationalSemanticPlan;

pub(super) fn packet_partition_scope(
    semantic_plan: Option<RelationalSemanticPlan<'_>>,
) -> Arc<[crate::identity::data::PartitionId]> {
    let mut touched = BTreeSet::new();
    if let Some(plan) = semantic_plan {
        if let Some(selection) = plan.selection() {
            touched.insert(selection.entity().partition_id);
        }
        for intent in &plan.commands().merged_intents {
            intent.seed_touched_partitions(&mut touched);
        }
    }
    touched.into_iter().collect::<Vec<_>>().into()
}

pub(super) fn packet_partition_scope_checked(
    semantic_plan: Option<RelationalSemanticPlan<'_>>,
    budget: &crate::validation::custom_rule::CustomPreparationBudget,
) -> Arc<[crate::identity::data::PartitionId]> {
    let mut touched = BTreeSet::new();
    if let Some(plan) = semantic_plan {
        if let Some(selection) = plan.selection() {
            let partition = selection.entity().partition_id;
            if !budget.try_plan_item(
                (std::mem::size_of::<crate::identity::data::PartitionId>()
                    + 4 * std::mem::size_of::<usize>()) as u64,
            ) {
                return Arc::from([]);
            }
            touched.insert(partition);
        }
        for intent in &plan.commands().merged_intents {
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
