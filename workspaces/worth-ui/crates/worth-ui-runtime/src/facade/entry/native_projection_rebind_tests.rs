use crate::runtime::rebind::{UiProjectionRebindRequest, UiRebindReceipt};
use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host_and_scalar_projection;
use crate::runtime::tests::active_application_session_test_support::{
    component_builder, source_backed_component_app_from_builder,
};

use super::native_identity_trace_host::NativeIdentityTraceHost;

#[test]
fn native_projection_rebind_returns_the_exact_fact_to_its_query_owner() {
    let plan = worth_ui_query_binding::WorthUiScalarProjectionHostPlan::prepare()
        .expect("product Query plan prepares");
    let installed = plan
        .install_for_certification()
        .expect("binding completion opens the product Query owner");
    let (registration, initial) = installed.into_parts();
    let mut shell = source_backed_component_app_with_host_and_scalar_projection(
        NativeIdentityTraceHost::default(),
        registration,
    )
    .launch_native_surface()
    .expect("source-backed application should launch through the native lifecycle");
    super::native_application_identity_trace_test_support::install_bound_surface_geometry(
        &mut shell,
    );

    let (pending, pending_completion) = initial.into_parts();
    let pending_receipt = published(
        shell
            .begin_projection_rebind(UiProjectionRebindRequest::new(pending).observed_at_tick(1))
            .expect("pending projection enters the standard native rebind"),
    );
    let pending_observation = match pending_receipt.release_scalar_projection_observation() {
        Ok(observation) => observation,
        Err(_) => panic!("the terminal plan did not return its only scalar predecessor"),
    };
    let owner = pending_completion
        .admit_publication(pending_observation)
        .expect("the exact pending fact readmits the Query owner");

    let current = owner
        .advance(
            worth_ui_query_binding::WorthUiScalarProjectionSourceRecord::new("ONLINE", 1)
                .expect("native source record"),
        )
        .expect("owner-issued refresh reaches Query");
    let (current, current_completion) = current.into_parts();
    let current_receipt = published(
        shell
            .begin_projection_rebind(UiProjectionRebindRequest::new(current).observed_at_tick(2))
            .expect("current projection enters the standard native rebind"),
    );
    let current_observation = match current_receipt.release_scalar_projection_observation() {
        Ok(observation) => observation,
        Err(_) => panic!("the terminal plan did not return its only current scalar predecessor"),
    };
    let owner = current_completion
        .admit_publication(current_observation)
        .expect("the exact current fact readmits the Query owner");

    let source_close = owner.close().expect("Query source closes terminally");
    assert!(source_close.owner_terminal());
    assert_eq!(source_close.live_source_count(), 0);
    let shutdown = shell.shutdown();
    assert!(shutdown.host_session_released());
    assert_eq!(shutdown.released_surface_count(), 1);
}

#[test]
fn authored_application_scalar_reaches_native_publication_and_returns_its_exact_fact() {
    let owner = worth_ui_query_binding::WorthUiStatusSourceOwner::install()
        .expect("authored status program installs");
    let (registration, pending) = owner
        .initial_projection()
        .expect("initial Query publication issues the UI registration");
    let builder = component_builder()
        .register_application_scalar_projection(registration.clone())
        .expect("the authored projection registers with the real application");
    let host = NativeIdentityTraceHost::default();
    let mut shell = source_backed_component_app_from_builder(builder, |application| {
        super::WorthUiCertificationApplicationTransition::activate_test_host(
            application,
            host.clone(),
        )
    })
    .launch_native_surface()
    .expect("source-backed application launches on the native host");
    super::native_application_identity_trace_test_support::install_bound_surface_geometry(
        &mut shell,
    );

    let foreign = worth_ui_query_binding::WorthUiStatusSourceOwner::install()
        .expect("foreign authored status program installs independently");
    let (_, foreign_observation) = foreign
        .initial_projection()
        .expect("foreign Query source issues a real same-identity fact");
    assert!(matches!(
        shell.begin_projection_rebind(
            UiProjectionRebindRequest::new(foreign_observation).observed_at_tick(1)
        ),
        Err(
            super::WorthUiNativeProjectionRebindDenial::ObservationAdmission(
                crate::runtime::observation::UiObservationAdmissionDenial::ForeignQueryProjection
            )
        )
    ));
    assert_eq!(host.presentation_calls(), 0);
    let (_, foreign_consequence) = foreign
        .initial_projection()
        .expect("foreign Query source issues a second real fact");
    let mut turn = shell
        .session
        .begin_observation_turn()
        .expect("intent consequence opens the ordinary observation turn");
    let stop = match turn.admit_intent_consequence_batch(
        crate::runtime::observation::UiIntentConsequenceObservationBatch::new(
            None,
            None,
            Some(foreign_consequence),
        ),
    ) {
        Ok(_) => panic!("foreign intent consequence must not enter publication"),
        Err(stop) => stop,
    };
    let (reason, batch) = stop.into_parts();
    assert!(matches!(
        reason,
        crate::runtime::observation::UiIntentConsequenceObservationAdmissionReason::Observation(
            crate::runtime::observation::UiObservationAdmissionDenial::ForeignQueryProjection
        )
    ));
    assert!(
        batch.into_parts().2.is_some(),
        "the denied fact is returned intact"
    );
    drop(turn);
    assert_eq!(host.presentation_calls(), 0);

    let pending_receipt = published(
        shell
            .begin_projection_rebind(UiProjectionRebindRequest::new(pending).observed_at_tick(1))
            .expect("pending authored fact enters native rebind"),
    );
    assert!(pending_receipt.mounted_publication().is_some());
    let pending = pending_receipt
        .release_application_scalar_projection_observation()
        .unwrap_or_else(|_| panic!("native publication must return its authored fact"));
    assert!(registration.admits(pending.fact()));
    assert_eq!(pending.fact().value().status, "PENDING");
    assert_eq!(
        pending.owner_order(),
        pending.fact().query_receipt().inspect().basis().version()
    );
    let (_, historical) = owner
        .initial_projection()
        .expect("a second real observation retains the predecessor basis");

    let current = owner
        .publish_source(
            worth_ui_query_binding::WorthUiScalarProjectionSourceRecord::new("ONLINE", 1)
                .expect("accepted source record"),
        )
        .expect("source edit commits and publishes through the authored Query program")
        .into_projection_observation()
        .expect("Query publication issues the successor UI observation");
    let current_receipt = published(
        shell
            .begin_projection_rebind(UiProjectionRebindRequest::new(current).observed_at_tick(2))
            .expect("current authored fact enters native rebind"),
    );
    assert!(current_receipt.mounted_publication().is_some());
    let current = current_receipt
        .release_application_scalar_projection_observation()
        .unwrap_or_else(|_| panic!("native successor must return its authored fact"));
    assert!(registration.admits(current.fact()));
    assert_eq!(current.fact().value().status, "ONLINE");
    assert_eq!(current.fact().value().revision, 1);
    assert_eq!(
        current.owner_order(),
        current.fact().query_receipt().inspect().basis().version()
    );
    assert!(current
        .fact()
        .query_receipt()
        .inspect()
        .terminal_resources_released());
    assert_eq!(host.presentation_calls(), 2);
    assert!(matches!(
        shell.begin_projection_rebind(
            UiProjectionRebindRequest::new(historical).observed_at_tick(3)
        ),
        Ok(super::WorthUiNativeManagedProjectionRebindOutcome::Stopped(
            super::native_managed_rebind::WorthUiNativeManagedRebindStop::SupersededBeforeEffects(
                _
            )
        ))
    ));
    assert_eq!(host.presentation_calls(), 2);

    let shutdown = shell.shutdown();
    assert!(shutdown.host_session_released());
    assert_eq!(shutdown.released_surface_count(), 1);
}

fn published(outcome: super::WorthUiNativeManagedProjectionRebindOutcome) -> UiRebindReceipt {
    match outcome {
        super::WorthUiNativeManagedProjectionRebindOutcome::Published(receipt) => receipt,
        _ => panic!("native projection rebind did not reach publication"),
    }
}
