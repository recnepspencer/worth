use bank_http_adapter::{
    BankHttpCredential, BankHttpElevationApprovalOutcome, BankHttpElevationApprovalRequest,
    BankHttpElevationRequest, BankHttpElevationRequestOutcome, BankHttpElevationRevocationOutcome,
    BankHttpElevationRevocationRequest, BankHttpMandatoryReviewOutcome,
    BankHttpMandatoryReviewRequest, BankHttpProtocolVersion,
};

use super::BankUserSession;
use crate::protocol::{
    BankUserNodeDenialKind, BankUserNodeElevationApprovalOutcome,
    BankUserNodeElevationApprovalRequest, BankUserNodeElevationRequest,
    BankUserNodeElevationRequestOutcome, BankUserNodeElevationRevocationOutcome,
    BankUserNodeElevationRevocationRequest, BankUserNodeMandatoryReviewOutcome,
    BankUserNodeMandatoryReviewRequest,
};

impl BankUserSession {
    pub(crate) async fn request_elevation(
        &self,
        request: BankUserNodeElevationRequest,
    ) -> BankUserNodeElevationRequestOutcome {
        let Some(credential) = self.credential.lock().await.clone() else {
            return request_denied(BankUserNodeDenialKind::NoAuthenticatedSession);
        };
        let upstream = BankHttpElevationRequest {
            protocol: BankHttpProtocolVersion::V4,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            idempotency_key: request.idempotency_key,
            estate: request.estate,
            access: request.access,
            mandatory_review: request.mandatory_review,
            upper_bound_grant: request.upper_bound_grant,
            reason: request.reason,
            field: request.field,
            duration_seconds: request.duration_seconds,
        };
        match self
            .forward::<_, BankHttpElevationRequestOutcome>(
                self.elevation_request_endpoint.clone(),
                &upstream,
                upstream.controls.deadline_milliseconds,
            )
            .await
        {
            Ok(response) => BankUserNodeElevationRequestOutcome::Forwarded { response },
            Err(_) => BankUserNodeElevationRequestOutcome::Forwarded {
                response: BankHttpElevationRequestOutcome::OutcomeUnknown {
                    request_id: upstream.request_id.clone(),
                    custody_id: upstream.request_id.clone(),
                },
            },
        }
    }

    pub(crate) async fn approve_elevation(
        &self,
        request: BankUserNodeElevationApprovalRequest,
    ) -> BankUserNodeElevationApprovalOutcome {
        let Some(credential) = self.credential.lock().await.clone() else {
            return approval_denied(BankUserNodeDenialKind::NoAuthenticatedSession);
        };
        let upstream = BankHttpElevationApprovalRequest {
            protocol: BankHttpProtocolVersion::V4,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            idempotency_key: request.idempotency_key,
            elevation: request.elevation,
        };
        match self
            .forward::<_, BankHttpElevationApprovalOutcome>(
                self.elevation_approval_endpoint.clone(),
                &upstream,
                upstream.controls.deadline_milliseconds,
            )
            .await
        {
            Ok(response) => BankUserNodeElevationApprovalOutcome::Forwarded { response },
            Err(_) => BankUserNodeElevationApprovalOutcome::Forwarded {
                response: BankHttpElevationApprovalOutcome::OutcomeUnknown {
                    request_id: upstream.request_id.clone(),
                    custody_id: upstream.request_id.clone(),
                },
            },
        }
    }

    pub(crate) async fn revoke_elevation(
        &self,
        request: BankUserNodeElevationRevocationRequest,
    ) -> BankUserNodeElevationRevocationOutcome {
        let Some(credential) = self.credential.lock().await.clone() else {
            return revocation_denied(BankUserNodeDenialKind::NoAuthenticatedSession);
        };
        let upstream = BankHttpElevationRevocationRequest {
            protocol: BankHttpProtocolVersion::V4,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            idempotency_key: request.idempotency_key,
            elevation: request.elevation,
        };
        match self
            .forward::<_, BankHttpElevationRevocationOutcome>(
                self.elevation_revocation_endpoint.clone(),
                &upstream,
                upstream.controls.deadline_milliseconds,
            )
            .await
        {
            Ok(response) => BankUserNodeElevationRevocationOutcome::Forwarded { response },
            Err(_) => BankUserNodeElevationRevocationOutcome::Forwarded {
                response: BankHttpElevationRevocationOutcome::OutcomeUnknown {
                    request_id: upstream.request_id.clone(),
                    custody_id: upstream.request_id.clone(),
                },
            },
        }
    }

    pub(crate) async fn complete_mandatory_review(
        &self,
        request: BankUserNodeMandatoryReviewRequest,
    ) -> BankUserNodeMandatoryReviewOutcome {
        let Some(credential) = self.credential.lock().await.clone() else {
            return review_denied(BankUserNodeDenialKind::NoAuthenticatedSession);
        };
        let upstream = BankHttpMandatoryReviewRequest {
            protocol: BankHttpProtocolVersion::V4,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            idempotency_key: request.idempotency_key,
            mandatory_review: request.mandatory_review,
        };
        match self
            .forward::<_, BankHttpMandatoryReviewOutcome>(
                self.mandatory_review_endpoint.clone(),
                &upstream,
                upstream.controls.deadline_milliseconds,
            )
            .await
        {
            Ok(response) => BankUserNodeMandatoryReviewOutcome::Forwarded { response },
            Err(_) => BankUserNodeMandatoryReviewOutcome::Forwarded {
                response: BankHttpMandatoryReviewOutcome::OutcomeUnknown {
                    request_id: upstream.request_id.clone(),
                    custody_id: upstream.request_id.clone(),
                },
            },
        }
    }
}

fn request_denied(kind: BankUserNodeDenialKind) -> BankUserNodeElevationRequestOutcome {
    BankUserNodeElevationRequestOutcome::Denied {
        denial: super::denial(kind),
    }
}

fn approval_denied(kind: BankUserNodeDenialKind) -> BankUserNodeElevationApprovalOutcome {
    BankUserNodeElevationApprovalOutcome::Denied {
        denial: super::denial(kind),
    }
}

fn revocation_denied(kind: BankUserNodeDenialKind) -> BankUserNodeElevationRevocationOutcome {
    BankUserNodeElevationRevocationOutcome::Denied {
        denial: super::denial(kind),
    }
}

fn review_denied(kind: BankUserNodeDenialKind) -> BankUserNodeMandatoryReviewOutcome {
    BankUserNodeMandatoryReviewOutcome::Denied {
        denial: super::denial(kind),
    }
}
