#[allow(unused_imports)] use crate::latch_algorithm_shape::*;
use {worth_store_test_support::harness::physical_isolation::epoch_scope as support};

use {std::collections::HashSet};
use {std::hash::BuildHasher, std::hash::Hasher};

use {worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_extent_reference, worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference, worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_segment_reference, worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority, worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout};
use {worth_store_physical_isolation::lower_latch_acquisition_plan, worth_store_physical_isolation::CanonicalLatchAcquisitionOrder, worth_store_physical_isolation::LatchAcquisitionDenial, worth_store_physical_isolation::LatchAcquisitionRequest, worth_store_physical_isolation::LatchAcquisitionStep, worth_store_physical_isolation::LatchUpgradeAuthority, worth_store_physical_isolation::LatchWaitForGraph, worth_store_physical_isolation::LatchWaitForGraphAdmissionDenial, worth_store_physical_isolation::PhysicalLatchClass, worth_store_physical_isolation::PhysicalLatchDeadlockPolicy, worth_store_physical_isolation::PhysicalLatchFamilyDeadlockPolicy, worth_store_physical_isolation::PhysicalLatchKey, worth_store_physical_isolation::PhysicalLatchWaitEdge};

#[test]
fn canonical_latch_order_survives_permutation_hash_seed_and_rebuild() {
    let expected_steps = mixed_mode_expected_steps();
    let expected_plan_steps = lower_steps(expected_steps.clone());
    let lanes = vec![
        expected_steps.clone(),
        reversed(expected_steps.clone()),
        rotated_left(expected_steps.clone(), 3),
        salted_hash_steps(expected_steps.clone(), 0xA5),
        salted_hash_steps(expected_steps, 0x5A),
        mixed_mode_expected_steps(),
    ];

    for lane in lanes {
        assert_eq!(lower_steps(lane), expected_plan_steps);
    }
}

#[test]
fn canonical_comparator_preserves_mode_and_action_tiebreakers() {
    let keys = algorithm_latch_keys();
    let exclusive = worth_store_physical_isolation::LatchAcquisitionStep::exclusive(keys.page_a);
    let upgrade = worth_store_physical_isolation::LatchAcquisitionStep::upgrade(keys.page_a, upgrade_authority());
    let shared = worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.page_a);

    assert!(worth_store_physical_isolation::CanonicalLatchAcquisitionOrder::compare_steps(&shared, &exclusive).is_lt());
    assert!(worth_store_physical_isolation::CanonicalLatchAcquisitionOrder::compare_steps(&exclusive, &upgrade).is_lt());
}

#[test]
fn latch_family_policy_is_explicit_for_every_class() {
    for class in all_latch_classes() {
        let prevention = worth_store_physical_isolation::PhysicalLatchFamilyDeadlockPolicy::for_class(class);
        let detection =
            worth_store_physical_isolation::PhysicalLatchFamilyDeadlockPolicy::detect_with_bounded_wait_for_graph(class);

        assert_eq!(prevention.class(), class);
        assert_eq!(
            prevention.policy(),
            worth_store_physical_isolation::PhysicalLatchDeadlockPolicy::PreventByCanonicalOrder
        );
        assert_eq!(detection.class(), class);
        assert_eq!(
            detection.policy(),
            worth_store_physical_isolation::PhysicalLatchDeadlockPolicy::DetectWithBoundedWaitForGraph
        );
    }
}

#[test]
fn bounded_detection_policy_denies_with_exact_counter_evidence() {
    let keys = algorithm_latch_keys();
    let denial = worth_store_physical_isolation::LatchWaitForGraph::bounded_with_evidence(
        vec![
            worth_store_physical_isolation::PhysicalLatchWaitEdge::new(1, 2, keys.page_a),
            worth_store_physical_isolation::PhysicalLatchWaitEdge::new(2, 3, keys.extent_a),
        ],
        1,
    )
    .unwrap_err();

    let worth_store_physical_isolation::LatchWaitForGraphAdmissionDenial::BoundExceeded(evidence) = denial else {
        panic!("bound excess must deny with pre-wait evidence");
    };
    assert_eq!(
        evidence.denial(),
        worth_store_physical_isolation::LatchAcquisitionDenial::WaitForGraphBoundExceeded
    );
    assert_counter_rows(
        evidence.counter_receipt(),
        &[
            ("s5.latch.attempts", 2),
            ("s5.latch.denied-upgrades", 0),
            ("s5.latch.detected-cycles", 0),
            ("s5.latch.execution-time-discovery-denials", 0),
            ("s5.latch.waits", 2),
        ],
    );
}
