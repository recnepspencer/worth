//! Rejection delivery starts dormant; the admitted rejection publishes its status definition.
use worth_query_host::facade::{application_contribution, domain};

use super::{
    PublishRejectedPaymentAssessment, RejectedPaymentAssessmentInput,
    RejectedPaymentAssessmentProducer,
};
use crate::schema::{BankSchema, PaymentStatusField};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RejectedPaymentAssessmentReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RejectedPaymentAssessmentReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RejectedPaymentAssessmentReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub RejectedPaymentAssessmentReadyNode in RejectedPaymentAssessmentReadinessDomain,
    RejectedPaymentAssessmentReadinessOperation,
    RejectedPaymentAssessmentReadinessFamily => operation "rejected-payment-assessment-ready"
);

pub struct RejectedPaymentAssessmentReadiness;
type ReadinessContract = application_contribution::WorthQueryOutputReadinessContract<
    RejectedPaymentAssessmentReadinessDomain,
    RejectedPaymentAssessmentReadinessOperation,
    RejectedPaymentAssessmentReadinessFamily,
>;

impl RejectedPaymentAssessmentReadiness {
    fn conditional_binding(
        contract: &ReadinessContract,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        BankSchema,
        PublishRejectedPaymentAssessment,
        RejectedPaymentAssessmentInput,
        RejectedPaymentAssessmentReadinessDomain,
        RejectedPaymentAssessmentReadinessOperation,
        RejectedPaymentAssessmentReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishRejectedPaymentAssessment::reference(),
            contract.definition().reference(),
        )
    }
}

impl application_contribution::WorthQueryApplicationConditionalBinding<BankSchema>
    for RejectedPaymentAssessmentReadiness
{
    type Configuration = ();
    type Installed = worth_query_host::facade::primary_graph::WorthQueryConditionalClockHandle<
        BankSchema,
        RejectedPaymentAssessmentReadyNode,
        (),
    >;
    type Operation = PublishRejectedPaymentAssessment;
    const IDENTITY: &'static str = "worth.bank.rejected-payment-assessment.readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.bank.rejected-payment-assessment.producer.v1"];

    fn package_contract(
        declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        application_contribution::WorthQueryApplicationConditionalPackageContract,
        application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        let contract = operation_definition(declaration)?;
        let binding = Self::conditional_binding(&contract).portable().clone();
        Ok(contract.into_package_contract(binding))
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<
            '_,
            BankSchema,
        >,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<BankSchema>,
    ) -> Result<
        Self::Installed,
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let contract =
            operation_definition(installation.installed_schema().installed_declaration())?;
        let ordinal = contract.delivery_ordinal(
            installation.installed_schema(),
            &contract.definition().semantics().conditional_nodes[0],
            PaymentStatusField::at("PaymentStatusField"),
        )?;
        let operation = installation
            .installed_schema()
            .installed_operation(PublishRejectedPaymentAssessment::reference())
            .expect("the payment assessment publication operation is installed");
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(
                operation,
                &Self::conditional_binding(&contract),
            )
            .expect("the payment assessment conditional binding is installed")
            .bind_node(RejectedPaymentAssessmentReadyNode::reference())
            .expect("the payment assessment readiness node is installed");
        installation.bind_output_readiness_definition::<RejectedPaymentAssessmentProducer, _, _, _, _, _, _, _>(
            node, ordinal, std::sync::Arc::new(super::RejectedPaymentAssessmentPredicate::awaiting_rejection()),
        )
    }
}

fn operation_definition(
    declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<ReadinessContract, application_contribution::WorthQueryOutputReadinessContractDenial> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        PaymentStatusField::at("PaymentStatusField"),
        domain::WorthQueryDomainOperationIdentity::new("rejected-payment-assessment-readiness", 1),
        "rejected-payment-assessment-ready",
        super::assessment_readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("assessment").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("rejected-payment-assessment-readiness")
            .unwrap(),
        "rejected-payment-assessment-readiness-v1",
    )?
    .field_dependency(
        declaration,
        PaymentStatusField::at("PaymentStatusField"),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
