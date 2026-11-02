//! Installed catalog operations issue the records and memberships used by the court.
use super::{bounds, schema::*};
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

mod handler;
mod population;
pub(super) use handler::Handler;
pub(super) use population::populate;

#[derive(Clone, Debug, serde::Serialize)]
pub enum Edit {
    Directory {
        identity: String,
    },
    Item {
        directory: String,
        identity: String,
        name: String,
    },
    Items {
        directory: String,
        items: Vec<ItemSeed>,
    },
    RetireItem {
        directory: String,
        item: String,
    },
    SetEnabled {
        directory: String,
        item: String,
        enabled: bool,
    },
    Share {
        directory: String,
        item: String,
    },
}
pub const MAX_ITEMS_PER_BATCH: usize = 256;
#[derive(Clone, Debug, serde::Serialize)]
pub struct ItemSeed {
    pub identity: String,
    pub name: String,
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {
    InvalidBatch,
}
pub struct Decision {
    directory: WorthQueryInvariantMutationTarget<Catalog, Directory>,
    item: Option<WorthQueryInvariantMutationTarget<Catalog, Item>>,
}
worth_query_structured_value_binding!(pub InputBinding for Edit { identity: "maintained-catalog.authoring.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity: "maintained-catalog.authoring.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "maintained-catalog.authoring.denial.v1" });
worth_query_operation!(pub AuthorCatalog for Catalog, input InputBinding);
worth_query_operation_reads!(AuthorCatalog => [Directory, DirectoryIdentity, Item, ItemIdentity, ItemEnabled, Contains]);
worth_query_operation_creates!(AuthorCatalog => [Directory, Item]);
worth_query_operation_links!(AuthorCatalog => [Contains]);
worth_query_operation_deletes!(AuthorCatalog => [Item]);
worth_query_operation_writes!(AuthorCatalog => [DirectoryIdentity, DirectoryNote, ItemIdentity, ItemName, ItemEnabled, bounds::ItemBounds]);

fn scope(_: &Edit) -> String {
    "catalog".into()
}
worth_query_mutation_binding!(
    pub Binding for Edit, schema Catalog, identity "maintained-catalog.authoring.binding.v1",
    input InputBinding, operation AuthorCatalog, result ResultBinding,
    idempotency u64, identity "maintained-catalog.authoring.key.v1",
    decision Decision, denial DenialBinding,
    handler identity "maintained-catalog.authoring.handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value scope,
);

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let op = AuthorCatalog::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 64)
        .operation_read_entity(op, Directory::reference())
        .operation_read_field(op, DirectoryIdentity::reference())
        .operation_read_entity(op, Item::reference())
        .operation_read_field(op, ItemIdentity::reference())
        .operation_read_field(op, ItemEnabled::reference())
        .operation_read_relation(op, Contains::reference())
        .operation_create(op, Directory::reference())
        .operation_create(op, Item::reference())
        .operation_link(op, Contains::reference())
        .operation_delete(op, Item::reference())
        .operation_write(op, DirectoryIdentity::reference())
        .operation_write(op, DirectoryNote::reference())
        .operation_write(op, ItemIdentity::reference())
        .operation_write(op, ItemName::reference())
        .operation_write(op, ItemEnabled::reference())
        .operation_write(op, bounds::ItemBounds::reference())
        .application_mutation_binding::<Binding>()
}
