//! Output observation retains source interruption and authentication causes.
use super::*;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial as Authorization,
    WorthQueryOperationAuthorizationDenialKind as AuthKind,
    WorthQueryOperationAuthorizationRefusalKind as Refusal,
};
use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;

#[test]
fn entry_custody_output_source_cancellation_and_expiry_keep_their_causes() {
    for (cause, expected) in [
        (AuthKind::Stopped(Err(Control::Cancelled)), Kind::Cancelled),
        (
            AuthKind::Refused(Refusal::ExpiredAuthentication),
            Kind::RequestAuthorization(Refusal::ExpiredAuthentication),
        ),
    ] {
        let source = crate::domain_computation::primary_graph::WorthQuerySourceExpectationDenial::admission_authorization("source", Authorization::new(cause, "operation"));
        let availability =
            WorthQueryOutputAvailabilityDenial::source_query_stop(QueryStop::SourceFacts(source));
        assert_eq!(availability.kind(), expected);
    }
}
