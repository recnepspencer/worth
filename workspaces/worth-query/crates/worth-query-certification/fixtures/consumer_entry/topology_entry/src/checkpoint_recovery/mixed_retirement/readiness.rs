use super::{
    producer::ReplacementProducer, ReplacePlanarVertex, TopologySchemaBinding, VertexReplacement,
};
use worth_query_host::facade::{application_contribution, domain};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ReplacementReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ReplacementReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ReplacementReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub(super) ReplacementReadyNode in ReplacementReadinessDomain, ReplacementReadinessOperation,
    ReplacementReadinessFamily => operation "mixed-replacement-output-ready"
);

pub(super) struct ReplacementReadiness<Schema>(std::marker::PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ReplacementReadiness<Schema> {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        ReplacePlanarVertex,
        VertexReplacement,
        ReplacementReadinessDomain,
        ReplacementReadinessOperation,
        ReplacementReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            ReplacePlanarVertex::reference::<Schema>(),
            operation_definition().reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for ReplacementReadiness<Schema>
{
    type Configuration = ();
    type Installed = ();
    type Operation = ReplacePlanarVertex;
    const IDENTITY: &'static str = "worth.query.certification.mixed-replacement-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.query.certification.mixed-replacement-producer.v1"];

    fn package_contract(
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            operation_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            ReplacementReadyNode::reference().node_identity(),
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
            .installed_operation(ReplacePlanarVertex::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .unwrap()
            .bind_node(ReplacementReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<ReplacementProducer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    ReplacementReadinessDomain,
    ReplacementReadinessOperation,
    ReplacementReadinessFamily,
> {
    let dependency = crate::readiness::output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("mixed-replacement-output-readiness", 1),
        "mixed-replacement-output-ready",
        crate::readiness::output_change_projection(),
        crate::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("replacement").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("mixed-replacement-readiness").unwrap(),
        128,
        128,
        "mixed-replacement-readiness-v1",
    )
    .semantic_reads([crate::readiness::output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("replacement readiness declaration is canonical")
}
