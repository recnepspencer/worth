//! A native add/remove restores the same observed relation set. Clearing its
//! dirty mark requires native verification and a still-current companion image.
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts;

use super::super::DirtyReverification;
use super::super::SettlementVerificationStop;
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::AccountBlocked;
use crate::domain_computation::primary_graph::tests::fixture::PrincipalIdentityField;
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
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
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
                .entity_id()
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
                .entity_id()
        })
        .expect("the fixture read owns its installed host call");
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let kind = graph
        .layout()
        .relation(AccountBlocked::reference().name())
        .unwrap()
        .kind;
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let identity = RecordedSettlementIdentity::retain(&SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity().clone(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(account),
        output_binding: OutputBindingIdentity::declared("RelationAbsenceOutput"),
    }, coordinate, 0);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let request_execution = &host_custody.borrowed_request_for_test();
            handle.with_runtime_mut(request_execution, |runtime| {
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
                let tracked_facts = crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false,
                    Arc::from([WorthQueryApplicationObservedFact::Relation {
                        relation_kind: kind, from: principal, to: account, matching_relations: Vec::new(),
                    }]));
                owner.register_settlement(SettlementRegistration {
                    computation: Some(super::super::ComputationRegistration::retained()),
                    work_membership: None, identity: identity.clone(), facts: tracked_facts.clone(),
                    output_facts: None, read_basis: basis.clone(), stale_at_read_basis: OrdSet::new(),
                    requirement: None, upstream: OrdSet::new(),
                }).unwrap();
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
                        .reverify_dirty(
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
                    .reverify_dirty(&host_custody.borrowed_request_for_test(), runtime, &restored_handle, &restored, &identity)
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
                        owner.clear_verified_dirty(verified),
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
                            .reverify_dirty(
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
                    .reverify_dirty(&host_custody.borrowed_request_for_test(), runtime, &latest_handle, &latest, &identity)
                    .unwrap()
                else {
                    panic!("fresh exact native comparison grants live clearing");
                };

                let diagnostic_image = owner.capture_full_verification(runtime, &latest_handle, &latest).unwrap();
                owner.clear_verified_dirty(verified).unwrap();
                {
                    diagnostic_image
                        .fence_semantic_image(&OrdSet::unit(identity.clone()))
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

                let changes = owner.computation_changes(runtime, &latest_handle, &identity, &tracked_facts).unwrap().expect("ordinary clear preserves original computation frontier");
                assert_eq!(changes.at_snapshot(runtime, &latest_handle).unwrap().iter().copied().collect::<Vec<_>>(), [0]);
                assert!(changes.at_snapshot(runtime, &basis_handle).is_none(), "later evidence cannot bless an earlier source snapshot");
                let original = tracked_facts.for_comparison().unwrap().facts().clone();
                let changes = changes.bind_original(original.clone());
                let control = crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(request_execution.allocation_policy(), None);
                let carry = changes.carry(&original, 1, &OrdSet::new(), &latest_handle, control).unwrap().unwrap();
                let empty = worth_relational::facade::history::RelationalDescriptiveTouchGraph::Exact(Vec::new());
                let selected = carry.select(&empty, 1, &[], control).unwrap().unwrap().ordinals;
                assert_eq!(selected.iter().copied().collect::<Vec<_>>(), [0], "ABA frontier survives zero fresh calls and exact-empty candidate touches");
                assert!(carry.select(&worth_relational::facade::history::RelationalDescriptiveTouchGraph::Unavailable, 1, &[], control).unwrap().is_none(), "unavailable is never exact empty");
                let foreign = Arc::<[WorthQueryApplicationObservedFact]>::from(original.iter().cloned().collect::<Vec<_>>()).into();
                assert!(changes.carry(&foreign, 1, &OrdSet::new(), &latest_handle, control).unwrap().is_none(), "equal data cannot substitute original backing");
                assert!(changes.carry(&original, 1, &OrdSet::new(), &basis_handle, control).unwrap().is_none(), "original body does not substitute the selected native pin");

                let (reader_handle, reader_position) = snapshot(runtime);
                assert_ne!(reader_handle, latest_handle, "a new reader owns a new native pin");
                assert_eq!(reader_position, latest);
                assert!(changes.carry(&original, 1, &OrdSet::new(), &reader_handle, control).unwrap().is_none(), "raw cutoff custody cannot substitute the reader pin");
                let reader_changes = changes.for_snapshot(runtime, &reader_handle).unwrap();
                assert_eq!(reader_changes.ordinals(), changes.ordinals());
                assert!(reader_changes.matches_publication_source(runtime, &reader_handle).unwrap());
                assert!(!reader_changes.matches_publication_source(runtime, &basis_handle).unwrap(), "provider readmission cannot carry revisions across a different native position");

                assert!(reader_changes.carry(&original, 1, &OrdSet::new(), &reader_handle, control).unwrap().is_some(), "the marking owner's actual position validation carries exact reader custody");
                assert!(changes.for_snapshot(runtime, &basis_handle).is_none());
                runtime.snapshots().release_snapshot(&reader_handle).unwrap();
let old = owner.computation_changes(runtime, &basis_handle, &identity, &tracked_facts).unwrap().unwrap();


                assert!(old.at_snapshot(runtime, &basis_handle).unwrap().is_empty(), "historical native frontier stays immutable");
                let different_backing = RetainedSourceFacts::for_test(false, Arc::from([]));
                assert!(owner.computation_changes(runtime, &latest_handle, &identity,
                    &different_backing).unwrap().is_none(),
                    "a source-current row cannot lend its frontier to foreign fact backing");
                for snapshot in [basis_handle, changed_handle, restored_handle, latest_handle] {
                    runtime.snapshots().release_snapshot(&snapshot).unwrap();
                }
            })
        })
        .expect("the fixture read owns its installed host call");
}
