//! unknown disabled cancelled and cross runtime resolution fail closed.
use super::*;

#[test]
fn unknown_disabled_cancelled_and_cross_runtime_resolution_fail_closed() {
    let enabled = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    let scope = live_scope();
    let unknown = enabled.authenticate("unknown", Duration::from_secs(60), &scope);
    enabled
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                enabled
                    .selected_product(&host_custody,)
                    .resolve_authenticated_principal(
                        &phase,
                        &enabled.binding,
                        &unknown,
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap_err()
                    .kind(),
                WorthQueryPrincipalResolutionDenialKind::UnknownPrincipal
            )
        })
        .expect("the fixture read owns its installed host call");

    let disabled = installed_world(&[("bob", WorthQueryPrincipalMappingStatus::Disabled)]);
    let disabled_scope = live_scope();
    let disabled_external = disabled.authenticate("bob", Duration::from_secs(60), &disabled_scope);
    disabled
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                disabled
                    .selected_product(&host_custody,)
                    .resolve_authenticated_principal(
                        &phase,
                        &disabled.binding,
                        &disabled_external,
                        &disabled_scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap_err()
                    .kind(),
                WorthQueryPrincipalResolutionDenialKind::DisabledPrincipal
            )
        })
        .expect("the fixture read owns its installed host call");

    let cancellation = WorthQueryCancellationSource::new();
    let cancelled_external = enabled.authenticate("alice", Duration::from_secs(60), &live_scope());
    cancellation.cancel();
    let cancelled_scope = WorthQueryRequestScope::new(
        std::time::Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    enabled
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                enabled
                    .selected_product(&host_custody,)
                    .resolve_authenticated_principal(
                        &phase,
                        &enabled.binding,
                        &cancelled_external,
                        &cancelled_scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap_err()
                    .kind(),
                WorthQueryPrincipalResolutionDenialKind::Cancelled
            )
        })
        .expect("the fixture read owns its installed host call");

    let foreign = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    let foreign_external = enabled.authenticate("alice", Duration::from_secs(60), &live_scope());
    foreign
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                foreign
                    .selected_product(&host_custody,)
                    .resolve_authenticated_principal(
                        &phase,
                        &foreign.binding,
                        &foreign_external,
                        &live_scope(),
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap_err()
                    .kind(),
                WorthQueryPrincipalResolutionDenialKind::ForeignRuntime
            )
        })
        .expect("the fixture read owns its installed host call");
}
