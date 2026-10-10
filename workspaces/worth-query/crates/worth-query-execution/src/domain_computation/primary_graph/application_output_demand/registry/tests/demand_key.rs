//! Lifecycle order does not collapse exact rows or cross output families.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
};
use std::{any::TypeId, cmp::Ordering};

#[test]
fn family_key_order_follows_declared_identity() {
    struct AlphaFamily;
    struct ZuluFamily;
    // Rust marker names are not the declared identity of their output.
    const ALPHA_IDENTITY: &str = "zulu-output";
    const ZULU_IDENTITY: &str = "alpha-output";
    let mut alpha = support::key("same-producer", 7, 1);
    let mut zulu = alpha.clone();
    assert_ne!(TypeId::of::<AlphaFamily>(), TypeId::of::<ZuluFamily>());
    alpha.family = crate::domain_computation::primary_graph::output_family_identity::OutputFamilyIdentity::declared(ALPHA_IDENTITY);
    zulu.family = crate::domain_computation::primary_graph::output_family_identity::OutputFamilyIdentity::declared(ZULU_IDENTITY);
    assert_eq!(ALPHA_IDENTITY.cmp(ZULU_IDENTITY), Ordering::Greater);
    assert_eq!(alpha.cmp(&zulu), Ordering::Greater);
}

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
    foreign.family = crate::domain_computation::primary_graph::output_family_identity::OutputFamilyIdentity::declared("other-output-family");
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
