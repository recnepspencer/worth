use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityEntitySelector, ApplicationCapabilityGovernedInputIdentity,
        ApplicationCapabilityRelatedEntitySelector, ApplicationCapabilityRequest,
        ApplicationCapabilityRequestContext, ApplicationCapabilityRequestProjection,
        ApplicationCapabilityRequestProjectionDenial,
    },
    application_schema::{
        ApplicationEncodedScalarValue, ApplicationStructuredValueBinding,
        ApplicationValueValidationDenial, StringApplicationValueBinding,
        U64ApplicationValueBinding,
    },
};
use worth_query_declaration::{
    worth_query_aspect, worth_query_capability, worth_query_capability_context,
    worth_query_capability_context_entity_slot, worth_query_capability_provenance,
    worth_query_entity, worth_query_field, worth_query_operation, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_relation,
};

use super::super::{Account, AccountIdentity, AccountLabel, IdentityExecutionSchema, Principal};
use super::governed_input::CapabilityGovernedInputIdentity;

#[path = "value_bindings.rs"]
mod value_bindings;
pub use value_bindings::{
    CapabilityActionBinding, CapabilityDisclosureBinding, CapabilityPurposeBinding,
    CapabilityStatusBinding,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityAction {
    Touch,
    Inspect,
    Disburse,
    RequestElevation,
    ApproveElevation,
    RevokeElevation,
    CompleteReview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityPurpose {
    AccountMaintenance,
    Audit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityStatus {
    Active,
    Revoked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityDisclosure {
    AccountActivity,
    PrivateLabel,
}

worth_query_entity!(pub CapabilityGrant for IdentityExecutionSchema);
worth_query_aspect!(pub CapabilityFacts for IdentityExecutionSchema, CapabilityGrant; identity = AspectIdentity(0x91611036), revision = AspectContractRevision(1),);
worth_query_field!(
    pub CapabilityIdentity for IdentityExecutionSchema, CapabilityGrant, CapabilityFacts:
    String => StringApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub CapabilityActionField for IdentityExecutionSchema, CapabilityGrant, CapabilityFacts:
    CapabilityAction => CapabilityActionBinding, read_only, no_equality
);
worth_query_field!(
    pub CapabilityPurposeField for IdentityExecutionSchema, CapabilityGrant, CapabilityFacts:
    CapabilityPurpose => CapabilityPurposeBinding, read_only, no_equality
);
worth_query_field!(
    pub CapabilityDisclosureField for IdentityExecutionSchema, CapabilityGrant, CapabilityFacts:
    CapabilityDisclosure => CapabilityDisclosureBinding, read_only, no_equality
);
worth_query_field!(
    pub CapabilityAmountField for IdentityExecutionSchema, CapabilityGrant, CapabilityFacts:
    u64 => U64ApplicationValueBinding, read_write, no_equality
);
worth_query_field!(
    pub CapabilityStatusField for IdentityExecutionSchema, CapabilityGrant, CapabilityFacts:
    CapabilityStatus => CapabilityStatusBinding, read_write, no_equality
);
worth_query_field!(
    pub CapabilityWorkflowField for IdentityExecutionSchema, CapabilityGrant, CapabilityFacts:
    String => StringApplicationValueBinding, read_write, no_equality
);
worth_query_field!(
    pub CapabilityNotBeforeField for IdentityExecutionSchema, CapabilityGrant, CapabilityFacts:
    u64 => U64ApplicationValueBinding, read_write, no_equality
);
worth_query_field!(
    pub CapabilityNotAfterField for IdentityExecutionSchema, CapabilityGrant, CapabilityFacts:
    u64 => U64ApplicationValueBinding, read_write, no_equality
);
worth_query_field!(
    pub CapabilityDelegationLimitField for IdentityExecutionSchema, CapabilityGrant, CapabilityFacts:
    u64 => U64ApplicationValueBinding, read_write, no_equality
);
worth_query_entity!(pub CapabilityActionRecord for IdentityExecutionSchema);
worth_query_aspect!(pub CapabilityActionRecordFacts for IdentityExecutionSchema,
    CapabilityActionRecord; identity = AspectIdentity(0x91611037), revision = AspectContractRevision(1),);
worth_query_field!(
    pub CapabilityActionRecordIdentity for IdentityExecutionSchema,
    CapabilityActionRecord, CapabilityActionRecordFacts:
    String => StringApplicationValueBinding, read_only, equality
);
worth_query_relation!(
    pub CapabilityGrantee in IdentityExecutionSchema,
    Principal => CapabilityGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityGrantor in IdentityExecutionSchema,
    Principal => CapabilityGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityCustodian in IdentityExecutionSchema,
    Principal => CapabilityGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityResource in IdentityExecutionSchema,
    CapabilityGrant => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityRelated in IdentityExecutionSchema,
    CapabilityGrant => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityParent in IdentityExecutionSchema,
    CapabilityGrant => CapabilityGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityExplicitDeny in IdentityExecutionSchema,
    Principal => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityConflictingBeneficiary in IdentityExecutionSchema,
    Principal => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityRequestActor in IdentityExecutionSchema,
    Principal => CapabilityActionRecord; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityPriorActor in IdentityExecutionSchema,
    Principal => CapabilityActionRecord; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub CapabilityActionResource in IdentityExecutionSchema,
    CapabilityActionRecord => Account; integrity = same_context_unbounded_retain_dangling);
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
    pub governed_input_identity: CapabilityGovernedInputIdentity,
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
worth_query_operation_reads!(CapabilityTouchOperation => [AccountLabel]);
worth_query_operation_writes!(CapabilityTouchOperation => [AccountLabel]);
worth_query_operation_reads!(ComposedCapabilityTouchOperation => [AccountLabel]);
worth_query_operation_writes!(ComposedCapabilityTouchOperation => [AccountLabel]);

impl ApplicationCapabilityRequest<IdentityExecutionSchema, TouchAccountCapability>
    for CapabilityTouchInput
{
    type Scope = Account;
    type Context = CapabilityRequestContext;

    fn governed_input_identity(&self) -> Option<ApplicationCapabilityGovernedInputIdentity> {
        self.governed_input_identity.materialize(self.amount)
    }

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

    fn governed_input_identity(&self) -> Option<ApplicationCapabilityGovernedInputIdentity> {
        self.governed_input_identity.materialize(self.amount)
    }

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
                        CapabilityActionRecordIdentity::reference(),
                        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                            self.request_record.clone(),
                        )
                        .expect("fixture request record identity must encode"),
                    ),
                )
                .entity(
                    CapabilityPriorActorSlot::reference(),
                    ApplicationCapabilityEntitySelector::new(
                        CapabilityActionRecordIdentity::reference(),
                        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                            self.prior_record.clone(),
                        )
                        .expect("fixture prior record identity must encode"),
                    ),
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
                AccountIdentity::reference(),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.account.clone(),
                )
                .expect("fixture account identity must encode"),
            ),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(self.action)
                .expect("fixture capability action must encode"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(self.purpose)
                .expect("fixture capability purpose must encode"),
            context,
        )
        .related_entity(ApplicationCapabilityRelatedEntitySelector::new(
            CapabilityRelated::reference(),
            ApplicationCapabilityEntitySelector::new(
                AccountIdentity::reference(),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.related_account.clone(),
                )
                .expect("fixture related account identity must encode"),
            ),
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
