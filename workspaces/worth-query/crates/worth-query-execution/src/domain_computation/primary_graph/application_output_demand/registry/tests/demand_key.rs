//! Lifecycle order does not collapse exact rows or cross output families.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
};
use std::{any::TypeId, cmp::Ordering};

struct OtherFamily;

fn initial() -> WorthQueryOutputDemandKey {
    let mut key = support::key("initial", 7, 1);
    key.applicability = WorthQueryProducerApplicability::new(
        "registry-fixture",
        WorthQueryProducerLifecyclePosture::Initial,
    );
    key
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
        assert!(super::super::refreshed_rejoin::newest_lawful_successor(
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
    foreign.family = TypeId::of::<OtherFamily>();
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

    let mut backdated = support::key("preserve", 6, 1);
    backdated.applicability = WorthQueryProducerApplicability::new(
        "registry-fixture",
        WorthQueryProducerLifecyclePosture::Preserve,
    );
    assert_eq!(backdated.replacement_order(&initial), Some(Ordering::Less));
    let other_root = support::key("preserve", 8, 2);
    assert_eq!(initial.replacement_order(&other_root), None);
}
