use std::collections::BTreeSet;
use std::sync::Arc;

use crate::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;
use crate::contracts::identity::data::primitives::PartitionId;
use crate::validation::custom_rule::CustomPreparationBudget;

pub(super) fn packet_partition_scope(
    semantic_plan: Option<RelationalSemanticPlan<'_>>,
    budget: Option<&CustomPreparationBudget>,
) -> Arc<[PartitionId]> {
    let admit = |bytes| budget.is_none_or(|budget| budget.try_plan_item(bytes));
    let mut touched = BTreeSet::new();
    if let Some(plan) = semantic_plan {
        let entry_bytes = (size_of::<PartitionId>() + 4 * size_of::<usize>()) as u64;
        if let Some(selection) = plan.selection() {
            if !admit(entry_bytes) {
                return Arc::from([]);
            }
            touched.insert(selection.entity().partition_id);
        }
        for intent in &plan.commands().merged_intents {
            if !admit(0) {
                break;
            }
            let visited = intent.try_visit_touched_partitions(&mut |partition| {
                let bytes = if touched.contains(&partition) {
                    0
                } else {
                    entry_bytes
                };
                if !admit(bytes) {
                    return Err(());
                }
                touched.insert(partition);
                Ok(())
            });
            if visited.is_err() {
                break;
            }
        }
    }
    if !admit((touched.len() as u64).saturating_mul(size_of::<PartitionId>() as u64)) {
        return Arc::from([]);
    }
    touched.into_iter().collect::<Vec<_>>().into()
}
