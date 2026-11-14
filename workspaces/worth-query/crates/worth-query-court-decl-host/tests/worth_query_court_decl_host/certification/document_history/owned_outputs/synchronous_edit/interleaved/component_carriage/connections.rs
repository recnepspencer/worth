//! Typed previous/foreign rows and the ordinary static child discovery.
use super::*;
macro_rules! relation {
    ($binding:ident, $connection:ident, $identity:literal, $port:ident, $slot:ident, $rows:ident) => {
        pub struct $binding;
        impl ApplicationConnectionIdentity for $binding {
            const IDENTITY: &'static str = $identity;
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Parent>
            for $binding
        {
        }
        pub type $connection = ApplicationConnectionRef<
            Schema,
            hierarchy::Parent,
            hierarchy::Output,
            hierarchy::Parent,
            hierarchy::$port,
            $binding,
        >;
        impl WorthQueryApplicationDependentOutputConnection<Schema> for $binding {
            type RootDemand = Demand;
            type Demand = Demand;
            type Discovery = topology::Read;
            const IDENTITY: &'static str = $identity;
            fn discovery_from_root(
                root: &Demand,
            ) -> Result<topology::Read, WorthQueryRequiredOutputConnectionDenial> {
                Ok(topology::Read(root.source.clone()))
            }
            fn demands_from_discovery(
                found: &topology::Topology,
            ) -> Result<Vec<Demand>, WorthQueryRequiredOutputConnectionDenial> {
                Ok(found
                    .children
                    .iter()
                    .map(|source| Demand {
                        source: source.clone(),
                        selector: None,
                    })
                    .collect())
            }
        }
        impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for $binding {
            type Prerequisites = topology::Read;
            type PrerequisiteRow = topology::ParentBinding;
            type Selection = ApplicationQueryResultRelationRef<
                topology::Query,
                topology::$slot,
                Schema,
                ModelMember,
                Model,
                Member,
                ForwardResultTraversal,
                ManyResults,
            >;
            fn prerequisite_rows() -> Self::Selection {
                topology::$rows()
            }
            fn prerequisites_from_target(
                target: &Demand,
            ) -> Result<topology::Read, WorthQueryRequiredOutputConnectionDenial> {
                Ok(topology::Read(target.source.clone()))
            }
            fn parent_from_prerequisite(
                row: &topology::Parent,
            ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
                Ok(Demand {
                    source: row.key.clone(),
                    selector: None,
                })
            }
        }
    };
}
relation!(
    Previous,
    PreviousConnection,
    "component-carriage.previous",
    PredecessorInput,
    PreviousParents,
    previous_parents
);
relation!(
    Foreign,
    ForeignConnection,
    "component-carriage.foreign",
    ForeignInput,
    ForeignParents,
    foreign_parents
);
pub struct Static;
impl ApplicationConnectionIdentity for Static {
    const IDENTITY: &'static str = "component-carriage.static";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Child>
    for Static
{
}
pub type StaticConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    hierarchy::ChildInput,
    Static,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Static {
    type RootDemand = Demand;
    type Demand = second::Demand;
    type Discovery = query::first_output::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!(
            "first:{}",
            root.selector.as_ref().unwrap_or(&root.source)
        )))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<second::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        match row.key.as_str() {
            "first:a" => Ok(vec![second::Demand("a".into())]),
            "first:carriage:d" => Ok(vec![second::Demand("carriage:d".into())]),
            _ => Ok(Vec::new()),
        }
    }
}
