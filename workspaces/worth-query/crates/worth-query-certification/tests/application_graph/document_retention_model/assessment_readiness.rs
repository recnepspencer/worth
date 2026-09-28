//! Native readiness contract for the installed retention-assessment producer.

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
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
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
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            operation_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            RetentionAssessmentReadyNode::reference().node_identity(),
        )
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
        installation.bind_output_readiness::<RetentionAssessmentProducer, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    RetentionAssessmentReadinessDomain,
    RetentionAssessmentReadinessOperation,
    RetentionAssessmentReadinessFamily,
> {
    let dependency = retention_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("retention-assessment-readiness", 1),
        "retention-assessment-ready",
        retention_projection(),
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("assessment").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("retention-assessment-readiness").unwrap(),
        32,
        32,
        "retention-assessment-readiness-v1",
    )
    .semantic_reads([retention_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("the document assessment readiness declaration is canonical")
}

fn retention_dependency() -> domain::WorthQuerySemanticTruthDependency {
    domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        document_contract(),
        retention_mask(),
        domain::AspectBinding::EntityField {
            field: domain::FieldKey::new("DocumentRetentionField").unwrap(),
        },
        domain::WorthQuerySemanticLocality::SourceRecord,
        [domain::AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap()
}

fn document_contract() -> domain::AspectContract {
    let field = |name, scalar| {
        domain::FieldDeclaration::new(
            domain::FieldKey::new(name).unwrap(),
            scalar,
            domain::FieldRequirement::Required,
            domain::AbsenceLaw::Required,
            domain::AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap()
    };
    domain::AspectContract::struct_aspect(
        domain::AspectKey::new("DocumentFacts").unwrap(),
        domain::AspectIdentity(0x9175_0103),
        domain::AspectContractRevision(1),
        domain::StructAspectShape::new([
            field("DocumentIdentityField", domain::ScalarAspectType::String),
            field("DocumentRetentionField", domain::ScalarAspectType::UInt64),
        ])
        .unwrap(),
    )
}

fn retention_mask() -> domain::AspectMask<domain::ProjectionMask> {
    domain::AspectMask::new([domain::CanonicalFieldPath::single(
        domain::FieldKey::new("DocumentRetentionField").unwrap(),
    )])
}

fn retention_projection() -> domain::WorthQueryOperationNativeProjectionContract {
    domain::WorthQueryOperationNativeProjectionContract::new(document_contract(), retention_mask())
        .unwrap()
}

fn canonical_query() -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new("Document").unwrap())
        .project(AspectFieldSelector::new("DocumentFacts", "DocumentRetentionField").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(
                "DocumentFacts",
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
