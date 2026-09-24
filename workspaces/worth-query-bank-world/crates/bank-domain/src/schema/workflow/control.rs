use worth_query_decl::facade::{
    application_capability::{
        ApplicationCapabilityEntitySelector, ApplicationCapabilityRequest,
        ApplicationCapabilityRequestContext, ApplicationCapabilityRequestProjection,
        ApplicationCapabilityRequestProjectionDenial,
    },
    application_operation::{
        ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
        ApplicationCandidateResourceCeiling, ApplicationCapabilityMutationBinding,
        ApplicationMutationBinding, ApplicationMutationFieldScope, ApplicationMutationIntent,
        NoApplicationMutationOutputs, NoApplicationMutationSource,
    },
    application_schema::{
        ApplicationEncodedScalarValue, ApplicationFieldRef, ApplicationPrincipalBindingRef,
        NoApplicationUnit, ReadOnly, StringApplicationValueBinding,
    },
    worth_query_operation, worth_query_operation_reads,
};

use crate::{
    model::{BankPrincipalId, PaymentId},
    proposals::{BankIdempotencyKey, BankProposalDenial},
};

use super::super::{
    approval_input_identity, client_key_identity, ApprovePayment, ApprovePaymentInputBinding,
    ApprovePaymentMutationBinding, BankPrincipalBinding, BankPrincipalIdBinding, BankSchema,
    ExternalPrincipalMapping, PaymentDecisionDenialBinding, PaymentDecisionResult,
    PaymentDecisionResultBinding, PaymentIdentity, PaymentIdentityField, PaymentIntent, Principal,
};
use super::{
    ApprovedBusinessPaymentAdvance, ApprovedBusinessPaymentApproval,
    ApprovedBusinessPaymentAuthoring, ApprovedBusinessPaymentInstanceStart,
};

pub struct ApprovedBusinessPaymentControlContext;

impl worth_query_decl::facade::portable_identity::WorthQueryPortableType
    for ApprovedBusinessPaymentControlContext
{
    const PORTABLE_TYPE_NAME: &'static str =
        "worth.bank.approved-business-payment-control-context.v1";
}

impl worth_query_decl::facade::application_capability::ApplicationCapabilityContextMarkerIdentity
    for ApprovedBusinessPaymentControlContext
{
    type Schema = BankSchema;
    const IDENTIFIER: &'static str = "ApprovedBusinessPaymentControlContext";
}

impl ApprovedBusinessPaymentControlContext {
    pub const fn reference(
    ) -> worth_query_decl::facade::application_capability::ApplicationCapabilityContextRef<
        BankSchema,
        Self,
    > {
        worth_query_decl::facade::application_capability::ApplicationCapabilityContextRef::from_declaration()
    }
}

type PaymentWorkflowScope = ApplicationMutationFieldScope<
    BankSchema,
    PaymentIntent,
    PaymentIdentity,
    PaymentIdentityField,
    PaymentId,
    ReadOnly,
    NoApplicationUnit,
>;

macro_rules! payment_workflow_control {
    (
        $operation:ident, $binding:ident, $intent:ident, $capability:ident,
        $binding_identity:literal, $handler_identity:literal, $command_identity:literal
    ) => {
        worth_query_operation!(
            pub $operation for BankSchema,
            input ApprovePaymentInputBinding
        );
        worth_query_operation_reads!($operation => [PaymentIdentityField]);

        pub struct $binding;

        impl ApplicationMutationBinding<BankSchema> for $binding {
            type Input = ApprovePayment;
            type InputBinding = ApprovePaymentInputBinding;
            type Result = PaymentDecisionResult;
            type ResultBinding = PaymentDecisionResultBinding;
            type IdempotencyKey = BankIdempotencyKey;
            type Operation = $operation;
            type Decision = ();
            type Denial = BankProposalDenial;
            type DenialBinding = PaymentDecisionDenialBinding;
            type Output = NoApplicationMutationOutputs;
            type ScopeBinding = PaymentWorkflowScope;
            type PrincipalBinding = BankPrincipalBinding;
            type Mapping = ExternalPrincipalMapping;
            type Principal = Principal;
            type PrincipalIdentity = BankPrincipalId;
            type PrincipalIdentityBinding = BankPrincipalIdBinding;
            type SourceExpectation = NoApplicationMutationSource;

            const IDENTITY: &'static str = $binding_identity;
            const HANDLER_IDENTITY: &'static str = $handler_identity;
            const IDEMPOTENCY_IDENTITY: &'static str = $command_identity;
            const REQUIRES_APPLICATION_PROGRAM: bool = true;
            const CANDIDATES: ApplicationCandidateRequirements =
                ApplicationCandidateRequirements::fixed_shape(
                    ApplicationCandidateCardinalityCeiling::fixed(64, 0, 128, 2, 512, 0),
                    ApplicationCandidateResourceCeiling::bounded(
                        2 * 1_024 * 1_024,
                        1_048_576,
                    ),
                );

            fn idempotency_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
                client_key_identity(key)
            }

            fn input_identity(input: &ApprovePayment) -> [u8; 32] {
                approval_input_identity(input)
            }

            fn scope_field() -> ApplicationFieldRef<
                BankSchema,
                PaymentIntent,
                PaymentIdentity,
                PaymentIdentityField,
                PaymentId,
                ReadOnly,
                worth_query_decl::facade::application_schema::EqualityPredicate,
                NoApplicationUnit,
            > {
                PaymentIdentityField::reference()
            }

            fn principal_binding() -> ApplicationPrincipalBindingRef<
                BankSchema,
                BankPrincipalBinding,
                ExternalPrincipalMapping,
                Principal,
                BankPrincipalId,
                BankPrincipalIdBinding,
            > {
                BankPrincipalBinding::reference()
            }
        }

        impl ApplicationCapabilityMutationBinding<BankSchema> for $binding {
            type Capability = $capability;
        }

        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $intent {
            pub input: ApprovePayment,
        }

        impl ApplicationMutationIntent<BankSchema> for $intent {
            type Binding = $binding;

            fn input(&self) -> &ApprovePayment {
                &self.input
            }

            fn scope_binding(&self) -> PaymentWorkflowScope {
                PaymentWorkflowScope::new(PaymentIdentityField::reference(), self.input.payment)
            }
        }

        impl ApplicationCapabilityRequest<BankSchema, $capability> for ApprovePayment {
            type Scope = PaymentIntent;
            type Context = ApprovedBusinessPaymentControlContext;

            fn capability_request(
                &self,
            ) -> Result<
                ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
                ApplicationCapabilityRequestProjectionDenial,
            > {
                Ok(ApplicationCapabilityRequestProjection::new(
                    ApplicationCapabilityEntitySelector::new(
                        PaymentIdentityField::reference(),
                        crate::schema::encoded_bank_value(self.payment),
                    ),
                    encoded("manage-approved-business-payment-workflow"),
                    encoded("business-payment-approval"),
                    ApplicationCapabilityRequestContext::new(
                        ApprovedBusinessPaymentControlContext::reference(),
                    ),
                ))
            }
        }
    };
}

payment_workflow_control!(
    ApprovedBusinessPaymentApprovalOperation,
    ApprovedBusinessPaymentApprovalBinding,
    ApprovedBusinessPaymentApprovalIntent,
    ApprovedBusinessPaymentApproval,
    "worth.bank.approved-business-payment-approval.binding.v1",
    "worth.bank.approved-business-payment-approval.handler.v1",
    "worth.bank.approved-business-payment-approval.command.v1"
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovedBusinessPaymentApplyIntent {
    pub input: ApprovePayment,
}

impl ApplicationMutationIntent<BankSchema> for ApprovedBusinessPaymentApplyIntent {
    type Binding = ApprovePaymentMutationBinding;

    fn input(&self) -> &ApprovePayment {
        &self.input
    }

    fn scope_binding(&self) -> PaymentWorkflowScope {
        PaymentWorkflowScope::new(PaymentIdentityField::reference(), self.input.payment)
    }
}
payment_workflow_control!(
    ApprovedBusinessPaymentAuthoringOperation,
    ApprovedBusinessPaymentAuthoringBinding,
    ApprovedBusinessPaymentAuthoringIntent,
    ApprovedBusinessPaymentAuthoring,
    "worth.bank.approved-business-payment-authoring.binding.v1",
    "worth.bank.approved-business-payment-authoring.handler.v1",
    "worth.bank.approved-business-payment-authoring.command.v1"
);
payment_workflow_control!(
    ApprovedBusinessPaymentInstanceStartOperation,
    ApprovedBusinessPaymentInstanceStartBinding,
    ApprovedBusinessPaymentInstanceStartIntent,
    ApprovedBusinessPaymentInstanceStart,
    "worth.bank.approved-business-payment-instance-start.binding.v1",
    "worth.bank.approved-business-payment-instance-start.handler.v1",
    "worth.bank.approved-business-payment-instance-start.command.v1"
);
payment_workflow_control!(
    ApprovedBusinessPaymentAdvanceOperation,
    ApprovedBusinessPaymentAdvanceBinding,
    ApprovedBusinessPaymentAdvanceIntent,
    ApprovedBusinessPaymentAdvance,
    "worth.bank.approved-business-payment-advance.binding.v1",
    "worth.bank.approved-business-payment-advance.handler.v1",
    "worth.bank.approved-business-payment-advance.command.v1"
);

fn encoded(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned())
        .expect("approved-payment workflow vocabulary is valid")
}
