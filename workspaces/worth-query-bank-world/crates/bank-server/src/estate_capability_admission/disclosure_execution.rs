use bank_domain::estate::{BankDisclosure, EstateWorkflowStage, RestrictedBankField};
use worth_query_host::facade::publication::domain_computation::{
    publish_application_field_omission, WorthQueryApplicationAuthorizationPublicationProfile,
    WorthQueryApplicationQueryPublicationInspection, WorthQueryPublishedApplicationDisclosure,
    WorthQueryPublishedApplicationDisclosurePosture,
    WorthQueryPublishedApplicationQueryOmissionPosture,
};

use super::fixture::{
    capability_world, governed_disclosure_world, request_scope, GrantSpec, APPROVER, DECEASED,
    ESTATE, EXECUTOR, SPECIALIST,
};
use super::publication_evidence::{
    assert_field_omission_publication, assert_omission_noninterference, publication_profile,
};
use crate::{queries, BankApplicationQueryDenial, BankReadControls};

#[test]
fn present_protected_beneficiary_value_cannot_influence_public_omission_material() {
    let first = governed_disclosure_world("governed-disclosure-hidden-twin", APPROVER);
    let second = governed_disclosure_world("governed-disclosure-hidden-twin", EXECUTOR);
    assert_eq!(first.source_beneficiaries(), vec![APPROVER]);
    assert_eq!(second.source_beneficiaries(), vec![EXECUTOR]);
    assert_ne!(first.source_beneficiaries(), second.source_beneficiaries());

    let first_principal = first.authenticate();
    let second_principal = second.authenticate();
    let first_result = first
        .runtime
        .query(queries::estate_customer_identity(ESTATE))
        .as_principal(&first_principal)
        .controls(BankReadControls::current(request_scope(), 1).unwrap())
        .execute()
        .unwrap();
    let second_result = second
        .runtime
        .query(queries::estate_customer_identity(ESTATE))
        .as_principal(&second_principal)
        .controls(BankReadControls::current(request_scope(), 1).unwrap())
        .execute()
        .unwrap();

    assert_eq!(first_result.rows(), second_result.rows());
    let first_inspection = first_result.receipt().inspect();
    let second_inspection = second_result.receipt().inspect();
    assert_ne!(
        first_inspection.basis().runtime_instance(),
        second_inspection.basis().runtime_instance(),
        "independent test worlds must retain distinct runtime basis"
    );
    assert_eq!(
        first_inspection.query_identity(),
        second_inspection.query_identity()
    );
    assert_eq!(
        first_inspection.parameter_binding_identity(),
        second_inspection.parameter_binding_identity()
    );
    assert_eq!(
        first_inspection.result_count(),
        second_inspection.result_count()
    );
    assert_eq!(
        first_inspection.ordinary_work_units(),
        second_inspection.ordinary_work_units()
    );
    assert_eq!(
        first_inspection.terminal_release(),
        second_inspection.terminal_release()
    );
    assert_eq!(
        first_inspection.basis().branch(),
        second_inspection.basis().branch()
    );
    assert_eq!(
        first_inspection.basis().snapshot(),
        second_inspection.basis().snapshot()
    );
    assert_eq!(
        first_inspection.basis().version(),
        second_inspection.basis().version()
    );
    assert_eq!(
        first_inspection.basis().posture(),
        second_inspection.basis().posture()
    );
    assert_eq!(
        first_result.receipt().disclosure(),
        second_result.receipt().disclosure()
    );
    let profile = publication_profile();
    let first_publication = publish_application_field_omission(
        first_result.receipt().disclosure(),
        WorthQueryApplicationAuthorizationPublicationProfile::exact(profile),
    )
    .unwrap();
    let second_publication = publish_application_field_omission(
        second_result.receipt().disclosure(),
        WorthQueryApplicationAuthorizationPublicationProfile::exact(profile),
    )
    .unwrap();
    assert_omission_noninterference(&first_publication, &second_publication, profile);
}

#[test]
fn authenticated_bank_consumer_receives_only_the_governed_published_shape() {
    let fixture = capability_world(
        "governed-disclosure",
        GrantSpec::identity_verification(),
        EstateWorkflowStage::Administration,
    );
    let principal = fixture.authenticate();
    let controls = BankReadControls::current(request_scope(), 1).unwrap();

    let published = fixture
        .runtime
        .query(queries::estate_customer_identity(ESTATE))
        .as_principal(&principal)
        .controls(controls)
        .execute()
        .unwrap();

    assert_eq!(published.rows().len(), 1);
    assert_eq!(
        published.rows()[0].customer(),
        BankDisclosure::Disclosed(DECEASED)
    );
    assert_eq!(
        published.rows()[0].beneficiaries(),
        &BankDisclosure::Omitted(RestrictedBankField::BeneficiaryIdentity.classification())
    );
    assert_query_publication(published.receipt().inspect(), published.rows().len());
    assert_governed_disclosure(published.receipt().disclosure());
    assert_field_omission_publication(published.receipt().disclosure());
}

#[test]
fn decision_relevant_protected_beneficiary_changes_the_public_outcome() {
    let admitted = governed_disclosure_world("governed-disclosure-decision-twin", APPROVER);
    let conflicted = governed_disclosure_world("governed-disclosure-decision-twin", SPECIALIST);
    assert_eq!(admitted.source_beneficiaries(), vec![APPROVER]);
    assert_eq!(conflicted.source_beneficiaries(), vec![SPECIALIST]);

    let admitted_result = admitted
        .runtime
        .query(queries::estate_customer_identity(ESTATE))
        .as_principal(&admitted.authenticate())
        .controls(BankReadControls::current(request_scope(), 1).unwrap())
        .execute()
        .expect("an unrelated beneficiary permits governed omission");
    let profile = publication_profile();
    let omission = publish_application_field_omission(
        admitted_result.receipt().disclosure(),
        WorthQueryApplicationAuthorizationPublicationProfile::exact(profile),
    )
    .expect("the admitted query has a governed omission");

    let conflict = match conflicted
        .runtime
        .query(queries::estate_customer_identity(ESTATE))
        .as_principal(&conflicted.authenticate())
        .controls(BankReadControls::current(request_scope(), 1).unwrap())
        .execute()
    {
        Err(BankApplicationQueryDenial::CapabilityAdmission(conflict)) => conflict,
        Err(other) => panic!("the decision-relevant twin denied at the wrong boundary: {other:?}"),
        Ok(_) => panic!("the requesting specialist cannot also be a beneficiary"),
    };
    assert_eq!(
        conflict.kind(),
        crate::BankAuthorizationDenialKind::ConflictRuleMatched
    );
    assert!(conflict.contributing_cause_count() > 0);

    let _omission_identity = omission
        .provenance()
        .source_basis()
        .boundary_artifact_locator()
        .expect("omission publication has a semantic source identity");
}

fn assert_governed_disclosure(disclosure: &WorthQueryPublishedApplicationDisclosure) {
    assert_eq!(
        disclosure.posture(),
        WorthQueryPublishedApplicationDisclosurePosture::Governed
    );
    assert_eq!(disclosure.disclosure_decision_count(), 4);
    assert_eq!(disclosure.disclosed_value_count(), 2);
    assert_eq!(disclosure.omitted_value_count(), 2);
    assert!(disclosure.authorization_decision_fact_count() > 0);
}

fn assert_query_publication(
    publication: WorthQueryApplicationQueryPublicationInspection<'_>,
    result_count: usize,
) {
    assert_eq!(
        publication.omission_posture(),
        WorthQueryPublishedApplicationQueryOmissionPosture::GovernedOmission
    );
    assert_eq!(publication.result_count(), result_count);
    assert!(publication.terminal_resources_released());
    assert_eq!(publication.publication_canonical_entries(), 0);
    assert_eq!(publication.publication_sha256_compression_blocks(), 0);
    assert_eq!(publication.publication_identity_text_materializations(), 0);
}

#[test]
fn bank_consumer_cannot_repurpose_an_administration_grant_for_identity_disclosure() {
    let fixture = capability_world(
        "wrong-purpose-disclosure",
        GrantSpec::view(),
        EstateWorkflowStage::Administration,
    );
    let principal = fixture.authenticate();
    let controls = BankReadControls::current(request_scope(), 1).unwrap();

    let denial = match fixture
        .runtime
        .query(queries::estate_customer_identity(ESTATE))
        .as_principal(&principal)
        .controls(controls)
        .execute()
    {
        Ok(_) => panic!("wrong-purpose grant must not publish identity disclosure"),
        Err(denial) => denial,
    };

    let BankApplicationQueryDenial::CapabilityAdmission(denial) = denial else {
        panic!("wrong-purpose grant must fail at capability admission")
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::CapabilityAuthorizationMissing
    );
    assert!(denial.contributing_cause_count() > 0);
}
