// Region output readiness beside its shared real producer binding.
use super::runtime_api::primary_graph::{
    WorthQueryConditionalApplicationRuntimeInstallation,
    WorthQueryConditionalRuntimeInstallationDenial,
};
use super::runtime_domain as domain;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RegionOutputDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RegionOutputReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RegionOutputReadinessFamily;

super::conditional_node!(
    pub(super) RegionOutputReadyNode in RegionOutputDomain, RegionOutputReadinessOperation,
    RegionOutputReadinessFamily => operation "region-output-ready"
);

pub(super) struct RegionOutputReadiness<Schema, const REUSE: bool = false, const MODE: u8 = 0>(
    PhantomData<fn() -> Schema>,
);

impl<Schema: TopologyModelBinding, const REUSE: bool, const MODE: u8>
    RegionOutputReadiness<Schema, REUSE, MODE>
{
    fn conditional_binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            RegionOutputDomain,
            RegionOutputReadinessOperation,
            RegionOutputReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        TotalRegionOutput,
        RegionOutputInput,
        RegionOutputDomain,
        RegionOutputReadinessOperation,
        RegionOutputReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            TotalRegionOutput::reference::<Schema>(),
            definition.reference(),
        )
    }
}

impl<Schema: TopologyModelBinding, const REUSE: bool, const MODE: u8>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for RegionOutputReadiness<Schema, REUSE, MODE>
{
    type Configuration = ();
    type Installed = ();
    type Operation = TotalRegionOutput;
    const IDENTITY: &'static str = "worth.query.certification.region-output-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &[PRODUCER];

    fn package_contract(
        declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        application_contribution::WorthQueryApplicationConditionalPackageContract,
        application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        let contract = operation_contract(declaration)?;
        let binding = Self::conditional_binding(contract.definition());
        Ok(contract.into_package_contract(binding.portable().clone()))
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        let contract = operation_contract(installation.installed_schema().installed_declaration())?;
        let binding = Self::conditional_binding(contract.definition());

        let operation = installation
            .installed_schema()
            .installed_operation(TotalRegionOutput::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(RegionOutputReadyNode::reference())
            .unwrap();
        installation
            .bind_output_readiness::<RegionOutputProducer<Schema, REUSE, MODE>, _, _, _, _, _, _>(
                node, 0,
            )
    }
}

fn operation_contract(
    declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<
    application_contribution::WorthQueryOutputReadinessContract<
        RegionOutputDomain,
        RegionOutputReadinessOperation,
        RegionOutputReadinessFamily,
    >,
    application_contribution::WorthQueryOutputReadinessContractDenial,
> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        super::readiness::delivery_field(),
        domain::WorthQueryDomainOperationIdentity::new("region-output-readiness", 1),
        "region-output-ready",
        super::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("region-output-readiness").unwrap(),
        "region-output-readiness-v1",
    )?
    .field_dependency(
        declaration,
        super::readiness::delivery_field(),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}
