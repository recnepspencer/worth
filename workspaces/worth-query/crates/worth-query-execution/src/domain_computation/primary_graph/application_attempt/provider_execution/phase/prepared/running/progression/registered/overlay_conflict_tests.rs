use super::WorthQueryRegisteredProviderAttempt;
use crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld;
use crate::domain_computation::{
    WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryProvisionalEffectProgramView,
    WorthQueryProvisionalOverlayAdmission,
};

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression) fn assert_second_real_overlay_is_rejected(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    registered: WorthQueryRegisteredProviderAttempt<'_>,
    world: &AuthorizationWorld,
) {
    let WorthQueryRegisteredProviderAttempt {
        staged,
        steps,
        dispatch_outbox: _dispatch_outbox,
    } = registered;
    let control = crate::domain_computation::primary_graph::StorageControl::new(
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        None,
    );
    let receipt = world
        .application
        .primary_provider
        .capture_application_read_set(execution, &staged.read_authority(), control)
        .expect("real registered facts must produce a decision read set");
    let fresh = match world
        .application
        .primary_provider
        .compare_application_read_set(execution, &staged.read_authority(), receipt, control)
        .expect("real registered facts must compare")
    {
        WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh) => fresh,
        WorthQueryDecisionReadSetFreshnessOutcome::Stale(_) => {
            panic!("unmodified fixture facts must remain fresh")
        }
    };
    let lowered = staged
        .effect_authority()
        .lower_shared_provisional_program(&fresh, steps)
        .expect("real effect steps must lower for the registered session");
    let session = staged.provider_session_view();
    let first_admission = WorthQueryProvisionalOverlayAdmission::new(
        session,
        staged.provisional_binding_identity(),
        lowered.identity(),
        0,
    );
    let first = staged
        .provisional_provider()
        .stage_provisional_overlay(
            session,
            WorthQueryProvisionalEffectProgramView::new(&lowered, 0),
            first_admission,
        )
        .expect("first real overlay must stage");
    let first_identity = first.view().physical_overlay_identity().to_owned();
    let substitute_admission = WorthQueryProvisionalOverlayAdmission::new(
        session,
        staged.provisional_binding_identity(),
        lowered.identity(),
        1,
    );
    let denial = match staged.provisional_provider().stage_provisional_overlay(
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
    assert_eq!(first.view().physical_overlay_identity(), first_identity);
    staged
        .provisional_provider()
        .discard_provisional_overlay(staged.execution_request(), first.view())
        .expect("the first exact overlay must remain independently discardable");
    let _ = staged.abort();
    assert_eq!(
        world.application.provider_session_resource_count_for_test(),
        0
    );
}
