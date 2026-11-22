use worth_query_decl::facade::{worth_query_operation, worth_query_structured_value_binding};

use worth_query_host::facade::declaration::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationFieldScope, ApplicationMutationIntent,
        ApplicationMutationOutputContract, ApplicationMutationOutputRoleDescriptor,
        ApplicationMutationOutputRoleFamilyDescriptor, ApplicationQueryMutationSource,
        WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationOutputRole,
        WorthQueryExactlyOneOutput, WorthQueryPreserveOutput,
    },
    application_schema::{ApplicationPrincipalBindingRef, ApplicationSchemaDeclarationBuilder},
};

use crate::{
    model::{BankPrincipalId, PaymentId},
    proposals::BankIdempotencyKey,
    queries::PaymentDetailQuery,
    schema::{
        BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, ExternalPrincipalMapping,
        PaymentIdentityField, PaymentIntent, PaymentStatus, PaymentStatusField, Principal,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct RejectedPaymentAssessmentInput {
    pub payment: PaymentId,
    pub status: PaymentStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RejectedPaymentAssessmentPublished {
    pub status: PaymentStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectedPaymentAssessmentDenial {
    SourceChanged,
}

worth_query_structured_value_binding!(pub RejectedPaymentAssessmentInputBinding for RejectedPaymentAssessmentInput { identity: "worth.bank.rejected-payment-assessment.input.v1" });
worth_query_structured_value_binding!(pub RejectedPaymentAssessmentPublishedBinding for RejectedPaymentAssessmentPublished { identity: "worth.bank.rejected-payment-assessment.published.v1" });
worth_query_structured_value_binding!(pub RejectedPaymentAssessmentDenialBinding for RejectedPaymentAssessmentDenial { identity: "worth.bank.rejected-payment-assessment.denial.v1" });

worth_query_operation!(pub PublishRejectedPaymentAssessment for BankSchema, input RejectedPaymentAssessmentInputBinding;
    reads [PaymentIntent];
);
pub struct RejectedPaymentAssessmentOutputs;

/// The approved payment an assessment preserves.
pub struct RejectedPaymentAssessmentOutput;

impl WorthQueryApplicationOutputRole for RejectedPaymentAssessmentOutput {
    type Schema = BankSchema;
    type Contract = RejectedPaymentAssessmentOutputs;
    type Entity = PaymentIntent;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "assessment";
}

impl ApplicationMutationOutputContract<BankSchema> for RejectedPaymentAssessmentOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <RejectedPaymentAssessmentOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[];
}

pub struct RejectedPaymentAssessmentBinding;
type AssessmentScope = ApplicationMutationFieldScope<PaymentIdentityField>;

impl ApplicationMutationBinding<BankSchema> for RejectedPaymentAssessmentBinding {
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = RejectedPaymentAssessmentInput;
    type InputBinding = RejectedPaymentAssessmentInputBinding;
    type Result = RejectedPaymentAssessmentPublished;
    type ResultBinding = RejectedPaymentAssessmentPublishedBinding;
    type IdempotencyKey = BankIdempotencyKey;
    type Operation = PublishRejectedPaymentAssessment;
    type Decision = worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget<
        BankSchema,
        PaymentIntent,
    >;
    type Denial = RejectedPaymentAssessmentDenial;
    type DenialBinding = RejectedPaymentAssessmentDenialBinding;
    type Output = RejectedPaymentAssessmentOutputs;
    type ScopeBinding = AssessmentScope;
    type PrincipalBinding = BankPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = BankPrincipalId;
    type PrincipalIdentityBinding = BankPrincipalIdBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PaymentDetailQuery>;

    const IDENTITY: &'static str = "worth.bank.rejected-payment-assessment.binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.bank.rejected-payment-assessment.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.bank.rejected-payment-assessment.command.v1";

    fn scope_field() -> PaymentIdentityField {
        PaymentIdentityField::at("PaymentIdentityField")
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

impl ApplicationMutationIntent<BankSchema> for RejectedPaymentAssessmentInput {
    type Binding = RejectedPaymentAssessmentBinding;

    fn input(&self) -> &Self {
        self
    }

    fn scope_binding(&self) -> AssessmentScope {
        AssessmentScope::new(
            PaymentIdentityField::at("PaymentIdentityField"),
            self.payment,
        )
    }
}

pub(crate) fn install_rejected_payment_assessment(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation(
            PublishRejectedPaymentAssessment::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(PublishRejectedPaymentAssessment::reference(), 1_024)
        .operation_read_field(
            PublishRejectedPaymentAssessment::reference(),
            PaymentIdentityField::at("PaymentIdentityField"),
        )
        .operation_read_field(
            PublishRejectedPaymentAssessment::reference(),
            PaymentStatusField::at("PaymentStatusField"),
        )
        .application_mutation_binding::<RejectedPaymentAssessmentBinding>()
}
