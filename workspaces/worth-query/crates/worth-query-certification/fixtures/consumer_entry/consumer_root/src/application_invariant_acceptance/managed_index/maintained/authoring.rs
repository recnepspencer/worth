//! Installed catalog operations issue the records and memberships used by the court.
use super::protected::access::grants::{BoxGrant, GrantIdentity, GrantResource};
use super::{bounds, ordered_decision::Entries, schema::*};
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
    Note {
        directory: String,
        note: String,
    },
    /// A new Directory under the watch grant: a peer of `catalog`.
    GrantedDirectory {
        identity: String,
    },
    RetireDirectory {
        directory: String,
    },
    /// Removes `directory` from the watch grant: no longer a peer.
    UngrantDirectory {
        directory: String,
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
    grant: Option<WorthQueryInvariantMutationTarget<Catalog, BoxGrant>>,
}
worth_query_structured_value_binding!(pub InputBinding for Edit { identity: "maintained-catalog.authoring.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity: "maintained-catalog.authoring.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "maintained-catalog.authoring.denial.v1" });
worth_query_operation!(pub AuthorCatalog for Catalog, input InputBinding);
worth_query_operation_reads!(AuthorCatalog => [Directory,Item,Contains,BoxGrant,GrantResource]);
worth_query_operation_creates!(AuthorCatalog => [Directory, Item]);
worth_query_operation_links!(AuthorCatalog => [Contains, Entries, GrantResource]);
worth_query_operation_deletes!(AuthorCatalog => [Item, Directory]);
worth_query_operation_unlinks!(AuthorCatalog => [GrantResource]);

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
    scope DirectoryIdentity,
    field DirectoryIdentity::at("DirectoryIdentity"), value scope,
);

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let op = AuthorCatalog::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 64)
        .operation_read_entity(op, Directory::reference())
        .operation_read_field(op, DirectoryIdentity::at("DirectoryIdentity"))
        .operation_read_field(op, DirectoryNote::at("DirectoryNote"))
        .operation_read_entity(op, Item::reference())
        .operation_read_field(op, ItemIdentity::at("ItemIdentity"))
        .operation_read_field(op, ItemEnabled::at("ItemEnabled"))
        .operation_read_relation(op, Contains::reference())
        .operation_read_entity(op, BoxGrant::reference())
        .operation_read_field(op, GrantIdentity::at("GrantIdentity"))
        .operation_link(op, GrantResource::reference())
        .operation_read_relation(op, GrantResource::reference())
        .operation_unlink(op, GrantResource::reference())
        .operation_delete(op, Directory::reference())
        .operation_create(op, Directory::reference())
        .operation_create(op, Item::reference())
        .operation_link(op, Contains::reference())
        .operation_link(op, Entries::reference())
        .operation_delete(op, Item::reference())
        .operation_write(op, DirectoryIdentity::at("DirectoryIdentity"))
        .operation_write(op, DirectoryNote::at("DirectoryNote"))
        .operation_write(op, ItemIdentity::at("ItemIdentity"))
        .operation_write(op, ItemName::at("ItemName"))
        .operation_write(op, ItemEnabled::at("ItemEnabled"))
        .operation_write(op, bounds::ItemBounds::at("ItemBounds"))
        .application_mutation_binding::<Binding>()
}
