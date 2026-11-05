use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity;
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
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
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
    let checkpoint = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointIdentity::new_for_test(worth_relational::facade::mvcc::RequiredOutputProducingImplementation::from_checkpoint(
                [0x11; 16], [0x22; 32],
            )
            .unwrap(), ("producer".to_owned()).into(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_output_demand::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointPosture::Performed, [1; 32], scope, [2; 32], None, [3; 32], (Vec::new()).into(), None, 0, None);
    let completion = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand::new_for_test(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority::Restored(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::restoration::WorthQueryRestoredAcceptedOutput::new_for_test(checkpoint, Arc::new(correspondence), observation, scope, WorthQueryCheckpointSourceIdentity::new_for_test([1; 32]), Arc::<[_]>::from([]).into(), None),
        ), WorthQueryOutputReadinessDeliveryEvidence::without_execution_for_test());
    let wake = Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new()));
    (*(*registry.state_for_test()).lock().unwrap().records_mut_for_test()).insert_for_test(
        output_key.clone(),
        { let __source_scope = Some(scope); let __wake = Arc::clone(&wake); let mut __base = record(
                product_occurrence,
                DemandState::Output(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress::restored_for_test(
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::ReadyCompletion::for_test(completion),
                )),
                1,
            ); *__base.source_scope_mut_for_test() = __source_scope; *__base.wake_mut_for_test() = __wake; __base },
    );
    drop(interest(&registry, output_key.clone(), wake));
    assert!((*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_for_test())[&output_key]
        .has_cached_ready_for_test());
    assert!(!(*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_for_test())[&output_key]
        .is_required_for_test());
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
        (*(*restored.checkpoint_mut_for_test()).idempotency_key_mut_for_test()) = [4; 32];
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
    assert!(!(*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_for_test())
    .contains_key_for_test(&output_key));
}
