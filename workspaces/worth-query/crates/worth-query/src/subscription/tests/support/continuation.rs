use super::world::{continuation_report_for, declaration_for};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn continuation_subject_only_certifies_through_continuation() {
    let declaration = declaration_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let (admission, continuation) = continuation_report_for(&declaration);
    let subject =
        QuerySubscriptionSupportSubject::continuation(&declaration, &admission, &continuation)
            .expect("fixture digest must fit");

    let (report, _) = report_query_subscription_support(
        subject,
        QuerySubscriptionSupportEvidence::admission(&declaration, &admission).unwrap(),
    )
    .unwrap();

    assert_eq!(
        report.support_posture(),
        &QuerySubscriptionSupportPosture::RuntimeBackedCertified
    );
    assert_eq!(
        report
            .support_matrix()
            .row_for_class(QuerySubscriptionSupportClass::Activation)
            .unwrap()
            .posture(),
        &QuerySubscriptionSupportPosture::RuntimeBackedCertified
    );
    assert_eq!(
        report
            .support_matrix()
            .row_for_class(QuerySubscriptionSupportClass::ActiveLifecycle)
            .unwrap()
            .posture(),
        &QuerySubscriptionSupportPosture::RuntimeBackedCertified
    );
    assert_eq!(
        report
            .support_matrix()
            .row_for_class(QuerySubscriptionSupportClass::Continuation)
            .unwrap()
            .posture(),
        &QuerySubscriptionSupportPosture::RuntimeBackedCertified
    );
    assert_eq!(
        report
            .support_matrix()
            .row_for_class(QuerySubscriptionSupportClass::PreviewCloseout)
            .unwrap()
            .posture(),
        &QuerySubscriptionSupportPosture::UncertifiedDenied
    );
}
