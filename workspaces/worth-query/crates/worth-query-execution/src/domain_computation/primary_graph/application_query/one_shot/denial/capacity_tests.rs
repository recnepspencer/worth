//! Spare capacity and the nested boxed payload are part of memory custody.
use super::*;
use worth_execution::ChargedBytes;
#[test]
fn boxed_authorization_and_spare_string_capacity_are_charged() {
    let mut query = String::with_capacity(64);
    query.push('q');
    let mut subject = String::with_capacity(128);
    subject.push('s');
    let mut authorization_subject = String::with_capacity(256);
    authorization_subject.push('a');
    let authorization = WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::PermissionDenied,
        authorization_subject,
    );
    let denial = WorthQueryApplicationOneShotDenial {
        kind: WorthQueryApplicationOneShotDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::PermissionDenied,
        ),
        authorization_denial: Some(Box::new(authorization)),
        query,
        subject,
    };
    let expected = 64
        + 128
        + 256
        + std::mem::size_of::<WorthQueryOperationAuthorizationDenial>()
        + std::mem::size_of::<WorthQueryOperationAuthorizationDenialKind>();
    assert_eq!(
        denial.additional_charged_bytes(),
        expected as u64,
        "the box allocation, nested cause vector and all capacities must be reserved"
    );
}
