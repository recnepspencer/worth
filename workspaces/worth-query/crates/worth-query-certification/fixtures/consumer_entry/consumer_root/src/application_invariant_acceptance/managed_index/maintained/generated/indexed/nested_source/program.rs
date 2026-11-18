//! One actual output graph connects the existing Catalog feature to its producer.
use super as producer;
use crate::application_invariant_acceptance::managed_index::maintained as catalog;
use catalog::generated::indexed::request;
use catalog::generated::source;
use catalog::{host::Feature, schema::Catalog};
use worth_query_decl::facade::application_program::*;
pub struct OutputFeature;
pub struct ItemsOutput;
pub struct ItemsInput;
pub struct Connection;
impl ApplicationFeature<Catalog> for OutputFeature {
    type Inputs = ApplicationFeatureInputList<ItemsInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "maintained-generated.applied-nested-source.output-feature.v1";
}
impl ApplicationOutputPort<Catalog, Feature> for ItemsOutput {
    type Value = catalog::geometry::query::ResultBinding;
    const IDENTITY: &'static str = "applied-nested-items";
}
impl ApplicationInputPort<Catalog, OutputFeature> for ItemsInput {
    type Value = catalog::geometry::query::ResultBinding;
    const IDENTITY: &'static str = "applied-nested-items";
    const REQUIRED: bool = true;
}
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "maintained-generated.applied-nested-source.connection.v1";
}
impl ApplicationOccurrenceConnectionBinding<Catalog, Feature, OutputFeature> for Connection {}
impl WorthQueryApplicationRequiredOutputConnection<Catalog> for Connection {
    type Source = request::Binding;
    type Demand = producer::Demand;
    const IDENTITY: &'static str = "maintained-generated.applied-nested-source.connection.v1";
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
pub(in crate::application_invariant_acceptance::managed_index::maintained) fn feature(
) -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Catalog, OutputFeature>()
        .conditional_mutation::<super::operation::Binding>()
        .finish()
}
