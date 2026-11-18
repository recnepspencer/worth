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
#[cfg(test)]
mod replay;
use super::component_carriage::parent as owner;
use super::super::super::family::diamond as joined;

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
// The child's genuine parent is the first output whose source model it reads.
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Discover {
    type Prerequisites = owner::Read;
    type PrerequisiteRow = owner::ParentBinding;
    type Selection = owner::Selection;
    fn prerequisite_rows() -> Self::Selection {
        owner::owner()
    }
    fn prerequisites_from_target(
        target: &second::Demand,
    ) -> Result<owner::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(owner::Read(format!("first:{}", target.0)))
    }
    fn parent_from_prerequisite(
        row: &owner::Parent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(row.key.clone()))
    }
}
type Children = ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>;

/// The second occurrence hop: `second:X` discovers the join `joined:first:X`,
/// whose value sums the seconds of model X, so it reads the child's value.
pub struct Grandchild;
impl ApplicationConnectionIdentity for Grandchild {
    const IDENTITY: &'static str = "interleaved.seed-children.grandchild";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Child, joined::Join> for Grandchild {}
type GrandchildConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Child,
    hierarchy::ChildOutput,
    joined::Join,
    joined::RightInput,
    Grandchild,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Grandchild {
    type RootDemand = second::Demand;
    type Discovery = query::second_output::Read;
    type Demand = joined::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &second::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::second_output::Read(format!("second:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<joined::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        let seed = row.key.strip_prefix("second:").expect("a real second output key");
        Ok(vec![joined::Demand(format!("first:{seed}"))])
    }
}
// The join's genuine parent is the child of the same seed model.
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Grandchild {
    type Prerequisites = owner::Read;
    type PrerequisiteRow = owner::ParentBinding;
    type Selection = owner::Selection;
    fn prerequisite_rows() -> Self::Selection {
        owner::owner()
    }
    fn prerequisites_from_target(
        target: &joined::Demand,
    ) -> Result<owner::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(owner::Read(target.0.clone()))
    }
    fn parent_from_prerequisite(
        row: &owner::Parent,
    ) -> Result<second::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(second::Demand(row.key.clone()))
    }
}
/// The child and its join reached only through occurrence arcs, two hops from
/// the seed: the tree declares no parent-to-child edge, as a CAD seed's
/// occurrence-discovered consumers.
pub struct Occurrence;
impl ApplicationOccurrenceComponent<Schema> for Occurrence {
    type Connections = (ChildConnection, GrandchildConnection);
}
pub type OccurrenceWriteTree =
    ApplicationOutputGraph<SeedWriteConnection, ApplicationOutputOccurrences<Occurrence>>;
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

pub struct OccurrenceProgram;
impl ApplicationProgramDefinition<Schema> for OccurrenceProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(
        super::program::ModelTree,
        (super::program::ProducedTree, OccurrenceWriteTree),
    )>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.interleaved-seed-occurrences");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        super::program::feature_specs::<Completion>()
    }
}
