use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
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
    mandatory_review_projection::EstateMandatoryReviewResult,
    mandatory_review_rows::{EstateRow, ReviewRow, ReviewerRow},
};

worth_query_decl::facade::worth_query_disclosure_classification!(EstateMandatoryReviewDisclosure);

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
worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateMandatoryReviewRequestBinding for EstateMandatoryReviewRequest { identity: "EstateMandatoryReviewRequest" });
worth_query_decl::facade::worth_query_query_binding!(
    pub EstateMandatoryReviewQueryBinding for EstateMandatoryReviewRequest, schema BankSchema, context crate::schema::BankBindingContext,
    identity "worth.bank.estate-mandatory-review-query-binding.v1",
    input EstateMandatoryReviewRequestBinding,
    query EstateMandatoryReviewQuery,
    parameters EstateMandatoryReviewQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result EstateMandatoryReviewQueryResultBinding,
    scope crate::schema::EstateCaseIdentityField,
    field crate::schema::EstateCaseIdentityField::at("EstateCaseIdentityField"),
    value EstateMandatoryReviewRequest::estate,
    limits results 1_024
);

pub fn estate_mandatory_review_definition() -> ApplicationQueryDefinitionBuilder<
    BankSchema,
    EstateMandatoryReviewQuery,
    EstateMandatoryReviewQueryParameters,
    EstateMandatoryReviewResult,
    EstateCase,
> {
    ApplicationQueryDefinitionBuilder::declare(EstateMandatoryReviewQuery::reference())
        .root(EstateCase::reference())
        .scope(EstateCase::reference())
        .result_shape(
            EstateRow::result_shape_builder::<
                EstateMandatoryReviewQuery,
                EstateMandatoryReviewQueryResultBinding,
            >()
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(disclosure_contract())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewEstateMandatoryReview::reference())
}

fn disclosure_contract() -> ApplicationQueryDisclosureContract {
    let field = || {
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::AuditTrail,
        )
    };
    type Q = EstateMandatoryReviewQuery;
    let influence = ApplicationQueryInfluenceContract::forbid_all();
    ApplicationQueryDisclosureContract::governed_by(
        EstateMandatoryReviewDisclosure,
        ViewEstateMandatoryReviewCapability::reference(),
    )
    .disclose_field_by(EstateRow::estate::<Q>(), field(), influence.clone())
    .disclose_relation_by(EstateRow::reviews::<Q>(), field(), influence.clone())
    .disclose_field_by(ReviewRow::review::<Q>(), field(), influence.clone())
    .disclose_field_by(ReviewRow::kind::<Q>(), field(), influence.clone())
    .disclose_field_by(ReviewRow::status::<Q>(), field(), influence.clone())
    .disclose_relation_by(ReviewRow::reviewer::<Q>(), field(), influence.clone())
    .disclose_field_by(ReviewerRow::reviewer::<Q>(), field(), influence)
}
