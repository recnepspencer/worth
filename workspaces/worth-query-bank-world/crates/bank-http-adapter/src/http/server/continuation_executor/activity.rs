//! Current activity orchestration lends inventory only around record transitions.
use super::*;
pub(super) async fn execute_page<A>(
    application: &A,
    registry: &SharedContinuationRegistry,
    job: &ContinuationJob<AdmittedPageRequest>,
) -> BankHttpAccountActivityPageOutcome
where
    A: BankHttpApplicationAuthenticator,
{
    let request = &job.request;
    let scope = WorthQueryRequestScope::new(Some(request.deadline), job.cancellation.token());
    let principal = match application
        .authenticate(request.credential.clone(), &scope)
        .await
    {
        Ok(principal) => principal,
        Err(denial) => return denied(Some(request.request_id.clone()), denial),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    if let Some((token, outcome)) = access(registry, |registry| {
        registry.replay_initial(&owner, request.account, &request.request_id)
    })
    .await
    {
        if matches!(
            outcome,
            BankHttpAccountActivityPageOutcome::Delivered { .. }
        ) && let Err(denial) = revalidate_replay(application, &principal, request, &scope)
        {
            if revokes_cached_access(denial) {
                access(registry, |registry| {
                    registry.deny_protected_replay(&token, denial)
                })
                .await;
            }
            return denied(Some(request.request_id.clone()), denial);
        }
        return outcome;
    }
    let controls = match page_controls(&request.controls, &scope) {
        Some(controls) => controls,
        None => return denied(Some(request.request_id.clone()), malformed()),
    };
    let page = match application
        .runtime()
        .account_activity(request.account)
        .as_principal(&principal)
        .page(controls)
    {
        Ok(page) => page,
        Err(error) => return denied(Some(request.request_id.clone()), query_denial(error)),
    };
    let (published, continuation) = page.into_parts();
    let Some(activity) = published.rows().first().map(BankHttpAccountActivity::from) else {
        return denied(Some(request.request_id.clone()), internal());
    };
    let publication = describe_query_publication(
        published.receipt(),
        BankHttpQueryCapabilityPurpose::AccountActivityReview,
    );
    access(registry, |registry| {
        registry.register_initial(
            owner,
            request.account,
            request.request_id.clone(),
            continuation,
            activity,
            publication,
        )
    })
    .await
    .unwrap_or_else(|custody| {
        drop(custody);
        denied(Some(request.request_id.clone()), saturated())
    })
}

pub(super) async fn execute_resume<A>(
    application: &A,
    registry: &SharedContinuationRegistry,
    job: &ContinuationJob<AdmittedResumeRequest>,
) -> BankHttpAccountActivityPageOutcome
where
    A: BankHttpApplicationAuthenticator,
{
    let request = &job.request.page;
    let scope = WorthQueryRequestScope::new(Some(request.deadline), job.cancellation.token());
    let principal = match application
        .authenticate(request.credential.clone(), &scope)
        .await
    {
        Ok(principal) => principal,
        Err(denial) => return denied(Some(request.request_id.clone()), denial),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    let controls = match resume_controls(&request.controls, &scope) {
        Some(controls) => controls,
        None => return denied(Some(request.request_id.clone()), malformed()),
    };
    if let Some(interruption) = scope.interruption() {
        let denial = match interruption {
            WorthQueryRequestInterruption::Cancelled => {
                BankHttpDenial::new(BankHttpDenialKind::Cancelled, BankHttpNextAction::Retry)
            }
            WorthQueryRequestInterruption::DeadlineExceeded => deadline_denial(),
        };
        return denied(Some(request.request_id.clone()), denial);
    }
    let continuation = match access(registry, |registry| {
        registry.begin_resume(
            &owner,
            request.account,
            &request.request_id,
            &job.request.continuation,
        )
    })
    .await
    {
        ResumeAdmission::Execute(continuation) => continuation,
        ResumeAdmission::Replay(outcome) => {
            if matches!(
                outcome,
                BankHttpAccountActivityPageOutcome::Delivered { .. }
            ) && let Err(denial) = revalidate_replay(application, &principal, request, &scope)
            {
                if revokes_cached_access(denial) {
                    access(registry, |registry| {
                        registry.deny_protected_replay(&job.request.continuation, denial)
                    })
                    .await;
                }
                return denied(Some(request.request_id.clone()), denial);
            }
            return outcome;
        }
        ResumeAdmission::InFlight => return denied(Some(request.request_id.clone()), saturated()),
        ResumeAdmission::Unavailable => return denied(Some(request.request_id.clone()), stale()),
    };
    let page = application
        .runtime()
        .account_activity(request.account)
        .as_principal(&principal)
        .resume(continuation, controls);
    complete_resume(registry, &job.request.continuation, request, page).await
}

async fn complete_resume(
    registry: &SharedContinuationRegistry,
    token: &str,
    request: &AdmittedPageRequest,
    page: Result<bank_server::BankAccountActivityPageResult, BankApplicationQueryDenial>,
) -> BankHttpAccountActivityPageOutcome {
    match page {
        Ok(page) => {
            let (published, continuation) = page.into_parts();
            let Some(activity) = published.rows().first().map(BankHttpAccountActivity::from) else {
                return fail_resume(registry, token, request, internal()).await;
            };
            let publication = describe_query_publication(
                published.receipt(),
                BankHttpQueryCapabilityPurpose::AccountActivityReview,
            );
            access(registry, |registry| {
                registry.complete_resume(
                    token,
                    request.request_id.clone(),
                    activity,
                    continuation,
                    publication,
                )
            })
            .await
            .unwrap_or_else(|custody| {
                drop(custody);
                denied(Some(request.request_id.clone()), stale())
            })
        }
        Err(error) => fail_resume(registry, token, request, query_denial(error)).await,
    }
}

fn page_controls(
    controls: &BankHttpRequestControls,
    scope: &WorthQueryRequestScope,
) -> Option<BankReadControls> {
    BankReadControls::current(scope.clone(), controls.maximum_results).ok()
}

fn resume_controls<'a>(
    controls: &BankHttpRequestControls,
    scope: &'a WorthQueryRequestScope,
) -> Option<WorthQueryApplicationQueryResumeControls<'a>> {
    Some(WorthQueryApplicationQueryResumeControls::new(
        NonZeroUsize::new(controls.maximum_results)?,
        scope,
    ))
}
