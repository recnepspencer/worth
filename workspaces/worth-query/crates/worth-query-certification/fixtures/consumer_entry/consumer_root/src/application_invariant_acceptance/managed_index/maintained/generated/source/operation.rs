//! This actual source action owns the generated feature's fixed output discharge.
use super::super::super::schema::*;
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
#[derive(Clone, Debug, serde::Serialize)]
pub struct RequestItem {
    pub directory: String,
    pub name: String,
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {}
worth_query_structured_value_binding!(pub InputBinding for RequestItem { identity: "maintained-generated.request-input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity: "maintained-generated.request-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "maintained-generated.request-denial.v1" });
worth_query_operation!(pub RequestGeneratedItem for Catalog, input InputBinding);
worth_query_operation_reads!(RequestGeneratedItem => [Directory, Item]);
worth_query_operation_creates!(RequestGeneratedItem => [Item, Directory]);
worth_query_operation_links!(RequestGeneratedItem => [Contains]);

fn scope(input: &RequestItem) -> String {
    input.directory.clone()
}
worth_query_mutation_binding!(pub Binding for RequestItem, schema Catalog, identity "maintained-generated.request-binding.v1",
    input InputBinding, operation RequestGeneratedItem, result ResultBinding,
    idempotency u64, identity "maintained-generated.request-key.v1",
    decision WorthQueryInvariantMutationTarget<Catalog, Directory>, denial DenialBinding,
    handler identity "maintained-generated.request-handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity,
    field DirectoryIdentity::at("DirectoryIdentity"), value scope,
);
pub struct Handler;
impl<B> OperationHandler<Catalog, B> for Handler
where
    B: ApplicationMutationBinding<
        Catalog,
        Input = RequestItem,
        Operation = RequestGeneratedItem,
        Decision = WorthQueryInvariantMutationTarget<Catalog, Directory>,
        Result = Changed,
        Denial = Denial,
    >,
{
    fn decide(
        &self,
        input: &RequestItem,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, B>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Directory>, Denial> {
        HandlerResult::from_result((|| {
            let directory = reader.resolve_entity(
                DirectoryIdentity::at("DirectoryIdentity"),
                input.directory.clone(),
            )?;
            reader.field(&directory, DirectoryNote::at("DirectoryNote"))?;
            Ok(reader.mutation_target(&directory)?)
        })())
    }
    fn build_candidate(
        &self,
        input: &RequestItem,
        target: WorthQueryInvariantMutationTarget<Catalog, Directory>,
        writer: &mut CandidateWriter<'_, Catalog, B>,
    ) -> HandlerResult<Changed, Denial> {
        HandlerResult::from_result((|| {
            let directory = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(
                    &directory,
                    DirectoryNote::at("DirectoryNote"),
                    input.name.clone(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            if let Some(identity) = super::super::super::peer_scope::SOURCE_CREATES.get() {
                create_item(writer, &directory, identity)?;
            }
            if let Some(identity) = super::super::super::peer_scope::SOURCE_SIBLING.get() {
                create_sibling(writer, &directory, identity)?;
            }
            Ok(Changed)
        })())
    }
}
pub(in super::super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let operation = RequestGeneratedItem::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 64)
        .operation_read_entity(operation, Directory::reference())
        .operation_read_field(operation, DirectoryIdentity::at("DirectoryIdentity"))
        .operation_read_field(operation, DirectoryNote::at("DirectoryNote"))
        .operation_write(operation, DirectoryNote::at("DirectoryNote"))
        .operation_create(operation, Item::reference())
        .operation_create(operation, Directory::reference())
        .operation_write(operation, DirectoryIdentity::at("DirectoryIdentity"))
        .operation_link(operation, Contains::reference())
        .operation_write(operation, ItemIdentity::at("ItemIdentity"))
        .operation_write(operation, ItemName::at("ItemName"))
        .operation_write(operation, ItemEnabled::at("ItemEnabled"))
        .operation_write(operation, super::super::super::bounds::ItemBounds::at("ItemBounds"))
        .application_mutation_binding::<Binding>()
}

/// An authored Item the source action adds beside its note, when armed.
fn create_item<B: ApplicationMutationBinding<Catalog, Operation = RequestGeneratedItem>>(
    writer: &mut CandidateWriter<'_, Catalog, B>,
    directory: &WorthQueryApplicationEffectEntity<Catalog, Directory>,
    identity: &str,
) -> Result<(), HandlerExecutionDenial> {
    let item = writer
        .create_entity_in_context(
            directory,
            Item::reference(),
            WorthQueryApplicationEntityKey::new(identity).unwrap(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(&item, ItemIdentity::at("ItemIdentity"), identity.to_owned())
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(&item, ItemName::at("ItemName"), identity.to_owned().into())
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(&item, ItemEnabled::at("ItemEnabled"), true)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(
            &item,
            super::super::super::bounds::ItemBounds::at("ItemBounds"),
            super::super::super::bounds::Coordinates::new(0, 0, 0, 0),
        )
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .link(Contains::reference(), &format!("{identity}-source"), directory, &item)
        .map_err(HandlerExecutionDenial::new)
}

/// A sibling Directory the source action adds beside its note, when armed: a
/// new scope entity with no Items, in the same coupled envelope.
fn create_sibling<B: ApplicationMutationBinding<Catalog, Operation = RequestGeneratedItem>>(
    writer: &mut CandidateWriter<'_, Catalog, B>,
    directory: &WorthQueryApplicationEffectEntity<Catalog, Directory>,
    identity: &str,
) -> Result<(), HandlerExecutionDenial> {
    let sibling = writer
        .create_entity_in_context(
            directory,
            Directory::reference(),
            WorthQueryApplicationEntityKey::new(identity).unwrap(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(
            &sibling,
            DirectoryIdentity::at("DirectoryIdentity"),
            identity.to_owned(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(&sibling, DirectoryNote::at("DirectoryNote"), String::new())
        .map_err(HandlerExecutionDenial::new)
}
