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
        payload: std::sync::Arc::new(OneShotDenialPayload {
            authorization_denial: Some(Box::new(authorization)),
            query,
            subject,
            custody: std::sync::OnceLock::new(),
        }),
    };
    // One shared Arc payload, two reference counters, and the original owned allocations.
    let expected = std::mem::size_of::<OneShotDenialPayload>()
        + 2 * std::mem::size_of::<usize>()
        + 64
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

#[test]
fn denial_clones_share_payload_and_keep_its_reservation_until_last_drop() {
    let denial = denial(
        WorthQueryApplicationOneShotDenialKind::WorkLimitExceeded,
        "query",
        "subject",
    );
    let bytes = denial.additional_charged_bytes();
    let budget = worth_execution::SerialMemoryBudget::new(bytes);
    denial.retain_custody(budget.reserve(bytes).unwrap());
    let clone = denial.clone();
    assert_eq!(denial, clone);
    drop(denial);
    assert_eq!(budget.reserve(1).unwrap_err().admitted, 0);
    drop(clone);
    assert!(budget.reserve(bytes).is_ok());
}
