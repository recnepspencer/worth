//! Region output readiness beside its shared real producer binding.
include!("output_producer/binding.rs");
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

impl<Schema: TopologySchemaBinding, const REUSE: bool, const MODE: u8>
    RegionOutputReadiness<Schema, REUSE, MODE>
{
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        TotalRegionOutput,
        RegionOutputInput,
        RegionOutputDomain,
        RegionOutputReadinessOperation,
        RegionOutputReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            TotalRegionOutput::reference::<Schema>(),
            operation_definition().reference(),
        )
    }
}

impl<Schema: TopologySchemaBinding, const REUSE: bool, const MODE: u8>
    application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for RegionOutputReadiness<Schema, REUSE, MODE>
{
    type Configuration = ();
    type Installed = ();
    type Operation = TotalRegionOutput;
    const IDENTITY: &'static str = "worth.query.certification.region-output-readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &[PRODUCER];

    fn package_contract(
    ) -> application_contribution::WorthQueryApplicationConditionalPackageContract {
        application_contribution::WorthQueryApplicationConditionalPackageContract::new(
            operation_definition().into_portable(),
            Self::conditional_binding().portable().clone(),
            RegionOutputReadyNode::reference().node_identity(),
        )
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        let operation = installation
            .installed_schema()
            .installed_operation(TotalRegionOutput::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .unwrap()
            .bind_node(RegionOutputReadyNode::reference())
            .unwrap();
        installation
            .bind_output_readiness::<RegionOutputProducer<Schema, REUSE, MODE>, _, _, _, _, _, _>(
                node, 0,
            )
    }
}

fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    RegionOutputDomain,
    RegionOutputReadinessOperation,
    RegionOutputReadinessFamily,
> {
    let dependency = super::readiness::output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("region-output-readiness", 1),
        "region-output-ready",
        super::readiness::output_change_projection(),
        super::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("region-output-readiness").unwrap(),
        128,
        128,
        "region-output-readiness-v1",
    )
    .semantic_reads([super::readiness::output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("the region output's readiness declaration is canonical")
}
