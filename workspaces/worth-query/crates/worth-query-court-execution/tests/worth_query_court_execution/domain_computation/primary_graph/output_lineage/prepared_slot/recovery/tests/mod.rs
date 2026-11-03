use worth_query_execution::domain_computation::primary_graph::output_lineage::prepared_slot::computation::PreparedComputationCustody;
    use worth_query_execution::domain_computation::primary_graph::output_lineage::prepared_slot::PreparedOutputLineageSlot;
    use worth_query_execution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention;
    use worth_query_execution::equality_type_id::EqualityTypeId as TypeId;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::sealed_run_for_lineage_test;
    use worth_query_execution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence;
    use worth_query_execution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention;
    use worth_query_execution::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
    use worth_query_execution::domain_computation::primary_graph::output_lineage::ProductCoordinate;
    use worth_query_execution::domain_computation::primary_graph::output_lineage::settlement_identity::RecordedSettlementIdentity;
    use worth_query_execution::domain_computation::primary_graph::output_lineage::semantic_source::SemanticSource;
    use worth_query_execution::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::OnceLock;

#[test]
fn world_recovery_metadata_carries_the_original_computation_and_absence() {
    let _observation =
            worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let source = SemanticSource {
            runtime_authority: world.application.runtime.authority_identity().as_u64(),
            schema: world.application.installed_schema.binding_identity().clone(),
            scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity(
                worth_relational::facade::identity::EntityId::new(worth_relational::facade::identity::PartitionId::main(), 1, 1)),
            output_binding: OutputBindingIdentity::declared("()"),
        };
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get() + 1,
    };
    let owner = Arc::new(Mutex::new(WorthQueryApplicationOutputLineage::default()));
    // These are private vacant addresses, never publication authority.
    // Reserve their ledger custody before constructing their test slots.
    let generation = std::cell::Cell::new(coordinate.generation);

    let slot = || {
        let coordinate = ProductCoordinate {
            generation: generation.get(),
            ..coordinate
        };
        generation.set(generation.get() + 1);
        let mut lineage = owner.lock().unwrap();
        let identity = RecordedSettlementIdentity::retain(&source, coordinate, 0);
        let record_cell = Arc::new(OnceLock::new());
        lineage
            .by_source
            .entry(source.clone())
            .or_default()
            .entry(coordinate.occurrence)
            .or_default()
            .insert(coordinate.generation, vec![Arc::clone(&record_cell)]);
        let partition_cell =
            lineage
                .partition_index
                .insert_vacancy(source.clone(), coordinate, None);
        PreparedOutputLineageSlot {
                owner: Arc::clone(&owner),
                output_binding_type: TypeId::of::<()>(),
                source: source.clone(),
                coordinate,
                partition: None,
                identity: Arc::clone(&identity),
                record_cell,
                partition_cell,
                cancellation: Some(Box::new(worth_query_execution::domain_computation::primary_graph::output_lineage::prepared_slot::CancelledLineageSlot {
                    identity,
                    partition: None,
                    next: None,
                })),
                completed_handler_facts: None,
                completed_decision_reuse: None,
                prepared_input_reuse_key: None,
                native_output_witness: None,
                required_publication: None,
                native_prior_checkpoint: None,
                prior_computation: None,
                filled: false,
                computation: PreparedComputationCustody::Unassigned,
            }
    };
    for computation in [
        SealedComputationRetention::Produced(sealed_run_for_lineage_test()),
        SealedComputationRetention::Absent(PriorAbsence::Stopped),
    ] {
        let produced = matches!(&computation, SealedComputationRetention::Produced(_));
        let mut original = slot();
        original.retain_computation(computation, None);
        let metadata = original.into_recovery_metadata();
        let mut replacement = slot();
        replacement.retain_recovery_metadata(metadata);
        assert_eq!(
            matches!(
                &replacement.computation,
                PreparedComputationCustody::Assigned(SealedComputationRetention::Produced(_))
            ),
            produced
        );
        if !produced {
            assert!(matches!(
                replacement.computation,
                PreparedComputationCustody::Assigned(SealedComputationRetention::Absent(
                    PriorAbsence::Stopped
                ))
            ));
        }
    }
}
