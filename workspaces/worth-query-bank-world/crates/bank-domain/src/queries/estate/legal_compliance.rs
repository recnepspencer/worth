use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
    ApplicationQueryLaneEligibility,
};
use worth_query_decl::facade::worth_query_application_query;

use crate::{
    authorization::ViewEstateLegalCompliance,
    estate::{EstateAction, EstateCapabilityPurpose, EstateCaseId, RestrictedBankField},
    schema::{BankSchema, EstateCase, ViewEstateLegalComplianceCapability},
};

use super::{
    legal_compliance_projection::EstateLegalComplianceResult, legal_compliance_selectors::*,
    legal_compliance_shape::legal_compliance_shape,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateLegalComplianceQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateLegalComplianceRequest {
    estate: EstateCaseId,
}

impl EstateLegalComplianceRequest {
    pub const fn estate(self) -> EstateCaseId {
        self.estate
    }

    pub const fn capability_request(self) -> EstateAction {
        EstateAction::ViewRestrictedEstate {
            estate: self.estate,
            field: RestrictedBankField::LegalDocument,
            purpose: EstateCapabilityPurpose::LegalCompliance,
        }
    }
}

pub const fn estate_legal_compliance(estate: EstateCaseId) -> EstateLegalComplianceRequest {
    EstateLegalComplianceRequest { estate }
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateLegalComplianceQueryParametersBinding for EstateLegalComplianceQueryParameters { identity: "EstateLegalComplianceQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateLegalComplianceQueryResultBinding for EstateLegalComplianceResult { identity: "EstateLegalComplianceResult" });
worth_query_application_query!(
    pub EstateLegalComplianceQuery for BankSchema,
    identity "EstateLegalComplianceQuery",
    parameters EstateLegalComplianceQueryParametersBinding,
    result EstateLegalComplianceQueryResultBinding,
    scope EstateCase => "EstateCase",
    name "estate_legal_compliance"
);
worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateLegalComplianceRequestBinding for EstateLegalComplianceRequest { identity: "EstateLegalComplianceRequest" });
worth_query_decl::facade::worth_query_query_binding!(
    pub EstateLegalComplianceQueryBinding for EstateLegalComplianceRequest, schema BankSchema,
    identity "worth.bank.estate-legal-compliance-query-binding.v1",
    input EstateLegalComplianceRequestBinding,
    query EstateLegalComplianceQuery,
    parameters EstateLegalComplianceQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result EstateLegalComplianceQueryResultBinding,
    principal crate::schema::BankPrincipalBinding, mapping crate::schema::ExternalPrincipalMapping, principal_entity crate::schema::Principal,
        principal_identity crate::model::BankPrincipalId, identity_binding crate::schema::BankPrincipalIdBinding,
    scope EstateCase, crate::schema::EstateCaseRecord, crate::schema::EstateCaseIdentityField,
        EstateCaseId, worth_query_decl::facade::application_schema::ReadOnly,
        worth_query_decl::facade::application_schema::NoApplicationUnit,
    field crate::schema::EstateCaseIdentityField::reference(),
    value EstateLegalComplianceRequest::estate,
    limits results 1_024, work 100_000
);

pub fn estate_legal_compliance_definition() -> ApplicationQueryDefinition<
    BankSchema,
    EstateLegalComplianceQuery,
    EstateLegalComplianceQueryParameters,
    EstateLegalComplianceResult,
    EstateCase,
> {
    ApplicationQueryDefinitionBuilder::declare(EstateLegalComplianceQuery::reference())
        .root(EstateCase::reference())
        .scope(EstateCase::reference())
        .result_shape(legal_compliance_shape())
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(3, 16, 16))
        .disclosure(disclosure_contract())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewEstateLegalCompliance::reference())
        .build()
        .expect("bank estate legal-compliance query is statically canonical")
}

fn disclosure_contract() -> ApplicationQueryDisclosureContract {
    let field = || {
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::LegalDocument,
        )
    };
    let influence = ApplicationQueryInfluenceContract::forbid_all();
    ApplicationQueryDisclosureContract::governed_by(
        "estate-legal-compliance",
        ViewEstateLegalComplianceCapability::reference(),
    )
    .disclose_field_by(estate_identity(), field(), influence.clone())
    .disclose_relation_by(estate_authorities(), field(), influence.clone())
    .disclose_field_by(authority_identity(), field(), influence.clone())
    .disclose_field_by(authority_kind(), field(), influence.clone())
    .disclose_field_by(authority_recognized(), field(), influence.clone())
    .disclose_relation_by(authority_holder(), field(), influence.clone())
    .disclose_field_by(authority_holder_identity(), field(), influence)
}
