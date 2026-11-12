//! Actual installed host phase for owner-local, non-I/O lifecycle axis courts.
pub(super) fn in_phase<R>(
    body: impl for<'scope> FnOnce(
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'scope>,
    ) -> R,
) -> R {
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world.application.with_host_advancement(body).unwrap()
}
