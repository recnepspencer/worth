use std::num::NonZeroU64;
use std::sync::Arc;

use super::{structural_views::StructuralRelationView, CustomInvariantWorkMeter};
use crate::tests::support::{create_entity, create_relation, runtime_with_test_schema};
use crate::validation::data::CustomInvariantAccessContract;
use crate::validation::engine::state_view::InvariantStateView;

#[test]
fn adjacency_budget_charges_candidates_even_when_access_filters_every_relation() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    create_relation(&runtime, source, target, "edge");
    let observation = crate::validation::engine::InvariantObservation::committed(
        runtime.storage_access().current_edition(),
    );
    let state = InvariantStateView::new(
        observation.committed_partition_access(),
        runtime.current_version_id(),
    );
    let work = CustomInvariantWorkMeter::new(NonZeroU64::new(4).unwrap());
    let relations = StructuralRelationView::new(
        state,
        work.clone(),
        Arc::new(CustomInvariantAccessContract::default()),
    );
    assert!(relations
        .outgoing_relations_for_entity(source)
        .unwrap()
        .is_empty());
    assert_eq!(work.consumed().get(), 4);
    assert!(!work.exceeded());
    assert!(relations.outgoing_relations_for_entity(source).is_err());
    assert!(work.exceeded());
}

#[test]
fn rejected_adjacency_budget_never_consumes_the_denied_scan() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    create_relation(&runtime, source, target, "edge");
    let observation = crate::validation::engine::InvariantObservation::committed(
        runtime.storage_access().current_edition(),
    );
    let state = InvariantStateView::new(
        observation.committed_partition_access(),
        runtime.current_version_id(),
    );
    let work = CustomInvariantWorkMeter::new(NonZeroU64::new(3).unwrap());
    let relations = StructuralRelationView::new(
        state,
        work.clone(),
        Arc::new(CustomInvariantAccessContract::default()),
    );
    assert!(relations.outgoing_relations_for_entity(source).is_err());
    assert_eq!(work.consumed().get(), 1);
    assert!(work.exceeded());
    assert!(relations.entity_kind(source).is_none());
    assert_eq!(work.consumed().get(), 1);
}
