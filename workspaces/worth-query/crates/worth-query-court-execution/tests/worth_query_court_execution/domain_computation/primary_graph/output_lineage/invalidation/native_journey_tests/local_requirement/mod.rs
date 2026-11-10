//! A post-effect lineage requirement cannot be hidden by an older Clean mark.

use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryCheckpointSourceIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::verification::ConsumedOutputVerification;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_source_identity::RecordedSourceIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;

#[test]
fn local_full_verification_requirement_overrides_a_clean_actor_row() {
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
    let (before_handle, before, retained_fact) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            handle.with_runtime(request_execution, |runtime| {
                let (snapshot, basis) = snapshot(runtime);
                let fact = field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &snapshot,
                    entity,
                    status.clone(),
                );
                let facts: Arc<[_]> = Arc::from([fact]);
                (snapshot, basis, facts)
            })
        })
        .expect("the fixture read owns its installed host call");

    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.fixture_binding(TypeId::of_for_test::<StatusOutput>(), "StatusOutput");

    lineage.install_output_families_for_test(BTreeMap::from([(
        "local-requirement".to_owned().into(),
        vec![(TypeId::of_for_test::<StatusOutput>(), "output".to_owned())],
    )]));
    let runtime_authority = (*world.application.runtime_for_test())
        .authority_identity_for_test()
        .as_u64();
    let schema = (*world.application.installed_schema_for_test()).binding_identity();
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity);
    lineage.record_restoration_for_test(
        TypeId::of_for_test::<StatusOutput>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::new(WorthQueryApplicationOutputCorrespondence::default()),
        RecordedSourceIdentity::Checkpoint(WorthQueryCheckpointSourceIdentity::new_for_test([0x71; 32])),
        [0x72; 32],
        None,
        [0x73; 32],
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts_for_test(Arc::clone(&retained_fact)),
        None,
        None,
        None,
    );
    let identity = Arc::clone(
        (*lineage
            .resolve_current_family_for_test(
                request_execution,
                runtime_authority,
                &schema,
                scope,
                "local-requirement",
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
            )
            .unwrap()
            .candidates_mut_for_test())
        .pop()
        .expect("the lineage owner selects its retained output")
        .settlement_identity_for_test(),
    );
    // The old actor row has no posting for this source field. A failed
    // post-effect registration leaves it Clean while lineage retains the
    // actual fact and records a full-verification requirement.
    register(
        owner,
        Arc::clone(&identity),
        Arc::from([]),
        &before,
        OrdSet::new(),
    );
    lineage.require_settlement_verification_for_test(
        &identity,
        FullVerificationReason::NativeRevisionUnavailable,
    );
    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        write_field(request_execution, runtime, entity, status, "closed")
    });
    let (after_handle, after) = handle.with_runtime(request_execution, snapshot);
    assert!(matches!(
        owner.currentness_for_test(&after, &identity),
        Ok(SourceSettlementCurrentness::Clean)
    ));

    let candidate = (*lineage
        .resolve_current_family_for_test(
            request_execution,
            runtime_authority,
            &schema,
            scope,
            "local-requirement",
            observation.lifecycle_incarnation(),
            observation.reference_generation().get(),
        )
        .unwrap()
        .candidates_mut_for_test())
    .pop()
    .expect("the local requirement stays on the selected owner row");
    assert_eq!(
        (*candidate.verification_requirement_for_test()),
        Some(FullVerificationReason::NativeRevisionUnavailable)
    );

    let result = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            handle.with_runtime(request_execution, |runtime| {
                ConsumedOutputEvidence::verify_candidate_with_admission_for_test(
                    &host_custody.borrowed_request_for_test(),
                    candidate.settlement_identity_for_test(),
                    &(*candidate.observed_source_facts_for_test())
                        .for_comparison()
                        .unwrap(),
                    candidate.consumed_outputs_for_test(),
                    *candidate.verification_requirement_for_test(),
                    &before,
                    owner,
                    runtime,
                    &after_handle,
                    &after,
                )
            })
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(result, Ok(ConsumedOutputVerification::ChangedDirectFact(0)));
    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        runtime
            .snapshots()
            .release_snapshot(&before_handle)
            .unwrap();
        runtime.snapshots().release_snapshot(&after_handle).unwrap();
    });
}
