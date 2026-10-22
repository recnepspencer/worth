//! The source action owns its actual transitive output tree and complete projections.
use super::*;
use worth_query_decl::facade::application_program::*;

type SourceConnection<S> = ApplicationConnectionRef<
    S,
    PlanarSourceFeature,
    PlanarBodyOutput,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToOutputConnection,
>;
type FinalConnection<S> = ApplicationConnectionRef<
    S,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToFinalConnection,
>;
type AlternateConnection<S> = ApplicationConnectionRef<
    S,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarAlternateFinalOutputFeature,
    PlanarAlternateDerivedBodyInput,
    PlanarOutputToAlternateFinalConnection,
>;
pub type PlanarSourceFeatureRoot<S> = ApplicationOutputGraph<
    SourceConnection<S>,
    (
        ApplicationOutputEdge<FinalConnection<S>, ApplicationOutputLeaf>,
        ApplicationOutputEdge<AlternateConnection<S>, ApplicationOutputLeaf>,
    ),
>;
pub struct SourceDelivery<S>(PhantomData<fn() -> S>);
impl<S: TopologySchemaBinding> ApplicationActionDelivery<S, PlanarSourceAdjustmentBinding<S>>
    for SourceDelivery<S>
{
    type Mode = ApplicationFixedDeliveryMode<PlanarSourceFeatureRoot<S>, ApplicationNoProjection>;
    const IDENTITY: &'static str = "required-feature.source-delivery.v1";
}
impl<S: TopologySchemaBinding> ApplicationFixedActionDelivery<S, PlanarSourceAdjustmentBinding<S>>
    for SourceDelivery<S>
{
    type Root = PlanarSourceFeatureRoot<S>;
    type Projection = ApplicationNoProjection;
}

worth_query_host::facade::worth_query_application_contribution! {
    pub feature PlanarSourceFeatureDefinition<Schema: TopologySchemaBinding>
        for PlanarSourceFeature => PlanarSourceRuntime {
        schema: |schema| { schema };
        provides(PlanarBodyOutput);
        action adjust(PlanarSourceAdjustmentBinding<Schema>, SourceDelivery<Schema>, PlanarSourceAdjustmentHandler) {
            operation: AdjustPlanarSource::reference::<Schema>().definition().no_external_effect().no_aftermath().finish();
        };
    }
}
