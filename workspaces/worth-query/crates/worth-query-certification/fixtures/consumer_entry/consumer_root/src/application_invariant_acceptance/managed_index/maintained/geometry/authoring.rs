//! Installed geometry actions observe targets before constructing Native effects.
use super::super::schema::*;
use super::schema::*;
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

mod handler;
pub(super) use handler::Handler;
#[derive(Clone, Debug, serde::Serialize)]
pub enum Edit {
    Chain {
        directory: String,
        shared_directory: Option<String>,
        prefix: String,
        vertices: u32,
        origin: i64,
    },
    Vertex {
        directory: String,
        identity: String,
        x: i64,
        y: i64,
    },
    Edge {
        directory: String,
        identity: String,
        start: String,
        end: String,
    },
    Move {
        directory: String,
        vertex: String,
        x: i64,
        y: i64,
    },
    Share {
        directory: String,
        edge: String,
    },
    RewireEnd {
        directory: String,
        edge: String,
        end: String,
    },
    DuplicateEnd {
        directory: String,
        edge: String,
    },
}
impl Edit {
    pub fn directory(&self) -> &str {
        match self {
            Self::Vertex { directory, .. }
            | Self::Chain { directory, .. }
            | Self::Edge { directory, .. }
            | Self::Move { directory, .. }
            | Self::Share { directory, .. }
            | Self::RewireEnd { directory, .. }
            | Self::DuplicateEnd { directory, .. } => directory,
        }
    }
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {}
type Target<T> = WorthQueryInvariantMutationTarget<Catalog, T>;
pub enum Decision {
    Chain {
        directory: Target<Directory>,
        shared_directory: Option<Target<Directory>>,
    },
    Vertex(Target<Directory>),
    Edge {
        directory: Target<Directory>,
        start: Target<Vertex>,
        end: Target<Vertex>,
    },
    Move(Target<Vertex>),
    Share {
        directory: Target<Directory>,
        edge: Target<Edge>,
    },
    Rewire {
        edge: Target<Edge>,
        old: Target<Vertex>,
        new: Target<Vertex>,
    },
    Duplicate {
        edge: Target<Edge>,
        end: Target<Vertex>,
    },
}
worth_query_structured_value_binding!(pub InputBinding for Edit { identity: "maintained-geometry.edit-input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity: "maintained-geometry.changed.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "maintained-geometry.denial.v1" });
worth_query_operation!(pub EditGeometry for Catalog, input InputBinding);
worth_query_operation_reads!(EditGeometry => [Directory, DirectoryIdentity, Edge, EdgeIdentity, Vertex, VertexIdentity, VertexRegion, Starts, Ends, ContainsEdge]);
worth_query_operation_creates!(EditGeometry => [Vertex, Edge]);
worth_query_operation_links!(EditGeometry => [ContainsEdge, Starts, Ends]);
worth_query_operation_unlinks!(EditGeometry => [Ends]);
worth_query_operation_writes!(EditGeometry => [VertexIdentity, VertexRegion, EdgeIdentity]);
fn scope(input: &Edit) -> String {
    input.directory().to_owned()
}
worth_query_mutation_binding!(pub Binding for Edit, schema Catalog, identity "maintained-geometry.edit-binding.v1",
    input InputBinding, operation EditGeometry, result ResultBinding,
    idempotency u64, identity "maintained-geometry.edit-key.v1", decision Decision, denial DenialBinding,
    handler identity "maintained-geometry.edit-handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(), outputs NoApplicationMutationOutputs,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit, field DirectoryIdentity::reference(), value scope,);

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let op = EditGeometry::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 128)
        .operation_read_entity(op, Directory::reference())
        .operation_read_field(op, DirectoryIdentity::reference())
        .operation_read_entity(op, Edge::reference())
        .operation_read_field(op, EdgeIdentity::reference())
        .operation_read_entity(op, Vertex::reference())
        .operation_read_field(op, VertexIdentity::reference())
        .operation_read_field(op, VertexRegion::reference())
        .operation_read_relation(op, ContainsEdge::reference())
        .operation_read_relation(op, Starts::reference())
        .operation_read_relation(op, Ends::reference())
        .operation_create(op, Vertex::reference())
        .operation_create(op, Edge::reference())
        .operation_link(op, ContainsEdge::reference())
        .operation_link(op, Starts::reference())
        .operation_link(op, Ends::reference())
        .operation_unlink(op, Ends::reference())
        .operation_write(op, VertexIdentity::reference())
        .operation_write(op, VertexRegion::reference())
        .operation_write(op, EdgeIdentity::reference())
        .operation_managed_index_read::<super::query::BoxBinding, _, _>(op)
        .application_mutation_binding::<Binding>()
}
