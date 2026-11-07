use super::world::{declaration_for, preview_closeout_for};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn preview_closeout_subject_certifies_preview_closeout() {
    let declaration = declaration_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let (admission, closeout) = preview_closeout_for(&declaration);
    let subject =
        QuerySubscriptionSupportSubject::preview_closeout(&declaration, &admission, &closeout)
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
        &QuerySubscriptionSupportPosture::RuntimeBackedCertified
    );
}
