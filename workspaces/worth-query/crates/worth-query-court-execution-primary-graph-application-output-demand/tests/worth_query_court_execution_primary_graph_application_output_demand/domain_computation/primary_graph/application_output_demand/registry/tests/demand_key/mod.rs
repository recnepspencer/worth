//! Lifecycle order does not collapse exact rows or cross output families.

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::demand_key::WorthQueryOutputDemandKey;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::demand_key::family_declarations;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::demand_key::initial;

use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use std::cmp::Ordering;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
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
                WorthQueryOutputDemandKey::new_for_test(
                    identity,
                    source.producer_identity_for_test().to_owned(),
                    source.applicability(),
                    source.source_epoch_for_test().clone(),
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
    (*preserve.producer_mut_for_test()) = "preserve".into();
    (*preserve.applicability_mut_for_test()) = WorthQueryProducerApplicability::new(
        "registry-fixture",
        WorthQueryProducerLifecyclePosture::Preserve,
    );
    assert_ne!(initial, preserve);
    assert!(initial.same_occurrence_for_test(&preserve));
    assert!(!initial.same_semantic_source_for_test(&preserve));
    assert_eq!(initial.replacement_order_for_test(&preserve), Some(Ordering::Less));
    assert_eq!(
        preserve.replacement_order_for_test(&initial),
        Some(Ordering::Greater)
    );
    (*preserve.producer_mut_for_test()) = (*initial.producer_for_test()).clone();
    assert_ne!(initial, preserve);
    assert!(initial.same_semantic_source_for_test(&preserve));
    assert_eq!(initial.replacement_order_for_test(&preserve), Some(Ordering::Less));
    assert_eq!(
        preserve.replacement_order_for_test(&initial),
        Some(Ordering::Greater)
    );
}
#[test]
fn incomparable_preserve_bindings_do_not_pick_a_successor_by_iteration_order() {
    let initial = initial();
    let mut left = initial.clone();
    (*left.producer_mut_for_test()) = "left-preserve".into();
    (*left.applicability_mut_for_test()) = WorthQueryProducerApplicability::new(
        "registry-fixture",
        WorthQueryProducerLifecyclePosture::Preserve,
    );
    let mut right = left.clone();
    (*right.producer_mut_for_test()) = "right-preserve".into();
    for candidates in [[&left, &right], [&right, &left]] {
        assert!(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::newest_lawful_successor(
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
    (*foreign.family_mut_for_test()) = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_family_identity::OutputFamilyIdentity::declared_for_test("other-output-family");
    assert!(!initial.same_occurrence_for_test(&foreign));
    assert!(!initial.same_semantic_source_for_test(&foreign));
    assert_eq!(initial.replacement_order_for_test(&foreign), None);

    let mut unrelated = initial.clone();
    (*unrelated.producer_mut_for_test()) = "unrelated".into();
    assert_eq!(initial.replacement_order_for_test(&unrelated), None);
    (*unrelated.applicability_mut_for_test()) = WorthQueryProducerApplicability::new(
        "another-profile",
        WorthQueryProducerLifecyclePosture::Preserve,
    );
    assert_eq!(initial.replacement_order_for_test(&unrelated), None);

    let mut backdated = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key("preserve", 6, 1);
    (*backdated.applicability_mut_for_test()) = WorthQueryProducerApplicability::new(
        "registry-fixture",
        WorthQueryProducerLifecyclePosture::Preserve,
    );
    assert_eq!(backdated.replacement_order_for_test(&initial), Some(Ordering::Less));
    let other_root = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key("preserve", 8, 2);
    assert_eq!(initial.replacement_order_for_test(&other_root), None);
}
