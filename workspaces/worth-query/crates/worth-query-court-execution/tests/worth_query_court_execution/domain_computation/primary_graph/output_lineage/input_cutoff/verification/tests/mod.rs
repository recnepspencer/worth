//! The currentness allowance decides reuse against a real native fact.

use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::tests::TestsWorthQueryObservedSourceSelectionForTest;

use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::LocatorAuthority;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_cutoff::verification::source_facts::marked_facts_permit_reuse;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn a_marked_fact_is_reverified_at_the_real_native_snapshot() {
    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let entity = world
                .selected_product(&phase.test_host_request_for_test())
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test();
            let graph = (*world.application.runtime_for_test())
                .primary_graph()
                .unwrap();
            let status_ref = AccountStatus::reference();
            let status = graph
                .layout_for_test()
                .field_locator_for_test(
                    status_ref.entity(),
                    status_ref.aspect(),
                    status_ref.field(),
                )
                .unwrap()
                .clone();
            let locator = AspectFieldLocator::new(
                LocatorAuthority::Authoritative,
                status.aspect().aspect_key().clone(),
                status.field_path().clone(),
            );

            (*(*world.application.primary_provider_for_test()).graph_for_test()).with_runtime(
                &execution,
                |runtime| {
                    let basis = runtime
                        .admit_branch_basis(&runtime.main_branch_identity())
                        .unwrap();
                    let snapshot = runtime
                        .snapshots()
                        .snapshot_for_observation(&basis.observation())
                        .unwrap();
                    let revision = runtime
                        .read_truth()
                        .project_snapshot(&snapshot)
                        .unwrap()
                        .entity_field_revision(&execution, entity, &locator)
                        .expect("fixture read request remains live")
                        .unwrap();
                    let facts = [WorthQueryApplicationObservedFact::SourceFieldRevision {
                        entity_id: entity,
                        locator: locator.clone(),
                        native_revision: Some(revision),
                    }];

                    // An ample allowance proves the marked fact current, so any `false`
                    // below comes from exhaustion alone, never from a changed source.

                    assert!(matches!(
                        marked_facts_permit_reuse(&execution, &facts, runtime, &snapshot),
                        Ok(true)
                    ));

                    runtime.snapshots().release_snapshot(&snapshot).unwrap();
                },
            );
        })
        .expect("the fixture borrows its installed host phase");
}

#[test]
fn a_clean_settlement_reuses_without_fact_reverification() {
    use worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::handler_fact_boundary::CompletedHandlerFactBoundary;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::decision_reuse::PreparedDecisionReuseContext;
    use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryCheckpointOutputRole;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::semantic_edition::InstalledProducerEdition;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence;
    use worth_query_execution::facade::application_contribution::WorthQueryDecisionContextDependencies;
    use worth_query_execution::facade::application_contribution::WorthQueryProducerInputReuseContract;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::settlement::SettlementRegistration;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::derived::SourceSettlementCurrentness;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutput;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutputMutable;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::retained_computation::RecordedComputation;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_reuse_key::PreparedInputReuseKey;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::native_output_witness::PreparedNativeOutputWitness;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::ProductCoordinate;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::settlement_identity::RecordedSettlementIdentity;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::semantic_source::SemanticSource;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::handler::decision_context_use::DecisionContextUse;
    use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let selected_product = world.selected_product(&phase.test_host_request_for_test());
            let entity = selected_product
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test();
            let (_, product, _) = selected_product.into_parts_for_test();
            let observation = product.observation_for_test();
            let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
            let handle = graph.integration_handle_for_test();
            let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
            let status_ref = AccountStatus::reference();
            let status = graph
                .layout_for_test()
                .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
                .unwrap();
            let locator = AspectFieldLocator::new(
                LocatorAuthority::Authoritative,
                status.aspect().aspect_key().clone(),
                status.field_path().clone(),
            );
            let correspondence = Arc::new(
                WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
                    TypeId::of_for_test::<()>(),
                    TypeId::of_for_test::<()>(),
                    std::iter::empty(),
                    &[WorthQueryCheckpointOutputRole::new_for_test("account".to_owned(), WorthQueryApplicationOutputPosture::Preserve, "Account".to_owned(), entity)],
                    |_| Some(TypeId::of_for_test::<()>()),
                    &execution,
                )
                .unwrap(),
            );
            let source = SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity().as_u64(), (*world.application.installed_schema_for_test()).binding_identity(), WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), OutputBindingIdentity::declared_for_test("ProbeOutput"));
            let identity = RecordedSettlementIdentity::retain_for_test(
                &source,
                ProductCoordinate::new_for_test(observation.lifecycle_incarnation(), observation.reference_generation().get()),
                0,
            );
            let selection = WorthQueryObservedSourceSelection::selecting_for_test(1);
            let input_key = || {
                PreparedInputReuseKey::new_for_test(
                    selection.clone(),
                    [0x61; 32],
                    InstalledProducerEdition::for_test_for_test([0x51; 32]),
                )
            };
            let context = || {
                PreparedDecisionReuseContext::new_for_test(
                    WorthQueryProducerInputReuseContract::canonical_bitwise(
                        WorthQueryDecisionContextDependencies::NONE,
                    ),
                    None,
                    None,
                    None,
                )
                .unwrap()
            };

            handle.with_runtime(&execution, |runtime| {
                let basis = runtime
                    .admit_branch_basis(&runtime.main_branch_identity())
                    .unwrap();
                let snapshot = runtime
                    .snapshots()
                    .snapshot_for_observation(&basis.observation())
                    .unwrap();
                let selected = runtime.read_truth().positioned_snapshot(&snapshot).unwrap();
                let revision = runtime
                    .read_truth()
                    .project_snapshot(&snapshot)
                    .unwrap()
                    .entity_field_revision(&execution, entity, &locator)
                    .expect("fixture read request remains live")
                    .unwrap();
                let facts: Arc<[_]> =
                    Arc::from([WorthQueryApplicationObservedFact::SourceFieldRevision {
                        entity_id: entity,
                        locator: locator.clone(),
                        native_revision: Some(revision),
                    }]);

                let witness = PreparedNativeOutputWitness::prepare_republication_for_test(
                    &correspondence,
                    graph.layout_for_test(),
                )
                .unwrap()
                .expect("the installed Account admits its native witness")
                .finish_for_test(&correspondence, runtime, &snapshot)
                .unwrap();
                let evidence = ComputationSourceEvidence::for_test(false);
                owner
                    .register_settlement_for_test(SettlementRegistration::new_for_test(None, None, Arc::clone(&identity), evidence.retain_facts_for_test(Arc::clone(&facts)), None, selected.clone(), Default::default(), None, Default::default()))
                    .unwrap();
                assert!(
                    matches!(
                        owner.currentness_for_test(&selected, &identity),
                        Ok(SourceSettlementCurrentness::Clean)
                    ),
                    "the native settlement must start Clean"
                );
                assert!(
                    matches!(
                        marked_facts_permit_reuse(&execution, facts.iter(), runtime, &snapshot),
                        Ok(true)
                    ),
                    "the native probe confirms the original handler fact"
                );

                let boundary = CompletedHandlerFactBoundary::completed_for_test(1);
                let completed = boundary
                    .seal_decision_reuse_for_test(context(), DecisionContextUse::default())
                    .unwrap();
                let row = RecordedOutput::new_for_test(None, None, evidence, None, None, Arc::from([]), Some(boundary), Some(completed), Some(input_key()), OnceLock::from(witness), Arc::clone(&identity), Arc::clone(&correspondence), None, None, None, [0x41; 32], Mutex::new(RecordedOutputMutable::new_for_test(
                        None,
                        Some(evidence.retain_facts_for_test(facts)),
                        RecordedComputation::Absent(PriorAbsence::FirstRun),
                    )));
                let candidate = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_cutoff::RetainedInputCutoffCandidate::from_exact_cell_for_test(Arc::new(
                    OnceLock::from(row),
                ));

                let decision = candidate
                    .eligible_for_reuse_for_test(
                        &execution,
                        &input_key(),
                        &context(),
                        None,
                        runtime,
                        &snapshot,
                        &selected,
                        owner,
                    )
                    .unwrap();
                assert!(
                    decision.is_some(),
                    "a Clean settlement must reuse without loading its facts"
                );
                runtime.snapshots().release_snapshot(&snapshot).unwrap();
            });
        })
        .expect("the fixture borrows its installed host phase");
}
