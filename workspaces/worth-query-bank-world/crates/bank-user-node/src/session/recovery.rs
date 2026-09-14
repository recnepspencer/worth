use bank_http_adapter::{
    BankHttpCredential, BankHttpEstateNotificationRequest, BankHttpProtocolVersion,
    BankHttpRecoveryRequest,
};

use crate::protocol::{
    BankUserNodeDenialKind, BankUserNodeEstateNotificationOutcome,
    BankUserNodeEstateNotificationRequest, BankUserNodeRecoveryInspectionOutcome,
    BankUserNodeRecoveryRequest,
};

use super::{denial, BankUserSession};

impl BankUserSession {
    pub(crate) async fn notify_estate_death(
        &self,
        request: BankUserNodeEstateNotificationRequest,
    ) -> BankUserNodeEstateNotificationOutcome {
        let credential = match self.credential.lock().await.clone() {
            Some(credential) => credential,
            None => return notification_denied(BankUserNodeDenialKind::NoAuthenticatedSession),
        };
        let upstream = BankHttpEstateNotificationRequest {
            protocol: BankHttpProtocolVersion::V1,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            idempotency_key: request.idempotency_key,
            estate: request.estate,
            notice: request.notice,
            subject: request.subject,
        };
        match self
            .forward(
                self.estate_notification_endpoint.clone(),
                &upstream,
                upstream.controls.deadline_milliseconds,
            )
            .await
        {
            Ok(response) => BankUserNodeEstateNotificationOutcome::Forwarded { response },
            Err(kind) => notification_denied(kind),
        }
    }

    pub(crate) async fn inspect_recovery(
        &self,
        request: BankUserNodeRecoveryRequest,
    ) -> BankUserNodeRecoveryInspectionOutcome {
        let upstream = match self.recovery_request(request).await {
            Ok(upstream) => upstream,
            Err(kind) => return inspection_denied(kind),
        };
        match self
            .forward(
                self.recovery_inspection_endpoint.clone(),
                &upstream,
                upstream.controls.deadline_milliseconds,
            )
            .await
        {
            Ok(response) => BankUserNodeRecoveryInspectionOutcome::Forwarded { response },
            Err(kind) => inspection_denied(kind),
        }
    }

    async fn recovery_request(
        &self,
        request: BankUserNodeRecoveryRequest,
    ) -> Result<BankHttpRecoveryRequest, BankUserNodeDenialKind> {
        let credential = self
            .credential
            .lock()
            .await
            .clone()
            .ok_or(BankUserNodeDenialKind::NoAuthenticatedSession)?;
        Ok(BankHttpRecoveryRequest {
            protocol: BankHttpProtocolVersion::V1,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            recovery: request.recovery,
        })
    }
}

fn notification_denied(kind: BankUserNodeDenialKind) -> BankUserNodeEstateNotificationOutcome {
    BankUserNodeEstateNotificationOutcome::Denied {
        denial: denial(kind),
    }
}

fn inspection_denied(kind: BankUserNodeDenialKind) -> BankUserNodeRecoveryInspectionOutcome {
    BankUserNodeRecoveryInspectionOutcome::Denied {
        denial: denial(kind),
    }
}
