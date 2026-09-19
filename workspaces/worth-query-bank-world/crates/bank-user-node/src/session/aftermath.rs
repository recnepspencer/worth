use bank_http_adapter::{
    BankHttpCredential, BankHttpEstateDisbursementRequest, BankHttpProtocolVersion,
};

use crate::protocol::{
    BankUserNodeDenialKind, BankUserNodeEstateDisbursementOutcome,
    BankUserNodeEstateDisbursementRequest,
};

use super::{denial, BankUserSession};

impl BankUserSession {
    pub(crate) async fn disburse_estate(
        &self,
        request: BankUserNodeEstateDisbursementRequest,
    ) -> BankUserNodeEstateDisbursementOutcome {
        let credential = match self.credential.lock().await.clone() {
            Some(credential) => credential,
            None => return disbursement_denied(BankUserNodeDenialKind::NoAuthenticatedSession),
        };
        let upstream = BankHttpEstateDisbursementRequest {
            protocol: BankHttpProtocolVersion::V1,
            request_id: request.request_id,
            credential: BankHttpCredential::from_authentik(&credential),
            controls: request.controls,
            idempotency_key: request.idempotency_key,
            estate: request.estate,
            source_account: request.source_account,
            destination_account: request.destination_account,
            beneficiary: request.beneficiary,
            amount_minor_units: request.amount_minor_units,
        };
        match self
            .forward(
                self.estate_disbursement_endpoint.clone(),
                &upstream,
                upstream.controls.deadline_milliseconds,
            )
            .await
        {
            Ok(response) => BankUserNodeEstateDisbursementOutcome::Forwarded { response },
            Err(kind) => disbursement_denied(kind),
        }
    }
}

fn disbursement_denied(kind: BankUserNodeDenialKind) -> BankUserNodeEstateDisbursementOutcome {
    BankUserNodeEstateDisbursementOutcome::Denied {
        denial: denial(kind),
    }
}
