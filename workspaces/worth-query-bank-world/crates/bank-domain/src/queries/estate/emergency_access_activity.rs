use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
        ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
        ApplicationQueryLaneEligibility, ApplicationQueryLiveResourceContract,
        ApplicationQueryObservableInfluence, ApplicationQueryOrderingDirection,
    },
    worth_query_application_query,
};

use crate::{
    authorization::ViewEstateCase,
    estate::{
        EmergencyAccessId, EmergencyAccessReason, EmergencyAccessStatus, EstateAction,
        EstateCaseId, EstateMoment, MandatoryReviewId, MandatoryReviewStatus, RestrictedBankField,
    },
    schema::{
        BankSchema, EmergencyAccess, EmergencyAccessIdentityField, EmergencyAccessIssuedAtField,
        EstateCase, EstateCaseIdentityField, ViewEstateEmergencyProtectionCapability,
    },
};

use self::{
    selectors::{access_id, access_issued_at, estate_accesses, estate_id},
    shape::emergency_access_activity_shape,
};

mod live_cause;
mod projection;
mod selectors;
mod shape;
#[cfg(test)]
mod tests;

pub use live_cause::EstateEmergencyAccessActivityLiveCause;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateEmergencyAccessActivityQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateEmergencyAccessActivityRequest {
    estate: EstateCaseId,
    access: EmergencyAccessId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EstateEmergencyAccessActivity {
    estate: EstateCaseId,
    accesses: Vec<EstateEmergencyAccessActivityItem>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateEmergencyAccessActivityItem {
    access: EmergencyAccessId,
    reason: EmergencyAccessReason,
    status: EmergencyAccessStatus,
    issued_at: EstateMoment,
    expires_at: EstateMoment,
    review: MandatoryReviewId,
    review_status: MandatoryReviewStatus,
}

impl EstateEmergencyAccessActivityRequest {
    pub const fn estate(self) -> EstateCaseId {
        self.estate
    }

    pub const fn capability_request(self) -> EstateAction {
        EstateAction::ViewRestrictedEstateWithEmergencyAccess {
            estate: self.estate,
            access: self.access,
            field: RestrictedBankField::EmergencyAccessActivity,
        }
    }
}

impl EstateEmergencyAccessActivity {
    pub const fn estate(&self) -> EstateCaseId {
        self.estate
    }

    pub fn accesses(&self) -> &[EstateEmergencyAccessActivityItem] {
        &self.accesses
    }

    pub(super) fn from_projection(
        estate: EstateCaseId,
        accesses: Vec<EstateEmergencyAccessActivityItem>,
    ) -> Self {
        Self { estate, accesses }
    }
}

impl EstateEmergencyAccessActivityItem {
    pub const fn access(self) -> EmergencyAccessId {
        self.access
    }

    pub const fn reason(self) -> EmergencyAccessReason {
        self.reason
    }

    pub const fn status(self) -> EmergencyAccessStatus {
        self.status
    }

    pub const fn issued_at(self) -> EstateMoment {
        self.issued_at
    }

    pub const fn expires_at(self) -> EstateMoment {
        self.expires_at
    }

    pub const fn review(self) -> MandatoryReviewId {
        self.review
    }

    pub const fn review_status(self) -> MandatoryReviewStatus {
        self.review_status
    }
}

pub const fn estate_emergency_access_activity(
    estate: EstateCaseId,
    access: EmergencyAccessId,
) -> EstateEmergencyAccessActivityRequest {
    EstateEmergencyAccessActivityRequest { estate, access }
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateEmergencyAccessActivityQueryParametersBinding for EstateEmergencyAccessActivityQueryParameters { identity: "EstateEmergencyAccessActivityQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateEmergencyAccessActivityQueryResultBinding for EstateEmergencyAccessActivity { identity: "EstateEmergencyAccessActivity" });
worth_query_application_query!(
    pub EstateEmergencyAccessActivityQuery for BankSchema,
    identity "EstateEmergencyAccessActivityQuery",
    parameters EstateEmergencyAccessActivityQueryParametersBinding,
    result EstateEmergencyAccessActivityQueryResultBinding,
    scope EstateCase => "EstateCase",
    name "estate_emergency_access_activity"
);
worth_query_decl::facade::worth_query_structured_value_binding!(pub EstateEmergencyAccessActivityRequestBinding for EstateEmergencyAccessActivityRequest { identity: "EstateEmergencyAccessActivityRequest" });
worth_query_decl::facade::worth_query_query_binding!(
    pub EstateEmergencyAccessActivityQueryBinding for EstateEmergencyAccessActivityRequest, schema BankSchema,
    identity "worth.bank.estate-emergency-access-activity-query-binding.v1",
    input EstateEmergencyAccessActivityRequestBinding,
    query EstateEmergencyAccessActivityQuery,
    parameters EstateEmergencyAccessActivityQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result EstateEmergencyAccessActivityQueryResultBinding,
    principal crate::schema::BankPrincipalBinding, mapping crate::schema::ExternalPrincipalMapping, principal_entity crate::schema::Principal,
        principal_identity crate::model::BankPrincipalId, identity_binding crate::schema::BankPrincipalIdBinding,
    scope EstateCase, crate::schema::EstateCaseRecord, crate::schema::EstateCaseIdentityField,
        EstateCaseId, worth_query_decl::facade::application_schema::ReadOnly,
        worth_query_decl::facade::application_schema::NoApplicationUnit,
    field crate::schema::EstateCaseIdentityField::reference(),
    value EstateEmergencyAccessActivityRequest::estate,
    limits results 1_024, work 100_000
);

pub fn estate_emergency_access_activity_definition() -> ApplicationQueryDefinition<
    BankSchema,
    EstateEmergencyAccessActivityQuery,
    EstateEmergencyAccessActivityQueryParameters,
    EstateEmergencyAccessActivity,
    EstateCase,
> {
    let field = || {
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::EmergencyAccessActivity,
        )
    };
    let no_influence = ApplicationQueryInfluenceContract::forbid_all();
    let live_scope = ApplicationQueryInfluenceContract::permit([
        ApplicationQueryObservableInfluence::LiveMembership,
    ]);
    let collection = ApplicationQueryInfluenceContract::permit([
        ApplicationQueryObservableInfluence::Pagination,
    ]);
    let access_ordering = ApplicationQueryInfluenceContract::permit([
        ApplicationQueryObservableInfluence::Ordering,
        ApplicationQueryObservableInfluence::Pagination,
        ApplicationQueryObservableInfluence::HistoricalMembership,
        ApplicationQueryObservableInfluence::Preview,
        ApplicationQueryObservableInfluence::LiveMembership,
    ]);
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "estate-emergency-access-activity",
        ViewEstateEmergencyProtectionCapability::reference(),
    )
    .use_field_by(
        EstateCaseIdentityField::reference(),
        field(),
        live_scope.clone(),
    )
    .use_field_by(
        EmergencyAccessIdentityField::reference(),
        field(),
        access_ordering.clone(),
    )
    .use_field_by(
        EmergencyAccessIssuedAtField::reference(),
        field(),
        access_ordering.clone(),
    )
    .disclose_field_by(estate_id(), field(), live_scope)
    .disclose_relation_by(estate_accesses(), field(), collection)
    .disclose_field_by(access_id(), field(), access_ordering.clone())
    .disclose_field_by(selectors::access_reason(), field(), no_influence.clone())
    .disclose_field_by(selectors::access_status(), field(), no_influence.clone())
    .disclose_field_by(selectors::access_issued_at(), field(), access_ordering)
    .disclose_field_by(
        selectors::access_expires_at(),
        field(),
        no_influence.clone(),
    )
    .disclose_relation_by(selectors::access_review(), field(), no_influence.clone())
    .disclose_field_by(selectors::review_id(), field(), no_influence.clone())
    .disclose_field_by(selectors::review_status(), field(), no_influence);
    ApplicationQueryDefinitionBuilder::declare(EstateEmergencyAccessActivityQuery::reference())
        .root(EstateCase::reference())
        .scope(EstateCase::reference())
        .result_shape(emergency_access_activity_shape())
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(3, 2, 8))
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned().with_preview())
        .lanes(
            ApplicationQueryLaneEligibility::one_shot()
                .with_historical()
                .with_preview()
                .with_live(),
        )
        .requires_ability(ViewEstateCase::reference())
        .order_by(
            access_issued_at(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .order_by(access_id(), ApplicationQueryOrderingDirection::Ascending)
        .continue_by(estate_accesses())
        .live_by::<EmergencyAccess, EstateEmergencyAccessActivityLiveCause, _, _, _, _, _, _, _, _>(
            estate_id(),
            access_id(),
            ApplicationQueryLiveResourceContract::bounded(64, 2_048, 4_096),
        )
        .build()
        .expect("estate emergency-access activity query is statically canonical")
}
