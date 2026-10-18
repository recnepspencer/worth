//! An ordinary complete consumer root beside the existing upstream root.
use super::super::*;
use super::{binding::PublishValueSource, ValueSourceDemand};
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    WorthQueryRequiredOutputConnectionDenial,
};
worth_query_decl::facade::worth_query_application! {
 pub(in crate::checkpoint_recovery::required_chain) ValueSourceSchema {
  owner: "worth.query.certification.consumed-value-source", version: (1,0), contributions: [TopologyContribution],
 }
}
impl TopologySchemaBinding for ValueSourceSchema {}
pub(in crate::checkpoint_recovery::required_chain) struct ValueFeature;
pub(in crate::checkpoint_recovery::required_chain) struct ValueInput;
pub(in crate::checkpoint_recovery::required_chain) struct ValueOutput;
pub(in crate::checkpoint_recovery::required_chain) struct SourceToValue;
impl ApplicationFeature<ValueSourceSchema> for ValueFeature {
    type Inputs = ApplicationFeatureInputList<ValueInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.certification.consumed-value-source-feature.v1";
}
impl ApplicationInputPort<ValueSourceSchema, ValueFeature> for ValueInput {
    type Value = PlanarReadResultBinding;
    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}
impl ApplicationOutputPort<ValueSourceSchema, ValueFeature> for ValueOutput {
    type Value = PlanarReadResultBinding;
    const IDENTITY: &'static str = "value";
}
impl ApplicationConnectionIdentity for SourceToValue {
    const IDENTITY: &'static str = "worth.query.certification.consumed-value-source-root.v1";
}
impl ApplicationOccurrenceConnectionBinding<ValueSourceSchema, PlanarSourceFeature, ValueFeature>
    for SourceToValue
{
}
impl WorthQueryApplicationRequiredOutputConnection<ValueSourceSchema> for SourceToValue {
    type Source = PlanarSourceAdjustmentBinding<ValueSourceSchema>;
    type Demand = ValueSourceDemand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        source: &PlanarSourceAdjustment,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(ValueSourceDemand(source.scope_key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<ValueSourceSchema, SourceToValue>
    for PlanarSourceAdjustmentBinding<ValueSourceSchema>
{
    fn demand_from_source(
        source: &PlanarSourceAdjustment,
    ) -> Result<ValueSourceDemand, WorthQueryRequiredOutputConnectionDenial> {
        <SourceToValue as WorthQueryApplicationRequiredOutputConnection<ValueSourceSchema>>::demand_from_source(source)
    }
}
pub(in crate::checkpoint_recovery::required_chain) type ValueSourceRoot = ApplicationOutputGraph<
    ApplicationConnectionRef<
        ValueSourceSchema,
        PlanarSourceFeature,
        PlanarBodyOutput,
        ValueFeature,
        ValueInput,
        SourceToValue,
    >,
    ApplicationOutputLeaf,
>;
pub(in crate::checkpoint_recovery::required_chain) struct ValueSourceProgram;
impl ApplicationProgramDefinition<ValueSourceSchema> for ValueSourceProgram {
    type Contributions = <ValueSourceSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<(UpstreamRoot, ValueSourceRoot)>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<ValueSourceSchema, PositivePlanarTurn>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.consumed-value-source-program.v1",
    );
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<ValueSourceSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .native_action::<PlanarSourceAdjustmentBinding<ValueSourceSchema>>()
                .finish_action()
                .native_action::<PlanarEditBinding<ValueSourceSchema>>()
                .finish_action()
                .finish(),
            super::super::output_feature_spec_for::<ValueSourceSchema>(),
            ApplicationFeatureSpec::root::<ValueSourceSchema, PlanarFinalOutputFeature>()
                .provides::<PlanarFinalBodyOutput>()
                .conditional_operation::<PublishFinalPlanarOutput>()
                .conditional_operation::<PreserveFinalPlanarOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<ValueSourceSchema, PlanarAlternateFinalOutputFeature>()
                .provides::<PlanarAlternateFinalBodyOutput>()
                .conditional_operation::<PublishAlternatePlanarOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<ValueSourceSchema, ValueFeature>()
                .provides::<ValueOutput>()
                .conditional_operation::<PublishValueSource>()
                .finish(),
        ]
    }
}

pub(in crate::checkpoint_recovery::required_chain) type UpstreamRoot = ApplicationOutputGraph<
    ApplicationConnectionRef<
        ValueSourceSchema,
        PlanarSourceFeature,
        PlanarBodyOutput,
        PlanarOutputFeature,
        PlanarBodyInput,
        PlanarSourceToOutputConnection,
    >,
    (
        ApplicationOutputEdge<
            ApplicationConnectionRef<
                ValueSourceSchema,
                PlanarOutputFeature,
                PlanarDerivedBodyOutput,
                PlanarFinalOutputFeature,
                PlanarDerivedBodyInput,
                PlanarOutputToFinalConnection,
            >,
            ApplicationOutputLeaf,
        >,
        ApplicationOutputEdge<
            ApplicationConnectionRef<
                ValueSourceSchema,
                PlanarOutputFeature,
                PlanarDerivedBodyOutput,
                PlanarAlternateFinalOutputFeature,
                PlanarAlternateDerivedBodyInput,
                PlanarOutputToAlternateFinalConnection,
            >,
            ApplicationOutputLeaf,
        >,
    ),
>;
