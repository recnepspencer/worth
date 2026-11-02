//! One actual output graph connects the existing Catalog feature to its producer.
use super::super::super::{host::Feature, schema::Catalog};
use super::{producer, request, source};
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{application_contribution::*, primary_graph::*};
pub struct OutputFeature;
pub struct DirectoryOutput;
pub struct DirectoryInput;
pub struct Connection;
impl ApplicationFeature<Catalog> for OutputFeature {
    type Inputs = ApplicationFeatureInputList<DirectoryInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "maintained-generated-geometry.output-feature.v1";
}
impl ApplicationOutputPort<Catalog, Feature> for DirectoryOutput {
    type Value = source::ResultBinding;
    const IDENTITY: &'static str = "geometry-directory";
}
impl ApplicationInputPort<Catalog, OutputFeature> for DirectoryInput {
    type Value = source::ResultBinding;
    const IDENTITY: &'static str = "geometry-directory";
    const REQUIRED: bool = true;
}
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "maintained-generated-geometry.connection.v1";
}
impl ApplicationOccurrenceConnectionBinding<Catalog, Feature, OutputFeature> for Connection {}
impl WorthQueryApplicationRequiredOutputConnection<Catalog> for Connection {
    type Source = request::Binding;
    type Demand = producer::Demand;
    const IDENTITY: &'static str = "maintained-generated-geometry.connection.v1";
    fn demand_from_source(
        input: &request::RequestEdge,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(producer::Demand {
            directory: input.directory.clone(),
        })
    }
}
pub type Link = ApplicationConnectionRef<
    Catalog,
    Feature,
    DirectoryOutput,
    OutputFeature,
    DirectoryInput,
    Connection,
>;
pub type Root = ApplicationOutputGraph<Link, ApplicationOutputLeaf>;
impl WorthQueryApplicationRequiredOutputSource<Catalog, Connection> for request::Binding {
    fn demand_from_source(
        input: &request::RequestEdge,
    ) -> Result<producer::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Connection as WorthQueryApplicationRequiredOutputConnection<Catalog>>::demand_from_source(
            input,
        )
    }
}
pub(in super::super::super) struct Delivery;
impl ApplicationActionDelivery<Catalog, request::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "maintained-generated-geometry.request-delivery.v1";
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
