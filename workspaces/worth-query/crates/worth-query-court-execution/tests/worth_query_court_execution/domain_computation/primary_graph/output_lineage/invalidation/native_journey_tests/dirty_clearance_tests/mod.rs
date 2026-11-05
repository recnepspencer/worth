//! A native add/remove restores the same observed relation set. Clearing its
//! dirty mark requires native verification and a still-current companion image.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::verification::DirtyReverification;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::verification::SettlementVerificationStop;
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountBlocked;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PrincipalIdentityField;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::CreatedRelationRef;
use worth_relational::facade::transactions::DeleteRelationIntent;
use worth_relational::facade::transactions::EntityReference;
use worth_relational::facade::transactions::RelationMutationIntent;
use worth_relational::facade::transactions::RelationSpec;

#[test]
fn native_reverification_clears_only_the_verified_live_image() {
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
    let account = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test()
        })
        .expect("the fixture read owns its installed host call");
    let principal = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    PrincipalIdentityField::reference(),
                    1,
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test()
        })
        .expect("the fixture read owns its installed host call");
    let (_, product, _) = selected.into_parts_for_test();
    let coordinate = ProductCoordinate::new_for_test(
        product.observation_for_test().lifecycle_incarnation(),
        product.observation_for_test().reference_generation().get(),
    );
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    let kind = *graph
        .layout_for_test()
        .relation(AccountBlocked::reference().name())
        .unwrap()
        .kind_for_test();
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout_for_test()
        .field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let identity = RecordedSettlementIdentity::retain_for_test(&SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity().clone(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(account), OutputBindingIdentity::declared_for_test("RelationAbsenceOutput")), coordinate, 0);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let request_execution = &host_custody.borrowed_request_for_test();
            handle.with_runtime_for_test(request_execution, |runtime| {
                write_field(request_execution, runtime, account, label.clone(), "prime");
                let (basis_handle, basis) = snapshot(runtime);
                assert!(runtime
                    .read_truth()
                    .visible_relations_of_kind(
                        &host_custody.borrowed_request_for_test(),
                        kind,
                        basis_handle.version_id()
                    )
                    .expect("fixture read request remains live")
                    .is_empty());
                let tracked_facts = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts::for_test_for_test(false,
                    Arc::from([WorthQueryApplicationObservedFact::Relation {
                        relation_kind: kind, from: principal, to: account, matching_relations: Vec::new(),
                    }]));
                owner.register_settlement_for_test(SettlementRegistration::new_for_test(Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::computation_changes::ComputationRegistration::retained_for_test()), None, identity.clone(), tracked_facts.clone(), None, basis.clone(), OrdSet::new(), None, OrdSet::new())).unwrap();
                let created = CreatedRelationRef {
                    partition_id: PartitionId::main(),
                    kind_id: kind,
                    client_key: ClientKey::raw("clearance-relation"),
                    source: EntityReference::Existing(principal),
                    target: EntityReference::Existing(account),
                };
                let result = write_batch(
                    request_execution,
                    runtime,
                    WorkerIntentBatch::new("add-clearance-relation").push(MutationIntent::Create(
                        CreateIntent::Relation(RelationSpec {
                            partition_id: created.partition_id,
                            kind_id: created.kind_id,
                            client_key: created.client_key.clone(),
                            source: created.source.clone(),
                            target: created.target.clone(),
                            fields: AspectFieldPatch::default(),
                        }),
                    )),
                );
                let relation = result.created_relation(&created).unwrap();
                release_test_commit_snapshot(runtime, &result);
                let (changed_handle, changed) = snapshot(runtime);
                assert!(matches!(
                    owner
                        .reverify_dirty_for_test(
                            &host_custody.borrowed_request_for_test(),
                            runtime,
                            &changed_handle,
                            &changed,
                            &identity
                        )
                        .unwrap(),
                    DirtyReverification::ChangedOrdinal(0)
                ));
                let deleted = write_batch(
                    request_execution,
                    runtime,
                    WorkerIntentBatch::new("remove-clearance-relation").push(
                        MutationIntent::Relation(RelationMutationIntent::Delete(
                            DeleteRelationIntent {
                                relation_id: relation,
                            },
                        )),
                    ),
                );
                release_test_commit_snapshot(runtime, &deleted);
                let (restored_handle, restored) = snapshot(runtime);

                assert!(matches!(
                    currentness(owner, &restored, &identity),
                    SourceSettlementCurrentness::Dirty(_)
                ));
                let DirtyReverification::Verified(verified) = owner
                    .reverify_dirty_for_test(&host_custody.borrowed_request_for_test(), runtime, &restored_handle, &restored, &identity)
                    .unwrap()
                else {
                    panic!("the actual restored relation set verifies at the live root");
                };
                write_field(
                    request_execution,
                    runtime,
                    account,
                    label,
                    "unrelated-label",
                );
                assert!(
                    matches!(
                        owner.clear_verified_dirty_for_test(verified),
                        Err(SettlementVerificationStop::Edit(_))
                    ),
                    "a native head move must invalidate old clearing authority"
                );
                let (latest_handle, latest) = snapshot(runtime);
                assert!(matches!(
                    currentness(owner, &latest, &identity),
                    SourceSettlementCurrentness::Dirty(_)
                ));
                assert!(
                    matches!(
                        owner
                            .reverify_dirty_for_test(
                                &host_custody.borrowed_request_for_test(),
                                runtime,
                                &restored_handle,
                                &restored,
                                &identity
                            )
                            .unwrap(),
                        DirtyReverification::HistoricalCurrent
                    ),
                    "a historical native comparison grants only a view witness"
                );

                let DirtyReverification::Verified(verified) = owner
                    .reverify_dirty_for_test(&host_custody.borrowed_request_for_test(), runtime, &latest_handle, &latest, &identity)
                    .unwrap()
                else {
                    panic!("fresh exact native comparison grants live clearing");
                };

                let diagnostic_image = owner.capture_full_verification_for_test(runtime, &latest_handle, &latest).unwrap();
                owner.clear_verified_dirty_for_test(verified).unwrap();
                {
                    diagnostic_image
                        .fence_semantic_image_for_test(&OrdSet::unit(identity.clone()))
                        .unwrap();
                    drop(diagnostic_image);
                }
                assert!(matches!(
                    currentness(owner, &latest, &identity),
                    SourceSettlementCurrentness::Clean
                ));
                assert!(
                    matches!(
                        currentness(owner, &restored, &identity),
                        SourceSettlementCurrentness::Dirty(_)
                    ),
                    "live clearing cannot rewrite a retained dirty image"
                );

                let changes = owner.computation_changes_for_test(runtime, &latest_handle, &identity, &tracked_facts).unwrap().expect("ordinary clear preserves original computation frontier");
                assert_eq!(changes.at_snapshot_for_test(runtime, &latest_handle).unwrap().iter().copied().collect::<Vec<_>>(), [0]);
                assert!(changes.at_snapshot_for_test(runtime, &basis_handle).is_none(), "later evidence cannot bless an earlier source snapshot");
                let original = tracked_facts.for_comparison().unwrap().facts_for_test().clone();
                let changes = changes.bind_original_for_test(original.clone());
                let control = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(request_execution.allocation_policy(), None);
                let carry = changes.carry_for_test(&original, 1, &OrdSet::new(), &latest_handle, control).unwrap().unwrap();
                let empty = worth_relational::facade::history::RelationalDescriptiveTouchGraph::Exact(Vec::new());
                let selected = carry.select_for_test(&empty, 1, &[], control).unwrap().unwrap().ordinals_into_for_test();
                assert_eq!(selected.iter().copied().collect::<Vec<_>>(), [0], "ABA frontier survives zero fresh calls and exact-empty candidate touches");
                assert!(carry.select_for_test(&worth_relational::facade::history::RelationalDescriptiveTouchGraph::Unavailable, 1, &[], control).unwrap().is_none(), "unavailable is never exact empty");
                let foreign = Arc::<[WorthQueryApplicationObservedFact]>::from(original.iter_for_test().cloned().collect::<Vec<_>>()).into();
                assert!(changes.carry_for_test(&foreign, 1, &OrdSet::new(), &latest_handle, control).unwrap().is_none(), "equal data cannot substitute original backing");
                assert!(changes.carry_for_test(&original, 1, &OrdSet::new(), &basis_handle, control).unwrap().is_none(), "original body does not substitute the selected native pin");

                let (reader_handle, reader_position) = snapshot(runtime);
                assert_ne!(reader_handle, latest_handle, "a new reader owns a new native pin");
                assert_eq!(reader_position, latest);
                assert!(changes.carry_for_test(&original, 1, &OrdSet::new(), &reader_handle, control).unwrap().is_none(), "raw cutoff custody cannot substitute the reader pin");
                let reader_changes = changes.for_snapshot_for_test(runtime, &reader_handle).unwrap();
                assert_eq!(reader_changes.ordinals_for_test(), changes.ordinals_for_test());
                assert!(reader_changes.matches_publication_source_for_test(runtime, &reader_handle).unwrap());
                assert!(!reader_changes.matches_publication_source_for_test(runtime, &basis_handle).unwrap(), "provider readmission cannot carry revisions across a different native position");

                assert!(reader_changes.carry_for_test(&original, 1, &OrdSet::new(), &reader_handle, control).unwrap().is_some(), "the marking owner's actual position validation carries exact reader custody");
                assert!(changes.for_snapshot_for_test(runtime, &basis_handle).is_none());
                runtime.snapshots().release_snapshot(&reader_handle).unwrap();
let old = owner.computation_changes_for_test(runtime, &basis_handle, &identity, &tracked_facts).unwrap().unwrap();


                assert!(old.at_snapshot_for_test(runtime, &basis_handle).unwrap().is_empty(), "historical native frontier stays immutable");
                let different_backing = RetainedSourceFacts::for_test_for_test(false, Arc::from([]));
                assert!(owner.computation_changes_for_test(runtime, &latest_handle, &identity,
                    &different_backing).unwrap().is_none(),
                    "a source-current row cannot lend its frontier to foreign fact backing");
                for snapshot in [basis_handle, changed_handle, restored_handle, latest_handle] {
                    runtime.snapshots().release_snapshot(&snapshot).unwrap();
                }
            })
        })
        .expect("the fixture read owns its installed host call");
}
