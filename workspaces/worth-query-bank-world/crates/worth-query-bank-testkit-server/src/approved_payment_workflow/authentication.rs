use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

pub use bank_server::{BankApprovalAuthenticationConfiguration, BankApprovalCredential};
pub use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
pub use worth_query_host::facade::admission::authentication_event::{
    WorthQueryAuthenticationEventChallenge, WorthQueryAuthenticationEventFuture,
    WorthQueryAuthenticationEventVerifier, WorthQueryAuthenticationEventVerifierFailure,
};

pub const TEST_FACTOR: &[u8] = b"approved-payment-test-factor";

pub fn approval_configuration() -> BankApprovalAuthenticationConfiguration {
    BankApprovalAuthenticationConfiguration::new(ApprovedPaymentTestVerifier)
}

pub fn valid_credential() -> BankApprovalCredential {
    BankApprovalCredential::new(TEST_FACTOR.to_vec())
}

pub fn invalid_credential() -> BankApprovalCredential {
    BankApprovalCredential::new(b"wrong-factor".to_vec())
}

pub struct ApprovedPaymentTestVerifier;

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

/// Completes one workflow approval on the calling thread.
///
/// Step-up approval verification is asynchronous by contract: the Host's
/// authentication-event verifier has no local, executor-free mode. The
/// in-process test verifier is always ready, so polling it completes it.
pub fn complete_approval<F: Future>(approval: F) -> F::Output {
    let mut approval = pin!(approval);
    let mut context = Context::from_waker(Waker::noop());
    loop {
        match approval.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
