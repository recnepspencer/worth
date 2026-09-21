//! Native readiness contract for the installed part-assessment producer.

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
    assessment_output::{PartAssessmentInput, PartAssessmentProducer, PublishPartAssessment},
    schema::BoundedDimensionSchema,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PartAssessmentReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PartAssessmentReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PartAssessmentReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub PartAssessmentReadyNode in PartAssessmentReadinessDomain,
    PartAssessmentReadinessOperation,
    PartAssessmentReadinessFamily => operation "part-assessment-ready"
);

pub struct PartAssessmentReadiness;

impl PartAssessmentReadiness {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        BoundedDimensionSchema,
        PublishPartAssessment,
        PartAssessmentInput,
        PartAssessmentReadinessDomain,
        PartAssessmentReadinessOperation,
        PartAssessmentReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishPartAssessment::reference(),
            operation_definition().reference(),
        )
    }
}

impl application_contribution::WorthQueryApplicationConditionalBinding<BoundedDimensionSchema>
    for PartAssessmentReadiness
{
    type Configuration = ();
    type Installed = ();
    type Operation = PublishPartAssessment;

    const IDENTITY: &'static str = "worth.query.certification.part-assessment.readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.query.certification.part-assessment.producer.v1"];

    fn package_contract(
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            operation_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            PartAssessmentReadyNode::reference().node_identity(),
        )
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<
            '_,
            BoundedDimensionSchema,
        >,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<BoundedDimensionSchema>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let operation = installation
            .installed_schema()
            .installed_operation(PublishPartAssessment::reference())
            .expect("the assessment publication operation is installed");
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .expect("the assessment conditional binding is installed")
            .bind_node(PartAssessmentReadyNode::reference())
            .expect("the assessment readiness node is installed");
        installation.bind_output_readiness::<PartAssessmentProducer, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    PartAssessmentReadinessDomain,
    PartAssessmentReadinessOperation,
    PartAssessmentReadinessFamily,
> {
    let dependency = dimension_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("part-assessment-readiness", 1),
        "part-assessment-ready",
        dimension_projection(),
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("assessment").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("part-assessment-readiness").unwrap(),
        32,
        32,
        "part-assessment-readiness-v1",
    )
    .semantic_reads([dimension_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("the part assessment readiness declaration is canonical")
}

fn dimension_dependency() -> domain::WorthQuerySemanticTruthDependency {
    domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        part_contract(),
        dimension_mask(),
        domain::AspectBinding::EntityField {
            field: domain::FieldKey::new("PartDimensionField").unwrap(),
        },
        domain::WorthQuerySemanticLocality::SourceRecord,
        [domain::AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap()
}

fn part_contract() -> domain::AspectContract {
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
        domain::AspectKey::new("PartFacts").unwrap(),
        domain::AspectIdentity(0x9175_0103),
        domain::AspectContractRevision(1),
        domain::StructAspectShape::new([
            field("PartIdentityField", domain::ScalarAspectType::String),
            field("PartDimensionField", domain::ScalarAspectType::UInt64),
        ])
        .unwrap(),
    )
}

fn dimension_mask() -> domain::AspectMask<domain::ProjectionMask> {
    domain::AspectMask::new([domain::CanonicalFieldPath::single(
        domain::FieldKey::new("PartDimensionField").unwrap(),
    )])
}

fn dimension_projection() -> domain::WorthQueryOperationNativeProjectionContract {
    domain::WorthQueryOperationNativeProjectionContract::new(part_contract(), dimension_mask())
        .unwrap()
}

fn canonical_query() -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new("Part").unwrap())
        .project(AspectFieldSelector::new("PartFacts", "PartDimensionField").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new("PartFacts", "PartDimensionField", "part_dimension")
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
