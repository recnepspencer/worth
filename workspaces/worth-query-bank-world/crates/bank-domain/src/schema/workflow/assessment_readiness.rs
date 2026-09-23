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
use crate::schema::BankSchema;

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

impl ApprovedPaymentAssessmentReadiness {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        BankSchema,
        PublishApprovedPaymentAssessment,
        ApprovedPaymentAssessmentInput,
        ApprovedPaymentAssessmentReadinessDomain,
        ApprovedPaymentAssessmentReadinessOperation,
        ApprovedPaymentAssessmentReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishApprovedPaymentAssessment::reference(),
            operation_definition().reference(),
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
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            operation_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            ApprovedPaymentAssessmentReadyNode::reference().node_identity(),
        )
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
        let operation = installation
            .installed_schema()
            .installed_operation(PublishApprovedPaymentAssessment::reference())
            .expect("the payment assessment publication operation is installed");
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .expect("the payment assessment conditional binding is installed")
            .bind_node(ApprovedPaymentAssessmentReadyNode::reference())
            .expect("the payment assessment readiness node is installed");
        installation
            .bind_output_readiness::<ApprovedPaymentAssessmentProducer, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    ApprovedPaymentAssessmentReadinessDomain,
    ApprovedPaymentAssessmentReadinessOperation,
    ApprovedPaymentAssessmentReadinessFamily,
> {
    let dependency = payment_status_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("approved-payment-assessment-readiness", 1),
        "approved-payment-assessment-ready",
        payment_status_projection(),
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("assessment").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("approved-payment-assessment-readiness")
            .unwrap(),
        32,
        32,
        "approved-payment-assessment-readiness-v1",
    )
    .semantic_reads([payment_status_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("the payment assessment readiness declaration is canonical")
}

fn payment_status_dependency() -> domain::WorthQuerySemanticTruthDependency {
    domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        payment_state_contract(),
        payment_status_mask(),
        domain::AspectBinding::EntityField {
            field: domain::FieldKey::new("PaymentStatusField").unwrap(),
        },
        domain::WorthQuerySemanticLocality::SourceRecord,
        [domain::AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap()
}

fn payment_state_contract() -> domain::AspectContract {
    let status = domain::FieldDeclaration::new(
        domain::FieldKey::new("PaymentStatusField").unwrap(),
        domain::ScalarAspectType::String,
        domain::FieldRequirement::Required,
        domain::AbsenceLaw::Required,
        domain::AspectEvolutionPolicy::AdditiveFieldsAllowed,
    )
    .unwrap();
    domain::AspectContract::struct_aspect(
        domain::AspectKey::new("PaymentState").unwrap(),
        domain::AspectIdentity(0x9161_1017),
        domain::AspectContractRevision(1),
        domain::StructAspectShape::new([status]).unwrap(),
    )
}

fn payment_status_mask() -> domain::AspectMask<domain::ProjectionMask> {
    domain::AspectMask::new([domain::CanonicalFieldPath::single(
        domain::FieldKey::new("PaymentStatusField").unwrap(),
    )])
}

fn payment_status_projection() -> domain::WorthQueryOperationNativeProjectionContract {
    domain::WorthQueryOperationNativeProjectionContract::new(
        payment_state_contract(),
        payment_status_mask(),
    )
    .unwrap()
}

fn canonical_query() -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new("PaymentIntent").unwrap())
        .project(AspectFieldSelector::new("PaymentState", "PaymentStatusField").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new("PaymentState", "PaymentStatusField", "payment_status")
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
