//! Static authority shared with ordinary mutation admission after the exact
//! installation owner has sealed a cold-compiled producer mutation binding.

use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryCurrentRetainedMutationBinding,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::primary_graph::{
    InvalidationEditAdmission, WorthQueryApplicationEntityIdentity,
    WorthQueryAuthenticatedPrincipal, WorthQueryPrimaryGraphApplicationRuntime,
};

use super::{
    denial, validate_static_authority_prefix, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

pub(in crate::domain_computation) enum WorthQueryRetainedMutationStaticStop {
    Admission(CompanionPreflightStop),
    Authorization(WorthQueryOperationAuthorizationDenial),
    AccountingOverflow,
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
    admission: &mut InvalidationEditAdmission,
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
    admission.charge_external_work(8).map_err(Stop::Admission)?;
    let operation = current.operation();
    let subject_bytes = operation.operation().len().max(principal.binding().len());
    let subject_bytes = u64::try_from(subject_bytes).map_err(|_| Stop::AccountingOverflow)?;
    let required_work = subject_bytes
        .checked_add(352)
        .ok_or(Stop::AccountingOverflow)?;
    admission
        .admit_read_scratch(subject_bytes)
        .and_then(|()| admission.charge_external_work(required_work))
        .map_err(Stop::Admission)?;
    validate_static_authority_prefix(runtime, principal, scope, operation)
        .map_err(Stop::Authorization)?;
    if !current.belongs_to(
        runtime.runtime.installed_packages(),
        runtime.installed_schema(),
    ) {
        return Err(Stop::Authorization(denial(
            WorthQueryOperationAuthorizationDenialKind::StaleInstalledOperation,
            operation.operation(),
        )));
    }
    Ok(())
}
