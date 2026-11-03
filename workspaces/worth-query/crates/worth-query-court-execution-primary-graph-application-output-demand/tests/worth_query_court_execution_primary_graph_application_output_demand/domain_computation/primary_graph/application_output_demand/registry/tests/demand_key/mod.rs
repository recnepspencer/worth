//! Lifecycle order does not collapse exact rows or cross output families.

use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::demand_key::WorthQueryOutputDemandKey;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::demand_key::family_declarations;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::demand_key::initial;

use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use std::cmp::Ordering;
use worth_query_execution::equality_type_id::EqualityTypeId as TypeId;
use worth_query_execution::facade::application_contribution::WorthQueryProducerApplicability;
use worth_query_execution::facade::application_contribution::WorthQueryProducerLifecyclePosture;
#[test]
fn family_key_order_follows_declared_identity() {
    for families in family_declarations() {
        assert_ne!(families[0].0, families[1].0);
        assert!(families[0].1 < families[1].1);
        let source = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key("same-producer", 7, 1);
        let keys = families.map(|(type_id, identity)| {
            (
                type_id,
                WorthQueryOutputDemandKey::new(
                    identity,
                    source.producer_identity().to_owned(),
                    source.applicability(),
                    source.source_epoch().clone(),
                ),
            )
        });
        assert_ne!(keys[0].0, keys[1].0);
        assert_eq!(keys[0].1.cmp(&keys[1].1), Ordering::Less);
    }
}
#[test]
fn same_observation_preserve_is_a_strict_successor_but_a_distinct_exact_row() {
    let initial = initial();
    let mut preserve = initial.clone();
    preserve.producer = "preserve".into();
    preserve.applicability = WorthQueryProducerApplicability::new(
        "registry-fixture",
        WorthQueryProducerLifecyclePosture::Preserve,
    );
    assert_ne!(initial, preserve);
    assert!(initial.same_occurrence(&preserve));
    assert!(!initial.same_semantic_source(&preserve));
    assert_eq!(initial.replacement_order(&preserve), Some(Ordering::Less));
    assert_eq!(
        preserve.replacement_order(&initial),
        Some(Ordering::Greater)
    );
    preserve.producer = initial.producer.clone();
    assert_ne!(initial, preserve);
    assert!(initial.same_semantic_source(&preserve));
    assert_eq!(initial.replacement_order(&preserve), Some(Ordering::Less));
    assert_eq!(
        preserve.replacement_order(&initial),
        Some(Ordering::Greater)
    );
}
#[test]
fn incomparable_preserve_bindings_do_not_pick_a_successor_by_iteration_order() {
    let initial = initial();
    let mut left = initial.clone();
    left.producer = "left-preserve".into();
    left.applicability = WorthQueryProducerApplicability::new(
        "registry-fixture",
        WorthQueryProducerLifecyclePosture::Preserve,
    );
    let mut right = left.clone();
    right.producer = "right-preserve".into();
    for candidates in [[&left, &right], [&right, &left]] {
        assert!(worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::newest_lawful_successor(
            candidates.into_iter(),
            &initial
        )
        .is_none());
    }
}
#[test]
fn family_scope_profile_and_source_movement_do_not_forge_a_successor() {
    let initial = initial();
    let mut foreign = initial.clone();
    foreign.family = worth_query_execution::domain_computation::primary_graph::output_family_identity::OutputFamilyIdentity::declared("other-output-family");
    assert!(!initial.same_occurrence(&foreign));
    assert!(!initial.same_semantic_source(&foreign));
    assert_eq!(initial.replacement_order(&foreign), None);

    let mut unrelated = initial.clone();
    unrelated.producer = "unrelated".into();
    assert_eq!(initial.replacement_order(&unrelated), None);
    unrelated.applicability = WorthQueryProducerApplicability::new(
        "another-profile",
        WorthQueryProducerLifecyclePosture::Preserve,
    );
    assert_eq!(initial.replacement_order(&unrelated), None);

    let mut backdated = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key("preserve", 6, 1);
    backdated.applicability = WorthQueryProducerApplicability::new(
        "registry-fixture",
        WorthQueryProducerLifecyclePosture::Preserve,
    );
    assert_eq!(backdated.replacement_order(&initial), Some(Ordering::Less));
    let other_root = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key("preserve", 8, 2);
    assert_eq!(initial.replacement_order(&other_root), None);
}
