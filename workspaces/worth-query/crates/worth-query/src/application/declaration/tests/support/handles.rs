use super::*;

pub fn admitted_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    regime: GeometryOperatingContext,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<
    GeometryDomain,
    GeometryOperatingContext,
> {
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        GeometryDomain,
        regime,
        [
            crate::application::domain_test_support::family::<GeometryDomain, SplitEdgeFamily>(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                SplitEdgeSingleOnlyFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, TemporalReadFamily>(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                DeferredTemporalReadFamily,
            >(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                super::super::async_support::AsyncResourceReadFamily,
            >(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                super::super::async_support::DeferredAsyncResourceReadFamily,
            >(),
        ],
    )
}

pub fn admitted_topology_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<
    TopologyDomain,
    GeometryOperatingContext,
> {
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        TopologyDomain,
        GeometryOperatingContext::collaborative(),
        [crate::application::domain_test_support::family::<
            TopologyDomain,
            SplitEdgeTopologyFamily,
        >()],
    )
}
