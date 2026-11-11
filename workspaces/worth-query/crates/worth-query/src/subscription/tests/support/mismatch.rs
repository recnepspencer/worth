use super::world::{admission_for, declaration_for};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn support_report_denies_mismatched_declaration_subject_sources() {
    let detail_declaration = declaration_for(LiveQueryFamily::Detail, None);
    let grouped_declaration = declaration_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::KanbanGrouped),
    );
    let detail_admission = admission_for(&detail_declaration);
    let mismatched_subject = QuerySubscriptionSupportSubject::declaration(&grouped_declaration)
        .expect("fixture digest must fit");

    let error = report_query_subscription_support(
        mismatched_subject,
        QuerySubscriptionSupportEvidence::admission(&detail_declaration, &detail_admission)
            .unwrap(),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionSupportReportDenialKind::DeclarationSourceMismatch
    );
    assert!(error.message().contains("same declaration artifact"));
    assert!(!error
        .failure_projection()
        .expect("fixture digest must fit")
        .label()
        .is_empty());
}

#[test]
fn support_report_denies_mismatched_admission_subject_sources() {
    let table_declaration = declaration_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let table_admission = admission_for(&table_declaration);
    let alternate_lowering =
        lower_query_subscription_to_bridge(table_declaration.clone(), roomy_lowering_budget())
            .unwrap();
    let alternate_admission = admit_query_subscription(
        alternate_lowering,
        QuerySubscriptionAdmissionBudget::admitted(2, 1, 1, 1, 1),
    )
    .unwrap();
    let activation = prepare_subscription_activation(alternate_admission);
    let subject = QuerySubscriptionSupportSubject::activation(&table_declaration, &activation)
        .expect("fixture digest must fit");

    let error = report_query_subscription_support(
        subject,
        QuerySubscriptionSupportEvidence::admission(&table_declaration, &table_admission).unwrap(),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionSupportReportDenialKind::AdmissionSourceMismatch
    );
    assert!(error.message().contains("same admission artifact"));
    assert!(!error
        .failure_projection()
        .expect("fixture digest must fit")
        .label()
        .is_empty());
}

#[test]
fn support_evidence_rejects_mismatched_declaration_and_admission_sources_early() {
    let detail_declaration = declaration_for(LiveQueryFamily::Detail, None);
    let table_declaration = declaration_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let table_admission = admission_for(&table_declaration);

    let error = QuerySubscriptionSupportEvidence::admission(&detail_declaration, &table_admission)
        .unwrap_err();

    assert!(error
        .message()
        .contains("same canonical query subscription family"));
    assert!(!error
        .failure_projection()
        .expect("fixture digest must fit")
        .label()
        .is_empty());
}

#[test]
fn declaration_support_can_be_reported_without_admission_evidence() {
    let declaration = declaration_for(LiveQueryFamily::Detail, None);
    let subject = QuerySubscriptionSupportSubject::declaration(&declaration)
        .expect("fixture digest must fit");

    let (report, lookup_receipt) = report_query_subscription_support(
        subject,
        QuerySubscriptionSupportEvidence::declaration(&declaration),
    )
    .unwrap();

    assert_eq!(
        report.support_subject().support_class(),
        &QuerySubscriptionSupportClass::Declaration
    );
    assert_eq!(
        report.support_posture(),
        &QuerySubscriptionSupportPosture::RuntimeBackedCertified
    );
    assert_eq!(
        lookup_receipt.resolution_posture(),
        &SupportResolutionPosture::IndexedFamilyLookup
    );
    assert_eq!(
        report
            .support_matrix()
            .row_for_class(QuerySubscriptionSupportClass::Activation)
            .unwrap()
            .posture(),
        &QuerySubscriptionSupportPosture::UncertifiedDenied
    );
}

#[test]
fn activation_support_denies_when_only_declaration_evidence_is_available() {
    let declaration = declaration_for(LiveQueryFamily::Detail, None);
    let admission = admission_for(&declaration);
    let activation = prepare_subscription_activation(admission);
    let subject = QuerySubscriptionSupportSubject::activation(&declaration, &activation)
        .expect("fixture digest must fit");

    let error = report_query_subscription_support(
        subject,
        QuerySubscriptionSupportEvidence::declaration(&declaration),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionSupportReportDenialKind::AdmissionEvidenceRequired
    );
    assert!(error.message().contains("requires admission evidence"));
    assert!(!error
        .failure_projection()
        .expect("fixture digest must fit")
        .label()
        .is_empty());
}
