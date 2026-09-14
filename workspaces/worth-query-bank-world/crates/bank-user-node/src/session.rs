use std::time::{Duration, Instant};

use bank_http_adapter::{
    AuthentikAuthorizationCallback, AuthentikOidcAdapter, AuthentikOidcCredential,
    AuthentikPendingAuthorization, BankHttpAccountActivityPageOutcome,
    BankHttpAccountActivityPageRequest, BankHttpAccountActivityResumeRequest,
    BankHttpAccountActivityStreamRequest, BankHttpAccountSummaryOutcome,
    BankHttpAccountSummaryRequest, BankHttpCredential, BankHttpMutationOutcome,
    BankHttpMutationRequest, BankHttpProtocolVersion,
};
use tokio::sync::{watch, Mutex};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use crate::configuration::BankUserNodeConfiguration;
use crate::protocol::{
    BankUserNodeAccountActivityPageOutcome, BankUserNodeAccountActivityPageRequest,
    BankUserNodeAccountActivityResumeRequest, BankUserNodeAccountActivityStreamRequest,
    BankUserNodeAccountSummaryOutcome, BankUserNodeAccountSummaryRequest,
    BankUserNodeAuthorizationOutcome, BankUserNodeDenial, BankUserNodeDenialKind,
    BankUserNodeMutationOutcome, BankUserNodeMutationRequest,
};

mod aftermath;
mod elevation;
mod recovery;
mod upstream_response;

pub(super) struct BankUserSession {
    oidc: AuthentikOidcAdapter,
    pending: Mutex<Option<AuthentikPendingAuthorization>>,
    lifecycle_transition: Mutex<()>,
    credential: Mutex<Option<AuthentikOidcCredential>>,
    session_revision: watch::Sender<u64>,
    client: reqwest::Client,
    account_summary_endpoint: url::Url,
    account_activity_endpoint: url::Url,
    account_activity_page_endpoint: url::Url,
    account_activity_resume_endpoint: url::Url,
    mutation_endpoint: url::Url,
    estate_notification_endpoint: url::Url,
    recovery_inspection_endpoint: url::Url,
    estate_disbursement_endpoint: url::Url,
    elevation_request_endpoint: url::Url,
    elevation_approval_endpoint: url::Url,
    elevation_revocation_endpoint: url::Url,
    mandatory_review_endpoint: url::Url,
    maximum_deadline: Duration,
}

pub(super) struct BankUserActivityStream {
    response: reqwest::Response,
    session_revision: watch::Receiver<u64>,
}

impl BankUserActivityStream {
    pub(super) fn into_transport(self) -> (reqwest::Response, watch::Receiver<u64>) {
        (self.response, self.session_revision)
    }
}

impl BankUserSession {
    pub(super) fn new(
        oidc: AuthentikOidcAdapter,
        configuration: &BankUserNodeConfiguration,
    ) -> Result<Self, url::ParseError> {
        let account_summary_endpoint = configuration
            .bank_server_origin
            .join("v1/queries/account-summary")?;
        let account_activity_endpoint = configuration
            .bank_server_origin
            .join("v1/live/account-activity")?;
        let account_activity_page_endpoint = configuration
            .bank_server_origin
            .join("v1/queries/account-activity/page")?;
        let account_activity_resume_endpoint = configuration
            .bank_server_origin
            .join("v1/queries/account-activity/resume")?;
        let mutation_endpoint = configuration.bank_server_origin.join("v1/mutations")?;
        let estate_notification_endpoint = configuration
            .bank_server_origin
            .join("v1/estate/notify-death")?;
        let recovery_inspection_endpoint = configuration
            .bank_server_origin
            .join("v1/recovery/inspect")?;
        let estate_disbursement_endpoint = configuration
            .bank_server_origin
            .join("v1/estate/disburse")?;
        let elevation_request_endpoint = configuration
            .bank_server_origin
            .join("v1/estate/elevation/request")?;
        let elevation_approval_endpoint = configuration
            .bank_server_origin
            .join("v1/estate/elevation/approve")?;
        let elevation_revocation_endpoint = configuration
            .bank_server_origin
            .join("v1/estate/elevation/revoke")?;
        let mandatory_review_endpoint = configuration
            .bank_server_origin
            .join("v1/estate/elevation/review")?;
        let (session_revision, _) = watch::channel(0);
        Ok(Self {
            oidc,
            pending: Mutex::new(None),
            lifecycle_transition: Mutex::new(()),
            credential: Mutex::new(None),
            session_revision,
            client: reqwest::Client::new(),
            account_summary_endpoint,
            account_activity_endpoint,
            account_activity_page_endpoint,
            account_activity_resume_endpoint,
            mutation_endpoint,
            estate_notification_endpoint,
            recovery_inspection_endpoint,
            estate_disbursement_endpoint,
            elevation_request_endpoint,
            elevation_approval_endpoint,
            elevation_revocation_endpoint,
            mandatory_review_endpoint,
            maximum_deadline: configuration.maximum_deadline,
        })
    }

    pub(super) async fn begin_authorization(&self) -> BankUserNodeAuthorizationOutcome {
        let mut pending = self.pending.lock().await;
        if pending.is_some() {
            return authorization_denied(BankUserNodeDenialKind::AuthorizationAlreadyPending);
        }
        let authorization = self.oidc.begin_authorization().await;
        let authorization_url = authorization.authorization_url().to_owned();
        *pending = Some(authorization.into_pending());
        BankUserNodeAuthorizationOutcome::AuthorizationRequired { authorization_url }
    }

    pub(super) async fn finish_authorization(
        &self,
        code: String,
        state: String,
    ) -> BankUserNodeAuthorizationOutcome {
        let _transition = self.lifecycle_transition.lock().await;
        let Some(pending) = self.pending.lock().await.take() else {
            return authorization_denied(BankUserNodeDenialKind::AuthorizationNotPending);
        };
        let Ok(callback) = AuthentikAuthorizationCallback::new(code, state) else {
            return authorization_denied(BankUserNodeDenialKind::AuthorizationRejected);
        };
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Instant::now() + self.maximum_deadline,
            cancellation.token(),
        );
        match self
            .oidc
            .finish_authorization(pending, callback, &scope)
            .await
        {
            Ok(credential) => {
                let mut installed = self.credential.lock().await;
                *installed = Some(credential);
                self.session_revision.send_modify(|revision| *revision += 1);
                BankUserNodeAuthorizationOutcome::Authenticated
            }
            Err(_) => authorization_denied(BankUserNodeDenialKind::AuthorizationRejected),
        }
    }

    pub(super) async fn revoke_authorization(&self) -> BankUserNodeAuthorizationOutcome {
        let _transition = self.lifecycle_transition.lock().await;
        let Some(credential) = self.credential.lock().await.take() else {
            return authorization_denied(BankUserNodeDenialKind::NoAuthenticatedSession);
        };
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Instant::now() + self.maximum_deadline,
            cancellation.token(),
        );
        if self
            .oidc
            .revoke_credential(&credential, &scope)
            .await
            .is_err()
        {
            *self.credential.lock().await = Some(credential);
            return authorization_denied(BankUserNodeDenialKind::AuthorizationRejected);
        }
        self.session_revision.send_modify(|revision| *revision += 1);
        BankUserNodeAuthorizationOutcome::Revoked
    }

    pub(super) async fn account_summary(
        &self,
        request: BankUserNodeAccountSummaryRequest,
    ) -> BankUserNodeAccountSummaryOutcome {
        let credential = match self.credential.lock().await.clone() {
            Some(credential) => credential,
            None => {
                return BankUserNodeAccountSummaryOutcome::Denied {
                    denial: denial(BankUserNodeDenialKind::NoAuthenticatedSession),
                }
            }
        };
        let upstream = BankHttpAccountSummaryRequest {
            protocol: BankHttpProtocolVersion::V1,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            account: request.account,
        };
        match self
            .forward::<_, BankHttpAccountSummaryOutcome>(
                self.account_summary_endpoint.clone(),
                &upstream,
                upstream.controls.deadline_milliseconds,
            )
            .await
        {
            Ok(response) => BankUserNodeAccountSummaryOutcome::Forwarded { response },
            Err(kind) => BankUserNodeAccountSummaryOutcome::Denied {
                denial: denial(kind),
            },
        }
    }

    pub(super) async fn open_account_activity(
        &self,
        request: BankUserNodeAccountActivityStreamRequest,
    ) -> Result<BankUserActivityStream, BankUserNodeDenial> {
        let (credential, session_revision) = {
            let credential = self.credential.lock().await;
            let credential = credential
                .clone()
                .ok_or_else(|| denial(BankUserNodeDenialKind::NoAuthenticatedSession))?;
            (credential, self.session_revision.subscribe())
        };
        let upstream = BankHttpAccountActivityStreamRequest {
            protocol: BankHttpProtocolVersion::V1,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            account: request.account,
            source_buffer_capacity: request.source_buffer_capacity,
        };
        let response = self
            .send_upstream(
                self.account_activity_endpoint.clone(),
                &upstream,
                upstream.controls.deadline_milliseconds,
            )
            .await
            .map_err(denial)?;
        if !response.status().is_success() {
            return Err(denial(BankUserNodeDenialKind::UpstreamProtocolViolation));
        }
        if session_revision.has_changed().unwrap_or(true) {
            return Err(denial(BankUserNodeDenialKind::NoAuthenticatedSession));
        }
        Ok(BankUserActivityStream {
            response,
            session_revision,
        })
    }

    pub(super) async fn account_activity_page(
        &self,
        request: BankUserNodeAccountActivityPageRequest,
    ) -> BankUserNodeAccountActivityPageOutcome {
        let credential = match self.credential.lock().await.clone() {
            Some(credential) => credential,
            None => return activity_denied(BankUserNodeDenialKind::NoAuthenticatedSession),
        };
        let upstream = BankHttpAccountActivityPageRequest {
            protocol: BankHttpProtocolVersion::V1,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            account: request.account,
        };
        self.forward_activity_page(
            self.account_activity_page_endpoint.clone(),
            &upstream,
            upstream.controls.deadline_milliseconds,
        )
        .await
    }

    pub(super) async fn account_activity_resume(
        &self,
        request: BankUserNodeAccountActivityResumeRequest,
    ) -> BankUserNodeAccountActivityPageOutcome {
        let credential = match self.credential.lock().await.clone() {
            Some(credential) => credential,
            None => return activity_denied(BankUserNodeDenialKind::NoAuthenticatedSession),
        };
        let upstream = BankHttpAccountActivityResumeRequest {
            protocol: BankHttpProtocolVersion::V1,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            account: request.account,
            continuation: request.continuation,
        };
        self.forward_activity_page(
            self.account_activity_resume_endpoint.clone(),
            &upstream,
            upstream.controls.deadline_milliseconds,
        )
        .await
    }

    async fn forward_activity_page<T: serde::Serialize + ?Sized>(
        &self,
        endpoint: url::Url,
        request: &T,
        deadline_milliseconds: u64,
    ) -> BankUserNodeAccountActivityPageOutcome {
        match self
            .forward::<_, BankHttpAccountActivityPageOutcome>(
                endpoint,
                request,
                deadline_milliseconds,
            )
            .await
        {
            Ok(response) => BankUserNodeAccountActivityPageOutcome::Forwarded { response },
            Err(kind) => activity_denied(kind),
        }
    }

    pub(super) async fn mutate(
        &self,
        request: BankUserNodeMutationRequest,
    ) -> BankUserNodeMutationOutcome {
        let credential = match self.credential.lock().await.clone() {
            Some(credential) => credential,
            None => return mutation_denied(BankUserNodeDenialKind::NoAuthenticatedSession),
        };
        let upstream = BankHttpMutationRequest {
            protocol: BankHttpProtocolVersion::V1,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            idempotency_key: request.idempotency_key,
            operation: request.operation,
        };
        match self
            .forward::<_, BankHttpMutationOutcome>(
                self.mutation_endpoint.clone(),
                &upstream,
                upstream.controls.deadline_milliseconds,
            )
            .await
        {
            Ok(response) => BankUserNodeMutationOutcome::Forwarded { response },
            Err(kind) => mutation_denied(kind),
        }
    }
}

fn authorization_denied(kind: BankUserNodeDenialKind) -> BankUserNodeAuthorizationOutcome {
    BankUserNodeAuthorizationOutcome::Denied {
        denial: denial(kind),
    }
}

const fn denial(kind: BankUserNodeDenialKind) -> BankUserNodeDenial {
    BankUserNodeDenial::new(kind)
}

fn activity_denied(kind: BankUserNodeDenialKind) -> BankUserNodeAccountActivityPageOutcome {
    BankUserNodeAccountActivityPageOutcome::Denied {
        denial: denial(kind),
    }
}

fn mutation_denied(kind: BankUserNodeDenialKind) -> BankUserNodeMutationOutcome {
    BankUserNodeMutationOutcome::Denied {
        denial: denial(kind),
    }
}
