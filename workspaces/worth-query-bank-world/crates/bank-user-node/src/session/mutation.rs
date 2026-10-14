//! Fresh session credentials accompany each owning custody command.
use super::*;
use bank_http_adapter::BankHttpMutationRequest;
impl BankUserSession {
    pub(crate) async fn mutate(
        &self,
        request: BankUserNodeMutationRequest,
    ) -> BankUserNodeMutationOutcome {
        let credential = match self.credential.lock().await.clone() {
            Some(credential) => credential,
            None => return mutation_denied(BankUserNodeDenialKind::NoAuthenticatedSession),
        };
        let credential = BankHttpCredential::from_authentik(&credential);
        let protocol = BankHttpProtocolVersion::V4;
        let (upstream, deadline, request_id, custody_id) = match request {
            BankUserNodeMutationRequest::Create {
                request_id,
                custody_id,
                sequence,
                controls,
                idempotency_key,
                operation,
            } => (
                BankHttpMutationRequest::Create {
                    protocol,
                    request_id: request_id.clone(),
                    credential,
                    custody_id: custody_id.clone(),
                    sequence,
                    controls,
                    idempotency_key,
                    operation,
                },
                controls.deadline_milliseconds,
                request_id,
                custody_id,
            ),
            BankUserNodeMutationRequest::Resume {
                request_id,
                custody_id,
                sequence,
                controls,
            } => (
                BankHttpMutationRequest::Resume {
                    protocol,
                    request_id: request_id.clone(),
                    credential,
                    custody_id: custody_id.clone(),
                    sequence,
                    controls,
                },
                controls.deadline_milliseconds,
                request_id,
                custody_id,
            ),
            BankUserNodeMutationRequest::Read {
                request_id,
                custody_id,
                sequence,
                controls,
            } => (
                BankHttpMutationRequest::Read {
                    protocol,
                    request_id: request_id.clone(),
                    credential,
                    custody_id: custody_id.clone(),
                    sequence,
                    controls,
                },
                controls.deadline_milliseconds,
                request_id,
                custody_id,
            ),
            BankUserNodeMutationRequest::Close {
                request_id,
                custody_id,
                sequence,
                controls,
            } => (
                BankHttpMutationRequest::Close {
                    protocol,
                    request_id: request_id.clone(),
                    credential,
                    custody_id: custody_id.clone(),
                    sequence,
                    controls,
                },
                controls.deadline_milliseconds,
                request_id,
                custody_id,
            ),
        };
        match self
            .forward::<_, BankHttpMutationOutcome>(
                self.mutation_endpoint.clone(),
                &upstream,
                deadline,
            )
            .await
        {
            Ok(response) => BankUserNodeMutationOutcome::Forwarded { response },
            Err(_) => BankUserNodeMutationOutcome::Forwarded {
                response: BankHttpMutationOutcome::OutcomeUnknown {
                    request_id,
                    custody_id,
                },
            },
        }
    }
}
