use super::*;
#[test]
fn subscription_capacity_preserves_admission_kind_and_query_subject() {
    use super::super::super::super::WorthQueryApplicationQueryAdmissionDenialKind as Admission;
    let denial = subscription_capacity_denial("live-query");
    assert!(matches!(
        denial.kind(),
        WorthQueryApplicationLiveOpenDenialKind::Admission(Admission::RetentionCapacityExhausted)
    ));
    assert_eq!(denial.subject(), "live-query");
}
