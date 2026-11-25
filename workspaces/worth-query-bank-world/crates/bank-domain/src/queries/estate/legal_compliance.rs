use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
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
    legal_compliance_projection::EstateLegalComplianceResult,
    legal_compliance_rows::{AuthorityRow, EstateRow, HolderRow},
};

worth_query_decl::facade::worth_query_disclosure_classification!(EstateLegalComplianceDisclosure);

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
    pub EstateLegalComplianceQueryBinding for EstateLegalComplianceRequest, schema BankSchema, context crate::schema::BankBindingContext,
    identity "worth.bank.estate-legal-compliance-query-binding.v1",
    input EstateLegalComplianceRequestBinding,
    query EstateLegalComplianceQuery,
    parameters EstateLegalComplianceQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result EstateLegalComplianceQueryResultBinding,
    scope crate::schema::EstateCaseIdentityField,
    field crate::schema::EstateCaseIdentityField::at("EstateCaseIdentityField"),
    value EstateLegalComplianceRequest::estate,
    limits results 1_024
);

pub fn estate_legal_compliance_definition() -> ApplicationQueryDefinitionBuilder<
    BankSchema,
    EstateLegalComplianceQuery,
    EstateLegalComplianceQueryParameters,
    EstateLegalComplianceResult,
    EstateCase,
> {
    ApplicationQueryDefinitionBuilder::declare(EstateLegalComplianceQuery::reference())
        .root(EstateCase::reference())
        .scope(EstateCase::reference())
        .result_shape(
            EstateRow::result_shape_builder::<
                EstateLegalComplianceQuery,
                EstateLegalComplianceQueryResultBinding,
            >()
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(disclosure_contract())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewEstateLegalCompliance::reference())
}

fn disclosure_contract() -> ApplicationQueryDisclosureContract {
    let field = || {
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::LegalDocument,
        )
    };
    type Q = EstateLegalComplianceQuery;
    let influence = ApplicationQueryInfluenceContract::forbid_all();
    ApplicationQueryDisclosureContract::governed_by(
        EstateLegalComplianceDisclosure,
        ViewEstateLegalComplianceCapability::reference(),
    )
    .disclose_field_by(EstateRow::estate::<Q>(), field(), influence.clone())
    .disclose_relation_by(EstateRow::authorities::<Q>(), field(), influence.clone())
    .disclose_field_by(AuthorityRow::authority::<Q>(), field(), influence.clone())
    .disclose_field_by(AuthorityRow::kind::<Q>(), field(), influence.clone())
    .disclose_field_by(AuthorityRow::recognized::<Q>(), field(), influence.clone())
    .disclose_relation_by(AuthorityRow::holder::<Q>(), field(), influence.clone())
    .disclose_field_by(HolderRow::holder::<Q>(), field(), influence)
}
