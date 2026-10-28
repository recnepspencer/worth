//! Freshly authenticated traversal retains only the latest native page.
use super::*;
use crate::http::protocol::{BankHttpCorrectionHistoryOutcome, BankHttpCorrectionSource};
use crate::http::server::continuation_registry::correction_history::HistoryResumeAdmission;
use crate::http::server::mutation_construction::correction::correction_denial;
use worth_query_host::facade::application_entry::WorthQueryCorrectionHistoryLimits;

pub(in crate::http::server) struct AdmittedHistoryRequest {
    pub request_id: String,
    pub credential: BankHttpCredential,
    pub limits: WorthQueryCorrectionHistoryLimits,
    pub page_width: NonZeroUsize,
    pub source: BankHttpCorrectionSource,
    pub deadline: Instant,
    pub continuation: Option<String>,
}
pub(super) struct HistoryJob {
    request: AdmittedHistoryRequest,
    cancellation: WorthQueryCancellationSource,
    pub(super) response: oneshot::Sender<BankHttpCorrectionHistoryOutcome>,
}
impl BankHttpContinuationExecutor {
    pub(in crate::http::server) async fn correction_history(
        &self,
        request: AdmittedHistoryRequest,
    ) -> BankHttpCorrectionHistoryOutcome {
        let request_id = request.request_id.clone();
        let deadline = request.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let _cancel = CancelOnDrop(cancellation.clone());
        let (response, receiver) = oneshot::channel();
        if self
            .sender
            .try_send(ContinuationCommand::Correction(HistoryJob {
                request,
                cancellation,
                response,
            }))
            .is_err()
        {
            return refused(&request_id, saturated());
        }
        match tokio::time::timeout_at(deadline.into(), receiver).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => refused(&request_id, unavailable()),
            Err(_) => refused(&request_id, deadline_denial()),
        }
    }
}
struct CancelOnDrop(WorthQueryCancellationSource);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

pub(super) async fn execute<A: BankHttpApplicationAuthenticator>(
    application: &A,
    registry: &SharedContinuationRegistry,
    job: &HistoryJob,
) -> BankHttpCorrectionHistoryOutcome {
    let request = &job.request;
    let scope = WorthQueryRequestScope::new(Some(request.deadline), job.cancellation.token());
    let principal = match application
        .authenticate(request.credential.clone(), &scope)
        .await
    {
        Ok(principal) => principal,
        Err(cause) => return refused(&request.request_id, cause),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    let mut replay = None;
    let previous = match &request.continuation {
        None => {
            replay = access(registry, |registry| {
                registry.replay_correction_initial(&owner, &request.source, &request.request_id)
            })
            .await;
            // This executor serializes registrations; worker lookups can only free slots.
            if replay.is_none()
                && !access(registry, |registry| {
                    registry.correction_initial_has_capacity()
                })
                .await
            {
                return refused(&request.request_id, saturated());
            }
            None
        }
        Some(token) => match access(registry, |registry| {
            registry.begin_correction_resume(&owner, &request.source, &request.request_id, token)
        })
        .await
        {
            HistoryResumeAdmission::Execute(page) => Some(page),
            HistoryResumeAdmission::Replay(outcome) => {
                replay = Some((token.clone(), outcome));
                None
            }
            HistoryResumeAdmission::Denied(cause) => return refused(&request.request_id, cause),
        },
    };
    if let Some((token, outcome)) = replay {
        if matches!(outcome, BankHttpCorrectionHistoryOutcome::Delivered { .. })
            && let Err(cause) = disclose(application, &principal, &scope, request)
        {
            if revokes_cached_access(cause) {
                access(registry, |registry| {
                    registry.deny_correction_replay(&token, cause)
                })
                .await;
            }
            return refused(&request.request_id, cause);
        }
        return outcome;
    }
    let result = read_page(
        application,
        &principal,
        &scope,
        request,
        previous.as_deref(),
    );
    let page = match result {
        Ok(page) => page,
        Err(cause) => {
            if let (Some(token), Some(previous)) = (&request.continuation, previous) {
                let returned = access(registry, |registry| {
                    let returned = registry.fail_correction_resume(
                        token,
                        &request.request_id,
                        cause,
                        previous,
                        !revokes_cached_access(cause),
                    );
                    if revokes_cached_access(cause) {
                        registry.deny_correction_replay(token, cause);
                    }
                    returned
                })
                .await;
                // Expired or revoked custody remains owned until the lock has been released.
                drop(returned);
            }
            return refused(&request.request_id, cause);
        }
    };
    drop(previous);
    let stored = match &request.continuation {
        None => {
            access(registry, |registry| {
                registry.register_correction_initial(
                    owner,
                    request.source.clone(),
                    request.request_id.clone(),
                    page,
                )
            })
            .await
        }
        Some(token) => {
            access(registry, |registry| {
                registry.complete_correction_resume(token, &request.request_id, page)
            })
            .await
        }
    };
    stored.unwrap_or_else(|page| {
        drop(page);
        refused(
            &request.request_id,
            if request.continuation.is_some() {
                stale()
            } else {
                saturated()
            },
        )
    })
}
fn disclose<A: BankHttpApplicationAuthenticator>(
    application: &A,
    principal: &bank_server::BankAuthenticatedPrincipal,
    scope: &WorthQueryRequestScope,
    request: &AdmittedHistoryRequest,
) -> Result<(), BankHttpDenial> {
    application
        .runtime()
        .request(principal, scope)
        .corrections()
        .select_coordinates(&request.source.clone().into(), request.limits)
        .map(|_| ())
        .map_err(correction_denial)
}
fn read_page<A: BankHttpApplicationAuthenticator>(
    application: &A,
    principal: &bank_server::BankAuthenticatedPrincipal,
    scope: &WorthQueryRequestScope,
    request: &AdmittedHistoryRequest,
    previous: Option<
        &worth_query_host::facade::primary_graph::WorthQueryCorrectionAlternativesPage,
    >,
) -> Result<
    Arc<worth_query_host::facade::primary_graph::WorthQueryCorrectionAlternativesPage>,
    BankHttpDenial,
> {
    let native_request = application.runtime().request(principal, scope);
    let limits = request.limits;
    let selected = native_request
        .corrections()
        .select_coordinates(&request.source.clone().into(), limits)
        .map_err(correction_denial)?;
    let width = request.page_width;
    let page = match previous {
        None => native_request
            .corrections()
            .alternatives(&selected, limits, width),
        Some(previous) => native_request.corrections().continue_alternatives(
            &selected,
            limits,
            width,
            previous.continuation().ok_or_else(stale)?,
        ),
    }
    .map_err(correction_denial)?;
    Ok(Arc::new(page.into_page()))
}

fn refused(request_id: &str, denial: BankHttpDenial) -> BankHttpCorrectionHistoryOutcome {
    BankHttpCorrectionHistoryOutcome::Denied {
        request_id: Some(request_id.to_owned()),
        denial,
    }
}
