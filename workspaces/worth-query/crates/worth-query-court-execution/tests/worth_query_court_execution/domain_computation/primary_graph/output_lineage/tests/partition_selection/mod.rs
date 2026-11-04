use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
use std::sync::Arc;
use std::sync::OnceLock;

use crate::domain_computation::primary_graph::output_lineage::tests::checkpoint_identity;
use crate::domain_computation::primary_graph::output_lineage::tests::source_facts;
use crate::domain_computation::primary_graph::output_lineage::tests::RestoredOutputBinding;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::ProductCoordinate;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutput;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::semantic_source::SemanticSource;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;

pub fn partition(number: u64) -> [u8; 32] {
    let mut partition = [0; 32];
    partition[..8].copy_from_slice(&number.to_le_bytes());
    partition
}

pub fn unrelated_partition_selection(population: u64) {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let product = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the fixture's default product occurrence is live");
    let observation = product.observation_for_test();
    let occurrence = observation.lifecycle_incarnation();
    let runtime_authority = (*world.application.runtime_for_test())
        .authority_identity()
        .as_u64();
    let schema = (*world.application.installed_schema_for_test()).binding_identity();
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            1,
            1,
        ),
    );
    let source = SemanticSource::new_for_test(
        runtime_authority,
        schema.clone(),
        scope,
        OutputBindingIdentity::declared_for_test("RestoredOutputBinding"),
    );
    let target = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let sibling = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.fixture_binding(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        "RestoredOutputBinding",
    );

    lineage.record_recovered_prior_output_for_test(
        lineage.binding_type_for_test(&(*source.output_binding_for_test())),
        runtime_authority,
        schema.clone(),
        scope,
        occurrence,
        1,
        Arc::clone(&target),
        checkpoint_identity([0x31; 32]),
        partition(0),
        None,
        [0x41; 32],
        None,
    );
    for number in 1..=population {
        lineage.record_recovered_prior_output_for_test(
            lineage.binding_type_for_test(&(*source.output_binding_for_test())),
            runtime_authority,
            schema.clone(),
            scope,
            occurrence,
            number + 1,
            Arc::clone(&sibling),
            checkpoint_identity([0x31; 32]),
            partition(number),
            None,
            [0x41; 32],
            None,
        );
    }
    let coordinate = ProductCoordinate::new_for_test(occurrence, population + 1);

    let selected = lineage.latest_output_in_partition_for_test(&source, coordinate, partition(0));
    assert!(Arc::ptr_eq(
        &(*selected.unwrap().correspondence_for_test()),
        &target
    ));
    let candidates = lineage
        .retained_output_candidates_metered_for_test(
            runtime_authority,
            &schema,
            scope,
            occurrence,
            population + 1,
            &[lineage.binding_type_for_test(&(*source.output_binding_for_test()))],
            partition(0),
            execution,
        )
        .expect("unrelated partitions cannot exhaust one selection lookup");
    assert_eq!(candidates.len(), 1);
    assert!(Arc::ptr_eq(
        &(*candidates[0].correspondence_for_test()),
        &target
    ));
    let missing = lineage.latest_output_in_partition_for_test(&source, coordinate, [0xff; 32]);
    assert!(missing.is_none());

    let revised_target = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let revised_generation = population + 2;
    let records = (*lineage.by_source_mut_for_test())
        .get_mut(&source)
        .unwrap()
        .get_mut(&occurrence)
        .unwrap()
        .entry(revised_generation)
        .or_default();
    let slot = records.len();
    let recorded = {
        // Bound in the original literal order, passed in the door's.
        let (producing_implementation, required_publication, native_prior_checkpoint, computation_source, performed_origin, consumed_outputs, completed_handler_facts, completed_decision_reuse, prepared_input_reuse_key, native_output_witness, mutable, settlement_identity, correspondence, source_identity, source_partition_identity, producer_dependency_identity, idempotency_key_identity) = (None, None, None, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false), None, Arc::from([]), None, None, None, OnceLock::new(), std::sync::Mutex::new(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutputMutable::new_for_test(None, None, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::retained_computation::RecordedComputation::Absent(PriorAbsence::Restored))), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::settlement_identity::RecordedSettlementIdentity::retain_for_test(
            &source,
            ProductCoordinate::new_for_test(occurrence, revised_generation),
            slot,
        ), Arc::clone(&revised_target), Some(checkpoint_identity([0x31; 32])), Some(partition(0)), None, [0x42; 32]);
        RecordedOutput::new_for_test(
            producing_implementation,
            required_publication,
            computation_source,
            native_prior_checkpoint,
            performed_origin,
            consumed_outputs,
            completed_handler_facts,
            completed_decision_reuse,
            prepared_input_reuse_key,
            native_output_witness,
            settlement_identity,
            correspondence,
            source_identity,
            source_partition_identity,
            producer_dependency_identity,
            idempotency_key_identity,
            mutable,
        )
    };
    let cell = Arc::new(std::sync::OnceLock::new());
    assert!(cell.set(recorded).is_ok());
    records.push(cell);
    (*lineage.partition_index_mut_for_test()).insert_for_test(
        source.clone(),
        occurrence,
        revised_generation,
        Some(partition(0)),
        slot,
    );
    let revised_coordinate = ProductCoordinate::new_for_test(occurrence, revised_generation);

    let (prior, work) = lineage
        .matching_output_in_partition_budgeted_for_test(
            &source,
            revised_coordinate,
            partition(0),
            execution,
            |recorded| Arc::ptr_eq(&(*recorded.correspondence_for_test()), &target),
        )
        .expect("same-partition predecessors, not siblings, consume the budget");
    assert_eq!(work, 2);
    assert!(Arc::ptr_eq(
        &(*prior.unwrap().correspondence_for_test()),
        &target
    ));

    // An unavailable derived locator must never guess from a sibling record.
    (*lineage.partition_index_mut_for_test()) = Default::default();
    let unavailable =
        lineage.latest_output_in_partition_for_test(&source, revised_coordinate, partition(0));
    assert!(unavailable.is_none());

    (*(*lineage.by_source_mut_for_test())
        .get_mut(&source)
        .unwrap()
        .get_mut(&occurrence)
        .unwrap()
        .get_mut(&1)
        .unwrap()
        .get_mut(0)
        .and_then(Arc::get_mut)
        .and_then(std::sync::OnceLock::get_mut)
        .unwrap()
        .source_partition_identity_mut_for_test()) = None;

    let (legacy, work) = lineage
        .matching_legacy_output_budgeted_for_test(
            &source,
            revised_coordinate,
            execution,
            |recorded| Arc::ptr_eq(&(*recorded.correspondence_for_test()), &target),
        )
        .expect("legacy unpartitioned correspondence remains discoverable within its budget");
    assert_eq!(work, population as usize + 2);
    assert!(Arc::ptr_eq(
        &(*legacy.unwrap().correspondence_for_test()),
        &target
    ));
}

#[test]
fn retained_output_selection_has_fixed_work_with_1k_and_10k_unrelated_partitions() {
    for population in [1_000, 10_000] {
        unrelated_partition_selection(population);
    }
}

#[test]
fn retained_output_selection_has_fixed_work_with_100k_unrelated_partitions() {
    unrelated_partition_selection(100_000);
}

#[test]
fn fork_selection_uses_ancestor_partition_and_retirement_prunes_its_locator() {
    use worth_runtime_world::facade::ProductBranchCreationIntent;
    use worth_runtime_world::facade::ProductBranchCreationPlans;
    use worth_runtime_world::facade::RelationalBranchCreationPlan;
    use worth_runtime_world::facade::RuntimeWorldBranchCreationOutcome;
    use worth_runtime_world::facade::RuntimeWorldCancellationSource;
    use worth_runtime_world::facade::SignalBranchCreationPlan;

    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let product = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the fixture's default product occurrence is live");
    let parent = product.observation_for_test();
    let intent = ProductBranchCreationIntent::from_source(
        "partition-index-fork",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ReuseExact,
            SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .expect("the fork intent is valid");
    let RuntimeWorldBranchCreationOutcome::Performed(child) = world
        .application
        .product_runtime()
        .create_product_branch_for_test(
            execution,
            &product,
            None,
            intent,
            &RuntimeWorldCancellationSource::new().token(),
        )
        .expect("the product owner admits the fork")
    else {
        panic!("the product owner must create the fork");
    };
    let runtime_authority = (*world.application.runtime_for_test())
        .authority_identity()
        .as_u64();
    let schema = (*world.application.installed_schema_for_test()).binding_identity();
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            1,
            1,
        ),
    );
    let source = SemanticSource::new_for_test(
        runtime_authority,
        schema.clone(),
        scope,
        OutputBindingIdentity::declared_for_test("RestoredOutputBinding"),
    );
    let correspondence = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.fixture_binding(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        "RestoredOutputBinding",
    );

    lineage.record_restoration_for_test(
        lineage.binding_type_for_test(&(*source.output_binding_for_test())),
        runtime_authority,
        schema.clone(),
        scope,
        parent,
        Arc::clone(&correspondence),
        checkpoint_identity([0x31; 32]),
        partition(0),
        None,
        [0x41; 32],
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts_for_test(source_facts()),
        None,
        None,
        None,
    );
    lineage.register_fork_for_test(parent, &child);
    let candidates = lineage
        .retained_output_candidates_metered_for_test(
            runtime_authority,
            &schema,
            scope,
            child.lifecycle_incarnation(),
            child.reference_generation().get(),
            &[lineage.binding_type_for_test(&(*source.output_binding_for_test()))],
            partition(0),
            execution,
        )
        .expect("a fork traverses its parent coordinate");
    assert!(Arc::ptr_eq(
        &(*candidates[0].correspondence_for_test()),
        &correspondence
    ));

    lineage.release_occurrence_for_test(parent.lifecycle_incarnation());
    assert!(
        (*lineage.partition_index_for_test())
            .latest_for_test(
                &source,
                ProductCoordinate::new_for_test(
                    parent.lifecycle_incarnation(),
                    parent.reference_generation().get()
                ),
                partition(0),
            )
            .is_some(),
        "a live child retains its ancestor's output locator"
    );
    lineage.release_occurrence_for_test(child.lifecycle_incarnation());
    assert!(
        (*lineage.partition_index_for_test())
            .latest_for_test(
                &source,
                ProductCoordinate::new_for_test(
                    parent.lifecycle_incarnation(),
                    parent.reference_generation().get()
                ),
                partition(0),
            )
            .is_none(),
        "retired lineage must prune its derived locator"
    );
    assert!(!(*lineage.by_source_for_test()).contains_key(&source));
}
