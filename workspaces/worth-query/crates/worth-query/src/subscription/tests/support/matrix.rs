use super::world::{admission_for, declaration_for};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn support_matrix_is_family_aware_for_all_admitted_subscription_families() {
    for (live_family, view_family, expected_family) in [
        (
            LiveQueryFamily::Detail,
            None,
            QuerySubscriptionFamily::DetailExact,
        ),
        (
            LiveQueryFamily::Detail,
            Some(LiveViewShapeFamily::InspectorDetailFocused),
            QuerySubscriptionFamily::InspectorDetailExact,
        ),
        (
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::Table),
            QuerySubscriptionFamily::CollectionMembership,
        ),
        (
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::KanbanGrouped),
            QuerySubscriptionFamily::GroupedCollectionMembership,
        ),
        (
            LiveQueryFamily::BoundedMaterialization,
            None,
            QuerySubscriptionFamily::BoundedMaterialization,
        ),
    ] {
        let declaration = declaration_for(live_family, view_family);
        let subject = QuerySubscriptionSupportSubject::declaration(&declaration)
            .expect("fixture digest must fit");
        let evidence = QuerySubscriptionSupportEvidence::declaration(&declaration);

        let (report, lookup_receipt) =
            report_query_subscription_support(subject, evidence).unwrap();

        assert_eq!(report.support_subject().family(), &expected_family);
        assert_eq!(report.support_matrix().family(), &expected_family);
        assert_eq!(
            report.support_posture(),
            &QuerySubscriptionSupportPosture::RuntimeBackedCertified
        );
        assert!(!report
            .support_matrix()
            .capability_projection()
            .label()
            .is_empty());
        assert_eq!(
            report
                .support_matrix()
                .row_for_class(QuerySubscriptionSupportClass::Declaration)
                .unwrap()
                .posture(),
            &QuerySubscriptionSupportPosture::RuntimeBackedCertified
        );
        assert_eq!(
            report
                .support_matrix()
                .row_for_class(QuerySubscriptionSupportClass::Activation)
                .unwrap()
                .posture(),
            &QuerySubscriptionSupportPosture::UncertifiedDenied
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
        assert_eq!(
            report
                .support_matrix()
                .row_for_class(QuerySubscriptionSupportClass::DurableReplay)
                .unwrap()
                .posture(),
            &QuerySubscriptionSupportPosture::RuntimeBackedDeferred
        );
        assert_eq!(
            report
                .support_matrix()
                .row_for_class(QuerySubscriptionSupportClass::StoreBackedRestart)
                .unwrap()
                .posture(),
            &QuerySubscriptionSupportPosture::RuntimeBackedDeferred
        );
        assert_eq!(
            lookup_receipt.resolution_posture(),
            &SupportResolutionPosture::IndexedFamilyLookup
        );
        assert_eq!(lookup_receipt.consumed_lookup_width(), 1);
        assert_eq!(lookup_receipt.remaining_lookup_width(), 6);
        assert_eq!(report.counters().support_report_request_count(), 1);
        assert_eq!(report.counters().supported_family_count(), 1);
        assert_eq!(report.counters().deferred_family_count(), 0);
        assert_eq!(report.counters().denied_family_count(), 0);
        assert_eq!(report.counters().uncertified_family_denial_count(), 0);
        assert_eq!(report.counters().support_matrix_emission_count(), 1);
        assert_eq!(report.counters().support_family_index_lookup_count(), 1);
        assert_eq!(report.counters().support_matrix_scan_debt_count(), 0);
        assert_eq!(
            report.lookup_receipt_projection().label(),
            lookup_receipt.lookup_receipt_projection().label()
        );
        assert!(!report.report_projection().label().is_empty());
        assert!(!report.counter_snapshot_projection().label().is_empty());
    }
}

#[test]
fn durable_and_store_backed_subjects_remain_explicitly_deferred() {
    let declaration = declaration_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let admission = admission_for(&declaration);
    let admission_evidence =
        QuerySubscriptionSupportEvidence::admission(&declaration, &admission).unwrap();

    for subject in [
        QuerySubscriptionSupportSubject::durable_replay(&declaration),
        QuerySubscriptionSupportSubject::store_backed_restart(&declaration),
    ] {
        let (report, lookup_receipt) = report_query_subscription_support(
            subject.expect("fixture digest must fit"),
            admission_evidence.clone(),
        )
        .unwrap();

        assert_eq!(
            report.support_posture(),
            &QuerySubscriptionSupportPosture::RuntimeBackedDeferred
        );
        assert_eq!(report.counters().supported_family_count(), 0);
        assert_eq!(report.counters().deferred_family_count(), 1);
        assert_eq!(report.counters().denied_family_count(), 0);
        assert_eq!(
            lookup_receipt.resolution_posture(),
            &SupportResolutionPosture::IndexedFamilyLookup
        );
    }
}
