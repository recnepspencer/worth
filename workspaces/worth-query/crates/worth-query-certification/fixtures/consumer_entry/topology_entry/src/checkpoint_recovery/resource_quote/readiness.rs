use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};
use worth_query_host::facade::{application_contribution, domain};

use super::{Input, Produce, Producer, TopologySchemaBinding};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountedReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountedReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountedReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub CountedReadyNode in CountedReadinessDomain, CountedReadinessOperation,
    CountedReadinessFamily => operation "counted-output-ready"
);

pub struct Readiness<Schema>(std::marker::PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> Readiness<Schema> {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        Produce,
        Input,
        CountedReadinessDomain,
        CountedReadinessOperation,
        CountedReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            Produce::reference::<Schema>(),
            operation_definition().reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for Readiness<Schema>
{
    type Configuration = ();
    type Installed = ();
    type Operation = Produce;

    const IDENTITY: &'static str = "worth.query.certification.counted-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &["quote-count.producer"];

    fn package_contract(
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            operation_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            CountedReadyNode::reference().node_identity(),
        )
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let operation = installation
            .installed_schema()
            .installed_operation(Produce::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .unwrap()
            .bind_node(CountedReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<Producer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}
pub(super) fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    CountedReadinessDomain,
    CountedReadinessOperation,
    CountedReadinessFamily,
> {
    let dependency = output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("counted-output-readiness", 1),
        "counted-output-ready",
        output_change_projection(),
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("counted-readiness").unwrap(),
        128,
        128,
        "counted-readiness-v1",
    )
    .semantic_reads([output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("planar readiness declaration is canonical")
}

pub(super) fn output_change_dependency() -> domain::WorthQuerySemanticTruthDependency {
    domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        output_contract(),
        output_change_mask(),
        domain::AspectBinding::EntityField {
            field: domain::FieldKey::new("QuoteOutputFacts").unwrap(),
        },
        domain::WorthQuerySemanticLocality::SourceRecord,
        [domain::AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap()
}

fn output_contract() -> domain::AspectContract {
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
        domain::AspectKey::new("QuoteOutputFacts").unwrap(),
        domain::AspectIdentity(0x9174_10e2),
        domain::AspectContractRevision(1),
        domain::StructAspectShape::new([field("CountedTotal", domain::ScalarAspectType::UInt64)])
            .unwrap(),
    )
}

fn output_change_mask() -> domain::AspectMask<domain::ProjectionMask> {
    domain::AspectMask::new([domain::CanonicalFieldPath::single(
        domain::FieldKey::new("CountedTotal").unwrap(),
    )])
}

pub(super) fn output_change_projection() -> domain::WorthQueryOperationNativeProjectionContract {
    domain::WorthQueryOperationNativeProjectionContract::new(
        output_contract(),
        output_change_mask(),
    )
    .unwrap()
}

pub(super) fn canonical_query(
) -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("Body").unwrap())
        .project(AspectFieldSelector::new("QuoteOutputFacts", "CountedTotal").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("QuoteOutputFacts", "CountedTotal", "total").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
