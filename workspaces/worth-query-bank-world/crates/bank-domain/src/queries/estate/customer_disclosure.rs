use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
    ApplicationQueryLaneEligibility,
};
use worth_query_decl::facade::worth_query_application_query;

use crate::authorization::ViewEstateCase;
use crate::estate::{
    BankDisclosure, EstateAction, EstateCapabilityPurpose, EstateCaseId, RestrictedBankField,
};
use crate::model::BankPrincipalId;
use crate::schema::{BankSchema, EstateCase, ViewEstateIdentityVerificationCapability};

use super::customer_disclosure_selectors::{
    beneficiary_identity, customer_identity, estate_beneficiaries, estate_customer,
};
use super::customer_disclosure_shape::customer_disclosure_shape;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateCustomerDisclosureQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateCustomerDisclosureRequest {
    estate: EstateCaseId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EstateCustomerDisclosure {
    customer: BankDisclosure<BankPrincipalId>,
    beneficiaries: BankDisclosure<Vec<BankPrincipalId>>,
}

impl EstateCustomerDisclosureRequest {
    pub const fn estate(self) -> EstateCaseId {
        self.estate
    }

    pub const fn capability_request(self) -> EstateAction {
        EstateAction::ViewRestrictedEstate {
            estate: self.estate,
            field: RestrictedBankField::CustomerIdentity,
            purpose: EstateCapabilityPurpose::IdentityVerification,
        }
    }
}

impl EstateCustomerDisclosure {
    pub fn new(
        customer: BankDisclosure<BankPrincipalId>,
        beneficiaries: BankDisclosure<Vec<BankPrincipalId>>,
    ) -> Self {
        Self {
            customer,
            beneficiaries,
        }
    }

    pub const fn customer(&self) -> BankDisclosure<BankPrincipalId> {
        self.customer
    }

    pub const fn beneficiaries(&self) -> &BankDisclosure<Vec<BankPrincipalId>> {
        &self.beneficiaries
    }
}

pub const fn estate_customer_identity(estate: EstateCaseId) -> EstateCustomerDisclosureRequest {
    EstateCustomerDisclosureRequest { estate }
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateCustomerDisclosureQueryParametersBinding for EstateCustomerDisclosureQueryParameters { identity: "EstateCustomerDisclosureQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateCustomerDisclosureQueryResultBinding for EstateCustomerDisclosure { identity: "EstateCustomerDisclosure" });
worth_query_application_query!(
    pub EstateCustomerDisclosureQuery for BankSchema,
    identity "EstateCustomerDisclosureQuery",
    parameters EstateCustomerDisclosureQueryParametersBinding,
    result EstateCustomerDisclosureQueryResultBinding,
    scope EstateCase => "EstateCase",
    name "estate_customer_identity"
);
worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateCustomerDisclosureRequestBinding for EstateCustomerDisclosureRequest { identity: "EstateCustomerDisclosureRequest" });
worth_query_decl::facade::worth_query_query_binding!(
    pub EstateCustomerDisclosureQueryBinding for EstateCustomerDisclosureRequest, schema BankSchema,
    identity "worth.bank.estate-customer-disclosure-query-binding.v1",
    input EstateCustomerDisclosureRequestBinding,
    query EstateCustomerDisclosureQuery,
    parameters EstateCustomerDisclosureQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result EstateCustomerDisclosureQueryResultBinding,
    principal crate::schema::BankPrincipalBinding, mapping crate::schema::ExternalPrincipalMapping, principal_entity crate::schema::Principal,
        principal_identity crate::model::BankPrincipalId, identity_binding crate::schema::BankPrincipalIdBinding,
    scope EstateCase, crate::schema::EstateCaseRecord, crate::schema::EstateCaseIdentityField,
        EstateCaseId, worth_query_decl::facade::application_schema::ReadOnly,
        worth_query_decl::facade::application_schema::NoApplicationUnit,
    field crate::schema::EstateCaseIdentityField::reference(),
    value EstateCustomerDisclosureRequest::estate,
    limits results 1_024, work 100_000
);

pub fn estate_customer_disclosure_definition() -> ApplicationQueryDefinition<
    BankSchema,
    EstateCustomerDisclosureQuery,
    EstateCustomerDisclosureQueryParameters,
    EstateCustomerDisclosure,
    EstateCase,
> {
    let influence = ApplicationQueryInfluenceContract::forbid_all();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "estate-customer-identity",
        ViewEstateIdentityVerificationCapability::reference(),
    )
    .disclose_relation_by(
        estate_customer(),
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            crate::estate::RestrictedBankField::CustomerIdentity,
        ),
        influence.clone(),
    )
    .disclose_field_by(
        customer_identity(),
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            crate::estate::RestrictedBankField::CustomerIdentity,
        ),
        influence.clone(),
    )
    .disclose_relation_by(
        estate_beneficiaries(),
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::BeneficiaryIdentity,
        ),
        influence.clone(),
    )
    .disclose_field_by(
        beneficiary_identity(),
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::BeneficiaryIdentity,
        ),
        influence,
    );
    ApplicationQueryDefinitionBuilder::declare(EstateCustomerDisclosureQuery::reference())
        .root(EstateCase::reference())
        .scope(EstateCase::reference())
        .result_shape(customer_disclosure_shape())
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 2, 2))
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned().with_preview())
        .lanes(
            ApplicationQueryLaneEligibility::one_shot()
                .with_historical()
                .with_preview(),
        )
        .requires_ability(ViewEstateCase::reference())
        .build()
        .expect("estate customer disclosure query is statically canonical")
}
