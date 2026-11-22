//! A retained managed Source fact has no comparator observation: whoever
//! retained it reads the Source afresh, so it can never compare unchanged.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::ManagedSourceReadKey;
use worth_relational::facade::identity::{EntityId, PartitionId};

#[test]
fn retained_managed_source_fact_is_unbounded_for_the_comparator() {
    let key = WorthQueryApplicationFactKey::ManagedIndexSource(ManagedSourceReadKey {
        obligation: [1; 32],
        authorization: [2; 32],
        scope: EntityId::new(PartitionId::new(1), 1, 1),
        influence: None,
    });
    // An unbounded observation sends an accounted producer run in full; an
    // unaccounted run marks every call that read it. Neither carries it.
    assert_eq!(observation_work_bound(&key), None);
}
