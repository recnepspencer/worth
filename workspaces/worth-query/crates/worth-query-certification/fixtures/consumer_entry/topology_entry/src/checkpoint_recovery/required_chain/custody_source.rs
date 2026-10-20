//! Same native member set and equal declared input after a source-only sibling value edit.
use super::*;
use std::marker::PhantomData;
use worth_query_host::facade::application_contribution::*;
mod binding;
mod program;
mod readiness;
use binding::ValueSourceInput;
pub(super) use binding::{ValueSourceBinding, ValueSourceHandler};
pub(super) use program::{UpstreamRoot, ValueSourceProgram, ValueSourceRoot, ValueSourceSchema};
pub(super) use readiness::ValueSourceReadiness;
const APPLICABILITY: &[WorthQueryProducerApplicability] = &[
    WorthQueryProducerApplicability::new(
        "consumed-value-source",
        WorthQueryProducerLifecyclePosture::Initial,
    ),
    WorthQueryProducerApplicability::new(
        "consumed-value-source",
        WorthQueryProducerLifecyclePosture::Preserve,
    ),
];
pub(super) struct ValueSourceFamily;
impl<Schema: TopologySchemaBinding> WorthQueryProducerOutputFamily<Schema> for ValueSourceFamily {
    type Source = PlanarReadBinding<Schema>;
    type Entity = Body;
    const IDENTITY: &'static str = "worth.query.certification.consumed-value-source-family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(_: &PlanarReadResult) -> &'static str {
        "consumed-value-source"
    }
}
#[derive(Clone)]
pub(super) struct ValueSourceDemand(pub String);
impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputDemand<Schema>
    for ValueSourceDemand
{
    type OutputFamily = ValueSourceFamily;
    fn source_intent(&self) -> PlanarRead {
        PlanarRead {
            body_key: self.0.clone(),
        }
    }
}
pub(super) struct ValueSourceProducer<Schema>(PhantomData<fn() -> Schema>);
impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for ValueSourceProducer<Schema>
{
    type Operation = ValueSourceBinding<Schema>;
    type OutputFamily = ValueSourceFamily;
    type Provider = ValueSourceProvider;
    type OutputRole = PlanarAnchorOutput<Schema>;
    const IDENTITY: &'static str = "worth.query.certification.consumed-value-source-producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        ));
}
pub(super) struct ValueSourceProvider;
impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationProducerProvider<Schema, ValueSourceProducer<Schema>>
    for ValueSourceProvider
{
    const SEMANTIC_IDENTITY: &'static str =
        "worth.query.certification.consumed-value-source-provider.v1";
    fn operation_input(&self, source: &PlanarReadResult) -> ValueSourceInput {
        ValueSourceInput {
            scope_key: source.body_key.clone(),
            value: length(worth_query_consumer_values::PositiveLength::get(&source.y) + 1),
        }
    }
    fn idempotency_key(&self, _: &PlanarReadResult, identity: &[u8; 32]) -> u64 {
        planar_source_key(identity) ^ 0x9176_33a0
    }
    fn demand_resources(
        &self,
        _: &PlanarReadResult,
    ) -> Result<
        WorthQueryProducerDemandResources,
        worth_query_host::facade::application_contribution::WorthQueryProducerResourceQuoteDenial,
    > {
        Ok(planar_producer_resources())
    }
}

// Like mixed_retirement, this authored domain belongs only to its isolated
// schema. Ordinary checkpoint programs never acquire its conditional.
pub(super) fn declare<Schema: TopologySchemaBinding>(
    schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
        Schema,
    >,
) -> worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema> {
    if std::any::TypeId::of::<Schema>() == std::any::TypeId::of::<ValueSourceSchema>() {
        binding::declare(schema)
    } else {
        schema
    }
}
pub(super) fn contracts<Schema: TopologySchemaBinding>(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    if std::any::TypeId::of::<Schema>() != std::any::TypeId::of::<ValueSourceSchema>() {
        return Ok(());
    }
    contracts.producer::<ValueSourceProducer<Schema>>()?;
    contracts.conditional::<ValueSourceReadiness<Schema>>()?;
    Ok(())
}
pub(super) fn configure<Schema: TopologySchemaBinding>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    if std::any::TypeId::of::<Schema>() != std::any::TypeId::of::<ValueSourceSchema>() {
        return Ok(());
    }
    setup.handler::<ValueSourceBinding<Schema>, _>(ValueSourceHandler)?;
    setup.producer::<ValueSourceProducer<Schema>>(ValueSourceProvider)?;
    setup.conditional::<ValueSourceReadiness<Schema>>(())
}
mod installation;
pub(super) use installation::install;
