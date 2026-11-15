//! Typed previous/foreign rows and the ordinary static child discovery.
use super::*;
macro_rules! relation {
    ($binding:ident, $connection:ident, $identity:literal, $port:ident, $slot:ident, $rows:ident, $row:ident, $binding_type:ident) => {
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
            type PrerequisiteRow = topology::$binding_type;
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
                row: &topology::$row,
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
    previous_parents,
    PreviousParent,
    PreviousParentBinding
);
relation!(
    Foreign,
    ForeignConnection,
    "component-carriage.foreign",
    ForeignInput,
    ForeignParents,
    foreign_parents,
    ForeignParent,
    ForeignParentBinding
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
    type Demand = from_first::Demand;
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
    ) -> Result<Vec<from_first::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        match row.key.as_str() {
            "first:a" | "first:carriage:d" => Ok(vec![from_first::Demand(row.key.clone())]),
            _ => Ok(Vec::new()),
        }
    }
}

pub struct Parent;
impl ApplicationConnectionIdentity for Parent {
    const IDENTITY: &'static str = "component-carriage.child-parent";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Child>
    for Parent
{
}
pub type ParentConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    hierarchy::SeedInput,
    Parent,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Parent {
    type RootDemand = Demand;
    type Demand = from_first::Demand;
    type Discovery = query::first_output::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.source)))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        // Only Static's captured a/d selectors request children. This connection
        // supplies their real incoming Parents, never an alternative child list.
        Ok(Vec::new())
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Parent {
    type Prerequisites = parent::Read;
    type PrerequisiteRow = parent::ParentBinding;
    type Selection = parent::Selection;
    fn prerequisite_rows() -> Self::Selection {
        parent::owner()
    }
    fn prerequisites_from_target(
        target: &from_first::Demand,
    ) -> Result<parent::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(parent::Read(target.0.clone()))
    }
    fn parent_from_prerequisite(
        row: &parent::Parent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand {
            source: row.key.clone(),
            selector: None,
        })
    }
}
