use worth_query_decl::facade::{
    worth_query_operation, worth_query_operation_reads, worth_query_structured_value_binding,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationOutputDemand, WorthQueryApplicationProducerBinding,
        WorthQueryApplicationProducerProvider, WorthQueryProducerApplicability,
        WorthQueryProducerDemandResources, WorthQueryProducerInvariantRequirement,
        WorthQueryProducerLifecyclePosture, WorthQueryProducerOutputFamily,
        WorthQueryWorkflowAssessmentOutputFamily, WorthQueryWorkflowAssessmentPosture,
    },
    declaration::{
        application_operation::{
            ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
            ApplicationCandidateResourceCeiling, ApplicationMutationBinding,
            ApplicationMutationFieldScope, ApplicationMutationIntent,
            ApplicationMutationOutputContract, ApplicationMutationOutputPosture,
            ApplicationMutationOutputRoleDescriptor, ApplicationMutationOutputRoleFamilyDescriptor,
            ApplicationQueryMutationSource,
        },
        application_schema::{
            ApplicationFieldRef, ApplicationPrincipalBindingRef,
            ApplicationSchemaDeclarationBuilder, EqualityPredicate, NoApplicationUnit, ReadOnly,
        },
    },
};

use crate::{
    model::{BankPrincipalId, PaymentId},
    proposals::{BankIdempotencyKey, CanonicalProposalPayload},
    queries::{PaymentDetailQuery, PaymentDetailQueryBinding, PaymentDetailRequest},
    reads::PaymentSummary,
    schema::{
        BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, ExternalPrincipalMapping,
        PaymentIdentity, PaymentIdentityField, PaymentIntent, PaymentStatus, PaymentStatusField,
        Principal,
    },
};

use super::super::client_key_identity;

const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "approved-payment-assessment",
    WorthQueryProducerLifecyclePosture::Initial,
);
const PRESERVE: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "approved-payment-assessment",
    WorthQueryProducerLifecyclePosture::Preserve,
);
const APPLICABILITY: &[WorthQueryProducerApplicability] = &[INITIAL, PRESERVE];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovedPaymentAssessmentInput {
    pub payment: PaymentId,
    pub status: PaymentStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovedPaymentAssessmentPublished {
    pub status: PaymentStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovedPaymentAssessmentDenial {
    SourceChanged,
}

worth_query_structured_value_binding!(pub ApprovedPaymentAssessmentInputBinding for ApprovedPaymentAssessmentInput { identity: "worth.bank.approved-payment-assessment.input.v1" });
worth_query_structured_value_binding!(pub ApprovedPaymentAssessmentPublishedBinding for ApprovedPaymentAssessmentPublished { identity: "worth.bank.approved-payment-assessment.published.v1" });
worth_query_structured_value_binding!(pub ApprovedPaymentAssessmentDenialBinding for ApprovedPaymentAssessmentDenial { identity: "worth.bank.approved-payment-assessment.denial.v1" });
worth_query_operation!(pub PublishApprovedPaymentAssessment for BankSchema, input ApprovedPaymentAssessmentInputBinding);
worth_query_operation_reads!(PublishApprovedPaymentAssessment => [PaymentIntent, PaymentIdentityField, PaymentStatusField]);

pub struct ApprovedPaymentAssessmentOutputs;

impl ApplicationMutationOutputContract<BankSchema> for ApprovedPaymentAssessmentOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[ApplicationMutationOutputRoleDescriptor::for_entity::<
            BankSchema,
            PaymentIntent,
        >(
            "assessment", ApplicationMutationOutputPosture::Preserve
        )];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[];
}

pub struct ApprovedPaymentAssessmentBinding;
type AssessmentScope = ApplicationMutationFieldScope<
    BankSchema,
    PaymentIntent,
    PaymentIdentity,
    PaymentIdentityField,
    PaymentId,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationMutationBinding<BankSchema> for ApprovedPaymentAssessmentBinding {
    type Input = ApprovedPaymentAssessmentInput;
    type InputBinding = ApprovedPaymentAssessmentInputBinding;
    type Result = ApprovedPaymentAssessmentPublished;
    type ResultBinding = ApprovedPaymentAssessmentPublishedBinding;
    type IdempotencyKey = BankIdempotencyKey;
    type Operation = PublishApprovedPaymentAssessment;
    type Decision = worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget<
        BankSchema,
        PaymentIntent,
    >;
    type Denial = ApprovedPaymentAssessmentDenial;
    type DenialBinding = ApprovedPaymentAssessmentDenialBinding;
    type Output = ApprovedPaymentAssessmentOutputs;
    type ScopeBinding = AssessmentScope;
    type PrincipalBinding = BankPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = BankPrincipalId;
    type PrincipalIdentityBinding = BankPrincipalIdBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PaymentDetailQuery>;

    const IDENTITY: &'static str = "worth.bank.approved-payment-assessment.binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.bank.approved-payment-assessment.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.bank.approved-payment-assessment.command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 0, 0),
            ApplicationCandidateResourceCeiling::bounded(1_024, 512),
        );

    fn idempotency_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
        client_key_identity(key)
    }

    fn input_identity(input: &ApprovedPaymentAssessmentInput) -> [u8; 32] {
        *CanonicalProposalPayload::new("approved-payment-assessment")
            .text("payment", &input.payment.canonical_text())
            .text("status", &format!("{:?}", input.status))
            .derive_identity()
            .bytes()
    }

    fn scope_field() -> ApplicationFieldRef<
        BankSchema,
        PaymentIntent,
        PaymentIdentity,
        PaymentIdentityField,
        PaymentId,
        ReadOnly,
        EqualityPredicate,
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

impl ApplicationMutationIntent<BankSchema> for ApprovedPaymentAssessmentInput {
    type Binding = ApprovedPaymentAssessmentBinding;

    fn input(&self) -> &Self {
        self
    }

    fn scope_binding(&self) -> AssessmentScope {
        AssessmentScope::new(PaymentIdentityField::reference(), self.payment)
    }
}

pub struct ApprovedPaymentAssessmentOutputFamily;

impl WorthQueryProducerOutputFamily<BankSchema> for ApprovedPaymentAssessmentOutputFamily {
    type Source = PaymentDetailQueryBinding;
    const IDENTITY: &'static str = "worth.bank.approved-payment-assessment.output.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;

    fn profile_kind(_: &PaymentSummary) -> &'static str {
        "approved-payment-assessment"
    }
}

impl WorthQueryWorkflowAssessmentOutputFamily<BankSchema>
    for ApprovedPaymentAssessmentOutputFamily
{
    fn assessment_posture(row: &PaymentSummary) -> WorthQueryWorkflowAssessmentPosture {
        if row.status() == PaymentStatus::ApprovalRequired {
            WorthQueryWorkflowAssessmentPosture::Passing
        } else {
            WorthQueryWorkflowAssessmentPosture::Failing
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovedPaymentAssessmentDemand {
    payment: PaymentId,
}

impl ApprovedPaymentAssessmentDemand {
    pub const fn new(payment: PaymentId) -> Self {
        Self { payment }
    }
}

impl WorthQueryApplicationOutputDemand<BankSchema> for ApprovedPaymentAssessmentDemand {
    type OutputFamily = ApprovedPaymentAssessmentOutputFamily;

    fn source_intent(&self) -> PaymentDetailRequest {
        PaymentDetailRequest::new(self.payment)
    }
}

pub struct ApprovedPaymentAssessmentProducer;
pub struct ApprovedPaymentAssessmentProvider;

impl WorthQueryApplicationProducerBinding<BankSchema> for ApprovedPaymentAssessmentProducer {
    type Operation = ApprovedPaymentAssessmentBinding;
    type OutputFamily = ApprovedPaymentAssessmentOutputFamily;
    type Provider = ApprovedPaymentAssessmentProvider;
    const IDENTITY: &'static str = "worth.bank.approved-payment-assessment.producer.v1";
    const OUTPUT_ROLE: &'static str = "assessment";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
}

impl WorthQueryApplicationProducerProvider<BankSchema, ApprovedPaymentAssessmentProducer>
    for ApprovedPaymentAssessmentProvider
{
    const SEMANTIC_IDENTITY: &'static str = "worth.bank.approved-payment-assessment.provider.v1";

    fn operation_input(&self, source: &PaymentSummary) -> ApprovedPaymentAssessmentInput {
        ApprovedPaymentAssessmentInput {
            payment: source.id(),
            status: source.status(),
        }
    }

    fn idempotency_key(
        &self,
        _: &PaymentSummary,
        source_identity: &[u8; 32],
    ) -> BankIdempotencyKey {
        let mut value = "approved-payment-assessment:".to_owned();
        for byte in source_identity {
            use std::fmt::Write as _;
            write!(value, "{byte:02x}").expect("writing into a string cannot fail");
        }
        BankIdempotencyKey::new(value).expect("assessment identity fits the bank key contract")
    }

    fn demand_resources(&self, _: &PaymentSummary) -> WorthQueryProducerDemandResources {
        WorthQueryProducerDemandResources::new(512, 1_024)
    }
}

pub(crate) fn install_approved_payment_assessment(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation(
            PublishApprovedPaymentAssessment::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(PublishApprovedPaymentAssessment::reference(), 256)
        .operation_projection_work_budget(PublishApprovedPaymentAssessment::reference(), 1_024)
        .operation_read_field(
            PublishApprovedPaymentAssessment::reference(),
            PaymentIdentityField::reference(),
        )
        .operation_read_field(
            PublishApprovedPaymentAssessment::reference(),
            PaymentStatusField::reference(),
        )
        .application_mutation_binding::<ApprovedPaymentAssessmentBinding>()
}
