//! Static authority shared with ordinary mutation admission after the exact
//! installation owner has sealed a cold-compiled producer mutation binding.

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryCurrentRetainedMutationBinding;

use crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity;
use crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

use super::denial;
use super::validate_static_authority_prefix;
use super::WorthQueryOperationAuthorizationDenial;
use super::WorthQueryOperationAuthorizationDenialKind;

pub(in crate::domain_computation) enum WorthQueryRetainedMutationStaticStop {
    Authorization(WorthQueryOperationAuthorizationDenial),
}

pub(in crate::domain_computation::authorization) fn validate_static_authority_retained<
    Schema,
    Binding,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
    current: &WorthQueryCurrentRetainedMutationBinding<'_, Schema, Binding>,
) -> Result<(), WorthQueryRetainedMutationStaticStop>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    use WorthQueryRetainedMutationStaticStop as Stop;

    // The shared validator below compares two runtime identities and three
    // full schema identities. Its schema getter initializes one 80-byte value;
    // the three comparisons inspect 3 x 80 bytes; the two runtime identities
    // and fixed owner checks take the remaining 32 of the 352-unit claim. One
    // terminal subject copy may use either operation name or principal binding.

    let operation = current.operation();

    validate_static_authority_prefix(runtime, principal, scope, operation)
        .map_err(Stop::Authorization)?;
    if !current.belongs_to(
        runtime.runtime.installed_packages(),
        runtime.installed_schema(),
    ) {
        return Err(Stop::Authorization(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::StaleInstalledOperation,
            ),
            operation.operation(),
        )));
    }
    Ok(())
}
