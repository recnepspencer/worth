#[allow(unused_imports)]
use crate::latch_acquisition_order::*;
use worth_store_test_support::harness::physical_isolation::epoch_scope as support;

use {
    worth_store_physical_isolation::latch_counter_backed_performance_receipt,
    worth_store_physical_isolation::lower_latch_acquisition_plan,
    worth_store_physical_isolation::pre_wait_denial_for_execution_time_latch_discovery,
    worth_store_physical_isolation::pre_wait_denial_for_hierarchy_inversion,
    worth_store_physical_isolation::pre_wait_denial_for_unauthorized_latch_upgrade,
    worth_store_physical_isolation::pre_wait_denial_for_unordered_latch_set,
    worth_store_physical_isolation::DeadlockPreventionDenial,
    worth_store_physical_isolation::LatchAcquisitionDenial,
    worth_store_physical_isolation::LatchAcquisitionRequest,
    worth_store_physical_isolation::LatchAcquisitionStep,
    worth_store_physical_isolation::LatchUpgradeAuthority,
    worth_store_physical_isolation::LatchWaitCounterSnapshot,
    worth_store_physical_isolation::LatchWaitForGraph,
    worth_store_physical_isolation::PhysicalLatchClass,
    worth_store_physical_isolation::PhysicalLatchKey,
    worth_store_physical_isolation::PhysicalLatchWaitEdge,
};
use {
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_extent_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_segment_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority,
    worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout,
};

#[test]
fn declared_footprints_lower_to_same_canonical_latch_order() {
    let keys = representative_latch_keys();
    let requested_a = vec![
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.page),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.root),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.extent),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.segment),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.manifest),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.chunk),
    ];
    let requested_b = vec![
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.chunk),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.manifest),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.segment),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.root),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.page),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.extent),
    ];

    let plan_a = lower_latch_acquisition_plan(
        worth_store_physical_isolation::LatchAcquisitionRequest::for_declared_footprint(
            requested_a,
        ),
    )
    .unwrap();
    let plan_b = lower_latch_acquisition_plan(
        worth_store_physical_isolation::LatchAcquisitionRequest::for_declared_footprint(
            requested_b,
        ),
    )
    .unwrap();

    assert_eq!(plan_a.steps(), plan_b.steps());
    assert_eq!(
        latch_classes(plan_a.steps()),
        vec![
            worth_store_physical_isolation::PhysicalLatchClass::Root,
            worth_store_physical_isolation::PhysicalLatchClass::Manifest,
            worth_store_physical_isolation::PhysicalLatchClass::Segment,
            worth_store_physical_isolation::PhysicalLatchClass::Extent,
            worth_store_physical_isolation::PhysicalLatchClass::Page,
            worth_store_physical_isolation::PhysicalLatchClass::FutureChunk,
        ]
    );
    let _proof = plan_a.order_proof();
}

#[test]
fn upgrade_steps_require_physical_authority_witness() {
    let authority = physical_authority_from_complete_closeout();
    let keys = representative_latch_keys();
    let upgrade_authority =
        worth_store_physical_isolation::LatchUpgradeAuthority::from_physical_read_stability_authority(&authority);

    let upgraded = lower_latch_acquisition_plan(
        worth_store_physical_isolation::LatchAcquisitionRequest::for_declared_footprint(vec![
            worth_store_physical_isolation::LatchAcquisitionStep::upgrade(
                keys.page,
                upgrade_authority,
            ),
        ]),
    )
    .unwrap();

    assert_eq!(upgraded.steps()[0].key(), keys.page);
    let denied_upgrade = pre_wait_denial_for_unauthorized_latch_upgrade(keys.page).unwrap();

    assert_denial_evidence(
        &denied_upgrade,
        worth_store_physical_isolation::DeadlockPreventionDenial::UnauthorizedUpgrade(keys.page),
        &[
            ("s5.latch.attempts", 1),
            ("s5.latch.denied-upgrades", 1),
            ("s5.latch.detected-cycles", 0),
            ("s5.latch.execution-time-discovery-denials", 0),
            ("s5.latch.waits", 0),
        ],
    );
}

#[test]
fn duplicate_and_conflicting_latch_requests_deny_before_waiting() {
    let keys = representative_latch_keys();
    let duplicate = lower_latch_acquisition_plan(
        worth_store_physical_isolation::LatchAcquisitionRequest::for_declared_footprint(vec![
            worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.page),
            worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.page),
        ]),
    );
    let conflicting = lower_latch_acquisition_plan(
        worth_store_physical_isolation::LatchAcquisitionRequest::for_declared_footprint(vec![
            worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.extent),
            worth_store_physical_isolation::LatchAcquisitionStep::exclusive(keys.extent),
        ]),
    );

    assert_eq!(
        duplicate.unwrap_err(),
        worth_store_physical_isolation::LatchAcquisitionDenial::DuplicateLatchKey(keys.page)
    );
    assert!(matches!(
        conflicting.unwrap_err(),
        worth_store_physical_isolation::LatchAcquisitionDenial::ConflictingLatchMode { key, .. } if key == keys.extent
    ));
}

#[test]
fn unordered_execution_inputs_and_hierarchy_inversions_deny_before_waiting() {
    let keys = representative_latch_keys();
    let inverted = vec![
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.page),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.root),
    ];

    assert!(pre_wait_denial_for_unordered_latch_set(&[
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.root),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.page),
    ])
    .unwrap()
    .is_none());
    let unordered_denial = pre_wait_denial_for_unordered_latch_set(&inverted)
        .unwrap()
        .unwrap();
    assert_denial_evidence(
        &unordered_denial,
        worth_store_physical_isolation::DeadlockPreventionDenial::UnorderedLockSet,
        &[
            ("s5.latch.attempts", 2),
            ("s5.latch.denied-upgrades", 0),
            ("s5.latch.detected-cycles", 0),
            ("s5.latch.execution-time-discovery-denials", 0),
            ("s5.latch.waits", 0),
        ],
    );
    let hierarchy_denial = pre_wait_denial_for_hierarchy_inversion(&inverted)
        .unwrap()
        .unwrap();
    assert_denial_evidence(
        &hierarchy_denial,
        worth_store_physical_isolation::DeadlockPreventionDenial::HierarchyInversion,
        &[
            ("s5.latch.attempts", 2),
            ("s5.latch.denied-upgrades", 0),
            ("s5.latch.detected-cycles", 0),
            ("s5.latch.execution-time-discovery-denials", 0),
            ("s5.latch.waits", 0),
        ],
    );
    let discovery_denial = pre_wait_denial_for_execution_time_latch_discovery(keys.page).unwrap();

    assert_denial_evidence(
        &discovery_denial,
        worth_store_physical_isolation::DeadlockPreventionDenial::ExecutionTimeLatchDiscovery(
            keys.page,
        ),
        &[
            ("s5.latch.attempts", 1),
            ("s5.latch.denied-upgrades", 0),
            ("s5.latch.detected-cycles", 0),
            ("s5.latch.execution-time-discovery-denials", 1),
            ("s5.latch.waits", 0),
        ],
    );
}

#[test]
fn wait_for_graph_detects_multi_actor_cycles_with_exact_counter_evidence() {
    let keys = representative_latch_keys();
    let graph = worth_store_physical_isolation::LatchWaitForGraph::bounded(
        vec![
            worth_store_physical_isolation::PhysicalLatchWaitEdge::new(1, 2, keys.page),
            worth_store_physical_isolation::PhysicalLatchWaitEdge::new(2, 3, keys.extent),
            worth_store_physical_isolation::PhysicalLatchWaitEdge::new(3, 1, keys.segment),
        ],
        4,
    )
    .unwrap();

    let report = graph.detect_cycle().unwrap();

    assert_eq!(report.cycle_edges().len(), 3);
    assert_eq!(report.counters().attempt_count(), 3);
    assert_eq!(report.counters().wait_count(), 3);
    assert_eq!(report.counters().detected_cycle_count(), 1);
    assert_counter_rows(
        report.counter_receipt(),
        &[
            ("s5.latch.attempts", 3),
            ("s5.latch.denied-upgrades", 0),
            ("s5.latch.detected-cycles", 1),
            ("s5.latch.execution-time-discovery-denials", 0),
            ("s5.latch.waits", 3),
        ],
    );
    let cycle_denial = graph.pre_wait_cycle_denial().unwrap().unwrap();

    assert_latch_denial_evidence(
        &cycle_denial,
        worth_store_physical_isolation::LatchAcquisitionDenial::CyclicPlan,
        &[
            ("s5.latch.attempts", 3),
            ("s5.latch.denied-upgrades", 0),
            ("s5.latch.detected-cycles", 1),
            ("s5.latch.execution-time-discovery-denials", 0),
            ("s5.latch.waits", 3),
        ],
    );
}

#[test]
fn latch_counter_evidence_is_exact_and_counter_backed() {
    let counters =
        worth_store_physical_isolation::LatchWaitCounterSnapshot::from_exact_counts(6, 2, 1, 1, 1);
    let receipt = latch_counter_backed_performance_receipt(counters).unwrap();
    let observed: Vec<(&str, u64)> = receipt
        .counter_rows()
        .iter()
        .map(|row| (row.name().as_str(), row.observed_count()))
        .collect();

    assert_eq!(
        observed,
        vec![
            ("s5.latch.attempts", 6),
            ("s5.latch.denied-upgrades", 1),
            ("s5.latch.detected-cycles", 1),
            ("s5.latch.execution-time-discovery-denials", 1),
            ("s5.latch.waits", 2),
        ]
    );
}
