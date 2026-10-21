use bank_domain::estate::{
    EstateWorkflowStage, LegalAuthorityKind, MandatoryReviewKind, MandatoryReviewStatus,
};
use worth_query_host::facade::publication::domain_computation::{
    WorthQueryPublishedApplicationDisclosurePosture,
    WorthQueryPublishedApplicationQueryOmissionPosture, WorthQueryPublishedApplicationResult,
};

use super::fixture::{
    capability_world, product_projection_world, request_scope, GrantSpec, AUTHORITY,
    COMPLETED_REVIEW, ESTATE, REQUESTED_REVIEW,
};
use crate::{queries, BankApplicationQueryDenial, BankReadControls};

#[test]
fn public_legal_compliance_query_consumes_legal_compliance_capability() {
    let fixture = product_projection_world(
        "legal-compliance-product-query",
        GrantSpec::legal_compliance_view(),
    );
    let principal = fixture.authenticate_reviewer();
    let result = fixture
        .runtime
        .query(queries::estate_legal_compliance(ESTATE))
        .as_principal(&principal)
        .controls(controls())
        .execute()
        .expect("legal-compliance authority should open the product query");

    assert_eq!(result.rows().len(), 1);
    let product = &result.rows()[0];
    assert_eq!(product.estate(), ESTATE);
    assert_eq!(product.authorities().len(), 2);
    let authority = product
        .authorities()
        .iter()
        .find(|authority| authority.id() == AUTHORITY)
        .expect("the selected estate authority should be projected");
    assert_eq!(authority.kind(), LegalAuthorityKind::CourtAppointment);
    assert!(!authority.recognized());
    assert_governed_receipt(&result);
}

#[test]
fn public_mandatory_review_query_consumes_mandatory_review_capability() {
    let fixture = product_projection_world(
        "mandatory-review-product-query",
        GrantSpec::mandatory_review_view(),
    );
    let principal = fixture.authenticate_reviewer();
    let result = fixture
        .runtime
        .query(queries::estate_mandatory_reviews(ESTATE))
        .as_principal(&principal)
        .controls(controls())
        .execute()
        .expect("mandatory-review authority should open the product query");

    assert_eq!(result.rows().len(), 1);
    let product = &result.rows()[0];
    assert_eq!(product.estate(), ESTATE);
    assert_eq!(product.reviews().len(), 2);
    let required = product
        .reviews()
        .iter()
        .find(|review| review.id() == REQUESTED_REVIEW)
        .expect("the required review should be projected");
    assert_eq!(required.kind(), MandatoryReviewKind::EmergencyAccess);
    assert_eq!(required.status(), MandatoryReviewStatus::Required);
    assert_eq!(required.reviewer(), None);
    let completed = product
        .reviews()
        .iter()
        .find(|review| review.id() == COMPLETED_REVIEW)
        .expect("the completed review should be projected");
    assert_eq!(completed.status(), MandatoryReviewStatus::Completed);
    assert_eq!(completed.kind(), MandatoryReviewKind::EstateRelease);
    assert_governed_receipt(&result);
}

fn assert_governed_receipt<Query, QueryResult>(
    result: &WorthQueryPublishedApplicationResult<Query, QueryResult>,
) {
    let receipt = result.receipt();
    let inspection = receipt.inspect();
    assert_eq!(
        inspection.omission_posture(),
        WorthQueryPublishedApplicationQueryOmissionPosture::NoOmission
    );
    assert_eq!(
        receipt.disclosure().posture(),
        WorthQueryPublishedApplicationDisclosurePosture::Governed
    );
    assert!(receipt.disclosure().disclosure_decision_count() > 0);
    assert_eq!(receipt.disclosure().omitted_value_count(), 0);
    assert!(inspection.terminal_resources_released());
}

#[test]
fn lower_product_queries_reject_an_administration_grant() {
    let fixture = capability_world(
        "lower-product-query-purpose-denial",
        GrantSpec::governance_view(),
        EstateWorkflowStage::Administration,
    );
    let principal = fixture.authenticate();
    for denial in [
        fixture
            .runtime
            .query(queries::estate_legal_compliance(ESTATE))
            .as_principal(&principal)
            .controls(controls())
            .execute()
            .err(),
        fixture
            .runtime
            .query(queries::estate_mandatory_reviews(ESTATE))
            .as_principal(&principal)
            .controls(controls())
            .execute()
            .err(),
    ] {
        assert!(matches!(
            denial,
            Some(BankApplicationQueryDenial::CapabilityAdmission(_))
        ));
    }
}

fn controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 1).unwrap()
}
