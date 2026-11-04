use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::WorthQueryRegisteredProviderAttempt;
use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetFreshnessOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalEffectProgramView;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalOverlayAdmission;

pub fn assert_second_real_overlay_is_rejected(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    registered: WorthQueryRegisteredProviderAttempt<'_>,
    world: &AuthorizationWorld,
) {
    let (staged, steps, _dispatch_outbox) = registered.into_parts_for_test();
    let control = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        None,
    );
    let receipt = (*world.application.primary_provider_for_test())
        .capture_application_read_set_for_test(execution, &staged.read_authority(), control)
        .expect("real registered facts must produce a decision read set");
    let fresh = match (*world.application.primary_provider_for_test())
        .compare_application_read_set_for_test(
            execution,
            &staged.read_authority(),
            receipt,
            control,
        )
        .expect("real registered facts must compare")
    {
        WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh) => fresh,
        WorthQueryDecisionReadSetFreshnessOutcome::Stale(_) => {
            panic!("unmodified fixture facts must remain fresh")
        }
    };
    let lowered = staged
        .effect_authority()
        .lower_shared_provisional_program_for_test(&fresh, steps)
        .expect("real effect steps must lower for the registered session");
    let session = staged.provider_session_view_for_test();
    let first_admission = WorthQueryProvisionalOverlayAdmission::new_for_test(
        session,
        staged.provisional_binding_identity_for_test(),
        lowered.identity(),
        0,
    );
    let first = staged
        .provisional_provider_for_test()
        .stage_provisional_overlay_for_test(
            session,
            WorthQueryProvisionalEffectProgramView::new(&lowered, 0),
            first_admission,
        )
        .expect("first real overlay must stage");
    let first_identity = first.view_for_test().physical_overlay_identity().to_owned();
    let substitute_admission = WorthQueryProvisionalOverlayAdmission::new_for_test(
        session,
        staged.provisional_binding_identity_for_test(),
        lowered.identity(),
        1,
    );
    let denial = match staged
        .provisional_provider_for_test()
        .stage_provisional_overlay_for_test(
            session,
            WorthQueryProvisionalEffectProgramView::new(&lowered, 1),
            substitute_admission,
        ) {
        Ok(_) => panic!("second real overlay must not replace the first"),
        Err(denial) => denial,
    };
    assert!(denial
        .detail()
        .contains("cannot stage this application overlay"));
    assert_eq!(
        first.view_for_test().physical_overlay_identity(),
        first_identity
    );
    staged
        .provisional_provider_for_test()
        .discard_provisional_overlay_for_test(
            staged.execution_request_for_test(),
            first.view_for_test(),
        )
        .expect("the first exact overlay must remain independently discardable");
    let _ = staged.abort();
    assert_eq!(
        world.application.provider_session_resource_count_for_test(),
        0
    );
}
