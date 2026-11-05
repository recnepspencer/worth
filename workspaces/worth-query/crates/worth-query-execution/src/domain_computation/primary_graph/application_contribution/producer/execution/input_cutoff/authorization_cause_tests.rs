//! Producer source refusal classification preserves interruption and authorization.
use super::*;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial as Authorization,
    WorthQueryOperationAuthorizationDenialKind as AuthKind,
    WorthQueryOperationAuthorizationRefusalKind as Refusal,
};
use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;

#[test]
fn entry_custody_producer_source_cancellation_and_expiry_keep_their_causes() {
    for (cause, expected) in [
        (
            AuthKind::Stopped(Err(Control::Cancelled)),
            WorthQueryOutputDemandDenialKind::Cancelled,
        ),
        (
            AuthKind::Refused(Refusal::ExpiredAuthentication),
            WorthQueryOutputDemandDenialKind::RequestAuthorization(Refusal::ExpiredAuthentication),
        ),
    ] {
        let source = crate::domain_computation::primary_graph::WorthQuerySourceExpectationDenial::admission_authorization("source", Authorization::new(cause, "operation"));
        assert_eq!(
            source_preparation_denial("producer", source).kind(),
            expected
        );
    }
}
