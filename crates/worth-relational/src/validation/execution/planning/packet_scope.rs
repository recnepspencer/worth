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
) -> Option<Arc<[crate::identity::data::PartitionId]>> {
    use crate::identity::data::PartitionId;
    use crate::storage::substrate::SharedMap;
    let mut touched = SharedMap::<PartitionId, ()>::new();
    if let Some(plan) = merged_plan {
        for intent in &plan.merged_intents {
            if !budget.try_plan_item(0) {
                return None;
            }
            intent
                .try_visit_touched_partitions(&mut |partition| {
                    if !budget.try_plan_item(0) {
                        return Err(());
                    }
                    if !touched.contains_key(&partition) {
                        if !budget
                            .try_claim_bytes(SharedMap::<PartitionId, ()>::entry_allocation_bytes())
                        {
                            return Err(());
                        }
                        let copied = touched.insert(partition, ());
                        debug_assert_eq!(copied, 0);
                    }
                    Ok(())
                })
                .ok()?;
        }
    }
    let bytes = (touched.len() * std::mem::size_of::<PartitionId>()) as u64;
    if !budget.try_claim_bytes(
        bytes.saturating_add(super::retained_bytes::arc_slice_bytes::<PartitionId>(
            touched.len(),
        )),
    ) {
        return None;
    }
    let entries = touched
        .try_iter(|work, bytes| {
            if budget.try_visit(work as usize) && budget.try_claim_bytes(bytes) {
                Ok(())
            } else {
                Err(())
            }
        })
        .ok()?;
    let mut partitions = Vec::with_capacity(touched.len());
    for (partition, _) in entries {
        if !budget.try_visit(1) {
            return None;
        }
        partitions.push(*partition);
    }
    Some(partitions.into())
}
