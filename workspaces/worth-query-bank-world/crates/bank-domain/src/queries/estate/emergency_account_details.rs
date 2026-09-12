use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
    ApplicationQueryLaneEligibility,
};
use worth_query_decl::facade::worth_query_application_query;

use crate::{
    authorization::ViewEstateCase,
    estate::{BankDisclosure, EmergencyAccessId, EstateAction, EstateCaseId, RestrictedBankField},
    reads::EstateAccountView,
    schema::{BankSchema, EstateCase, ViewEstateEmergencyProtectionCapability},
};

use super::emergency_account_details_selectors::{
    account_identity, account_name, account_status, estate_account,
};
use super::emergency_account_details_shape::emergency_account_details_shape;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateEmergencyAccountDetailsQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateEmergencyAccountDetailsRequest {
    estate: EstateCaseId,
    access: EmergencyAccessId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EstateEmergencyAccountDetails {
    account: BankDisclosure<EstateAccountView>,
}

impl EstateEmergencyAccountDetailsRequest {
    pub const fn estate(self) -> EstateCaseId {
        self.estate
    }

    pub const fn capability_request(self) -> EstateAction {
        EstateAction::ViewRestrictedEstateWithEmergencyAccess {
            estate: self.estate,
            access: self.access,
            field: RestrictedBankField::AccountDetails,
        }
    }
}

impl EstateEmergencyAccountDetails {
    pub const fn new(account: BankDisclosure<EstateAccountView>) -> Self {
        Self { account }
    }

    pub const fn account(&self) -> &BankDisclosure<EstateAccountView> {
        &self.account
    }
}

pub const fn estate_emergency_account_details(
    estate: EstateCaseId,
    access: EmergencyAccessId,
) -> EstateEmergencyAccountDetailsRequest {
    EstateEmergencyAccountDetailsRequest { estate, access }
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateEmergencyAccountDetailsQueryParametersBinding for EstateEmergencyAccountDetailsQueryParameters { identity: "EstateEmergencyAccountDetailsQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateEmergencyAccountDetailsQueryResultBinding for EstateEmergencyAccountDetails { identity: "EstateEmergencyAccountDetails" });
worth_query_application_query!(
    pub EstateEmergencyAccountDetailsQuery for BankSchema,
    identity "EstateEmergencyAccountDetailsQuery",
    parameters EstateEmergencyAccountDetailsQueryParametersBinding,
    result EstateEmergencyAccountDetailsQueryResultBinding,
    scope EstateCase => "EstateCase",
    name "estate_emergency_account_details"
);
worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateEmergencyAccountDetailsRequestBinding for EstateEmergencyAccountDetailsRequest { identity: "EstateEmergencyAccountDetailsRequest" });
worth_query_decl::facade::worth_query_query_binding!(
    pub EstateEmergencyAccountDetailsQueryBinding for EstateEmergencyAccountDetailsRequest, schema BankSchema,
    identity "worth.bank.estate-emergency-account-details-query-binding.v1",
    input EstateEmergencyAccountDetailsRequestBinding,
    query EstateEmergencyAccountDetailsQuery,
    parameters EstateEmergencyAccountDetailsQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result EstateEmergencyAccountDetailsQueryResultBinding,
    principal crate::schema::BankPrincipalBinding, mapping crate::schema::ExternalPrincipalMapping, principal_entity crate::schema::Principal,
        principal_identity crate::model::BankPrincipalId, identity_binding crate::schema::BankPrincipalIdBinding,
    scope EstateCase, crate::schema::EstateCaseRecord, crate::schema::EstateCaseIdentityField,
        EstateCaseId, worth_query_decl::facade::application_schema::ReadOnly,
        worth_query_decl::facade::application_schema::NoApplicationUnit,
    field crate::schema::EstateCaseIdentityField::reference(),
    value EstateEmergencyAccountDetailsRequest::estate,
    limits results 1_024, work 100_000
);

pub fn estate_emergency_account_details_definition() -> ApplicationQueryDefinition<
    BankSchema,
    EstateEmergencyAccountDetailsQuery,
    EstateEmergencyAccountDetailsQueryParameters,
    EstateEmergencyAccountDetails,
    EstateCase,
> {
    let field = || {
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::AccountDetails,
        )
    };
    let influence = ApplicationQueryInfluenceContract::forbid_all();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "estate-emergency-account-details",
        ViewEstateEmergencyProtectionCapability::reference(),
    )
    .disclose_relation_by(estate_account(), field(), influence.clone())
    .disclose_field_by(account_identity(), field(), influence.clone())
    .disclose_field_by(account_name(), field(), influence.clone())
    .disclose_field_by(account_status(), field(), influence);
    ApplicationQueryDefinitionBuilder::declare(EstateEmergencyAccountDetailsQuery::reference())
        .root(EstateCase::reference())
        .scope(EstateCase::reference())
        .result_shape(emergency_account_details_shape())
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 3))
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned().with_preview())
        .lanes(
            ApplicationQueryLaneEligibility::one_shot()
                .with_historical()
                .with_preview(),
        )
        .requires_ability(ViewEstateCase::reference())
        .build()
        .expect("estate emergency account details query is statically canonical")
}
