//! The producer of a region output: a demand for one scope's output runs the
//! region output operation over the set the scope's ordinate names, first to
//! create the output and then to keep it.

use std::marker::PhantomData;

use worth_query_consumer_values::PositiveLength;
use worth_query_host::facade::application_contribution::{
    self, WorthQueryApplicationOutputDemand, WorthQueryApplicationProducerBinding,
    WorthQueryApplicationProducerProvider, WorthQueryProducerApplicability,
    WorthQueryProducerDemandResources, WorthQueryProducerInputReuseContract,
    WorthQueryProducerInvariantRequirement, WorthQueryProducerLifecyclePosture,
    WorthQueryProducerOutputFamily,
};
use worth_query_host::facade::domain;
use worth_query_host::facade::primary_graph::{
    WorthQueryConditionalApplicationRuntimeInstallation,
    WorthQueryConditionalRuntimeInstallationDenial,
};

use super::region_output::{RegionOutputBinding, RegionOutputInput, TotalRegionOutput};
use super::*;

const APPLICABILITY: &[WorthQueryProducerApplicability] = &[
    WorthQueryProducerApplicability::new(
        "region-output",
        WorthQueryProducerLifecyclePosture::Initial,
    ),
    WorthQueryProducerApplicability::new(
        "region-output",
        WorthQueryProducerLifecyclePosture::Preserve,
    ),
];

pub(super) struct RegionOutputFamily;
impl<Schema: TopologySchemaBinding> WorthQueryProducerOutputFamily<Schema> for RegionOutputFamily {
    type Source = PlanarReadBinding<Schema>;
    type Entity = Body;
    const IDENTITY: &'static str = "worth.query.certification.region-output-family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(_: &PlanarReadResult) -> &'static str {
        "region-output"
    }
}

/// A demand for the region output of the scope it names.
#[derive(Clone)]
pub(super) struct RegionOutputDemand(pub(super) String);
impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputDemand<Schema>
    for RegionOutputDemand
{
    type OutputFamily = RegionOutputFamily;
    fn source_intent(&self) -> PlanarRead {
        PlanarRead {
            body_key: self.0.clone(),
        }
    }
}

const PRODUCER: &str = "worth.query.certification.region-output-producer.v1";

pub(super) struct RegionOutputProducer<Schema, const REUSE: bool = false, const MODE: u8 = 0>(
    PhantomData<fn() -> Schema>,
);
impl<Schema: TopologySchemaBinding, const REUSE: bool, const MODE: u8>
    WorthQueryApplicationProducerBinding<Schema> for RegionOutputProducer<Schema, REUSE, MODE>
{
    type Operation = RegionOutputBinding<Schema>;
    type OutputFamily = RegionOutputFamily;
    type Provider = RegionOutputProvider<MODE>;
    const IDENTITY: &'static str = PRODUCER;
    type OutputRole = PlanarAnchorOutput<Schema>;
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> = if REUSE {
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            application_contribution::WorthQueryDecisionContextDependencies::NONE,
        ))
    } else {
        None
    };
}

/// The scope's ordinate names the set its output totals: an even ordinate
/// the even set, an odd one the odd set.
pub(super) struct RegionOutputProvider<const MODE: u8 = 0>;
impl<Schema: TopologySchemaBinding, const REUSE: bool, const MODE: u8>
    WorthQueryApplicationProducerProvider<Schema, RegionOutputProducer<Schema, REUSE, MODE>>
    for RegionOutputProvider<MODE>
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.region-output-provider.v1";
    fn operation_input(&self, source: &PlanarReadResult) -> RegionOutputInput {
        let entries = if PositiveLength::get(&source.y).is_multiple_of(2) {
            "even"
        } else {
            "odd"
        };
        RegionOutputInput {
            scope_key: source.body_key.clone(),
            entries: entries.to_owned(),
        }
    }
    fn idempotency_key(&self, _: &PlanarReadResult, identity: &[u8; 32]) -> u64 {
        let key = planar_source_key(identity) ^ 0x9176_3c0b;
        if MODE == 4 {
            key | (3 << 62)
        } else {
            key
        }
    }
    fn demand_resources(&self, _: &PlanarReadResult) -> WorthQueryProducerDemandResources {
        if MODE == 1 {
            // The wide-key decision may disclose every declared Native fact.
            // Reserve its finite projection bound, including own-write rebasing.
            WorthQueryProducerDemandResources::new(
                4096,
                super::region_output::DECISION_FACT_BUDGET * 512,
            )
        } else {
            planar_producer_resources()
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RegionOutputDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RegionOutputReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RegionOutputReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
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
    let dependency = crate::readiness::output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("region-output-readiness", 1),
        "region-output-ready",
        crate::readiness::output_change_projection(),
        crate::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("region-output-readiness").unwrap(),
        128,
        128,
        "region-output-readiness-v1",
    )
    .semantic_reads([crate::readiness::output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("the region output's readiness declaration is canonical")
}
