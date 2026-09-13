mod fields;
mod projection;
mod relations;
mod shape;

use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryOrderingDirection,
};
use worth_query_decl::facade::worth_query_application_query;

use crate::authorization::AuditInstitution;
use crate::model::InstitutionId;
use crate::reads::InstitutionAuditView;
use crate::schema::{BankSchema, Institution};

use self::fields::posting_sequence;
use self::shape::institution_audit_shape;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstitutionAuditQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstitutionAuditRequest {
    institution: InstitutionId,
}

impl InstitutionAuditRequest {
    pub const fn new(institution: InstitutionId) -> Self {
        Self { institution }
    }

    pub const fn institution(self) -> InstitutionId {
        self.institution
    }
}

pub const fn institution_audit(institution: InstitutionId) -> InstitutionAuditRequest {
    InstitutionAuditRequest::new(institution)
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub InstitutionAuditQueryParametersBinding for InstitutionAuditQueryParameters { identity: "InstitutionAuditQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub InstitutionAuditQueryResultBinding for InstitutionAuditView { identity: "InstitutionAuditView" });
worth_query_application_query!(
    pub InstitutionAuditQuery for BankSchema,
    identity "InstitutionAuditQuery",
    parameters InstitutionAuditQueryParametersBinding,
    result InstitutionAuditQueryResultBinding,
    scope Institution => "Institution",
    name "institution_audit"
);
worth_query_decl::facade::worth_query_structured_value_binding!(pub InstitutionAuditRequestBinding for InstitutionAuditRequest { identity: "InstitutionAuditRequest" });
worth_query_decl::facade::worth_query_query_binding!(
    pub InstitutionAuditQueryBinding for InstitutionAuditRequest, schema BankSchema,
    identity "worth.bank.institution-audit-query-binding.v1",
    input InstitutionAuditRequestBinding,
    query InstitutionAuditQuery,
    parameters InstitutionAuditQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result InstitutionAuditQueryResultBinding,
    principal crate::schema::BankPrincipalBinding, mapping crate::schema::ExternalPrincipalMapping, principal_entity crate::schema::Principal,
        principal_identity crate::model::BankPrincipalId, identity_binding crate::schema::BankPrincipalIdBinding,
    scope Institution, crate::schema::InstitutionIdentity,
        crate::schema::InstitutionIdentityField, InstitutionId,
        worth_query_decl::facade::application_schema::ReadOnly,
        worth_query_decl::facade::application_schema::NoApplicationUnit,
    field crate::schema::InstitutionIdentityField::reference(),
    value InstitutionAuditRequest::institution,
    limits results 1_024, work 100_000

);

pub fn institution_audit_definition() -> ApplicationQueryDefinition<
    BankSchema,
    InstitutionAuditQuery,
    InstitutionAuditQueryParameters,
    InstitutionAuditView,
    Institution,
> {
    ApplicationQueryDefinitionBuilder::declare(InstitutionAuditQuery::reference())
        .root(Institution::reference())
        .scope(Institution::reference())
        .result_shape(institution_audit_shape())
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(4, 4, 10))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(AuditInstitution::reference())
        .order_by(
            posting_sequence(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .build()
        .expect("bank institution audit query is statically canonical")
}
