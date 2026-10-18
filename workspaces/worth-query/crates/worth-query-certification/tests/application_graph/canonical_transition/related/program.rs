//! Program-owned source action and the real derived producer output root.
use super::{edit, producer, query, schema::*};
use worth_query_host::facade::{declaration::application_program::*, primary_graph::*};
pub struct Program;
pub struct SourceFeature;
pub struct DerivedFeature;
pub struct SourcePort;
pub struct DerivedInput;
pub struct Connection;
pub struct DerivedDelivery;
pub type Root = ApplicationOutputGraph<
    ApplicationConnectionRef<
        CanonicalSchema,
        SourceFeature,
        SourcePort,
        DerivedFeature,
        DerivedInput,
        Connection,
    >,
    ApplicationOutputLeaf,
>;
impl ApplicationFeature<CanonicalSchema> for SourceFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "canonical-transition.source.feature.v1";
}
impl ApplicationFeature<CanonicalSchema> for DerivedFeature {
    type Inputs = ApplicationFeatureInputList<DerivedInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "canonical-transition.derived.feature.v1";
}
impl ApplicationOutputPort<CanonicalSchema, SourceFeature> for SourcePort {
    type Value = query::ResultBinding;
    const IDENTITY: &'static str = "canonical-transition.source";
}
impl ApplicationInputPort<CanonicalSchema, DerivedFeature> for DerivedInput {
    type Value = query::ResultBinding;
    const IDENTITY: &'static str = "canonical-transition.derived-input";
    const REQUIRED: bool = true;
}
impl ApplicationOccurrenceConnectionBinding<CanonicalSchema, SourceFeature, DerivedFeature>
    for Connection
{
}
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "canonical-transition.required-derived.v1";
}
impl WorthQueryApplicationRequiredOutputConnection<CanonicalSchema> for Connection {
    type Source = edit::EditBinding;
    type Demand = producer::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &edit::Input,
    ) -> Result<producer::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(producer::Demand)
    }
}
impl WorthQueryApplicationRequiredOutputSource<CanonicalSchema, Connection> for edit::EditBinding {
    fn demand_from_source(
        source: &edit::Input,
    ) -> Result<producer::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Connection as WorthQueryApplicationRequiredOutputConnection<CanonicalSchema>>::demand_from_source(source)
    }
}
impl ApplicationActionDelivery<CanonicalSchema, edit::EditBinding> for DerivedDelivery {
    type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "canonical-transition.derived-delivery.v1";
}
impl ApplicationFixedActionDelivery<CanonicalSchema, edit::EditBinding> for DerivedDelivery {
    type Root = Root;
    type Projection = ApplicationNoProjection;
}
impl ApplicationProgramDefinition<CanonicalSchema> for Program {
    type Contributions = (CanonicalContribution,);
    type Outputs = ApplicationProgramOutputs<Root>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("canonical-transition.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<CanonicalSchema, SourceFeature>()
                .provides::<SourcePort>()
                .action::<edit::EditBinding, DerivedDelivery>()
                .finish_action()
                .native_action::<super::publish::PublishBinding>()
                .finish_action()
                .native_action::<super::retire::Binding>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<CanonicalSchema, DerivedFeature>()
                .conditional_operation::<producer::Produce>()
                .conditional_operation::<producer::preserve::Preserve>()
                .finish(),
        ]
    }
}
