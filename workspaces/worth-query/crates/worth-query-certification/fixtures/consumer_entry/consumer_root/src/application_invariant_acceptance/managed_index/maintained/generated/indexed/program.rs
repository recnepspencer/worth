//! One actual output graph connects the existing Catalog feature to its producer.
use super::super::super::{host::Feature, query, schema::Catalog};
use super::super::source;
use super::{producer, request};
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::primary_graph::*;
pub struct OutputFeature;
pub struct ItemsOutput;
pub struct ItemsInput;
pub struct Connection;
impl ApplicationFeature<Catalog> for OutputFeature {
    type Inputs = ApplicationFeatureInputList<ItemsInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "maintained-generated.indexed.output-feature.v1";
}
impl ApplicationOutputPort<Catalog, Feature> for ItemsOutput {
    type Value = query::ResultBinding;
    const IDENTITY: &'static str = "indexed-items";
}
impl ApplicationInputPort<Catalog, OutputFeature> for ItemsInput {
    type Value = query::ResultBinding;
    const IDENTITY: &'static str = "indexed-items";
    const REQUIRED: bool = true;
}
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "maintained-generated.indexed.connection.v1";
}
impl ApplicationOccurrenceConnectionBinding<Catalog, Feature, OutputFeature> for Connection {}
impl WorthQueryApplicationRequiredOutputConnection<Catalog> for Connection {
    type Source = request::Binding;
    type Demand = producer::Demand;
    const IDENTITY: &'static str = "maintained-generated.indexed.connection.v1";
    fn demand_from_source(
        input: &source::operation::RequestItem,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(producer::Demand {
            directory: input.directory.clone(),
        })
    }
}
pub type Link =
    ApplicationConnectionRef<Catalog, Feature, ItemsOutput, OutputFeature, ItemsInput, Connection>;
pub type Root = ApplicationOutputGraph<Link, ApplicationOutputLeaf>;
impl WorthQueryApplicationRequiredOutputSource<Catalog, Connection> for request::Binding {
    fn demand_from_source(
        input: &source::operation::RequestItem,
    ) -> Result<producer::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Connection as WorthQueryApplicationRequiredOutputConnection<Catalog>>::demand_from_source(
            input,
        )
    }
}
pub(in super::super::super) struct Delivery;
impl ApplicationActionDelivery<Catalog, request::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "maintained-generated.indexed.request-delivery.v1";
}
impl ApplicationFixedActionDelivery<Catalog, request::Binding> for Delivery {
    type Root = Root;
    type Projection = ApplicationNoProjection;
}
pub(in super::super::super) fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Catalog, OutputFeature>()
        .conditional_mutation::<super::operation::Binding>()
        .finish()
}
