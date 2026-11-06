use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};
use worth_query_host::facade::{application_contribution, domain};

use super::{
    ApprovedPaymentAssessmentInput, ApprovedPaymentAssessmentProducer,
    PublishApprovedPaymentAssessment,
};
use crate::schema::{BankSchema, PaymentStatusField};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovedPaymentAssessmentReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovedPaymentAssessmentReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovedPaymentAssessmentReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub ApprovedPaymentAssessmentReadyNode in ApprovedPaymentAssessmentReadinessDomain,
    ApprovedPaymentAssessmentReadinessOperation,
    ApprovedPaymentAssessmentReadinessFamily => operation "approved-payment-assessment-ready"
);

pub struct ApprovedPaymentAssessmentReadiness;
type ReadinessContract = application_contribution::WorthQueryOutputReadinessContract<
    ApprovedPaymentAssessmentReadinessDomain,
    ApprovedPaymentAssessmentReadinessOperation,
    ApprovedPaymentAssessmentReadinessFamily,
>;

impl ApprovedPaymentAssessmentReadiness {
    fn conditional_binding(
        contract: &ReadinessContract,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        BankSchema,
        PublishApprovedPaymentAssessment,
        ApprovedPaymentAssessmentInput,
        ApprovedPaymentAssessmentReadinessDomain,
        ApprovedPaymentAssessmentReadinessOperation,
        ApprovedPaymentAssessmentReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishApprovedPaymentAssessment::reference(),
            contract.definition().reference(),
        )
    }
}

impl application_contribution::WorthQueryApplicationConditionalBinding<BankSchema>
    for ApprovedPaymentAssessmentReadiness
{
    type Configuration = ();
    type Installed = ();
    type Operation = PublishApprovedPaymentAssessment;
    const IDENTITY: &'static str = "worth.bank.approved-payment-assessment.readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.bank.approved-payment-assessment.producer.v1"];

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
        (),
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
            .installed_operation(PublishApprovedPaymentAssessment::reference())
            .expect("the payment assessment publication operation is installed");
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(
                operation,
                &Self::conditional_binding(&contract),
            )
            .expect("the payment assessment conditional binding is installed")
            .bind_node(ApprovedPaymentAssessmentReadyNode::reference())
            .expect("the payment assessment readiness node is installed");
        installation.bind_output_readiness::<ApprovedPaymentAssessmentProducer, _, _, _, _, _, _>(
            node, ordinal,
        )
    }
}

fn operation_definition(
    declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<ReadinessContract, application_contribution::WorthQueryOutputReadinessContractDenial> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        PaymentStatusField::at("PaymentStatusField"),
        domain::WorthQueryDomainOperationIdentity::new("approved-payment-assessment-readiness", 1),
        "approved-payment-assessment-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("assessment").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("approved-payment-assessment-readiness")
            .unwrap(),
        "approved-payment-assessment-readiness-v1",
    )?
    .field_dependency(
        declaration,
        PaymentStatusField::at("PaymentStatusField"),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}

fn canonical_query() -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new("PaymentIntent").unwrap())
        .project(
            AspectFieldSelector::new(
                PaymentStatusField::at("PaymentStatusField").aspect(),
                "PaymentStatusField",
            )
            .unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(
                PaymentStatusField::at("PaymentStatusField").aspect(),
                "PaymentStatusField",
                "payment_status",
            )
            .unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
