use crate::runtime::rebind::{UiProjectionRebindRequest, UiRebindOutcome, UiRebindReceipt};
use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host_and_scalar_projection;

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
        1,
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
        2,
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

fn published(outcome: UiRebindOutcome<'_>, tick: u64) -> UiRebindReceipt {
    match outcome {
        UiRebindOutcome::Published(receipt) => receipt,
        UiRebindOutcome::InFlight(completion) => match completion.complete(tick) {
            UiRebindOutcome::Published(receipt) => receipt,
            _ => panic!("native projection completion did not publish"),
        },
        _ => panic!("native projection rebind did not reach publication"),
    }
}
