pub use worth_query::facade::domain;

pub mod declaration_contracts;

pub use super::conditional_node_contract::{dependency, node};
pub use super::installed_operation_fixture::{
    conditional_workspace, correspondence_bridge, fixture_record_identity, ConditionalModelGraph,
    GeometryDomain, ReadFamily, ReadVertex,
};

pub fn geometry_node_location() -> domain::WorthQueryConditionalNodeLocation {
    domain::WorthQueryConditionalNodeLocation::operation("geometry").unwrap()
}
