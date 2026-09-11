use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryLaneEligibility,
};
use worth_query_decl::facade::worth_query_application_query;

use crate::{
    authorization::ViewEstateCase,
    estate::{EstateAction, EstateCapabilityPurpose, EstateCaseId, RestrictedBankField},
    reads::EstateGovernanceContext,
    schema::{BankSchema, EstateCase},
};

use super::{governance_disclosure::governance_disclosure, governance_shape::governance_shape};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateGovernanceQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateGovernanceRequest {
    estate: EstateCaseId,
}

impl EstateGovernanceRequest {
    pub const fn estate(self) -> EstateCaseId {
        self.estate
    }

    pub const fn capability_request(self) -> EstateAction {
        EstateAction::ViewRestrictedEstate {
            estate: self.estate,
            field: RestrictedBankField::GovernanceMetadata,
            purpose: EstateCapabilityPurpose::EstateAdministration,
        }
    }
}

pub const fn estate_governance_context(estate: EstateCaseId) -> EstateGovernanceRequest {
    EstateGovernanceRequest { estate }
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateGovernanceQueryParametersBinding for EstateGovernanceQueryParameters { identity: "EstateGovernanceQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateGovernanceQueryResultBinding for EstateGovernanceContext { identity: "EstateGovernanceContext" });
worth_query_application_query!(
    pub EstateGovernanceQuery for BankSchema,
    identity "EstateGovernanceQuery",
    parameters EstateGovernanceQueryParametersBinding,
    result EstateGovernanceQueryResultBinding,
    scope EstateCase => "EstateCase",
    name "estate_governance_context"
);

pub fn estate_governance_definition() -> ApplicationQueryDefinition<
    BankSchema,
    EstateGovernanceQuery,
    EstateGovernanceQueryParameters,
    EstateGovernanceContext,
    EstateCase,
> {
    ApplicationQueryDefinitionBuilder::declare(EstateGovernanceQuery::reference())
        .root(EstateCase::reference())
        .scope(EstateCase::reference())
        .result_shape(governance_shape())
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(4, 16, 34))
        .disclosure(governance_disclosure())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewEstateCase::reference())
        .build()
        .expect("bank estate governance query is statically canonical")
}
