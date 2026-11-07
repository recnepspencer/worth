use super::world::{admission_for, declaration_for};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn active_lifecycle_subject_reports_certified_support_with_indexed_lookup_receipt() {
    let declaration = declaration_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::KanbanGrouped),
    );
    let admission = admission_for(&declaration);
    let activation = prepare_subscription_activation(admission.clone());
    let active_admission = admit_active_subscription_lane(
        activation,
        ActiveSubscriptionWorkBudget::admitted(
            ActiveRegistryLookupWidth::measured(1),
            ActiveFanoutWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPolicy::LifecycleArena,
        ),
    )
    .unwrap();
    let subject = QuerySubscriptionSupportSubject::active_lifecycle(
        &declaration,
        &admission,
        &active_admission,
    )
    .expect("fixture digest must fit");
    let evidence = QuerySubscriptionSupportEvidence::admission(&declaration, &admission).unwrap();

    let (report, lookup_receipt) = report_query_subscription_support(subject, evidence).unwrap();

    assert_eq!(
        report.support_subject().support_class(),
        &QuerySubscriptionSupportClass::ActiveLifecycle
    );
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
        &QuerySubscriptionSupportPosture::UncertifiedDenied
    );
    assert_eq!(
        report
            .support_matrix()
            .row_for_class(QuerySubscriptionSupportClass::PreviewCloseout)
            .unwrap()
            .posture(),
        &QuerySubscriptionSupportPosture::UncertifiedDenied
    );
    assert_eq!(
        lookup_receipt.resolution_posture(),
        &SupportResolutionPosture::IndexedFamilyLookup
    );
    assert_eq!(report.counters().supported_family_count(), 1);
}

#[test]
fn activation_subject_only_certifies_activation_and_keeps_later_runtime_rows_uncertified() {
    let declaration = declaration_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let admission = admission_for(&declaration);
    let activation = prepare_subscription_activation(admission.clone());
    let subject = QuerySubscriptionSupportSubject::activation(&declaration, &activation)
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
        &QuerySubscriptionSupportPosture::UncertifiedDenied
    );
    assert_eq!(
        report
            .support_matrix()
            .row_for_class(QuerySubscriptionSupportClass::Continuation)
            .unwrap()
            .posture(),
        &QuerySubscriptionSupportPosture::UncertifiedDenied
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
