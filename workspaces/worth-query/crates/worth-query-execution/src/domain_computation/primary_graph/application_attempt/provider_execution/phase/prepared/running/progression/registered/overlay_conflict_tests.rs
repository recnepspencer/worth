use super::WorthQueryRegisteredProviderAttempt;
use crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld;
use crate::domain_computation::{
    WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryProvisionalEffectProgramView,
    WorthQueryProvisionalOverlayAdmission,
};

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression) fn assert_second_real_overlay_is_rejected(
    registered: WorthQueryRegisteredProviderAttempt<'_>,
    world: &AuthorizationWorld,
) {
    let WorthQueryRegisteredProviderAttempt {
        staged,
        requests,
        steps,
        dispatch_outbox: _dispatch_outbox,
    } = registered;
    let receipt = staged
        .read_authority()
        .capture_decision_read_set(requests)
        .expect("real registered facts must produce a decision read set");
    let fresh = match staged
        .read_authority()
        .compare_decision_read_set(receipt)
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
        .discard_provisional_overlay(first.view())
        .expect("the first exact overlay must remain independently discardable");
    let _ = staged.abort();
    assert_eq!(world.application.provider_session_resource_count(), 0);
}
