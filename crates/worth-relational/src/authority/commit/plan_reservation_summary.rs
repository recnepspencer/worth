//! Count bulk reservations from the selected merged plan.
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use std::collections::BTreeMap;

pub(crate) fn bulk_reservations_for_plan(plan: &MergedCommitPlan) -> (usize, usize) {
    let mut entity_requests = BTreeMap::new();
    let mut relation_requests = BTreeMap::new();
    for intent in &plan.merged_intents {
        if let Some((partition_id, requested)) = intent.bulk_entity_reservation() {
            *entity_requests.entry(partition_id).or_insert(0usize) += requested;
        }
        if let Some((partition_id, requested)) = intent.bulk_relation_reservation() {
            *relation_requests.entry(partition_id).or_insert(0usize) += requested;
        }
    }

    let entity_reserved = entity_requests.into_values().sum();
    let relation_reserved = relation_requests.into_values().sum();
    (entity_reserved, relation_reserved)
}
