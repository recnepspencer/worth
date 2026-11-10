//! Native source custody, shared settlement installation, retirement and races.
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;

#[test]
fn native_projection_continuation_keeps_exact_bindings_and_rejects_source_races() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::at("AccountStatus"),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test()
        })
        .expect("the fixture read owns its installed host call");
    let (_, product, _) = selected.into_parts_for_test();
    let observation = product.observation_for_test();
    let coordinate = ProductCoordinate::new_for_test(
        observation.lifecycle_incarnation(),
        observation.reference_generation().get(),
    );
    let source = SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), OutputBindingIdentity::declared_for_test("StatusOutput"));
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = (*handle.source_owner_for_test()).invalidation_owner_for_test();
    let status_ref = AccountStatus::at("AccountStatus");
    let status = graph
        .layout_for_test()
        .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label_ref = AccountLabel::at("AccountLabel");
    let label = graph
        .layout_for_test()
        .field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        write_field(request_execution, runtime, entity, label, "prime")
    });
    let (before_handle, before, facts) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            handle.with_runtime(request_execution, |runtime| {
                let (snapshot, basis) = snapshot(runtime);
                let facts: Arc<[_]> = Arc::from([field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &snapshot,
                    entity,
                    status.clone(),
                )]);
                (snapshot, basis, facts)
            })
        })
        .expect("the fixture read owns its installed host call");

    let first = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 0);
    owner
        .register_settlement_for_test(SettlementRegistration::new_for_test(Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::computation_changes::ComputationRegistration::retained_for_test()), None, first.clone(), RetainedSourceFacts::for_test_for_test(false, facts.clone()), None, before.clone(), OrdSet::new(), None, OrdSet::new()))
        .unwrap();
    let cell = owner.cell_for_read_for_test(&before).unwrap().unwrap();
    let historical = cell.read_image();

    let original = (*(*(*historical.payload().current_for_test()).settlements_for_test())[&first]
        .facts_for_test())
    .clone();
    let original_body = original.postconditions_for_test().clone();
    let continuation = handle.with_runtime_mut_for_test(request_execution, |runtime| {
        let changes = owner
            .computation_changes_for_test(runtime, &before_handle, &first, &original)
            .unwrap()
            .unwrap()
            .bind_original_for_test(original_body.clone());
        let control = StorageControl::new_for_test(request_execution.allocation_policy(), None);
        let carry = changes
            .carry_for_test(
                &original_body,
                original_body.len_for_test(),
                &OrdSet::new(),
                &before_handle,
                control,
            )
            .unwrap()
            .unwrap();
        let basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap();
        let mut transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("posting-correspondence").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields: AspectFieldPatch::from(BTreeMap::from([(
                            status.clone(),
                            AspectValue::String(InternedString::Raw("closed".to_owned())),
                        )])),
                    }),
                )),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let candidate = runtime
            .prepare_branch_transaction(request_execution, transaction)
            .unwrap();
        assert!(carry
            .matches_publication_source_for_test(runtime, &before_handle)
            .unwrap());
        let selection = candidate
            .with_prepared_descriptive_touches(|touches| {
                carry.select_for_test(touches, original_body.len_for_test(), &[], control)
            })
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(
            (*selection.ordinals_for_test()).contains(&0),
            "the actual candidate selects the native field read"
        );
        drop(candidate);
        // This owner proof seals unchanged revisions at the same live image;
        // it does not claim a performed producer or original-read permission.
        selection
            .basis_into_for_test()
            .seal_for_test(original_body.clone(), before.clone(), control)
            .unwrap()
    });

    let second = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 1);
    let registration = |identity, body, continuation| {
        SettlementRegistration::new_for_test(
            None,
            None,
            identity,
            RetainedSourceFacts::for_test_for_test(false, body)
                .with_posting_continuation_for_test(continuation),
            None,
            before.clone(),
            OrdSet::new(),
            None,
            OrdSet::new(),
        )
    };
    // Exact completed-body correspondence, not data equality, permits sharing.
    owner
        .register_settlement_for_test(registration(
            second.clone(),
            facts.clone(),
            Some(continuation.clone()),
        ))
        .unwrap();
    let installed = cell.read_image();
    let a = Arc::clone(
        (*(*installed.payload().current_for_test()).settlements_for_test())[&first]
            .posting_projection_for_test(),
    );
    let b = Arc::clone(
        (*(*installed.payload().current_for_test()).settlements_for_test())[&second]
            .posting_projection_for_test(),
    );
    assert!(Arc::ptr_eq(&a, &b));
    assert_eq!(
        (*(*installed.payload().current_for_test()).posting_bindings_for_test()).len(),
        1
    );
    assert_eq!(
        (*(*(*installed.payload().current_for_test()).posting_bindings_for_test())
            [a.id_for_test()]
        .settlements_for_test()
        .unwrap()
        .settlements_for_test())
        .len(),
        2
    );
    let unrelated = Arc::from(facts.to_vec().into_boxed_slice());
    let third = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 2);
    owner
        .register_settlement_for_test(registration(
            third.clone(),
            unrelated,
            Some(continuation.clone()),
        ))
        .unwrap();
    let reconstructed = cell.read_image();
    assert!(
        !Arc::ptr_eq(
            (*(*reconstructed.payload().current_for_test()).settlements_for_test())[&third]
                .posting_projection_for_test(),
            &a
        ),
        "equal unrelated backing reconstructs"
    );
    let retired = owner.retire_settlements_for_test(std::slice::from_ref(&first));
    assert!(retired.contains(&first));
    let retired_image = cell.read_image();
    assert!(
        !(*(*retired_image.payload().current_for_test()).settlements_for_test())
            .contains_key(&first)
    );
    assert!(
        (*(*(*retired_image.payload().current_for_test()).posting_bindings_for_test())
            [a.id_for_test()]
        .settlements_for_test()
        .unwrap()
        .settlements_for_test())
        .contains(&second)
    );
    assert!(
        (*(*historical.payload().current_for_test()).settlements_for_test()).contains_key(&first)
    );
    assert!(
        !(*(*historical.payload().current_for_test()).settlements_for_test()).contains_key(&second)
    );
    // A prepared current image may not overwrite a real intervening native edit.
    let fourth = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 3);
    let prepared = owner
        .prepare_settlement_for_test(registration(
            fourth.clone(),
            facts.clone(),
            Some(continuation),
        ))
        .unwrap();
    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        write_field(request_execution, runtime, entity, status, "closed")
    });
    assert!(prepared.install_for_test().is_err());
    let (after_handle, after) = handle.with_runtime(request_execution, snapshot);
    assert!(matches!(
        currentness(owner, &after, &second),
        SourceSettlementCurrentness::Dirty(_)
    ));
    assert!(matches!(
        currentness(owner, &after, &fourth),
        SourceSettlementCurrentness::FullVerificationRequired(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason::MissingSettlement
        )
    ));
    drop(retired_image);
    drop(reconstructed);
    drop(installed);
    drop(historical);
    drop(original);

    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        runtime
            .snapshots()
            .release_snapshot(&before_handle)
            .unwrap();
        runtime.snapshots().release_snapshot(&after_handle).unwrap();
    });
}
