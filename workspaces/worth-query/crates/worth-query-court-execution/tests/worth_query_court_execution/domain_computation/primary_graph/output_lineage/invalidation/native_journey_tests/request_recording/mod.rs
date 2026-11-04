//! A request that compares a retained output in full pays to record its row.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;

use std::sync::OnceLock;

use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryCheckpointOutputRole;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::verification::ConsumedOutputVerification;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::current_output::RetainedOutputCurrentnessRead;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::native_output_witness::SealedNativeOutputWitness;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;

#[test]
fn a_demand_records_the_row_it_genuinely_verified() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (entity, identity) = world
        .application
        .with_host_advancement(|phase| primed_output(&phase, &world))
        .expect("the fixture read owns its installed host call");
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request_for_test(); handle.with_runtime(request_execution, |runtime| {
        let snapshot = selected.application_basis_for_test().snapshot_handle();
        let positioned = runtime.read_truth().positioned_snapshot(snapshot).unwrap();
        let mut read = RetainedOutputCurrentnessRead::new_for_test(Arc::clone(&identity), None, Arc::from([]), Some(account_witness(&host_custody.borrowed_request_for_test(), &world, runtime, snapshot, entity)), true, None);
let recorded = || {
            matches!(
                owner.currentness_for_test(&positioned, &identity),
                Ok(SourceSettlementCurrentness::Clean)
            )
        };


        assert!(!recorded());
        // Neither absence nor an empty comparison may replace the original
        // own-publication gate. These reads hold no receipt and cannot mark.
        for facts in [None, RetainedSourceFacts::for_test_for_test(false, Arc::from([])).for_comparison()] {
            (*read.facts_mut_for_test()) = facts;

            let stopped = selected.require_current_read_for_test(&host_custody.borrowed_request_for_test(),
                runtime, &read, None).unwrap_err();
            assert_eq!(stopped.kind(), worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind::Superseded);
            assert!(!recorded(), "factless refusal cannot establish a verified row");
        }
        let label = AccountLabel::reference();
        let locator = graph.layout_for_test().field_locator_for_test(label.entity(), label.aspect(), label.field()).unwrap().clone();
        let fact = field_fact(&host_custody.borrowed_request_for_test(), runtime, snapshot, entity, locator);
        (*read.facts_mut_for_test()) = RetainedSourceFacts::for_test_for_test(false, Arc::from([fact])).for_comparison();
        assert!(selected
            .require_current_read_for_test(&host_custody.borrowed_request_for_test(), runtime, &read, None)
            .is_ok());
        assert!(recorded(), "the genuine native verification registers the row");
    }) }).expect("the fixture read owns its installed host call");
}

#[test]
fn a_verification_records_the_row_it_compared() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (entity, identity) = world
        .application
        .with_host_advancement(|phase| primed_output(&phase, &world))
        .expect("the fixture read owns its installed host call");
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request_for_test(); handle.with_runtime(request_execution, |runtime| {
        let snapshot = selected.application_basis_for_test().snapshot_handle();
        let positioned = runtime.read_truth().positioned_snapshot(snapshot).unwrap();
        let witness = account_witness(&host_custody.borrowed_request_for_test(), &world, runtime, snapshot, entity);
        let facts: Arc<[WorthQueryApplicationObservedFact]> = Arc::from([]);
        owner
            .register_settlement_for_test(SettlementRegistration::new_for_test(None, None, Arc::clone(&identity), RetainedSourceFacts::for_test_for_test(false, Arc::clone(&facts)), None, positioned.clone(), OrdSet::new(), Some(FullVerificationReason::DeclaredChangeUnavailable), OrdSet::new()))
            .expect("the row requires verification at the selected source");
        let verify = || {
            ConsumedOutputEvidence::verify_at_observation_for_test(&host_custody.borrowed_request_for_test(), &identity, &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts::for_test_for_test(false, Arc::clone(&facts)).for_comparison().unwrap(), &[], None, &witness, owner, runtime, snapshot, &positioned)
        };
let recorded = || {
            matches!(
                owner.currentness_for_test(&positioned, &identity),
                Ok(SourceSettlementCurrentness::Clean)
            )
        };


        assert!(!recorded());
        assert_eq!(
            verify(),
            Ok(ConsumedOutputVerification::Current)
        );
        assert!(recorded(), "the genuine native verification registers the row");
    }) }).expect("the fixture read owns its installed host call");
}

#[test]
fn a_restored_output_registers_before_commit() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (_, identity) = world
        .application
        .with_host_advancement(|phase| primed_output(&phase, &world))
        .expect("the fixture read owns its installed host call");
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let (head_handle, head) = handle.with_runtime(request_execution, snapshot);
    let restored = ConsumedOutputEvidence::retained_for_test(
        identity,
        Arc::from([]),
        Vec::new(),
        Some(FullVerificationReason::CheckpointRestore),
        Arc::new(head),
    );

    assert_eq!(
        restored.establish_restored_before_commit_for_test(
            request_execution,
            &(*handle.source_owner_for_test())
        ),
        Ok(())
    );

    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        runtime.snapshots().release_snapshot(&head_handle).unwrap();
    });
}

/// The account's sealed native output witness at `snapshot`.
pub fn account_witness(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity: EntityId,
) -> Arc<OnceLock<SealedNativeOutputWitness>> {
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let layout = graph.layout_for_test();
    let truth = runtime.read_truth();
    let mut facts = vec![WorthQueryApplicationObservedFact::Entity {
        entity_id: entity,
        kind: truth
            .read_exact_snapshot_live_entity_kind(execution, snapshot, entity)
            .unwrap()
            .unwrap(),
    }];
    for aspect in layout.native_output_aspects_for_test("Account") {
        facts.push(WorthQueryApplicationObservedFact::SourceAspectRevision {
            entity_id: entity,
            aspect: aspect.clone(),
            native_revision: truth
                .read_exact_snapshot_entity_aspect_version(execution, snapshot, entity, aspect)
                .unwrap()
                .unwrap(),
        });
    }
    let correspondence = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
        TypeId::of_for_test::<()>(),
        TypeId::of_for_test::<()>(),
        std::iter::empty(),
        &[WorthQueryCheckpointOutputRole::new_for_test(
            "account".to_owned(),
            WorthQueryApplicationOutputPosture::Preserve,
            "Account".to_owned(),
            entity,
        )],
        |_| Some(TypeId::of_for_test::<()>()),
        execution,
    )
    .unwrap();
    SealedNativeOutputWitness::from_checkpoint_facts_for_test(&correspondence, layout, &facts)
        .unwrap()
        .expect("complete native facts reconstruct the witness")
}

/// Publishes through the World so the companion has a selected cell at the
/// head, and names one label output of the open account.
pub fn primed_output(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
) -> (EntityId, Arc<RecordedSettlementIdentity>) {
    let entity = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .resolve_entity(
            phase_custody,
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id_for_test();
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout_for_test()
        .field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    publish_relational_mutation(
        &phase_custody.borrowed_request_for_test(),
        world,
        WorkerIntentBatch::new("request-recording-prime").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    label,
                    AspectValue::String(InternedString::Raw("prime".to_owned())),
                )])),
            }),
        )),
    );
    let (_, product, _) = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .into_parts_for_test();
    let coordinate = ProductCoordinate::new_for_test(
        product.observation_for_test().lifecycle_incarnation(),
        product.observation_for_test().reference_generation().get(),
    );
    let source = SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), OutputBindingIdentity::declared_for_test("LabelOutput"));
    (
        entity,
        RecordedSettlementIdentity::retain_for_test(&source, coordinate, 0),
    )
}
