//! One actual output graph connects the existing Catalog feature to its producer.
use super::super::{composition::Feature, edit::Edit, query, schema::Catalog};
use super::{self as producer, source};
use worth_query_decl::facade::application_program::*;
pub struct OutputFeature;
pub struct ItemsOutput;
pub struct ItemsInput;
pub struct Connection;
impl ApplicationFeature<Catalog> for OutputFeature {
    type Inputs = ApplicationFeatureInputList<ItemsInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "nested-required-support.output-feature.v1";
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
    const IDENTITY: &'static str = "nested-required-support.connection.v1";
}
impl ApplicationOccurrenceConnectionBinding<Catalog, Feature, OutputFeature> for Connection {}
impl WorthQueryApplicationRequiredOutputConnection<Catalog> for Connection {
    type Source = source::Binding;
    type Demand = producer::Demand;
    const IDENTITY: &'static str = "nested-required-support.connection.v1";
    fn demand_from_source(
        input: &Edit,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        match input {
            Edit::Name { .. } => Ok(producer::Demand {
                directory: "A".into(),
                prefix: "match".into(),
            }),
            Edit::Link { .. } => Ok(producer::Demand {
                directory: "D".into(),
                prefix: "match-unrelated".into(),
            }),
            _ => Err(WorthQueryRequiredOutputConnectionDenial::new(
                "the adjacency court uses real rename and link actions",
            )),
        }
    }
}
pub type Link =
    ApplicationConnectionRef<Catalog, Feature, ItemsOutput, OutputFeature, ItemsInput, Connection>;
pub type Root = ApplicationOutputGraph<Link, ApplicationOutputLeaf>;
impl WorthQueryApplicationRequiredOutputSource<Catalog, Connection> for source::Binding {
    fn demand_from_source(
        input: &Edit,
    ) -> Result<producer::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Connection as WorthQueryApplicationRequiredOutputConnection<Catalog>>::demand_from_source(
            input,
        )
    }
}
pub(in super::super) fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Catalog, OutputFeature>()
        .conditional_mutation::<super::operation::Binding>()
        .finish()
}
pub(in super::super) struct Delivery;
impl ApplicationActionDelivery<Catalog, source::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "nested-required-support.delivery.v1";
}
impl ApplicationFixedActionDelivery<Catalog, source::Binding> for Delivery {
    type Root = Root;
    type Projection = ApplicationNoProjection;
}
