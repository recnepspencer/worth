//! Installed component contracts exercised through genuine Query and Native owners.
use super::super::decision::components::binding::{Shells, Visible};
use super::*;
use worth_query_declaration::{
    facade::{application_query::*, application_schema::*},
    *,
};
mod custody;
mod declarations;
mod disclosure;
mod execution;
mod planning;
pub struct ComponentSlot;
worth_query_portable_type!(ComponentSlot=>"worth.query.components.query-slot.v1");
worth_query_application_query!(pub Components for Schema:OrbitModel,identity "DeclaredComponents",parameters ParametersBinding,result metadata_only::MemberBinding,scope Node=>"Node",name "declared_components");
macro_rules! query {
    ($q:ident,$name:literal) => {
        worth_query_application_query!(pub $q for Schema:OrbitModel,identity stringify!($q),parameters ParametersBinding,result RowsBinding,scope Node=>"Node",name $name);
    };
}
query!(Census, "component_census");
query!(Of, "component_of");
query!(Partial, "component_partial");
query!(Empty, "component_empty");
query!(Allowed, "component_allowed");
query!(Omitted, "component_omitted");
query!(Missing, "component_missing");
query!(GuardDenied, "component_guarddenied");
query!(GuardAllowed, "component_guardallowed");
query!(AggregateDenied, "component_aggregatedenied");
query!(OrderingDenied, "component_orderingdenied");
impl<S: OrbitModel> WorthQueryApplicationProjection<S, Components> for metadata_only::Member {
    fn project(
        _: &WorthQueryApplicationProjectionRow<'_, S, Components>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self)
    }
}
fn selection<S: OrbitModel, Q: ApplicationQueryMarkerIdentity<S, Scope = Node>>(
) -> ApplicationQueryComponentRef<S, Q, ComponentSlot, Shells, Node> {
    ApplicationQueryComponentRef::census("shells", Shells::reference())
}
fn of<S: OrbitModel>() -> ApplicationQueryComponentRef<S, Of, ComponentSlot, Shells, Node> {
    ApplicationQueryComponentRef::of_scope_member("shells", Shells::reference())
}
pub(super) fn declare<const O: bool>(
    schema: ApplicationSchemaDeclarationBuilder<OrbitSchema<O>>,
) -> ApplicationSchemaDeclarationBuilder<OrbitSchema<O>> {
    declarations::declare(schema)
}
fn interleaved_world() -> World {
    world::install_orbit_with_component_visibility(
        Graph {
            nodes: 5,
            edges: vec![(0, 2), (2, 4), (4, 0), (1, 3), (3, 1)],
            active: false,
            blocked: vec![],
        },
        probe::Probe::new(|_, _, _| {}),
        vec![(0, 1)],
    )
}
