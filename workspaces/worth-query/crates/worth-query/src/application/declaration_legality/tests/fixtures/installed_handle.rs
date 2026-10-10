use super::*;

pub(crate) fn admitted_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    regime: &'static str,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<
    GeometryDomain,
    CollaborativeWorld,
> {
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        GeometryDomain,
        CollaborativeWorld::named(regime),
        [
            crate::application::domain_test_support::family::<GeometryDomain, LegalFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, IllegalRoleFamily>(),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                IllegalDispositionFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, DeferredLegalityFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, MaskedCoverageFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, TemporalCurrentFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, TemporalPreviewFamily>(
            ),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                TemporalHistoricalFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, AsyncCurrentFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, AsyncPreviewFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, AsyncHistoricalFamily>(
            ),
        ],
    )
}
