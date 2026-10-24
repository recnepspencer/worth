//! Session credentials accompany each explicit history request; no native custody lives here.
use super::*;
use crate::protocol::{
    BankUserNodeCorrectionHistoryOutcome, BankUserNodeCorrectionHistoryRequest,
    BankUserNodeCorrectionHistoryResumeRequest,
};
use bank_http_adapter::{
    BankHttpCorrectionHistoryOutcome, BankHttpCorrectionHistoryRequest,
    BankHttpCorrectionHistoryResumeRequest,
};

impl BankUserSession {
    pub(crate) async fn correction_history(
        &self,
        request: BankUserNodeCorrectionHistoryRequest,
    ) -> BankUserNodeCorrectionHistoryOutcome {
        let Some(credential) = self.credential.lock().await.clone() else {
            return refused(BankUserNodeDenialKind::NoAuthenticatedSession);
        };
        let upstream = BankHttpCorrectionHistoryRequest {
            protocol: BankHttpProtocolVersion::V4,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            source: request.source,
        };
        match self
            .forward::<_, BankHttpCorrectionHistoryOutcome>(
                self.correction_history_endpoint.clone(),
                &upstream,
                upstream.controls.deadline_milliseconds,
            )
            .await
        {
            Ok(response) => BankUserNodeCorrectionHistoryOutcome::Forwarded { response },
            Err(kind) => refused(kind),
        }
    }
    pub(crate) async fn resume_correction_history(
        &self,
        request: BankUserNodeCorrectionHistoryResumeRequest,
    ) -> BankUserNodeCorrectionHistoryOutcome {
        let Some(credential) = self.credential.lock().await.clone() else {
            return refused(BankUserNodeDenialKind::NoAuthenticatedSession);
        };
        let upstream = BankHttpCorrectionHistoryResumeRequest {
            protocol: BankHttpProtocolVersion::V4,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            source: request.source,
            continuation: request.continuation,
        };
        match self
            .forward::<_, BankHttpCorrectionHistoryOutcome>(
                self.correction_history_resume_endpoint.clone(),
                &upstream,
                upstream.controls.deadline_milliseconds,
            )
            .await
        {
            Ok(response) => BankUserNodeCorrectionHistoryOutcome::Forwarded { response },
            Err(kind) => refused(kind),
        }
    }
}
fn refused(kind: BankUserNodeDenialKind) -> BankUserNodeCorrectionHistoryOutcome {
    BankUserNodeCorrectionHistoryOutcome::Denied {
        denial: denial(kind),
    }
}
