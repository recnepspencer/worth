//! Owner controls on real native deletion, separate from the installed Ready journey.
use crate::domain_computation::primary_graph::output_lineage::native_output_witness::checkpoint::tests::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_relational::facade::transactions::DeleteEntityIntent;

#[test]
fn mixed_native_witness_retains_exact_retirement_and_refuses_tampered_recovery() {
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
    let resolve = |key: &str| {
        world
            .application
            .with_host_advancement(|phase| {
                selected
                    .resolve_entity(
                        &phase,
                        AccountIdentity::at("AccountIdentity"),
                        key.to_owned(),
                        &live_scope(),
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap()
                    .entity_id_for_test()
            })
            .expect("the fixture invocation owns its installed host call")
    };
    let live = resolve("account-1");
    let retired = resolve("account-2");
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let layout = graph.layout_for_test();
    let handle = graph.integration_handle_for_test();
    let correspondence = mixed_correspondence(request_execution, live, retired);
    let foreign = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let foreign_selected = foreign
        .application
        .with_host_advancement(|phase| {
            foreign.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let request_execution = &host_custody.borrowed_request_for_test();
            handle.with_runtime_mut_for_test(request_execution, |runtime| {
                let before = snapshot(runtime);
                let prepare = || {
                    PreparedNativeOutputWitness::prepare_republication_for_test(&correspondence, layout)
                        .unwrap()
                        .unwrap()
                };

                assert!(
                    prepare()
                        .finish_for_test(&correspondence, runtime, &before)
                        .is_none(),
                    "live is not retired"
                );

                let prepared = prepare();
                publish(
                    request_execution,
                    runtime,
                    WorkerIntentBatch::new("mixed-native-retirement").push(MutationIntent::Entity(
                        EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: retired }),
                    )),
                );
                let committed = snapshot(runtime);
                let witness = prepared
                    .finish_for_test(&correspondence, runtime, &committed)
                    .expect("actual commit seals every role");
                let witness = witness.get().unwrap();
                let equivalent = prepare()
                    .finish_for_test(&correspondence, runtime, &committed)
                    .unwrap();
                assert!(witness.same_output_as_for_test(equivalent.get().unwrap()).unwrap());
                let mut changed = prepare()
                    .finish_for_test(&correspondence, runtime, &committed)
                    .unwrap();
                let changed = Arc::get_mut(&mut changed).unwrap().get_mut().unwrap();
                (*(*changed.roles_mut_for_test())[1].posture_mut_for_test()) = WorthQueryApplicationOutputPosture::Preserve;
                assert!(
                    !witness.same_output_as_for_test(changed).unwrap(),
                    "matching entity and revisions cannot erase a retirement posture"
                );
                (*(*changed.roles_mut_for_test())[1].posture_mut_for_test()) = WorthQueryApplicationOutputPosture::Retire;
                let metadata =
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::native_output_witness::retirement::native_metadata(runtime, &committed, retired, request_execution)
                        .unwrap()
                        .unwrap();
                assert_ne!((*metadata.created_at_for_test()), (*metadata.deleted_at_for_test()));
                (*(*(*changed.roles_mut_for_test())[1].retirement_mut_for_test()).as_mut().unwrap().created_at_mut_for_test()) = *metadata.deleted_at_for_test();
                assert!(
                    !witness.same_output_as_for_test(changed).unwrap(),
                    "the original creation version belongs to exact retirement equality"
                );
                assert_eq!((*metadata.deleted_at_for_test()), committed.version_id());
                assert!(runtime
                    .read_truth()
                    .read_exact_snapshot_live_entity_kind(request_execution, &committed, retired)
                    .unwrap()
                    .is_none());
                assert_eq!(
                    runtime
                        .read_truth()
                        .read_exact_snapshot_live_entity_kind(request_execution, &committed, live)
                        .unwrap(),
                    Some(*metadata.kind_for_test())
                );
                assert!(witness
                    .unchanged_in_for_test(runtime, &committed, request_execution)
                    .unwrap());
                assert!(!witness
                    .unchanged_in_for_test(runtime, &before, request_execution)
                    .unwrap());
                assert!(!witness
                    .unchanged_in_for_test(
                        runtime,
                        foreign_selected.application_basis_for_test().snapshot_handle(),
                        request_execution
                    )
                    .unwrap());

                assert!(
                    witness.prepare_fact_projection_for_test().unwrap().is_none(),
                    "mixed retired output stays checkpoint-ineligible"
                );
                let facts = [Fact::RetiredOutputEntity {
                    entity_id: retired,
                    kind: (*metadata.kind_for_test()),
                    created_at: (*metadata.created_at_for_test()),
                    deleted_at: (*metadata.deleted_at_for_test()),
                    read_locator: "original-decision-read".into(),
                }];
                assert!(facts[0]
                    .remains_equal_in_for_test(
                        runtime,
                        &host_custody.borrowed_request_for_test(),
                        &committed
                    )
                    .unwrap());
                assert!(SealedNativeOutputWitness::from_checkpoint_facts_for_test(
                    &correspondence,
                    layout,
                    &facts
                )
                .unwrap()
                .is_none());
                let external_dead = Fact::SourceEntity { entity_id: retired };
                assert!(!witness.covers_content_fact_for_test(&external_dead).unwrap());
                assert!(!external_dead
                    .remains_equal_in_for_test(
                        runtime,
                        &host_custody.borrowed_request_for_test(),
                        &committed
                    )
                    .unwrap());
                assert!(
                    !witness
                        .checkpoint_facts_current_in_for_test(
                            &host_custody.borrowed_request_for_test(),
                            runtime,
                            &committed,
                            &facts
                        )
                        .unwrap(),
                    "even genuine retired facts do not grant checkpoint eligibility"
                );
                let wrong_generation = mixed_correspondence(
                    &host_custody.borrowed_request_for_test(),
                    live,
                    EntityId::new(
                        retired.partition_id,
                        retired.local_slot_value(),
                        retired.generation_value() + 1,
                    ),
                );
                assert!(prepare()
                    .finish_for_test(&wrong_generation, runtime, &committed)
                    .is_none());
                let missing = WorthQueryApplicationOutputCorrespondence::default();
                assert!(prepare().finish_for_test(&missing, runtime, &committed).is_none());
                let mut wrong_kind = prepare();
                (*(*wrong_kind.roles_mut_for_test())[1].kind_mut_for_test()) = KindId::new((*metadata.kind_for_test()).as_u32() + 1);
                assert!(wrong_kind
                    .finish_for_test(&correspondence, runtime, &committed)
                    .is_none());
                let mut wrong_posture = prepare();
                (*(*wrong_posture.roles_mut_for_test())[1].posture_mut_for_test()) = WorthQueryApplicationOutputPosture::Preserve;
                assert!(wrong_posture
                    .finish_for_test(&correspondence, runtime, &committed)
                    .is_none());
                let label_ref = AccountLabel::at("AccountLabel");
                let label = layout
                    .field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
                    .unwrap()
                    .clone();
                change_label(request_execution, runtime, live, label.clone());
                let later = snapshot(runtime);
                assert_eq!(
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::native_output_witness::retirement::native_metadata(runtime, &later, retired, request_execution)
                        .unwrap(),
                    Some(metadata),
                    "original tuple persists at later observations"
                );
                assert!(
                    prepare().finish_for_test(&correspondence, runtime, &later).is_none(),
                    "later version cannot issue an old retirement anew"
                );
                assert!(
                    !witness
                        .unchanged_in_for_test(runtime, &later, request_execution)
                        .unwrap(),
                    "live aspects still invalidate the mixed proof"
                );
                publish(
                    request_execution,
                    runtime,
                    WorkerIntentBatch::new("mixed-output-aba").push(MutationIntent::Entity(
                        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                            entity_id: live,
                            fields: AspectFieldPatch::from(BTreeMap::from([(
                                label,
                                AspectValue::String("primary".into()),
                            )])),
                        }),
                    )),
                );
                let aba = snapshot(runtime);
                assert!(
                    !witness
                        .unchanged_in_for_test(runtime, &aba, request_execution)
                        .unwrap(),
                    "same-value ABA keeps its changed revision"
                );
                assert!(
                    witness
                        .unchanged_in_for_test(runtime, &committed, request_execution)
                        .unwrap(),
                    "the original held native observation remains exact after ABA"
                );
                runtime.snapshots().release_snapshot(&aba).unwrap();
                assert!(
                    !witness
                        .unchanged_in_for_test(runtime, &aba, request_execution)
                        .unwrap(),
                    "released basis cannot authorize a witness"
                );
                for snapshot in [&before, &committed, &later] {
                    runtime.snapshots().release_snapshot(snapshot).unwrap();
                }
            })
        })
        .expect("the fixture read owns its installed host call");
}

pub fn mixed_correspondence(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    live: EntityId,
    retired: EntityId,
) -> WorthQueryApplicationOutputCorrespondence {
    WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
        TypeId::of_for_test::<()>(),
        TypeId::of_for_test::<()>(),
        std::iter::empty(),
        &[
            ("live", WorthQueryApplicationOutputPosture::Preserve, live),
            (
                "retired",
                WorthQueryApplicationOutputPosture::Retire,
                retired,
            ),
        ]
        .into_iter()
        .map(|(role, posture, entity)| {
            WorthQueryCheckpointOutputRole::new_for_test(
                role.into(),
                posture,
                "Account".into(),
                entity,
            )
        })
        .collect::<Vec<_>>(),
        |_| Some(TypeId::of_for_test::<()>()),
        request_execution,
    )
    .unwrap()
}

pub fn publish(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    batch: WorkerIntentBatch,
) {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(request_execution, transaction)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(request_execution, candidate)
    else {
        panic!("actual native mutation performs")
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}
