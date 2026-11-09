//! Native readiness contract for the installed retention-assessment producer.
use super::schema::DocumentRetentionField;

pub use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};
pub use worth_query_host::facade::{application_contribution, domain};

pub use super::{
    assessment_output::{
        PublishRetentionAssessment, RetentionAssessmentInput, RetentionAssessmentProducer,
    },
    schema::DocumentRetentionSchema,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionAssessmentReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionAssessmentReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionAssessmentReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub RetentionAssessmentReadyNode in RetentionAssessmentReadinessDomain,
    RetentionAssessmentReadinessOperation,
    RetentionAssessmentReadinessFamily => operation "retention-assessment-ready"
);

pub struct RetentionAssessmentReadiness;

impl RetentionAssessmentReadiness {
    pub fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        DocumentRetentionSchema,
        PublishRetentionAssessment,
        RetentionAssessmentInput,
        RetentionAssessmentReadinessDomain,
        RetentionAssessmentReadinessOperation,
        RetentionAssessmentReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishRetentionAssessment::reference(),
            operation_definition().reference(),
        )
    }
}

impl application_contribution::WorthQueryApplicationConditionalBinding<DocumentRetentionSchema>
    for RetentionAssessmentReadiness
{
    type Configuration = ();
    type Installed = ();
    type Operation = PublishRetentionAssessment;

    const IDENTITY: &'static str = "worth.query.certification.retention-assessment.readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.query.certification.retention-assessment.producer.v1"];

    fn package_contract(
        _declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        application_contribution::WorthQueryApplicationConditionalPackageContract,
        worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        let contract = readiness_contract(_declaration)?;
        Ok(contract.into_package_contract(Self::conditional_binding().portable().clone()))
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<
            '_,
            DocumentRetentionSchema,
        >,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<DocumentRetentionSchema>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let operation = installation
            .installed_schema()
            .installed_operation(PublishRetentionAssessment::reference())
            .expect("the assessment publication operation is installed");
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .expect("the assessment conditional binding is installed")
            .bind_node(RetentionAssessmentReadyNode::reference())
            .expect("the assessment readiness node is installed");
        let contract =
            readiness_contract(installation.installed_schema().installed_declaration()).unwrap();
        let ordinal = contract
            .delivery_ordinal(
                installation.installed_schema(),
                node.declaration(),
                DocumentRetentionField::at("DocumentRetentionField"),
            )
            .unwrap();
        installation
            .bind_output_readiness::<RetentionAssessmentProducer, _, _, _, _, _, _>(node, ordinal)
    }
}

type Contract = application_contribution::WorthQueryOutputReadinessContract<
    RetentionAssessmentReadinessDomain,
    RetentionAssessmentReadinessOperation,
    RetentionAssessmentReadinessFamily,
>;
pub fn readiness_contract(
    declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<Contract, application_contribution::WorthQueryOutputReadinessContractDenial> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        DocumentRetentionField::at("DocumentRetentionField"),
        domain::WorthQueryDomainOperationIdentity::new("retention-assessment-readiness", 1),
        "retention-assessment-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("assessment").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("retention-assessment-readiness").unwrap(),
        "retention-assessment-readiness-v1",
    )?
    .field_dependency(
        declaration,
        DocumentRetentionField::at("DocumentRetentionField"),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
pub fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    RetentionAssessmentReadinessDomain,
    RetentionAssessmentReadinessOperation,
    RetentionAssessmentReadinessFamily,
> {
    readiness_contract(DocumentRetentionSchema::declaration().unwrap().erased())
        .unwrap()
        .definition()
        .clone()
}

pub fn canonical_query(
) -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("Document").unwrap())
        .project(
            AspectFieldSelector::new(
                DocumentRetentionField::at("DocumentRetentionField").aspect(),
                "DocumentRetentionField",
            )
            .unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(
                DocumentRetentionField::at("DocumentRetentionField").aspect(),
                "DocumentRetentionField",
                "document_retention",
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
