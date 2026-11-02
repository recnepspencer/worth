//! Ordinary observed targets author the text graph consumed by nested Sources.
use super::schema::*;
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
#[path = "authoring/handler.rs"]
mod handler;
pub(super) use handler::Handler;
#[derive(Clone, Debug, serde::Serialize)]
pub enum Edit {
    Directory {
        identity: String,
    },
    Item {
        directory: String,
        identity: String,
        name: String,
        root: bool,
        incarnation: u64,
    },
    Name {
        item: String,
        value: String,
    },
    Enabled {
        item: String,
        value: bool,
    },
    Link {
        from: String,
        to: String,
        edition: u64,
    },
    Unlink {
        from: String,
        to: String,
    },
    Share {
        directory: String,
        item: String,
    },
    Retire {
        item: String,
    },
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {
    MissingEdge,
}
type Target<T> = WorthQueryInvariantMutationTarget<Catalog, T>;
pub enum Decision {
    Directory(Target<Directory>),
    Item(Target<Item>),
    Pair(Target<Item>, Target<Item>),
    Share(Target<Directory>, Target<Item>),
}
worth_query_structured_value_binding!(pub InputBinding for Edit { identity: "nested-catalog.edit.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity: "nested-catalog.edit.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "nested-catalog.edit.denial.v1" });
worth_query_operation!(pub EditGraph for Catalog, input InputBinding);
worth_query_operation_reads!(EditGraph => [Directory,DirectoryIdentity,Item,ItemIdentity,ItemName,ItemEnabled,Children]);
worth_query_operation_creates!(EditGraph => [Directory,Item]);
worth_query_operation_links!(EditGraph => [Contains,Children]);
worth_query_operation_unlinks!(EditGraph => [Children]);
worth_query_operation_deletes!(EditGraph => [Item]);
worth_query_operation_writes!(EditGraph => [DirectoryIdentity,ItemIdentity,ItemName,ItemEnabled]);
fn scope(_: &Edit) -> String {
    "catalog".into()
}
worth_query_mutation_binding!(pub Binding for Edit, schema Catalog, identity "nested-catalog.edit.binding.v1",
    input InputBinding, operation EditGraph, result ResultBinding, idempotency u64, identity "nested-catalog.edit.key.v1",
    decision Decision, denial DenialBinding, handler identity "nested-catalog.edit.handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(), outputs NoApplicationMutationOutputs,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit, field DirectoryIdentity::reference(), value scope,);
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let op = EditGraph::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 128)
        .operation_read_entity(op, Directory::reference())
        .operation_read_entity(op, Item::reference())
        .operation_read_field(op, DirectoryIdentity::reference())
        .operation_read_field(op, ItemIdentity::reference())
        .operation_read_field(op, ItemName::reference())
        .operation_read_field(op, ItemEnabled::reference())
        .operation_read_relation(op, Children::reference())
        .operation_create(op, Directory::reference())
        .operation_create(op, Item::reference())
        .operation_link(op, Contains::reference())
        .operation_link(op, Children::reference())
        .operation_unlink(op, Children::reference())
        .operation_delete(op, Item::reference())
        .operation_write(op, DirectoryIdentity::reference())
        .operation_write(op, ItemIdentity::reference())
        .operation_write(op, ItemName::reference())
        .operation_write(op, ItemEnabled::reference())
        .application_mutation_binding::<Binding>()
}

impl std::fmt::Display for Denial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Denial {}
