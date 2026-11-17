//! A parent whose driving discovers a child that reads the parent's own input.
//!
//! `first:X` reads model `X`; settling it discovers `second:X`, whose source is
//! that same model `X` (the seed). A stage that writes model `X` therefore
//! changes the child's input directly, without reading the child or the parent.
use super::super::super::hierarchy::program as hierarchy;
use super::super::super::program as base;
use super::*;
#[cfg(test)]
mod differential;
pub(super) mod frontier;

macro_rules! root {
    ($start:ident, $connection:ident, $port:ty, $binding:ty, $identity:literal, |$input:ident| $demand:expr) => {
        pub struct $start;
        impl ApplicationConnectionIdentity for $start {
            const IDENTITY: &'static str = $identity;
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, base::Entry, hierarchy::Parent>
            for $start
        {
        }
        type $connection = ApplicationConnectionRef<
            Schema,
            base::Entry,
            base::EntryOutput,
            hierarchy::Parent,
            $port,
            $start,
        >;
        impl WorthQueryApplicationRequiredOutputConnection<Schema> for $start {
            type Source = $binding;
            type Demand = first::Demand;
            const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
            fn demand_from_source(
                $input: &<$binding as ApplicationMutationBinding<Schema>>::Input,
            ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
                Ok($demand)
            }
        }
        impl WorthQueryApplicationRequiredOutputSource<Schema, $start> for $binding {
            fn demand_from_source(
                input: &<$binding as ApplicationMutationBinding<Schema>>::Input,
            ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
                <$start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
            }
        }
    };
}
// Writes the seed: model `a`, the parent's input. Its own driving discovers
// the child too, as an ordinary seed edit does.
root!(SeedWrite, SeedWriteConnection, hierarchy::ContinuationInput, fixture::model::Binding,
    "interleaved.seed-children.write", |input| first::Demand(input.key.clone()));
// Reads the parent's output, so the parent and its children settle before it.
root!(SeedRead, SeedReadConnection, hierarchy::PredecessorInput, dependent::Binding,
    "interleaved.seed-children.read", |_input| first::Demand("a".into()));

pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "interleaved.seed-children.discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Child>
    for Discover
{
}
type ChildConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    hierarchy::SeedInput,
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
        // The parent's own seed: the child reads model X, not first:X.
        Ok(vec![second::Demand(
            row.key.strip_prefix("first:").expect("a real first output key").into(),
        )])
    }
}
type Children = ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>;
pub type SeedWriteTree = ApplicationOutputGraph<SeedWriteConnection, Children>;
pub type SeedReadTree = ApplicationOutputGraph<SeedReadConnection, Children>;

pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(
        super::program::ModelTree,
        (super::program::ProducedTree, (SeedWriteTree, SeedReadTree)),
    )>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.interleaved-seed-children");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        super::program::feature_specs::<Completion>()
    }
}
