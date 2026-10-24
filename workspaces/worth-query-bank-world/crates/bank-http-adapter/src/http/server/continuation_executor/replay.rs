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
    request: &AdmittedPageRequest,
    scope: &WorthQueryRequestScope,
) -> Result<(), BankHttpDenial>
where
    A: BankHttpApplicationAuthenticator,
{
    #[cfg(test)]
    if let Some(denial) = fault::take(&request.request_id) {
        return Err(denial);
    }
    let controls = BankReadControls::current(scope.clone(), 1).map_err(|_| malformed())?;
    application
        .runtime()
        .account_activity(request.account)
        .as_principal(principal)
        .page(controls)
        .map_err(query_denial)?;
    Ok(())
}

/// Test-only replay-read faults, keyed by request id so parallel courts stay
/// apart. An armed fault replaces the next revalidation read's result.
#[cfg(test)]
pub(in crate::http::server) mod fault {
    use std::sync::Mutex;

    use super::BankHttpDenial;

    static ARMED: Mutex<Vec<(String, BankHttpDenial)>> = Mutex::new(Vec::new());

    pub(in crate::http::server) fn fail_next_replay_read(request_id: &str, denial: BankHttpDenial) {
        ARMED.lock().unwrap().push((request_id.to_owned(), denial));
    }

    pub(super) fn take(request_id: &str) -> Option<BankHttpDenial> {
        let mut armed = ARMED.lock().unwrap();
        let index = armed.iter().position(|(id, _)| id == request_id)?;
        Some(armed.remove(index).1)
    }
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
