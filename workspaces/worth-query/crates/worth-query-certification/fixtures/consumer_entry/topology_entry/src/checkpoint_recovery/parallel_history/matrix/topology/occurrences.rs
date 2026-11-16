//! One repeated-family connection retains real occurrence scope and prerequisite custody.
use super::*;
use crate::checkpoint_recovery::parallel_history::matrix::producer::Demand;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection,
    WorthQueryApplicationPrerequisiteOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};
pub struct Occurrence;
impl ApplicationConnectionIdentity for Occurrence {
    const IDENTITY: &'static str = "matrix-prerequisite";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Feature, Feature> for Occurrence {}
pub type Connection =
    ApplicationConnectionRef<Schema, Feature, OutputPort, Feature, PrerequisitePort, Occurrence>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Occurrence {
    type RootDemand = Demand;
    type Demand = Demand;
    type Discovery = Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Demand,
    ) -> Result<Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Read(root.0))
    }
    fn demands_from_discovery(
        found: &Topology,
    ) -> Result<Vec<Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(found.children.iter().copied().map(Demand).collect())
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Occurrence {
    type Prerequisites = Read;
    type PrerequisiteRow = ParentBinding;
    type Selection = ApplicationQueryResultRelationRef<
        Query,
        Parents,
        Schema,
        Prerequisite,
        Node,
        Node,
        ForwardResultTraversal,
        ManyResults,
    >;
    fn prerequisite_rows() -> Self::Selection {
        parents()
    }
    fn prerequisites_from_target(
        target: &Demand,
    ) -> Result<Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Read(target.0))
    }
    fn parent_from_prerequisite(
        row: &Parent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(member_key(&row.key)))
    }
}
