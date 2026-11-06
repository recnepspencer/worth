use crate::basis_lifecycle::basis_lifecycle;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::intent_admission::{
    worth_query_basis_observation_intent, worth_query_projection_consumption_intent,
    WorthQueryAdmittedIntentPlan, WorthQueryBasisObservationPlan,
    WorthQueryIntentAdmissionDecision, WorthQueryIntentAdmissionEligibility,
    WorthQueryProjectionConsumptionPlan, WorthQueryRawIntentAdmissionRequest,
};
use crate::projection_consumption::{
    intent_admission_admitted_projection_declaration,
    intent_admission_warning_projection_declaration,
};

#[derive(Clone)]
pub(in crate::intent_admission::certification) struct CertifiedBasisObservationIntentFixture {
    pub(in crate::intent_admission::certification) request: WorthQueryRawIntentAdmissionRequest,
    pub(in crate::intent_admission::certification) eligibility:
        WorthQueryIntentAdmissionEligibility,
    pub(in crate::intent_admission::certification) plan: WorthQueryBasisObservationPlan,
    pub(in crate::intent_admission::certification) scoped_basis_digest: String,
}

#[derive(Clone)]
pub(in crate::intent_admission::certification) struct CertifiedProjectionConsumptionAdmittedFixture
{
    pub(in crate::intent_admission::certification) request: WorthQueryRawIntentAdmissionRequest,
    pub(in crate::intent_admission::certification) eligibility:
        WorthQueryIntentAdmissionEligibility,
    pub(in crate::intent_admission::certification) plan: WorthQueryProjectionConsumptionPlan,
    pub(in crate::intent_admission::certification) contract_digest: String,
}

#[derive(Clone)]
pub(in crate::intent_admission::certification) struct CertifiedProjectionConsumptionWarningFixture {
    pub(in crate::intent_admission::certification) request: WorthQueryRawIntentAdmissionRequest,
    pub(in crate::intent_admission::certification) eligibility:
        WorthQueryIntentAdmissionEligibility,
    pub(in crate::intent_admission::certification) plan: WorthQueryProjectionConsumptionPlan,
    pub(in crate::intent_admission::certification) contract_digest: String,
}

pub(in crate::intent_admission::certification) fn certified_basis_observation_intent_fixture(
) -> Result<CertifiedBasisObservationIntentFixture, IdentityDerivationDenial> {
    let review = worth_query_basis_observation_intent(basis_lifecycle().current_head())
        .expect("basis observation intent should build")
        .review()?;
    let request = review.request().clone();
    let eligibility = review.eligibility().clone();
    let plan = match review.decision().clone() {
        WorthQueryIntentAdmissionDecision::Admitted(
            WorthQueryAdmittedIntentPlan::BasisObservation(plan),
        ) => plan,
        other => panic!("expected admitted basis plan, got {other:?}"),
    };
    let scoped_basis = worth_query_basis_observation_intent(basis_lifecycle().current_head())
        .expect("basis observation intent should build")
        .admit()
        .expect("basis observation should admit")
        .scope();
    Ok(CertifiedBasisObservationIntentFixture {
        request,
        eligibility,
        plan,
        scoped_basis_digest: scoped_basis.scoped_basis_digest().to_string(),
    })
}

pub(in crate::intent_admission::certification) fn certified_projection_consumption_admitted_fixture(
) -> Result<CertifiedProjectionConsumptionAdmittedFixture, IdentityDerivationDenial> {
    let declaration = intent_admission_admitted_projection_declaration()?;
    let review = worth_query_projection_consumption_intent(declaration)
        .expect("projection intent should build")
        .review()?;
    let request = review.request().clone();
    let eligibility = review.eligibility().clone();
    let plan = match review.decision().clone() {
        WorthQueryIntentAdmissionDecision::Admitted(
            WorthQueryAdmittedIntentPlan::ProjectionConsumption(plan),
        ) => plan,
        other => panic!("expected admitted projection plan, got {other:?}"),
    };
    let contract = review
        .admit()
        .expect("projection should admit")
        .bind_contract()?;
    Ok(CertifiedProjectionConsumptionAdmittedFixture {
        request,
        eligibility,
        plan,
        contract_digest: contract.contract_digest().to_string(),
    })
}

pub(in crate::intent_admission::certification) fn certified_projection_consumption_warning_fixture(
) -> Result<CertifiedProjectionConsumptionWarningFixture, IdentityDerivationDenial> {
    let declaration = intent_admission_warning_projection_declaration()?;
    let review = worth_query_projection_consumption_intent(declaration)
        .expect("warning projection intent should build")
        .review()?;
    let request = review.request().clone();
    let eligibility = review.eligibility().clone();
    let plan = match review.decision().clone() {
        WorthQueryIntentAdmissionDecision::Admitted(
            WorthQueryAdmittedIntentPlan::ProjectionConsumption(plan),
        ) => plan,
        other => panic!("expected warning-bearing admitted projection plan, got {other:?}"),
    };
    let contract = review
        .admit()
        .expect("warning-bearing projection should admit")
        .bind_contract()?;
    Ok(CertifiedProjectionConsumptionWarningFixture {
        request,
        eligibility,
        plan,
        contract_digest: contract.contract_digest().to_string(),
    })
}
