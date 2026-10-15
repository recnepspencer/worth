use std::sync::Arc;
use std::time::Instant;

use bank_domain::estate::EstateAction;
use bank_domain::proposals::BankIdempotencyKey;
use tokio::sync::oneshot;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;

use super::super::protocol::{
    BankHttpCredential, BankHttpElevationApprovalOutcome, BankHttpElevationRequestOutcome,
    BankHttpElevationRevocationOutcome, BankHttpMandatoryReviewOutcome,
};
use super::authentication::BankHttpApplicationAuthenticator;
use super::elevation_registry::{BankHttpElevationContext, BankHttpElevationRegistry};

mod approval;
mod mandatory_review;
mod outcome;
mod request;
mod revocation;

use approval::execute_approval;
use mandatory_review::execute_mandatory_review;
use outcome::*;
use request::execute_request;
use revocation::execute_revocation;

pub(super) struct AdmittedBankHttpElevationRequest {
    pub(super) request_id: String,
    pub(super) credential: BankHttpCredential,
    pub(super) idempotency_key: BankIdempotencyKey,
    pub(super) action: EstateAction,
    pub(super) context: BankHttpElevationContext,
    pub(super) deadline: Instant,
}

pub(super) struct AdmittedBankHttpElevationTransition {
    pub(super) request_id: String,
    pub(super) credential: BankHttpCredential,
    pub(super) idempotency_key: BankIdempotencyKey,
    pub(super) token: String,
    pub(super) deadline: Instant,
}

#[derive(Clone)]
pub(super) struct BankHttpElevationExecutor {
    request: Arc<
        dyn Fn(
                AdmittedBankHttpElevationRequest,
                WorthQueryCancellationSource,
                oneshot::Sender<BankHttpElevationRequestOutcome>,
            ) + Send
            + Sync,
    >,
    approve: Arc<
        dyn Fn(
                AdmittedBankHttpElevationTransition,
                WorthQueryCancellationSource,
                oneshot::Sender<BankHttpElevationApprovalOutcome>,
            ) + Send
            + Sync,
    >,
    revoke: Arc<
        dyn Fn(
                AdmittedBankHttpElevationTransition,
                WorthQueryCancellationSource,
                oneshot::Sender<BankHttpElevationRevocationOutcome>,
            ) + Send
            + Sync,
    >,
    review: Arc<
        dyn Fn(
                AdmittedBankHttpElevationTransition,
                WorthQueryCancellationSource,
                oneshot::Sender<BankHttpMandatoryReviewOutcome>,
            ) + Send
            + Sync,
    >,
}
impl BankHttpElevationExecutor {
    pub(super) fn start<A: BankHttpApplicationAuthenticator>(
        application: Arc<A>,
        _queue_capacity: usize,
        capacity: usize,
        lifetime: std::time::Duration,
        mutations: super::mutation_executor::BankHttpMutationContinuation,
    ) -> Self {
        let registry = Arc::new(tokio::sync::Mutex::new(BankHttpElevationRegistry::new(
            capacity, lifetime,
        )));
        let request = {
            let application = Arc::clone(&application);
            let registry = Arc::clone(&registry);
            let mutations = mutations.clone();
            Arc::new(
                move |request: AdmittedBankHttpElevationRequest, cancellation, response| {
                    let application = Arc::clone(&application);
                    let registry = Arc::clone(&registry);
                    let id = request.request_id.clone();
                    let denied_id = id.clone();
                    mutations.submit_native(
                        id,
                        super::native_route::job(
                            move || async move {
                                let mut registry = registry.lock().await;
                                execute_request(
                                    application.as_ref(),
                                    &mut registry,
                                    request,
                                    cancellation,
                                )
                                .await
                            },
                            move |kind| {
                                request_denied(
                                    Some(denied_id),
                                    super::mutation_refusal::denial(kind),
                                )
                            },
                            response,
                        ),
                    );
                },
            )
        };
        let approve = {
            let application = Arc::clone(&application);
            let registry = Arc::clone(&registry);
            let mutations = mutations.clone();
            Arc::new(
                move |request: AdmittedBankHttpElevationTransition, cancellation, response| {
                    let application = Arc::clone(&application);
                    let registry = Arc::clone(&registry);
                    let id = request.request_id.clone();
                    let denied_id = id.clone();
                    mutations.submit_native(
                        id,
                        super::native_route::job(
                            move || async move {
                                let mut registry = registry.lock().await;
                                execute_approval(
                                    application.as_ref(),
                                    &mut registry,
                                    request,
                                    cancellation,
                                )
                                .await
                            },
                            move |kind| {
                                approval_denied(
                                    Some(denied_id),
                                    super::mutation_refusal::denial(kind),
                                )
                            },
                            response,
                        ),
                    );
                },
            )
        };
        let revoke = {
            let application = Arc::clone(&application);
            let registry = Arc::clone(&registry);
            let mutations = mutations.clone();
            Arc::new(
                move |request: AdmittedBankHttpElevationTransition, cancellation, response| {
                    let application = Arc::clone(&application);
                    let registry = Arc::clone(&registry);
                    let id = request.request_id.clone();
                    let denied_id = id.clone();
                    mutations.submit_native(
                        id,
                        super::native_route::job(
                            move || async move {
                                let mut registry = registry.lock().await;
                                execute_revocation(
                                    application.as_ref(),
                                    &mut registry,
                                    request,
                                    cancellation,
                                )
                                .await
                            },
                            move |kind| {
                                revocation_denied(
                                    Some(denied_id),
                                    super::mutation_refusal::denial(kind),
                                )
                            },
                            response,
                        ),
                    );
                },
            )
        };
        let review = {
            let application = Arc::clone(&application);
            let registry = Arc::clone(&registry);
            let mutations = mutations.clone();
            Arc::new(
                move |request: AdmittedBankHttpElevationTransition, cancellation, response| {
                    let application = Arc::clone(&application);
                    let registry = Arc::clone(&registry);
                    let id = request.request_id.clone();
                    let denied_id = id.clone();
                    mutations.submit_native(
                        id,
                        super::native_route::job(
                            move || async move {
                                let mut registry = registry.lock().await;
                                execute_mandatory_review(
                                    application.as_ref(),
                                    &mut registry,
                                    request,
                                    cancellation,
                                )
                                .await
                            },
                            move |kind| {
                                review_denied(
                                    Some(denied_id),
                                    super::mutation_refusal::denial(kind),
                                )
                            },
                            response,
                        ),
                    );
                },
            )
        };
        Self {
            request,
            approve,
            revoke,
            review,
        }
    }
    pub(super) async fn request(
        &self,
        request: AdmittedBankHttpElevationRequest,
    ) -> BankHttpElevationRequestOutcome {
        let id = request.request_id.clone();
        let deadline = request.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let cancellation_guard = CancelOnDrop(cancellation.clone());
        let (response, receiver) = oneshot::channel();
        let response_wait = tokio::time::timeout_at(deadline.into(), receiver);
        tokio::pin!(response_wait);
        // Cancellation must precede disposal of the response future in every edition.
        let _cancel = cancellation_guard;
        (self.request)(request, cancellation, response);
        match response_wait.await {
            Ok(Ok(outcome)) => outcome,
            _ => BankHttpElevationRequestOutcome::OutcomeUnknown {
                request_id: id.clone(),
                custody_id: id,
            },
        }
    }
    pub(super) async fn approve(
        &self,
        request: AdmittedBankHttpElevationTransition,
    ) -> BankHttpElevationApprovalOutcome {
        let id = request.request_id.clone();
        let deadline = request.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let cancellation_guard = CancelOnDrop(cancellation.clone());
        let (response, receiver) = oneshot::channel();
        let response_wait = tokio::time::timeout_at(deadline.into(), receiver);
        tokio::pin!(response_wait);
        // Cancellation must precede disposal of the response future in every edition.
        let _cancel = cancellation_guard;
        (self.approve)(request, cancellation, response);
        match response_wait.await {
            Ok(Ok(outcome)) => outcome,
            _ => BankHttpElevationApprovalOutcome::OutcomeUnknown {
                request_id: id.clone(),
                custody_id: id,
            },
        }
    }
    pub(super) async fn revoke(
        &self,
        request: AdmittedBankHttpElevationTransition,
    ) -> BankHttpElevationRevocationOutcome {
        let id = request.request_id.clone();
        let deadline = request.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let cancellation_guard = CancelOnDrop(cancellation.clone());
        let (response, receiver) = oneshot::channel();
        let response_wait = tokio::time::timeout_at(deadline.into(), receiver);
        tokio::pin!(response_wait);
        // Cancellation must precede disposal of the response future in every edition.
        let _cancel = cancellation_guard;
        (self.revoke)(request, cancellation, response);
        match response_wait.await {
            Ok(Ok(outcome)) => outcome,
            _ => BankHttpElevationRevocationOutcome::OutcomeUnknown {
                request_id: id.clone(),
                custody_id: id,
            },
        }
    }
    pub(super) async fn review(
        &self,
        request: AdmittedBankHttpElevationTransition,
    ) -> BankHttpMandatoryReviewOutcome {
        let id = request.request_id.clone();
        let deadline = request.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let cancellation_guard = CancelOnDrop(cancellation.clone());
        let (response, receiver) = oneshot::channel();
        let response_wait = tokio::time::timeout_at(deadline.into(), receiver);
        tokio::pin!(response_wait);
        // Cancellation must precede disposal of the response future in every edition.
        let _cancel = cancellation_guard;
        (self.review)(request, cancellation, response);
        match response_wait.await {
            Ok(Ok(outcome)) => outcome,
            _ => BankHttpMandatoryReviewOutcome::OutcomeUnknown {
                request_id: id.clone(),
                custody_id: id,
            },
        }
    }
}
struct CancelOnDrop(WorthQueryCancellationSource);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
