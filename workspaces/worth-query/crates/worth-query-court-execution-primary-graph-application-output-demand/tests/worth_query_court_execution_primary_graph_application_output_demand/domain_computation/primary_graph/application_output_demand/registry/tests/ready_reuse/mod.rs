use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryCheckpointSourceIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
use worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;

#[test]
fn ready_output_reopens_and_only_its_exact_restored_predecessor_refreshes_it() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let product_occurrence = occurrence();
    let output_key = key("producer", 5, 1);
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let observation = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .product()
                .observation_for_test()
                .clone()
        })
        .expect("the fixture read owns its installed host call");
    let correspondence = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
        TypeId::of_for_test::<()>(),
        TypeId::of_for_test::<()>(),
        std::iter::empty(),
        &[],
        |_| None,
        &worth_execution::ExecutionRequest::serial(&worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        )),
    )
    .expect("empty restored correspondence is valid for a registry lifetime test");
    let checkpoint = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointIdentity {
        implementation:
            worth_relational::facade::mvcc::RequiredOutputProducingImplementation::from_checkpoint(
                [0x11; 16], [0x22; 32],
            )
            .unwrap(),
        producer: ("producer".to_owned()).into(),
        posture: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointPosture::Performed,
        source: [1; 32],
        scope,
        source_partition: [2; 32],
        producer_dependency: None,
        idempotency_key: [3; 32],
        roles: (Vec::new()).into(),
        producer_facts: None,
        producer_fact_wire_version: 0,
        metadata: None,
    };
    let completion = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand {
        authority: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority::Restored(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::restoration::WorthQueryRestoredAcceptedOutput {
                checkpoint,
                correspondence: Arc::new(correspondence),
                observation,
                source_scope: scope,
                source_identity: WorthQueryCheckpointSourceIdentity::new_for_test([1; 32]),
                observed_source_facts: Arc::<[_]>::from([]).into(),
                native_output_witness: None,
            },
        ),
        readiness: WorthQueryOutputReadinessDeliveryEvidence::without_execution_for_test(),
    };
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    (*registry.state_for_test()).lock().unwrap().records.insert(
        output_key.clone(),
        DemandRecord {
            source_scope: Some(scope),
            wake: Arc::clone(&wake),
            ..record(
                product_occurrence,
                DemandState::Output(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress::restored_for_test(
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::ReadyCompletion::for_test(completion),
                )),
                1,
            )
        },
    );
    drop(interest(&registry, output_key.clone(), wake));
    assert!((*registry.state_for_test()).lock().unwrap().records[&output_key].has_cached_ready());
    assert!(!(*registry.state_for_test()).lock().unwrap().records[&output_key].is_required());
    let reopened = registry
        .admit_for_test(
            output_key.clone(),
            None,
            scope,
            product_occurrence,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
            None,
            None,
        )
        .expect("the equivalent ordinary demand reopens")
        .0;

    assert!(registry
        .peek_ready_for_test(&reopened)
        .expect("ordered lookup and fixed Arc pin fit the admitted work")
        .is_some());
    assert!(matches!(
        registry.begin_for_test(&reopened),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Ready(_)
    ));
    let worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Ready(completion) =
        registry.begin_for_test(&reopened)
    else {
        panic!("retained output remains ready")
    };
    let mut lookalike = (*completion.authority_for_test()).clone();
    if let worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority::Restored(restored) = &mut lookalike {
        (*restored.checkpoint_for_test()).idempotency_key = [4; 32];
    }
    let unrelated = registry
        .admit_for_test(
            output_key.clone(),
            None,
            scope,
            product_occurrence,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
            None,
            Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refresh_predecessor::OutputRefreshPredecessor::of_for_test(
                &lookalike, &reopened,
            )),
        )
        .unwrap()
        .0;
    assert!(
        matches!(
            registry.begin_for_test(&unrelated),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Ready(_)
        ),
        "a lookalike predecessor cannot reopen the accepted output"
    );
    drop(unrelated);
    let refreshed = registry
        .admit_for_test(
            output_key.clone(),
            None,
            scope,
            product_occurrence,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
            None,
            Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refresh_predecessor::OutputRefreshPredecessor::of_for_test(
                &(*completion.authority_for_test()),
                &reopened,
            )),
        )
        .unwrap()
        .0;
    assert!(matches!(
        registry.begin_for_test(&refreshed),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Schedule(None)
    ));
    registry.finish_scheduling_for_test(
        &refreshed,
        None,
        &mut Ok(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputSchedulingResult::Scheduled),
    );
    assert!(
        matches!(registry.begin_for_test(&refreshed), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Execute { successor_of: Some(identity) } if identity == [3; 32]),
        "the exact restored predecessor reopens with its retained idempotency lineage"
    );
    drop(refreshed);
    drop(reopened);
    registry.release_product_occurrence_for_test(product_occurrence);
    assert!(!(*registry
        .state_for_test())
        .lock()
        .unwrap()
        .records
        .contains_key(&output_key));
}
