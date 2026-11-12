//! One preflight sequence for conventional typed operations and borrowed reads.
use super::*;
use worth_query_installation::facade::InstalledOperationView;
/// Both entrances reject capability-only operations; their established failure
/// precedence and missing-capability diagnostic remain distinct.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::authorization) enum ConventionalPreflightEntrance {
    TypedOperation,
    BorrowedRead,
}
pub(in crate::domain_computation::authorization) fn validate_conventional_preflight<
    S: ApplicationSchema,
    P,
    I,
    E,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    principal: &WorthQueryAuthenticatedPrincipal<S, P, I>,
    scope: &WorthQueryApplicationEntityIdentity<S, E>,
    operation: InstalledOperationView<'_>,
    request: &WorthQueryRequestScope,
    entrance: ConventionalPreflightEntrance,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    if matches!(entrance, ConventionalPreflightEntrance::BorrowedRead) {
        require_conventional_authorization(
            operation,
            AuthorizationRefusal::CapabilityAuthorizationMissing,
        )?;
    }
    validate_admission_clock(
        &runtime.runtime,
        principal.clock(),
        request,
        operation.operation(),
    )?;
    admit_request(request, operation.operation())?;
    if matches!(entrance, ConventionalPreflightEntrance::TypedOperation) {
        require_conventional_authorization(operation, AuthorizationRefusal::CapabilityRequired)?;
    }
    validate_static_authority_prefix(runtime, principal, scope, operation)
}
fn require_conventional_authorization(
    operation: InstalledOperationView<'_>,
    missing: AuthorizationRefusal,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    if operation.contracts().authorization().requires_capability() {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(missing),
            operation.operation(),
        ));
    }
    Ok(())
}
