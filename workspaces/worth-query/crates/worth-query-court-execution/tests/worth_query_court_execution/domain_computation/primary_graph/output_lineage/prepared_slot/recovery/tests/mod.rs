use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::prepared_slot::computation::PreparedComputationCustody;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::prepared_slot::PreparedOutputLineageSlot;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention;
    use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::sealed_run_for_lineage_test;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::ProductCoordinate;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::settlement_identity::RecordedSettlementIdentity;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::semantic_source::SemanticSource;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::OnceLock;

#[test]
fn world_recovery_metadata_carries_the_original_computation_and_absence() {
    let _observation =
            worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap();
    let source = SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity().clone(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(
                worth_relational::facade::identity::EntityId::new(worth_relational::facade::identity::PartitionId::main(), 1, 1)), OutputBindingIdentity::declared_for_test("()"));
    let coordinate = ProductCoordinate::new_for_test(
        product.observation_for_test().lifecycle_incarnation(),
        product.observation_for_test().reference_generation().get() + 1,
    );
    let owner = Arc::new(Mutex::new(WorthQueryApplicationOutputLineage::default()));
    // These are private vacant addresses, never publication authority.
    // Reserve their ledger custody before constructing their test slots.
    let generation = std::cell::Cell::new(*coordinate.generation_for_test());

    let slot = || {
        let coordinate = {
            let __generation = generation.get();
            let mut __base = coordinate;
            *__base.generation_mut_for_test() = __generation;
            __base
        };
        generation.set(generation.get() + 1);
        let mut lineage = owner.lock().unwrap();
        let identity = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 0);
        let record_cell = Arc::new(OnceLock::new());
        (*lineage.by_source_mut_for_test())
            .entry(source.clone())
            .or_default()
            .entry(*coordinate.occurrence_for_test())
            .or_default()
            .insert(
                *coordinate.generation_for_test(),
                vec![Arc::clone(&record_cell)],
            );
        let partition_cell = (*lineage.partition_index_mut_for_test()).insert_vacancy_for_test(
            source.clone(),
            coordinate,
            None,
        );
        PreparedOutputLineageSlot::new_for_test(Arc::clone(&owner), TypeId::of_for_test::<()>(), source.clone(), coordinate, None, Arc::clone(&identity), record_cell, partition_cell, Some(Box::new(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::prepared_slot::CancelledLineageSlot::new_for_test(identity, None, None))), None, None, None, None, None, None, None, false, PreparedComputationCustody::Unassigned)
    };
    for computation in [
        SealedComputationRetention::Produced(sealed_run_for_lineage_test()),
        SealedComputationRetention::Absent(PriorAbsence::Stopped),
    ] {
        let produced = matches!(&computation, SealedComputationRetention::Produced(_));
        let mut original = slot();
        original.retain_computation_for_test(computation, None);
        let metadata = original.into_recovery_metadata_for_test();
        let mut replacement = slot();
        replacement.retain_recovery_metadata_for_test(metadata);
        assert_eq!(
            matches!(
                &(*replacement.computation_for_test()),
                PreparedComputationCustody::Assigned(SealedComputationRetention::Produced(_))
            ),
            produced
        );
        if !produced {
            assert!(matches!(
                *replacement.computation_for_test(),
                PreparedComputationCustody::Assigned(SealedComputationRetention::Absent(
                    PriorAbsence::Stopped
                ))
            ));
        }
    }
}
