//! One native outbox handle is retried with the admission of this attempt only.
use super::*;
pub(in crate::application_entry) fn retry_original_handle<S, O, I, Scope>(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    admission: &WorthQueryAdmittedApplicationOperation<S, O, I, Scope>,
    handle: WorthQueryRecoveryHandle,
) -> Result<
    WorthQueryRecoverySafeRetryAdmission,
    (
        WorthQueryRecoveryHandleDenial,
        Option<WorthQueryRecoveryHandle>,
    ),
>
where
    S: ApplicationSchema,
{
    let authority = match application.admit_recovery_effect_authority(&handle, admission) {
        Ok(authority) => authority,
        Err(cause) => return Err(refused_handle(cause, handle)),
    };
    let redispatch = match application
        .redispatch_admitted_external_effect(phase, &handle, &authority, admission)
    {
        Ok(redispatch) => redispatch,
        Err(cause) => return Err(refused_handle(cause.into(), handle)),
    };
    safe_retry_recovery_handle(handle, &authority, redispatch).map_err(|cause| cause.into_parts())
}

fn refused_handle(
    cause: WorthQueryRecoveryHandleDenial,
    handle: WorthQueryRecoveryHandle,
) -> (
    WorthQueryRecoveryHandleDenial,
    Option<WorthQueryRecoveryHandle>,
) {
    let terminal = cause.kind() == WorthQueryRecoveryHandleDenialKind::AlreadyTerminal;
    (cause, (!terminal).then_some(handle))
}
