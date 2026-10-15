//! Each concrete command moves its original input/key into one native owner.
use super::estate_source_outcome::{close_completed, EstateSourceCommandOutcome};
use bank_domain::{estate::EstateAction, proposals::BankIdempotencyKey, schema::*};
use bank_server::{
    mutations, BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationControls,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

pub(super) fn notify(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    request: &WorthQueryRequestScope,
    action: EstateAction,
    key: BankIdempotencyKey,
) -> EstateSourceCommandOutcome<NotifyEstateDeath, bank_server::DeathNotificationAction> {
    let input = match NotifyEstateDeath::try_from(action) {
        Ok(input) => input,
        Err(action) => return EstateSourceCommandOutcome::WrongInput { action, key },
    };
    let operation = match runtime
        .mutate(mutations::notify_estate_death(input))
        .as_principal(principal)
        .controls(BankMutationControls::new(request.clone(), key))
        .retain_operation()
    {
        Ok(operation) => operation,
        Err(refusal) => return EstateSourceCommandOutcome::Construction(refusal),
    };
    // These existing command inputs declare no additional caller conditions.
    // Construction retains that exact typed empty condition set and owned key.
    close_completed(
        operation.advance(&runtime.request(principal, request)),
        &runtime.request(principal, request),
    )
}

pub(super) fn retransmit(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    request: &WorthQueryRequestScope,
    action: EstateAction,
    key: BankIdempotencyKey,
) -> EstateSourceCommandOutcome<
    RetransmitEstateDeathNotice,
    bank_server::DeathNoticeRetransmissionAction,
> {
    let input = match RetransmitEstateDeathNotice::try_from(action) {
        Ok(input) => input,
        Err(action) => return EstateSourceCommandOutcome::WrongInput { action, key },
    };
    let operation = match runtime
        .mutate(mutations::retransmit_estate_death_notice(input))
        .as_principal(principal)
        .controls(BankMutationControls::new(request.clone(), key))
        .retain_operation()
    {
        Ok(operation) => operation,
        Err(refusal) => return EstateSourceCommandOutcome::Construction(refusal),
    };
    // These existing command inputs declare no additional caller conditions.
    // Construction retains that exact typed empty condition set and owned key.
    close_completed(
        operation.advance(&runtime.request(principal, request)),
        &runtime.request(principal, request),
    )
}

pub(super) fn freeze(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    request: &WorthQueryRequestScope,
    action: EstateAction,
    key: BankIdempotencyKey,
) -> EstateSourceCommandOutcome<FreezeEstateAccount, bank_server::EstateAccountFreezeAction> {
    let input = match FreezeEstateAccount::try_from(action) {
        Ok(input) => input,
        Err(action) => return EstateSourceCommandOutcome::WrongInput { action, key },
    };
    let operation = match runtime
        .mutate(mutations::freeze_estate_account(input))
        .as_principal(principal)
        .controls(BankMutationControls::new(request.clone(), key))
        .retain_operation()
    {
        Ok(operation) => operation,
        Err(refusal) => return EstateSourceCommandOutcome::Construction(refusal),
    };
    // These existing command inputs declare no additional caller conditions.
    // Construction retains that exact typed empty condition set and owned key.
    close_completed(
        operation.advance(&runtime.request(principal, request)),
        &runtime.request(principal, request),
    )
}

pub(super) fn open(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    request: &WorthQueryRequestScope,
    action: EstateAction,
    key: BankIdempotencyKey,
) -> EstateSourceCommandOutcome<OpenEstateCase, bank_server::EstateCaseOpeningAction> {
    let input = match OpenEstateCase::try_from(action) {
        Ok(input) => input,
        Err(action) => return EstateSourceCommandOutcome::WrongInput { action, key },
    };
    let operation = match runtime
        .mutate(mutations::open_estate_case(input))
        .as_principal(principal)
        .controls(BankMutationControls::new(request.clone(), key))
        .retain_operation()
    {
        Ok(operation) => operation,
        Err(refusal) => return EstateSourceCommandOutcome::Construction(refusal),
    };
    // These existing command inputs declare no additional caller conditions.
    // Construction retains that exact typed empty condition set and owned key.
    close_completed(
        operation.advance(&runtime.request(principal, request)),
        &runtime.request(principal, request),
    )
}

pub(super) fn recognize(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    request: &WorthQueryRequestScope,
    action: EstateAction,
    key: BankIdempotencyKey,
) -> EstateSourceCommandOutcome<RecognizeEstateExecutor, bank_server::ExecutorRecognitionAction> {
    let input = match RecognizeEstateExecutor::try_from(action) {
        Ok(input) => input,
        Err(action) => return EstateSourceCommandOutcome::WrongInput { action, key },
    };
    let operation = match runtime
        .mutate(mutations::recognize_estate_executor(input))
        .as_principal(principal)
        .controls(BankMutationControls::new(request.clone(), key))
        .retain_operation()
    {
        Ok(operation) => operation,
        Err(refusal) => return EstateSourceCommandOutcome::Construction(refusal),
    };
    // These existing command inputs declare no additional caller conditions.
    // Construction retains that exact typed empty condition set and owned key.
    close_completed(
        operation.advance(&runtime.request(principal, request)),
        &runtime.request(principal, request),
    )
}

pub(super) fn release(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    request: &WorthQueryRequestScope,
    action: EstateAction,
    key: BankIdempotencyKey,
) -> EstateSourceCommandOutcome<ReleaseEstate, bank_server::EstateReleaseAction> {
    let input = match ReleaseEstate::try_from(action) {
        Ok(input) => input,
        Err(action) => return EstateSourceCommandOutcome::WrongInput { action, key },
    };
    let operation = match runtime
        .mutate(mutations::release_estate(input))
        .as_principal(principal)
        .controls(BankMutationControls::new(request.clone(), key))
        .retain_operation()
    {
        Ok(operation) => operation,
        Err(refusal) => return EstateSourceCommandOutcome::Construction(refusal),
    };
    // These existing command inputs declare no additional caller conditions.
    // Construction retains that exact typed empty condition set and owned key.
    close_completed(
        operation.advance(&runtime.request(principal, request)),
        &runtime.request(principal, request),
    )
}

pub(super) fn disburse(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    request: &WorthQueryRequestScope,
    action: EstateAction,
    key: BankIdempotencyKey,
) -> EstateSourceCommandOutcome<DisburseEstate, bank_server::EstateDisbursementAction> {
    let input = match DisburseEstate::try_from(action) {
        Ok(input) => input,
        Err(action) => return EstateSourceCommandOutcome::WrongInput { action, key },
    };
    let operation = match runtime
        .mutate(mutations::disburse_estate(input))
        .as_principal(principal)
        .controls(BankMutationControls::new(request.clone(), key))
        .retain_operation()
    {
        Ok(operation) => operation,
        Err(refusal) => return EstateSourceCommandOutcome::Construction(refusal),
    };
    // These existing command inputs declare no additional caller conditions.
    // Construction retains that exact typed empty condition set and owned key.
    close_completed(
        operation.advance(&runtime.request(principal, request)),
        &runtime.request(principal, request),
    )
}
