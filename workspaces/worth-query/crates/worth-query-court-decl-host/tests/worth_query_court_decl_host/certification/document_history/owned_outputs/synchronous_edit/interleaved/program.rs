//! Existing first/second/child families share one real ranked cabinet topology.
use super::super::super::hierarchy::program as hierarchy;
use super::super::super::program as base;
use super::*;
pub struct ModelRoot;
impl ApplicationConnectionIdentity for ModelRoot {
    const IDENTITY: &'static str = "interleaved.model-root";
}
impl ApplicationOccurrenceConnectionBinding<Schema, base::Entry, hierarchy::Parent> for ModelRoot {}
pub type ModelConnection = ApplicationConnectionRef<
    Schema,
    base::Entry,
    base::EntryOutput,
    hierarchy::Parent,
    hierarchy::Input,
    ModelRoot,
>;
pub type ModelTree = ApplicationOutputGraph<ModelConnection, ApplicationOutputLeaf>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for ModelRoot {
    type Source = fixture::model::Binding;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &fixture::model::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, ModelRoot> for fixture::model::Binding {
    fn demand_from_source(
        input: &fixture::model::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, ModelRoot> for fixture::cap::Binding {
    fn demand_from_source(
        _: &fixture::cap::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand("a".into()))
    }
}
pub struct ProducedRoot;
impl ApplicationConnectionIdentity for ProducedRoot {
    const IDENTITY: &'static str = "interleaved.produced-root";
}
impl ApplicationOccurrenceConnectionBinding<Schema, base::Entry, hierarchy::Parent>
    for ProducedRoot
{
}
pub type ProducedConnection = ApplicationConnectionRef<
    Schema,
    base::Entry,
    base::EntryOutput,
    hierarchy::Parent,
    hierarchy::FirstSeedInput,
    ProducedRoot,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for ProducedRoot {
    type Source = dependent::Binding;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &dependent::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand("a".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, ProducedRoot> for dependent::Binding {
    fn demand_from_source(
        _: &dependent::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand("a".into()))
    }
}
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "interleaved.dependents";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Child>
    for Discover
{
}
pub type ChildConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    hierarchy::ChildInput,
    Discover,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = first::Demand;
    type Discovery = query::first_output::Read;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<second::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok((0..row.value / 10)
            .map(|ordinal| second::Demand(format!("dependent:{ordinal}")))
            .collect())
    }
}
pub type ProducedTree = ApplicationOutputGraph<
    ProducedConnection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct SealRoot;
impl ApplicationConnectionIdentity for SealRoot {
    const IDENTITY: &'static str = "interleaved.seal-root";
}
impl ApplicationOccurrenceConnectionBinding<Schema, base::Entry, hierarchy::Child> for SealRoot {}
pub type SealConnection = ApplicationConnectionRef<
    Schema,
    base::Entry,
    base::EntryOutput,
    hierarchy::Child,
    hierarchy::SeedInput,
    SealRoot,
>;
pub type SealTree = ApplicationOutputGraph<SealConnection, ApplicationOutputLeaf>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for SealRoot {
    type Source = seal::Binding;
    type Demand = child::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &seal::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(child::Demand("m".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, SealRoot> for seal::Binding {
    fn demand_from_source(
        _: &seal::Intent,
    ) -> Result<child::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(child::Demand("m".into()))
    }
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(ModelTree, (ProducedTree, SealTree))>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.interleaved");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = base::entry_spec_with_envelope_deliveries::<
            Completion,
            Completion,
            fixture::Delivery<ModelTree>,
            Completion,
            fixture::Delivery<ModelTree>,
            Completion,
            Completion,
            Completion,
            Completion,
            Completion,
        >();
        specs
    }
}
