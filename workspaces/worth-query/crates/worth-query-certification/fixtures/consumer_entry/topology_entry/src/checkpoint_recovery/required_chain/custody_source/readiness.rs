use super::{
    binding::{PublishValueSource, ValueSourceInput},
    TopologySchemaBinding, ValueSourceProducer,
};
use worth_query_host::facade::{application_contribution, domain};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::checkpoint_recovery::required_chain) struct ValueSourceReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::checkpoint_recovery::required_chain) struct ValueSourceReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::checkpoint_recovery::required_chain) struct ValueSourceReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub(in crate::checkpoint_recovery::required_chain) ValueSourceReadyNode in ValueSourceReadinessDomain, ValueSourceReadinessOperation,
    ValueSourceReadinessFamily => operation "consumed-value-source-output-ready"
);

pub(in crate::checkpoint_recovery::required_chain) struct ValueSourceReadiness<Schema>(
    std::marker::PhantomData<fn() -> Schema>,
);

impl<Schema: TopologySchemaBinding> ValueSourceReadiness<Schema> {
    fn conditional_binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            ValueSourceReadinessDomain,
            ValueSourceReadinessOperation,
            ValueSourceReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        PublishValueSource,
        ValueSourceInput,
        ValueSourceReadinessDomain,
        ValueSourceReadinessOperation,
        ValueSourceReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishValueSource::reference::<Schema>(),
            definition.reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for ValueSourceReadiness<Schema>
{
    type Configuration = ();
    type Installed = ();
    type Operation = PublishValueSource;
    const IDENTITY: &'static str = "worth.query.certification.consumed-value-source-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &["worth.query.certification.consumed-value-source-producer.v1"];

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
            .installed_operation(PublishValueSource::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(ValueSourceReadyNode::reference())
            .unwrap();
        installation.bind_output_readiness::<ValueSourceProducer<Schema>, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_contract(
    declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<
    application_contribution::WorthQueryOutputReadinessContract<
        ValueSourceReadinessDomain,
        ValueSourceReadinessOperation,
        ValueSourceReadinessFamily,
    >,
    application_contribution::WorthQueryOutputReadinessContractDenial,
> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        crate::readiness::delivery_field(),
        domain::WorthQueryDomainOperationIdentity::new("consumed-value-source-output-readiness", 1),
        "consumed-value-source-output-ready",
        crate::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("consumed-value-source-readiness").unwrap(),
        "consumed-value-source-readiness-v1",
    )?
    .field_dependency(
        declaration,
        crate::readiness::delivery_field(),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
