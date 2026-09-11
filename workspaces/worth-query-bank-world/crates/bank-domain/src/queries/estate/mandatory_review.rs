use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
    ApplicationQueryLaneEligibility,
};
use worth_query_decl::facade::worth_query_application_query;

use crate::{
    authorization::ViewEstateMandatoryReview,
    estate::{EstateAction, EstateCapabilityPurpose, EstateCaseId, RestrictedBankField},
    schema::{BankSchema, EstateCase, ViewEstateMandatoryReviewCapability},
};

use super::{
    mandatory_review_projection::EstateMandatoryReviewResult, mandatory_review_selectors::*,
    mandatory_review_shape::mandatory_review_shape,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateMandatoryReviewQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateMandatoryReviewRequest {
    estate: EstateCaseId,
}

impl EstateMandatoryReviewRequest {
    pub const fn estate(self) -> EstateCaseId {
        self.estate
    }

    pub const fn capability_request(self) -> EstateAction {
        EstateAction::ViewRestrictedEstate {
            estate: self.estate,
            field: RestrictedBankField::AuditTrail,
            purpose: EstateCapabilityPurpose::MandatoryReview,
        }
    }
}

pub const fn estate_mandatory_reviews(estate: EstateCaseId) -> EstateMandatoryReviewRequest {
    EstateMandatoryReviewRequest { estate }
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateMandatoryReviewQueryParametersBinding for EstateMandatoryReviewQueryParameters { identity: "EstateMandatoryReviewQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateMandatoryReviewQueryResultBinding for EstateMandatoryReviewResult { identity: "EstateMandatoryReviewResult" });
worth_query_application_query!(
    pub EstateMandatoryReviewQuery for BankSchema,
    identity "EstateMandatoryReviewQuery",
    parameters EstateMandatoryReviewQueryParametersBinding,
    result EstateMandatoryReviewQueryResultBinding,
    scope EstateCase => "EstateCase",
    name "estate_mandatory_reviews"
);

pub fn estate_mandatory_review_definition() -> ApplicationQueryDefinition<
    BankSchema,
    EstateMandatoryReviewQuery,
    EstateMandatoryReviewQueryParameters,
    EstateMandatoryReviewResult,
    EstateCase,
> {
    ApplicationQueryDefinitionBuilder::declare(EstateMandatoryReviewQuery::reference())
        .root(EstateCase::reference())
        .scope(EstateCase::reference())
        .result_shape(mandatory_review_shape())
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(3, 16, 16))
        .disclosure(disclosure_contract())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewEstateMandatoryReview::reference())
        .build()
        .expect("bank estate mandatory-review query is statically canonical")
}

fn disclosure_contract() -> ApplicationQueryDisclosureContract {
    let field = || {
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::AuditTrail,
        )
    };
    let influence = ApplicationQueryInfluenceContract::forbid_all();
    ApplicationQueryDisclosureContract::governed_by(
        "estate-mandatory-review",
        ViewEstateMandatoryReviewCapability::reference(),
    )
    .disclose_field_by(estate_identity(), field(), influence.clone())
    .disclose_relation_by(estate_reviews(), field(), influence.clone())
    .disclose_field_by(review_identity(), field(), influence.clone())
    .disclose_field_by(review_kind(), field(), influence.clone())
    .disclose_field_by(review_status(), field(), influence.clone())
    .disclose_relation_by(review_principal(), field(), influence.clone())
    .disclose_field_by(review_principal_identity(), field(), influence)
}
