use super::{
    binding::{ChainInput, PublishChain},
    producer::ChainProducer,
    TopologySchemaBinding,
};
use worth_query_host::facade::{application_contribution, domain};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ChainReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ChainReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ChainReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub(super) ChainReadyNode in ChainReadinessDomain, ChainReadinessOperation,
    ChainReadinessFamily => operation "consumed-chain-output-ready"
);

pub(super) struct ChainReadiness<Schema>(std::marker::PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ChainReadiness<Schema> {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        PublishChain,
        ChainInput,
        ChainReadinessDomain,
        ChainReadinessOperation,
        ChainReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishChain::reference::<Schema>(),
            operation_definition().reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for ChainReadiness<Schema>
{
    type Configuration = ();
    type Installed = ();
    type Operation = PublishChain;
    const IDENTITY: &'static str = "worth.query.certification.consumed-chain-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.query.certification.consumed-chain-producer.v1"];

    fn package_contract(
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            operation_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            ChainReadyNode::reference().node_identity(),
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
            .installed_operation(PublishChain::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .unwrap()
            .bind_node(ChainReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<ChainProducer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    ChainReadinessDomain,
    ChainReadinessOperation,
    ChainReadinessFamily,
> {
    let dependency = crate::readiness::output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("consumed-chain-output-readiness", 1),
        "consumed-chain-output-ready",
        crate::readiness::output_change_projection(),
        crate::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("consumed-chain-readiness").unwrap(),
        128,
        128,
        "consumed-chain-readiness-v1",
    )
    .semantic_reads([crate::readiness::output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("chain readiness declaration is canonical")
}
