#[allow(unused_imports)]
use crate::stable_read_plan_admission::*;

use {
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_extent_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_segment_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority,
    worth_store_test_support::harness::physical_isolation::epoch_scope::generation_counted_page_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout,
    worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_operation_digest_closeout,
};

#[test]
fn proof_bearing_read_plan_admits_before_execution_handle() {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let references = protected_set(
        [
            current_generation_segment_reference(11),
            current_generation_extent_reference(13),
            current_generation_page_reference(17),
        ],
        16,
    );

    let plan = admit_plan(&authority, root, references, 8192, 8);

    assert_eq!(plan.root_epoch().get(), root.epoch().get());
    assert_eq!(plan.manifest_epoch().get(), root.manifest_epoch().get());
    assert!(plan.epoch_vector().segment_epoch().is_some());
    assert!(plan.epoch_vector().extent_epoch().is_some());
    assert!(plan.epoch_vector().page_epoch().is_some());
    assert_eq!(plan.footprint().protected().references().len(), 3);
    assert_eq!(plan.counters().protected_references(), 3);
    assert_eq!(plan.counters().release_obligations(), 3);
    assert_eq!(plan.counters().reachability_barriers(), 1);
    assert_eq!(plan.counters().resident_bytes(), 8192);
    assert_eq!(
        plan.retry_posture(),
        worth_store_physical_isolation::PhysicalReadPlanRetryPosture::Current
    );
    assert_eq!(plan.counters().retry_decisions(), 0);
    assert_eq!(plan.counters().scratch_capacity(), 8);
    assert_eq!(plan.counters().scratch_allocations(), 2);
    assert_eq!(plan.counters().allocation_events(), 3);
    assert_eq!(plan.reachability_barrier().protected_references(), 3);
    assert_eq!(
        plan.reachability_barrier().footprint_basis(),
        plan.footprint().declared_footprint_basis()
    );
    assert_eq!(plan.footprint().footprint_basis().protected_ranges(), 3);
    assert!(plan.release_semantics().release_required());
    assert_eq!(
        plan.latch_plan().steps().len() as u64,
        plan.counters().latch_requirements()
    );

    let expected_release_basis = plan.reachability_barrier().footprint_basis();
    let handle = plan.into_execution_ready_handle();
    let release = handle.release();
    assert_eq!(release.protected_references_released(), 3);
    assert_eq!(release.footprint_basis(), expected_release_basis);
}

#[test]
fn protected_reference_footprint_is_canonical_across_input_order() {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let first = protected_set(
        [
            current_generation_page_reference(23),
            current_generation_page_reference(29),
        ],
        4,
    );
    let second = protected_set(
        [
            current_generation_page_reference(29),
            current_generation_page_reference(23),
        ],
        4,
    );

    let first_plan = admit_plan(&authority, root, first, 4096, 4);
    let second_plan = admit_plan(&authority, root, second, 4096, 4);

    assert_eq!(
        first_plan.footprint().protected().references(),
        second_plan.footprint().protected().references()
    );
    assert_eq!(
        first_plan.footprint().protected().ranges().ranges(),
        second_plan.footprint().protected().ranges().ranges()
    );
}

#[test]
fn missing_release_stale_generation_and_unbounded_footprints_deny_before_handle() {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let references = protected_set([current_generation_page_reference(31)], 1);
    let missing_release =
        worth_store_physical_isolation::UnprotectedReadIntent::for_known_footprint(
            root, references, 4096,
        );
    assert_eq!(
        worth_store_physical_isolation::PublishedReaderHazard::publish(&authority, missing_release)
            .unwrap_err(),
        worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial::MissingReleaseSemantics
    );

    let stale =
        worth_store_physical_isolation::ProtectedPhysicalReferenceSet::from_generation_counted_refs(
            [(
                generation_counted_page_reference(37),
                worth_store_physical_format::PhysicalGeneration::from_raw(38).unwrap(),
            )],
        );
    assert!(matches!(
        stale,
        Err(worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial::StaleGeneration(_))
    ));

    let broad = protected_set(
        [
            current_generation_page_reference(41),
            current_generation_page_reference(43),
        ],
        2,
    );
    let observed_references = broad.clone();
    let intent = worth_store_physical_isolation::UnprotectedReadIntent::for_known_footprint(
        root, broad, 4096,
    )
    .with_release_semantics(
        worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics::reader_releases_all(),
    );
    let hazard =
        worth_store_physical_isolation::PublishedReaderHazard::publish(&authority, intent).unwrap();
    let observed = post_protection_observation(&authority, &hazard, root, observed_references);
    let validated = hazard
        .observe_authority_after_publication(&authority, observed)
        .unwrap()
        .validate()
        .unwrap();
    let denial = worth_store_physical_isolation::TraversalAdmissionGuard::from_validated_root(validated)
        .admit(worth_store_physical_isolation::ReadPlanAdmissionScratchArena::for_protected_reference_capacity(1))
        .unwrap_err();

    assert_eq!(
        denial,
        worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial::UnboundedProtectedFootprint {
            requested: 2,
            capacity: 1
        }
    );
}

#[test]
fn compatibility_footprint_constructor_cannot_reach_execution_handle() {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let references = worth_store_physical_isolation::ProtectedPhysicalReferenceSet::from_current_generation_refs([
        current_generation_page_reference(44),
    ])
    .unwrap();
    let observed_references = references.clone();
    let intent = worth_store_physical_isolation::UnprotectedReadIntent::for_known_footprint(
        root, references, 4096,
    )
    .with_release_semantics(
        worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics::reader_releases_all(),
    );
    let hazard =
        worth_store_physical_isolation::PublishedReaderHazard::publish(&authority, intent).unwrap();
    let observed = post_protection_observation(&authority, &hazard, root, observed_references);
    let validated = hazard
        .observe_authority_after_publication(&authority, observed)
        .unwrap()
        .validate()
        .unwrap();
    let denial = worth_store_physical_isolation::TraversalAdmissionGuard::from_validated_root(validated)
        .admit(worth_store_physical_isolation::ReadPlanAdmissionScratchArena::for_protected_reference_capacity(4))
        .unwrap_err();

    assert_eq!(
        denial,
        worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial::UnboundedProtectedFootprint {
            requested: 1,
            capacity: 0
        }
    );
}

#[test]
fn hazard_publication_rejects_root_from_different_store_authority() {
    let authority = physical_authority_from_complete_closeout();
    let other_authority = physical_authority_from_operation_digest_closeout("other-root");
    let root = current_root_from_authority(&other_authority);
    let references = protected_set([current_generation_page_reference(61)], 1);
    let intent = worth_store_physical_isolation::UnprotectedReadIntent::for_known_footprint(
        root, references, 4096,
    )
    .with_release_semantics(
        worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics::reader_releases_all(),
    );
    let denial = worth_store_physical_isolation::PublishedReaderHazard::publish(&authority, intent)
        .unwrap_err();

    assert!(matches!(
        denial,
        worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial::AuthorityRootMismatch { .. }
    ));
}

#[test]
fn post_publication_root_drift_denies_before_traversal_admission() {
    let authority = physical_authority_from_complete_closeout();
    let other_authority = physical_authority_from_operation_digest_closeout("post-hazard-root");
    let root = current_root_from_authority(&authority);
    let references = protected_set([current_generation_page_reference(53)], 1);
    let observed_root = current_root_from_authority(&other_authority);
    let observed_references = references.clone();
    let intent = worth_store_physical_isolation::UnprotectedReadIntent::for_known_footprint(
        root, references, 4096,
    )
    .with_release_semantics(
        worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics::reader_releases_all(),
    );
    let hazard =
        worth_store_physical_isolation::PublishedReaderHazard::publish(&authority, intent).unwrap();
    let observed = post_protection_observation(
        &other_authority,
        &hazard,
        observed_root,
        observed_references,
    );

    assert!(matches!(
        hazard
            .observe_authority_after_publication(&other_authority, observed)
            .unwrap()
            .validate()
            .unwrap_err(),
        worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial::StalePlan(_)
    ));
}

#[test]
fn post_publication_reference_drift_denies_before_traversal_admission() {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let protected_before_hazard = protected_set([current_generation_page_reference(53)], 1);
    let observed_after_hazard = protected_set([current_generation_page_reference(59)], 1);
    let intent = worth_store_physical_isolation::UnprotectedReadIntent::for_known_footprint(
        root,
        protected_before_hazard,
        4096,
    )
    .with_release_semantics(
        worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics::reader_releases_all(),
    );
    let hazard =
        worth_store_physical_isolation::PublishedReaderHazard::publish(&authority, intent).unwrap();
    let observed = post_protection_observation(&authority, &hazard, root, observed_after_hazard);

    assert!(matches!(
        hazard
            .observe_authority_after_publication(&authority, observed)
            .unwrap()
            .validate()
            .unwrap_err(),
        worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial::StalePlan(_)
    ));
}

#[test]
fn observation_from_different_hazard_denies_before_epoch_validation() {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let first_references = protected_set([current_generation_page_reference(67)], 1);
    let second_references = protected_set(
        [
            current_generation_page_reference(67),
            current_generation_page_reference(71),
        ],
        2,
    );
    let first_intent = worth_store_physical_isolation::UnprotectedReadIntent::for_known_footprint(
        root,
        first_references,
        4096,
    )
    .with_release_semantics(
        worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics::reader_releases_all(),
    );
    let second_intent = worth_store_physical_isolation::UnprotectedReadIntent::for_known_footprint(
        root,
        second_references,
        4096,
    )
    .with_release_semantics(
        worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics::reader_releases_all(),
    );
    let first_hazard =
        worth_store_physical_isolation::PublishedReaderHazard::publish(&authority, first_intent)
            .unwrap();
    let second_hazard =
        worth_store_physical_isolation::PublishedReaderHazard::publish(&authority, second_intent)
            .unwrap();
    let observed = post_protection_observation(
        &authority,
        &second_hazard,
        root,
        protected_set(
            [
                current_generation_page_reference(67),
                current_generation_page_reference(71),
            ],
            2,
        ),
    );

    assert_eq!(
        first_hazard
            .observe_authority_after_publication(&authority, observed)
            .unwrap_err(),
        worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial::PostProtectionObservationHazardMismatch {
            expected_protected_references: 1,
            observed_protected_references: 2
        }
    );
}

#[test]
fn execution_time_reference_discovery_is_denied_by_execution_handle() {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let admitted_reference = current_generation_page_reference(47);
    let discovered_reference = current_generation_page_reference(49);
    let references = protected_set([admitted_reference], 4);
    let handle = admit_plan(&authority, root, references, 4096, 4).into_execution_ready_handle();

    assert_eq!(handle.read_protected_reference(admitted_reference), Ok(()));
    assert_eq!(
        handle
            .read_protected_reference(discovered_reference)
            .unwrap_err(),
        worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial::ExecutionTimeReferenceDiscovery
    );
}
