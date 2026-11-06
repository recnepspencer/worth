use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityElevationRequest, ApplicationCapabilityElevationRequestProjection,
        ApplicationCapabilityElevationRequestProjectionDenial, ApplicationCapabilityEntitySelector,
        ApplicationCapabilityRequest, ApplicationCapabilityRequestContext,
        ApplicationCapabilityRequestProjection, ApplicationCapabilityRequestProjectionDenial,
        ApplicationCapabilityValueBinding,
    },
    application_schema::{
        ApplicationEncodedScalarValue, StringApplicationValueBinding, U64ApplicationValueBinding,
    },
};
use worth_query_declaration::{
    worth_query_capability, worth_query_capability_context_entity_slot, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
};

use super::super::{Account, AccountIdentity, IdentityExecutionSchema};
use super::declaration::*;

#[path = "elevated/value_bindings.rs"]
mod value_bindings;

pub use value_bindings::{
    CapabilityElevationStatusBinding, CapabilityReviewKindBinding, CapabilityReviewStatusBinding,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityElevationStatus {
    Requested,
    Approved,
    Expired,
    Revoked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityReviewStatus {
    Required,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityReviewKind {
    Elevation,
}

worth_query_capability_context_entity_slot!(pub CapabilityElevationSlot in IdentityExecutionSchema, CapabilityRequestContext => CapabilityElevation);
worth_query_capability_context_entity_slot!(pub CapabilityReviewSlot in IdentityExecutionSchema, CapabilityRequestContext => CapabilityReview);
worth_query_capability!(pub ElevatedTouchAccountCapability in IdentityExecutionSchema);
worth_query_capability!(pub RequestElevationCapability in IdentityExecutionSchema);
worth_query_capability!(pub ApproveElevationCapability in IdentityExecutionSchema);

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ElevatedCapabilityTouchInput {
    pub account: String,
    pub elevation: Option<String>,
    pub substitute_resource_selector: bool,
    pub action: CapabilityAction,
    pub purpose: CapabilityPurpose,
    pub disclosure: CapabilityDisclosure,
    pub amount: u64,
}
worth_query_declaration::worth_query_portable_type!(CapabilityElevationStatus => "worth.query.test.execution.capability.elevation_status.v1");
worth_query_declaration::worth_query_portable_type!(CapabilityReviewStatus => "worth.query.test.execution.capability.review_status.v1");
worth_query_declaration::worth_query_portable_type!(CapabilityReviewKind => "worth.query.test.execution.capability.review_kind.v1");
worth_query_declaration::worth_query_portable_type!(
    ElevatedCapabilityTouchInput => "worth.query.test.elevated-capability-touch-input.v1"
);

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct RequestElevationInput {
    pub account: String,
    pub target_account: String,
    pub grant: String,
    pub elevation_key: String,
    pub elevation_identity: String,
    pub review_key: String,
    pub review_identity: String,
    pub reason: String,
    pub duration: std::time::Duration,
    pub action: CapabilityAction,
    pub target_purpose: CapabilityPurpose,
    pub disclosure: CapabilityDisclosure,
    pub amount: u64,
}
worth_query_declaration::worth_query_portable_type!(
    RequestElevationInput => "worth.query.test.request-elevation-input.v1"
);

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ApproveElevationInput {
    pub account: String,
    pub elevation: String,
}
worth_query_declaration::worth_query_portable_type!(
    ApproveElevationInput => "worth.query.test.approve-elevation-input.v1"
);

worth_query_declaration::worth_query_structured_value_binding!(pub ElevatedCapabilityTouchOperationInputBinding for ElevatedCapabilityTouchInput { identity: "worth.query.test.elevated-capability-touch-input.v1" });
worth_query_operation!(pub ElevatedCapabilityTouchOperation for IdentityExecutionSchema, input ElevatedCapabilityTouchOperationInputBinding);
worth_query_declaration::worth_query_structured_value_binding!(pub RequestCapabilityElevationOperationInputBinding for RequestElevationInput { identity: "worth.query.test.request-elevation-input.v1" });
worth_query_operation!(pub RequestCapabilityElevationOperation for IdentityExecutionSchema, input RequestCapabilityElevationOperationInputBinding);
worth_query_declaration::worth_query_structured_value_binding!(pub ApproveCapabilityElevationOperationInputBinding for ApproveElevationInput { identity: "worth.query.test.approve-elevation-input.v1" });
worth_query_operation!(pub ApproveCapabilityElevationOperation for IdentityExecutionSchema, input ApproveCapabilityElevationOperationInputBinding);

worth_query_operation_creates!(RequestCapabilityElevationOperation => [CapabilityElevation, CapabilityReview]);

worth_query_operation_links!(RequestCapabilityElevationOperation => [CapabilityElevationRequester, CapabilityElevationGrant, CapabilityElevationResource, CapabilityElevationReview, CapabilityReviewResource]);
worth_query_operation_reads!(ApproveCapabilityElevationOperation => [CapabilityElevationRequester, CapabilityElevationApprover, CapabilityElevationGrant, CapabilityElevationResource, CapabilityElevationReview, CapabilityReviewResource, CapabilityReviewer]);

worth_query_operation_links!(ApproveCapabilityElevationOperation => [CapabilityElevationApprover]);

impl ApplicationCapabilityRequest<IdentityExecutionSchema, ElevatedTouchAccountCapability>
    for ElevatedCapabilityTouchInput
{
    type Scope = Account;
    type Context = CapabilityRequestContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            IdentityExecutionSchema,
            Account,
            CapabilityRequestContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let projection = ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                AccountIdentity::at("AccountIdentity"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.account.clone(),
                )
                .expect("fixture account identity must encode"),
            )
            .unwrap(),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(self.action)
                .expect("fixture capability action must encode"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(self.purpose)
                .expect("fixture capability purpose must encode"),
            ApplicationCapabilityRequestContext::new(CapabilityRequestContext::reference()),
        );
        let projection = match (&self.elevation, self.substitute_resource_selector) {
            (Some(_), true) => projection.elevation(ApplicationCapabilityEntitySelector::new(
                AccountIdentity::at("AccountIdentity"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.account.clone(),
                )
                .expect("fixture account identity must encode"),
            ).unwrap()),
            (Some(elevation), false) => {
                projection.elevation(ApplicationCapabilityEntitySelector::new(
                    CapabilityElevationIdentity::at("CapabilityElevationIdentity"),
                    ApplicationEncodedScalarValue::<<CapabilityElevationIdentity as worth_query_declaration::facade::application_schema::fragment::RecordSelection>::Binding>::try_new(elevation.clone().into())
                    .expect("fixture elevation identity must encode"),
                ).unwrap())
            }
            (None, _) => projection,
        };
        Ok(projection
            .field(
                ApplicationEncodedScalarValue::<CapabilityDisclosureBinding>::try_new(
                    self.disclosure,
                )
                .expect("fixture capability disclosure must encode"),
            )
            .magnitude(
                ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(self.amount)
                    .expect("fixture capability magnitude must encode"),
            ))
    }
}

impl ApplicationCapabilityRequest<IdentityExecutionSchema, RequestElevationCapability>
    for RequestElevationInput
{
    type Scope = Account;
    type Context = CapabilityRequestContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            IdentityExecutionSchema,
            Account,
            CapabilityRequestContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(self.ordinary_projection())
    }
}

impl ApplicationCapabilityRequest<IdentityExecutionSchema, ApproveElevationCapability>
    for ApproveElevationInput
{
    type Scope = Account;
    type Context = CapabilityRequestContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            IdentityExecutionSchema,
            Account,
            CapabilityRequestContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                AccountIdentity::at("AccountIdentity"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.account.clone(),
                )
                .expect("fixture account identity must encode"),
            ).unwrap(),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(
                CapabilityAction::ApproveElevation,
            )
            .expect("fixture capability action must encode"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .expect("fixture capability purpose must encode"),
            ApplicationCapabilityRequestContext::new(CapabilityRequestContext::reference()).entity(
                CapabilityElevationSlot::reference(),
                ApplicationCapabilityEntitySelector::new(
                    CapabilityElevationIdentity::at("CapabilityElevationIdentity"),
                    ApplicationEncodedScalarValue::<<CapabilityElevationIdentity as worth_query_declaration::facade::application_schema::fragment::RecordSelection>::Binding>::try_new(self.elevation.clone().into())
                    .expect("fixture elevation identity must encode"),
                ).unwrap(),
            ),
        ))
    }
}

impl
    ApplicationCapabilityElevationRequest<
        IdentityExecutionSchema,
        RequestCapabilityElevationOperation,
    > for RequestElevationInput
{
    type Scope = Account;
    type Context = CapabilityRequestContext;

    fn elevation_request(
        &self,
    ) -> Result<
        ApplicationCapabilityElevationRequestProjection<
            IdentityExecutionSchema,
            Account,
            CapabilityRequestContext,
        >,
        ApplicationCapabilityElevationRequestProjectionDenial,
    > {
        ApplicationCapabilityElevationRequestProjection::new(
            self.target_projection(),
            ApplicationCapabilityEntitySelector::new(
                CapabilityIdentity::at("CapabilityIdentity"),
                ApplicationEncodedScalarValue::<<CapabilityIdentity as worth_query_declaration::facade::application_schema::fragment::RecordSelection>::Binding>::try_new(self.grant.clone().into())
                .expect("fixture grant identity must encode"),
            ).unwrap(),
            self.elevation_key.clone(),
            ApplicationCapabilityValueBinding::new(
                CapabilityElevationIdentity::at("CapabilityElevationIdentity"),
                ApplicationEncodedScalarValue::<<CapabilityElevationIdentity as worth_query_declaration::facade::application_schema::fragment::RecordSelection>::Binding>::try_new(self.elevation_identity.clone().into())
                .expect("fixture elevation identity must encode"),
            ).unwrap(),
            self.review_key.clone(),
            ApplicationCapabilityValueBinding::new(
                CapabilityReviewIdentity::at("CapabilityReviewIdentity"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.review_identity.clone(),
                )
                .expect("fixture review identity must encode"),
            ).unwrap(),
            ApplicationCapabilityValueBinding::new(
                CapabilityElevationReason::at("CapabilityElevationReason"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.reason.clone(),
                )
                .expect("fixture elevation reason must encode"),
            ).unwrap(),
            self.duration,
        )
    }
}

impl RequestElevationInput {
    fn ordinary_projection(
        &self,
    ) -> ApplicationCapabilityRequestProjection<
        IdentityExecutionSchema,
        Account,
        CapabilityRequestContext,
    > {
        ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                AccountIdentity::at("AccountIdentity"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.account.clone(),
                )
                .expect("fixture account identity must encode"),
            )
            .unwrap(),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(
                CapabilityAction::RequestElevation,
            )
            .expect("fixture capability action must encode"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .expect("fixture capability purpose must encode"),
            ApplicationCapabilityRequestContext::new(CapabilityRequestContext::reference()),
        )
    }

    fn target_projection(
        &self,
    ) -> ApplicationCapabilityRequestProjection<
        IdentityExecutionSchema,
        Account,
        CapabilityRequestContext,
    > {
        ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                AccountIdentity::at("AccountIdentity"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.target_account.clone(),
                )
                .expect("fixture target account identity must encode"),
            )
            .unwrap(),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(self.action)
                .expect("fixture capability action must encode"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(self.target_purpose)
                .expect("fixture capability purpose must encode"),
            ApplicationCapabilityRequestContext::new(CapabilityRequestContext::reference()),
        )
        .field(
            ApplicationEncodedScalarValue::<CapabilityDisclosureBinding>::try_new(self.disclosure)
                .expect("fixture capability disclosure must encode"),
        )
        .magnitude(
            ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(self.amount)
                .expect("fixture capability magnitude must encode"),
        )
    }
}

#[path = "elevated/account_activity_query.rs"]
mod account_activity_query;
#[path = "elevated/close.rs"]
mod close;
#[path = "elevated/contract.rs"]
mod contract;
#[path = "elevated/review.rs"]
mod review;
pub(in crate::domain_computation::primary_graph) use account_activity_query::{
    elevated_account_activity_definition, elevated_account_activity_parameters,
    ElevatedAccountActivityCause, ElevatedAccountActivityQuery, ElevatedAccountActivityResult,
};
pub use close::*;
pub(super) use contract::install;
pub use review::*;

pub use crate::domain_computation::primary_graph::tests::fixture::schema_types::storage_model::{
    CapabilityElevation, CapabilityElevationApprover, CapabilityElevationClosedAt,
    CapabilityElevationGrant, CapabilityElevationIdentity, CapabilityElevationNotAfter,
    CapabilityElevationNotBefore, CapabilityElevationReason, CapabilityElevationRequester,
    CapabilityElevationResource, CapabilityElevationReview, CapabilityElevationStatusField,
    CapabilityReview, CapabilityReviewIdentity, CapabilityReviewKindField,
    CapabilityReviewResource, CapabilityReviewReviewedAt, CapabilityReviewStatusField,
    CapabilityReviewer,
};
