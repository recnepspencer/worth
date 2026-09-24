use super::*;

pub struct PlanarFinalPreserveReadiness<Schema>(PhantomData<fn() -> Schema>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarFinalPreserveReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarFinalPreserveReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarFinalPreserveReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub PlanarFinalPreserveReadyNode in PlanarFinalPreserveReadinessDomain,
    PlanarFinalPreserveReadinessOperation, PlanarFinalPreserveReadinessFamily
    => operation "planar-final-preserve-ready"
);

impl<Schema: TopologySchemaBinding> PlanarFinalPreserveReadiness<Schema> {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        PreserveFinalPlanarOutput,
        FinalPlanarPreserve,
        PlanarFinalPreserveReadinessDomain,
        PlanarFinalPreserveReadinessOperation,
        PlanarFinalPreserveReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PreserveFinalPlanarOutput::reference::<Schema>(),
            preserve_readiness_definition().reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for PlanarFinalPreserveReadiness<Schema>
{
    type Configuration = ();
    type Installed = ();
    type Operation = PreserveFinalPlanarOutput;

    const IDENTITY: &'static str = "worth.query.certification.planar-final-preserve-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.query.certification.planar-final-preserve-producer.v1"];

    fn package_contract(
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            preserve_readiness_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            PlanarFinalPreserveReadyNode::reference().node_identity(),
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
            .installed_operation(PreserveFinalPlanarOutput::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .unwrap()
            .bind_node(PlanarFinalPreserveReadyNode::reference())
            .unwrap();
        installation
            .bind_output_readiness::<PlanarFinalPreserveProducer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}

fn preserve_readiness_definition() -> domain::WorthQueryDomainOperationDefinition<
    PlanarFinalPreserveReadinessDomain,
    PlanarFinalPreserveReadinessOperation,
    PlanarFinalPreserveReadinessFamily,
> {
    let dependency = super::super::readiness::output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("planar-final-preserve-readiness", 1),
        "planar-final-preserve-ready",
        super::super::readiness::output_change_projection(),
        super::super::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("planar-final-preserve-readiness").unwrap(),
        128,
        128,
        "planar-final-preserve-readiness-v1",
    )
    .semantic_reads([super::super::readiness::output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("final planar preservation readiness declaration is canonical")
}
