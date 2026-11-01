//! Ordinary observed endpoints and declared writes author the physical event history.
use super::schema::*;
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
mod handler;
pub(super) use handler::Handler;
#[derive(Clone, Debug, serde::Serialize)]
pub enum Edit {
    Previous {
        event: String,
        previous: Option<String>,
        edition: u64,
    },
    Retirement {
        event: String,
        item: String,
        retired: bool,
        edition: u64,
    },
    Introduction {
        event: String,
        item: String,
        introduced: bool,
        edition: u64,
    },
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub struct Denial;
impl std::fmt::Display for Denial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "event history refusal")
    }
}
impl std::error::Error for Denial {}
type Target<T> = WorthQueryInvariantMutationTarget<Catalog, T>;
pub struct Decision {
    event: Target<Directory>,
    previous: Option<Target<Directory>>,
    old: Vec<Target<Directory>>,
    item: Option<Target<Item>>,
}
worth_query_structured_value_binding!(pub InputBinding for Edit { identity:"event-catalog.history.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity:"event-catalog.history.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity:"event-catalog.history.denial.v1" });
worth_query_operation!(pub EditHistory for Catalog,input InputBinding);
worth_query_operation_reads!(EditHistory=>[Directory,DirectoryIdentity,Item,ItemIdentity,Previous,Retires,Contains]);
worth_query_operation_links!(EditHistory=>[Previous,Retires,Contains]);
worth_query_operation_unlinks!(EditHistory=>[Previous,Retires,Contains]);
fn scope(_: &Edit) -> String {
    "catalog".into()
}
worth_query_mutation_binding!(pub Binding for Edit,schema Catalog,identity "event-catalog.history.binding.v1",
    input InputBinding,operation EditHistory,result ResultBinding,idempotency u64,identity "event-catalog.history.key.v1",
    decision Decision,denial DenialBinding,handler identity "event-catalog.history.handler.v1",program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),outputs NoApplicationMutationOutputs,
    principal CatalogPrincipal,mapping ExternalMapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Directory,DirectoryFacts,DirectoryIdentity,String,ReadOnly,NoApplicationUnit,field DirectoryIdentity::reference(),value scope,);
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let op = EditHistory::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 128)
        .operation_read_entity(op, Directory::reference())
        .operation_read_entity(op, Item::reference())
        .operation_read_field(op, DirectoryIdentity::reference())
        .operation_read_field(op, ItemIdentity::reference())
        .operation_read_relation(op, Previous::reference())
        .operation_read_relation(op, Retires::reference())
        .operation_read_relation(op, Contains::reference())
        .operation_link(op, Previous::reference())
        .operation_link(op, Retires::reference())
        .operation_link(op, Contains::reference())
        .operation_unlink(op, Previous::reference())
        .operation_unlink(op, Retires::reference())
        .operation_unlink(op, Contains::reference())
        .application_mutation_binding::<Binding>()
}
