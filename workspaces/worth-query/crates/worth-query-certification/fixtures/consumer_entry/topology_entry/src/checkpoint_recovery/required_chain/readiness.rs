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
    fn conditional_binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            ChainReadinessDomain,
            ChainReadinessOperation,
            ChainReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        PublishChain,
        ChainInput,
        ChainReadinessDomain,
        ChainReadinessOperation,
        ChainReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishChain::reference::<Schema>(),
            definition.reference(),
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
        declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        application_contribution::WorthQueryApplicationConditionalPackageContract,
        worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        let contract = operation_contract(declaration)?;
        let binding = Self::conditional_binding(contract.definition());
        Ok(contract.into_package_contract(binding.portable().clone()))
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let contract = operation_contract(installation.installed_schema().installed_declaration())?;
        let binding = Self::conditional_binding(contract.definition());

        let operation = installation
            .installed_schema()
            .installed_operation(PublishChain::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(ChainReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<ChainProducer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_contract(
    declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<
    application_contribution::WorthQueryOutputReadinessContract<
        ChainReadinessDomain,
        ChainReadinessOperation,
        ChainReadinessFamily,
    >,
    application_contribution::WorthQueryOutputReadinessContractDenial,
> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        crate::readiness::delivery_field(),
        domain::WorthQueryDomainOperationIdentity::new("consumed-chain-output-readiness", 1),
        "consumed-chain-output-ready",
        crate::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("consumed-chain-readiness").unwrap(),
        "consumed-chain-readiness-v1",
    )?
    .field_dependency(
        declaration,
        crate::readiness::delivery_field(),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
