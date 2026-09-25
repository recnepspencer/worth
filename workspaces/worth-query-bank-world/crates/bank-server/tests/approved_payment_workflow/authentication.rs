use bank_server::{BankApprovalAuthenticationConfiguration, BankApprovalCredential};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::admission::authentication_event::{
    WorthQueryAuthenticationEventChallenge, WorthQueryAuthenticationEventFuture,
    WorthQueryAuthenticationEventVerifier, WorthQueryAuthenticationEventVerifierFailure,
};

const TEST_FACTOR: &[u8] = b"approved-payment-test-factor";

pub(super) fn approval_configuration() -> BankApprovalAuthenticationConfiguration {
    BankApprovalAuthenticationConfiguration::new(ApprovedPaymentTestVerifier)
}

pub(super) fn valid_credential() -> BankApprovalCredential {
    BankApprovalCredential::new(TEST_FACTOR.to_vec())
}

pub(super) fn invalid_credential() -> BankApprovalCredential {
    BankApprovalCredential::new(b"wrong-factor".to_vec())
}

struct ApprovedPaymentTestVerifier;

impl WorthQueryAuthenticationEventVerifier for ApprovedPaymentTestVerifier {
    type Credential = BankApprovalCredential;

    fn configuration_identity(&self) -> &str {
        "bank.approved-payment.test-factor.v1"
    }

    fn verify<'a>(
        &'a self,
        credential: Self::Credential,
        challenge: &'a WorthQueryAuthenticationEventChallenge,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationEventFuture<'a> {
        Box::pin(async move {
            if credential.as_bytes() != TEST_FACTOR
                || challenge.intent().purpose() != "workflow-approval-signature"
                || *challenge.nonce() == [0; 32]
            {
                return Err(WorthQueryAuthenticationEventVerifierFailure::CredentialRejected);
            }
            Ok(())
        })
    }
}
