//! Stable preparation cannot reinterpret a cutoff at an older source root.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;

use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::derived::SettlementRegistrationStop;
use worth_relational::facade::mvcc::CompanionCellEditStop;

#[test]
fn current_source_registration_rejects_native_movement_while_ordinary_replay_remains_valid() {
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
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
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
    let identity = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 0);
    let registration = |identity| {
        // Bound in the original literal order, passed in the door's.
        let (
            work_membership,
            computation,
            identity,
            facts,
            output_facts,
            read_basis,
            stale_at_read_basis,
            requirement,
            upstream,
        ) = (
            None,
            None,
            identity,
            RetainedSourceFacts::for_test_for_test(false, Arc::clone(&facts)),
            None,
            before.clone(),
            OrdSet::new(),
            None,
            OrdSet::new(),
        );
        SettlementRegistration::new_for_test(
            computation,
            work_membership,
            identity,
            facts,
            output_facts,
            read_basis,
            stale_at_read_basis,
            requirement,
            upstream,
        )
    };
    let prepared = owner
        .prepare_current_settlement_for_test(registration(Arc::clone(&identity)), &before)
        .expect("the exact current native source can prepare");
    assert!(Arc::ptr_eq(prepared.identity_for_test(), &identity));
    assert!(std::ptr::eq(prepared.selected_source_for_test(), &before));
    let cleanup = prepared
        .install_for_test()
        .unwrap_or_else(|_| panic!("an unchanged source image installs its prepared registration"));
    drop(cleanup);
    assert!(matches!(
        currentness(owner, &before, &identity),
        SourceSettlementCurrentness::Clean
    ));

    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        write_field(request_execution, runtime, entity, status, "closed")
    });
    let (after_handle, after) = handle.with_runtime(request_execution, snapshot);
    let next = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 1);
    let stopped =
        owner.prepare_current_settlement_for_test(registration(Arc::clone(&next)), &before);
    assert!(matches!(
        stopped,
        Err(SettlementRegistrationStop::Edit(
            CompanionCellEditStop::TopologyGenerationChanged
        ))
    ));
    assert!(matches!(
        currentness(owner, &after, &next),
        SourceSettlementCurrentness::FullVerificationRequired(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason::MissingSettlement
        )
    ));
    // An ordinary post-effect registration must still replay deliveries from
    // its retained basis rather than pretending its old fact is current.
    owner
        .register_settlement_for_test(registration(Arc::clone(&next)))
        .unwrap();
    match currentness(owner, &after, &next) {
        SourceSettlementCurrentness::Dirty(ordinals) => assert!(ordinals.contains(&0)),
        _ => panic!("ordinary retained registration must mark the changed source fact"),
    }
    // A registrant that already knows its own effect moved a fact it read
    // is not clean at the post-effect basis: no later delivery marks it.
    let born_stale = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 2);
    owner
        .register_settlement_for_test({
            let __computation = None;
            let __read_basis = after.clone();
            let __stale_at_read_basis = OrdSet::unit(0);
            let mut __base = registration(Arc::clone(&born_stale));
            *__base.computation_mut_for_test() = __computation;
            *__base.read_basis_mut_for_test() = __read_basis;
            *__base.stale_at_read_basis_mut_for_test() = __stale_at_read_basis;
            __base
        })
        .unwrap();
    match currentness(owner, &after, &born_stale) {
        SourceSettlementCurrentness::Dirty(ordinals) => assert_eq!(ordinals, OrdSet::unit(0)),
        _ => panic!("a registration stale at its own read basis starts dirty"),
    }
    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        runtime
            .snapshots()
            .release_snapshot(&before_handle)
            .unwrap();
        runtime.snapshots().release_snapshot(&after_handle).unwrap();
    });
}
