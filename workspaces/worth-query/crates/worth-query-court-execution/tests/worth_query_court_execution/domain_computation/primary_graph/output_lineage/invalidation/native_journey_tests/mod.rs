//! The ordinary native writer must mark the installed Query actor before its
//! new source head can be read. No World publication or Query feed is involved.
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

// Fixture evidence carries no performed output projection, so the
// equivalence oracle correctly refuses it; this proof is about metering.

pub use std::collections::BTreeMap;
pub use std::sync::Arc;
pub use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;

pub use im::OrdSet;
pub use worth_foundational::facade::AspectFieldLocator;
pub use worth_foundational::facade::AspectValue;
pub use worth_foundational::facade::InternedString;
pub use worth_foundational::facade::LocatorAuthority;
pub use worth_relational::facade::identity::EntityId;
pub use worth_relational::facade::mvcc::RelationalPublicationOutcome;
pub use worth_relational::facade::mvcc::RelationalTransactionIntent;
pub use worth_relational::facade::runtime::PositionedRelationalSnapshot;
pub use worth_relational::facade::runtime::RelationalRuntime;
pub use worth_relational::facade::snapshots::SnapshotHandle;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::ProductCoordinate;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::settlement_identity::RecordedSettlementIdentity;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::semantic_source::SemanticSource;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::settlement::SettlementRegistration;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::derived::SourceSettlementCurrentness;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::commit_snapshot_closeout::release_test_commit_snapshot;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub struct StatusOutput;

pub fn snapshot(runtime: &RelationalRuntime) -> (SnapshotHandle, PositionedRelationalSnapshot) {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let handle = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let proof = runtime.read_truth().positioned_snapshot(&handle).unwrap();
    (handle, proof)
}

pub fn write_field(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
    locator: AspectFieldLocator,
    value: &str,
) {
    let committed = write_batch(
        execution,
        runtime,
        WorkerIntentBatch::new("native-invalidation-journey").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    locator,
                    AspectValue::String(InternedString::Raw(value.to_owned())),
                )])),
            }),
        )),
    );
    release_test_commit_snapshot(runtime, &committed);
}

pub fn write_batch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    batch: WorkerIntentBatch,
) -> worth_relational::facade::transactions::CommitResult {
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
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("ordinary native field mutation must publish");
    };
    runtime.settle_performed_publication(performed).unwrap()
}

pub fn field_fact(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    handle: &SnapshotHandle,
    entity: EntityId,
    locator: AspectFieldLocator,
) -> WorthQueryApplicationObservedFact {
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        locator.aspect().aspect_key().clone(),
        locator.field_path().clone(),
    );
    let revision = runtime
        .read_truth()
        .project_snapshot(handle)
        .unwrap()
        .entity_field_revision(execution, entity, &locator)
        .expect("fixture read request remains live")
        .unwrap();
    WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: entity,
        locator,
        native_revision: Some(revision),
    }
}

pub fn register(
    owner: &SourceInvalidationOwner,
    identity: Arc<RecordedSettlementIdentity>,
    facts: Arc<[WorthQueryApplicationObservedFact]>,
    basis: &PositionedRelationalSnapshot,
    upstream: OrdSet<Arc<RecordedSettlementIdentity>>,
) {
    owner
        .register_settlement_for_test(SettlementRegistration::new_for_test(None, None, identity, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts::for_test_for_test(false, facts), None, basis.clone(), im::OrdSet::new(), None, upstream))
        .expect("native aligned settlement registers");
}

pub fn currentness(
    owner: &SourceInvalidationOwner,
    basis: &PositionedRelationalSnapshot,
    identity: &RecordedSettlementIdentity,
) -> SourceSettlementCurrentness {
    owner.currentness_for_test(basis, identity).unwrap()
}

#[test]
fn ordinary_native_writer_marks_only_matched_fields_and_actual_downstream_edges() {
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
    let coordinate = ProductCoordinate::new_for_test(
        product.observation_for_test().lifecycle_incarnation(),
        product.observation_for_test().reference_generation().get(),
    );
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = (*handle.source_owner_for_test()).invalidation_owner_for_test();
    let status_ref = AccountStatus::at("AccountStatus");
    let label_ref = AccountLabel::at("AccountLabel");
    let status = graph
        .layout_for_test()
        .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label = graph
        .layout_for_test()
        .field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let source = |output_binding| {
        SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity().clone(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), output_binding)
    };
    let a = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("StatusOutput")),
        coordinate,
        0,
    );
    let b = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("LabelOutput")),
        coordinate,
        0,
    );
    let c = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("DownstreamOutput")),
        coordinate,
        0,
    );
    let late = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("LateOutput")),
        coordinate,
        0,
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let request_execution = &host_custody.borrowed_request_for_test();
            handle.with_runtime_for_test(request_execution, |runtime| {
                // Ensure the real companion has a selected cell even if fixture
                // installation happened before its subscription became active.
                write_field(request_execution, runtime, entity, label.clone(), "prime");
                let (before_handle, before) = snapshot(runtime);
                let status_facts: Arc<[_]> = Arc::from([field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &before_handle,
                    entity,
                    status.clone(),
                )]);
                register(
                    owner,
                    a.clone(),
                    status_facts.clone(),
                    &before,
                    OrdSet::new(),
                );
                register(
                    owner,
                    b.clone(),
                    Arc::from([field_fact(
                        &host_custody.borrowed_request_for_test(),
                        runtime,
                        &before_handle,
                        entity,
                        label.clone(),
                    )]),
                    &before,
                    OrdSet::new(),
                );
                register(
                    owner,
                    c.clone(),
                    Arc::from([]),
                    &before,
                    OrdSet::unit(a.clone()),
                );
                assert!(matches!(
                    currentness(owner, &before, &a),
                    SourceSettlementCurrentness::Clean
                ));
                write_field(request_execution, runtime, entity, label, "changed-label");
                let (label_handle, label_basis) = snapshot(runtime);
                assert!(
                    matches!(
                        currentness(owner, &label_basis, &a),
                        SourceSettlementCurrentness::Clean
                    ),
                    "the sibling aspect touch must not dirty a field-local settlement"
                );
                assert!(matches!(
                    currentness(owner, &label_basis, &c),
                    SourceSettlementCurrentness::Clean
                ));
                match currentness(owner, &label_basis, &b) {
                    SourceSettlementCurrentness::Dirty(ordinals) => {
                        assert_eq!(ordinals, OrdSet::unit(0))
                    }
                    _ => panic!("the matching label fact must be dirty before its head is read"),
                }
                assert!(
                    matches!(
                        currentness(owner, &before, &b),
                        SourceSettlementCurrentness::Clean
                    ),
                    "newer marks cannot leak into the retained earlier snapshot"
                );
                write_field(request_execution, runtime, entity, status, "closed");
                let (after_handle, after) = snapshot(runtime);
                assert!(matches!(
                    currentness(owner, &after, &a),
                    SourceSettlementCurrentness::Dirty(_)
                ));
                match currentness(owner, &after, &c) {
                    SourceSettlementCurrentness::PendingUpstream(upstream) => {
                        assert_eq!(upstream, OrdSet::unit(a))
                    }
                    _ => panic!("only the actual downstream edge must be pending"),
                }
                register(owner, late.clone(), status_facts, &before, OrdSet::new());
                assert!(
                    matches!(
                        currentness(owner, &after, &late),
                        SourceSettlementCurrentness::Dirty(_)
                    ),
                    "late registration must replay actual intervening deliveries"
                );
                for snapshot in [before_handle, label_handle, after_handle] {
                    runtime.snapshots().release_snapshot(&snapshot).unwrap();
                }
            })
        })
        .expect("the fixture read owns its installed host call");
}

#[path = "concurrent_first_writers/mod.rs"]
pub mod concurrent_first_writers;
#[path = "current_registration/mod.rs"]
pub mod current_registration;
#[path = "dirty_clearance_tests/mod.rs"]
pub mod dirty_clearance_tests;
#[path = "displaced_generation/mod.rs"]
pub mod displaced_generation;
#[path = "equal_field_write/mod.rs"]
pub mod equal_field_write;
#[path = "equality/mod.rs"]
pub mod equality;
#[path = "full_verification/mod.rs"]
pub mod full_verification;
#[path = "local_requirement/mod.rs"]
pub mod local_requirement;
#[path = "logical_work_scale/mod.rs"]
pub mod logical_work_scale;
#[path = "pending_own_evidence/mod.rs"]
pub mod pending_own_evidence;
#[path = "population_world/mod.rs"]
pub mod population_world;
#[path = "posting_continuation/mod.rs"]
pub mod posting_continuation;
#[path = "precommit_chain/mod.rs"]
pub mod precommit_chain;
#[path = "replay_propagation/mod.rs"]
pub mod replay_propagation;
#[path = "request_recording/mod.rs"]
pub mod request_recording;
#[path = "required_hints/mod.rs"]
pub mod required_hints;
#[path = "retained_headroom/mod.rs"]
pub mod retained_headroom;
#[path = "retired_image/mod.rs"]
pub mod retired_image;
#[path = "scope_hierarchy/mod.rs"]
pub mod scope_hierarchy;
#[path = "shared_fact_custody/mod.rs"]
pub mod shared_fact_custody;
#[path = "transitive_propagation/mod.rs"]
pub mod transitive_propagation;
#[path = "unchanged_history/mod.rs"]
pub mod unchanged_history;
#[path = "verified_current/mod.rs"]
pub mod verified_current;
