//! admitted handle.
use super::*;

pub(in super::super) fn admitted_handle(
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
            crate::application::domain_test_support::family::<GeometryDomain, AdmittedFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, ReceiptFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, DeferredFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, DeniedFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, StaleFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, FailedFamily>(),
            crate::application::domain_test_support::family::<GeometryDomain, AlternateAspectFamily>(
            ),
            crate::application::domain_test_support::family::<
                GeometryDomain,
                DescriptiveDeferredSignalFamily,
            >(),
            crate::application::domain_test_support::family::<GeometryDomain, MaskedCoverageFamily>(
            ),
            crate::application::domain_test_support::family::<GeometryDomain, RebindRequiredFamily>(
            ),
        ],
    )
}
