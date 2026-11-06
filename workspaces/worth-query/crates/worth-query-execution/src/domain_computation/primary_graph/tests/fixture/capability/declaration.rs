use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityEntitySelector, ApplicationCapabilityRelatedEntitySelector,
        ApplicationCapabilityRequest, ApplicationCapabilityRequestContext,
        ApplicationCapabilityRequestProjection, ApplicationCapabilityRequestProjectionDenial,
    },
    application_schema::{
        ApplicationEncodedScalarValue, ApplicationStructuredValueBinding,
        ApplicationValueValidationDenial, StringApplicationValueBinding,
        U64ApplicationValueBinding,
    },
};
use worth_query_declaration::{
    worth_query_capability, worth_query_capability_context,
    worth_query_capability_context_entity_slot, worth_query_capability_provenance,
    worth_query_operation,
};

use super::super::{Account, AccountIdentity, IdentityExecutionSchema};

#[path = "value_bindings.rs"]
mod value_bindings;
pub use value_bindings::{
    CapabilityActionBinding, CapabilityDisclosureBinding, CapabilityPurposeBinding,
    CapabilityStatusBinding,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum CapabilityAction {
    Touch,
    Inspect,
    Disburse,
    RequestElevation,
    ApproveElevation,
    RevokeElevation,
    CompleteReview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum CapabilityPurpose {
    AccountMaintenance,
    Audit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityStatus {
    Active,
    Revoked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum CapabilityDisclosure {
    AccountActivity,
    PrivateLabel,
}

worth_query_capability_context!(pub CapabilityRequestContext in IdentityExecutionSchema);
worth_query_capability_context_entity_slot!(
    pub CapabilityRequestActorSlot in IdentityExecutionSchema,
    CapabilityRequestContext => CapabilityActionRecord
);
worth_query_capability_context_entity_slot!(
    pub CapabilityPriorActorSlot in IdentityExecutionSchema,
    CapabilityRequestContext => CapabilityActionRecord
);
worth_query_capability_provenance!(pub CapabilityProvenance in IdentityExecutionSchema);
worth_query_capability!(pub TouchAccountCapability in IdentityExecutionSchema);
worth_query_capability!(pub ComposedTouchAccountCapability in IdentityExecutionSchema);

/// Serialized by `touch_input_encoding`, which counts every encoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityTouchInput {
    pub account: String,
    pub action: CapabilityAction,
    pub purpose: CapabilityPurpose,
    pub disclosure: CapabilityDisclosure,
    pub related_account: String,
    pub request_record: String,
    pub prior_record: String,
    pub amount: u64,
    pub caller_time: u64,
}
worth_query_declaration::worth_query_portable_type!(CapabilityAction => "worth.query.test.execution.capability.action.v1");
worth_query_declaration::worth_query_portable_type!(CapabilityPurpose => "worth.query.test.execution.capability.purpose.v1");
worth_query_declaration::worth_query_portable_type!(CapabilityStatus => "worth.query.test.execution.capability.status.v1");
worth_query_declaration::worth_query_portable_type!(CapabilityDisclosure => "worth.query.test.execution.capability.disclosure.v1");
pub struct CapabilityTouchOperationInputBinding;

impl ApplicationStructuredValueBinding for CapabilityTouchOperationInputBinding {
    type Value = CapabilityTouchInput;

    const IDENTITY_NAME: &'static str = "worth.query.test.capability-touch-input.v1";

    fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        (value.caller_time != u64::MAX)
            .then_some(())
            .ok_or_else(|| {
                ApplicationValueValidationDenial::rejected(
                    Self::IDENTITY,
                    "reserved caller time is invalid",
                )
            })
    }
}

worth_query_operation!(pub CapabilityTouchOperation for IdentityExecutionSchema, input CapabilityTouchOperationInputBinding);
worth_query_declaration::worth_query_structured_value_binding!(pub ComposedCapabilityTouchOperationInputBinding for CapabilityTouchInput { identity: "worth.query.test.capability-touch-input.v1" });
worth_query_operation!(pub ComposedCapabilityTouchOperation for IdentityExecutionSchema, input ComposedCapabilityTouchOperationInputBinding);

impl ApplicationCapabilityRequest<IdentityExecutionSchema, TouchAccountCapability>
    for CapabilityTouchInput
{
    type Scope = Account;
    type Context = CapabilityRequestContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<IdentityExecutionSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        self.project_capability_request(ApplicationCapabilityRequestContext::new(
            CapabilityRequestContext::reference(),
        ))
    }
}

impl ApplicationCapabilityRequest<IdentityExecutionSchema, ComposedTouchAccountCapability>
    for CapabilityTouchInput
{
    type Scope = Account;
    type Context = CapabilityRequestContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<IdentityExecutionSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        self.project_capability_request(
            ApplicationCapabilityRequestContext::new(CapabilityRequestContext::reference())
                .entity(
                    CapabilityRequestActorSlot::reference(),
                    ApplicationCapabilityEntitySelector::new(
                        CapabilityActionRecordIdentity::at("CapabilityActionRecordIdentity"),
                        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                            self.request_record.clone(),
                        )
                        .expect("fixture request record identity must encode"),
                    )
                    .unwrap(),
                )
                .entity(
                    CapabilityPriorActorSlot::reference(),
                    ApplicationCapabilityEntitySelector::new(
                        CapabilityActionRecordIdentity::at("CapabilityActionRecordIdentity"),
                        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                            self.prior_record.clone(),
                        )
                        .expect("fixture prior record identity must encode"),
                    )
                    .unwrap(),
                ),
        )
    }
}

impl CapabilityTouchInput {
    fn project_capability_request(
        &self,
        context: ApplicationCapabilityRequestContext<
            IdentityExecutionSchema,
            CapabilityRequestContext,
        >,
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
            )
            .unwrap(),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(self.action)
                .expect("fixture capability action must encode"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(self.purpose)
                .expect("fixture capability purpose must encode"),
            context,
        )
        .related_entity(ApplicationCapabilityRelatedEntitySelector::new(
            CapabilityRelated::reference(),
            ApplicationCapabilityEntitySelector::new(
                AccountIdentity::at("AccountIdentity"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.related_account.clone(),
                )
                .expect("fixture related account identity must encode"),
            )
            .unwrap(),
        ))
        .field(
            ApplicationEncodedScalarValue::<CapabilityDisclosureBinding>::try_new(self.disclosure)
                .expect("fixture capability disclosure must encode"),
        )
        .magnitude(
            ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(self.amount)
                .expect("fixture capability magnitude must encode"),
        ))
    }
}

pub use crate::domain_computation::primary_graph::tests::fixture::schema_types::storage_model::{
    CapabilityActionField, CapabilityActionRecord, CapabilityActionRecordIdentity,
    CapabilityActionResource, CapabilityAmountField, CapabilityConflictingBeneficiary,
    CapabilityCustodian, CapabilityDelegationLimitField, CapabilityDisclosureField,
    CapabilityExplicitDeny, CapabilityGrant, CapabilityGrantee, CapabilityGrantor,
    CapabilityIdentity, CapabilityNotAfterField, CapabilityNotBeforeField, CapabilityParent,
    CapabilityPriorActor, CapabilityPurposeField, CapabilityRelated, CapabilityRequestActor,
    CapabilityResource, CapabilityStatusField, CapabilityWorkflowField,
};
