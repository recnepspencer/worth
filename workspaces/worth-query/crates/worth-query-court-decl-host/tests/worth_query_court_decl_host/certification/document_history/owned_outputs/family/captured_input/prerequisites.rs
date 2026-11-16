//! Ordinary predecessor rows make the changing capture's control causal.
use super::super::super::hierarchy::program as hierarchy;
use super::super::super::synchronous_edit::interleaved::component_carriage::{outgoing, topology};
use super::*;

pub struct Previous;
impl ApplicationConnectionIdentity for Previous {
    const IDENTITY: &'static str = "captured-input.previous";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Parent>
    for Previous
{
}
pub type Connection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Parent,
    hierarchy::PredecessorInput,
    Previous,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Previous {
    type RootDemand = first::Demand;
    type Demand = first::Demand;
    type Discovery = outgoing::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<outgoing::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(outgoing::Read(root.0.clone()))
    }
    fn demands_from_discovery(
        row: &outgoing::Dependents,
    ) -> Result<Vec<first::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(row.children.iter().cloned().map(first::Demand).collect())
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Previous {
    type Prerequisites = topology::Read;
    type PrerequisiteRow = topology::PreviousParentBinding;
    type Selection = ApplicationQueryResultRelationRef<
        topology::Query,
        topology::PreviousParents,
        Schema,
        ModelMember,
        Model,
        Member,
        ForwardResultTraversal,
        ManyResults,
    >;
    fn prerequisite_rows() -> Self::Selection {
        topology::previous_parents()
    }
    fn prerequisites_from_target(
        target: &first::Demand,
    ) -> Result<topology::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(topology::Read(target.0.clone()))
    }
    fn parent_from_prerequisite(
        row: &topology::PreviousParent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(row.key.clone()))
    }
}
