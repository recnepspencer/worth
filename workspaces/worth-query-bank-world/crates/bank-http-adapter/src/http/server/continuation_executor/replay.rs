use bank_domain::model::AccountId;
use bank_server::{BankAuthenticatedPrincipal, BankReadControls};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

use super::{
    access, denied, malformed, query_denial, AdmittedPageRequest,
    BankHttpAccountActivityPageOutcome, BankHttpApplicationAuthenticator, BankHttpDenial,
    BankHttpDenialKind, BankHttpNextAction, SharedContinuationRegistry,
};

pub(super) fn revokes_cached_access(denial: BankHttpDenial) -> bool {
    matches!(
        denial.kind,
        BankHttpDenialKind::PermissionDenied
            | BankHttpDenialKind::Unauthenticated
            | BankHttpDenialKind::NotFound
            | BankHttpDenialKind::Stale
    )
}

pub(super) fn revalidate_replay<A>(
    application: &A,
    principal: &BankAuthenticatedPrincipal,
    account: AccountId,
    scope: &WorthQueryRequestScope,
) -> Result<(), BankHttpDenial>
where
    A: BankHttpApplicationAuthenticator,
{
    let controls = BankReadControls::current(scope.clone(), 1).map_err(|_| malformed())?;
    application
        .runtime()
        .account_activity(account)
        .as_principal(principal)
        .page(controls)
        .map_err(query_denial)?;
    Ok(())
}

pub(super) async fn fail_resume(
    registry: &SharedContinuationRegistry,
    token: &str,
    request: &AdmittedPageRequest,
    denial: BankHttpDenial,
) -> BankHttpAccountActivityPageOutcome {
    let scrub_initial = revokes_cached_access(denial);
    let denial = match denial.next_action {
        BankHttpNextAction::Retry
        | BankHttpNextAction::NarrowRequest
        | BankHttpNextAction::CorrectRequest => {
            BankHttpDenial::new(denial.kind, BankHttpNextAction::Refresh)
        }
        _ => denial,
    };
    let outcome = denied(Some(request.request_id.clone()), denial);
    access(registry, |registry| {
        registry.fail_resume(
            token,
            request.request_id.clone(),
            outcome.clone(),
            scrub_initial,
        )
    })
    .await;
    outcome
}
