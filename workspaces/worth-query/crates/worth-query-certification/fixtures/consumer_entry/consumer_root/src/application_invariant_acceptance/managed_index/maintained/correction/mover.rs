//! A plain recorded ItemName writer supplies authentic source/reverse occurrences.
use super::*;
use worth_query_decl::facade::{application_operation::*, *};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Input {
    pub directory: String,
    pub item: String,
    pub replacement: String,
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {
    MissingName,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "maintained-catalog.recorded-mover.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity: "maintained-catalog.recorded-mover.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "maintained-catalog.recorded-mover.denial.v1" });
worth_query_operation!(pub MoveName for Catalog, input InputBinding);
worth_query_operation_reads!(MoveName => [DirectoryIdentity, Item, ItemIdentity, ItemName]);
worth_query_operation_writes!(MoveName => [ItemName]);
fn scope(input: &Input) -> String {
    input.directory.clone()
}
worth_query_mutation_binding!(
    pub Binding for Input, schema Catalog, identity "maintained-catalog.recorded-mover.binding.v1",
    input InputBinding, operation MoveName, result ResultBinding,
    idempotency u64, identity "maintained-catalog.recorded-mover.key.v1",
    decision WorthQueryInvariantMutationTarget<Catalog, Item>, denial DenialBinding,
    handler identity "maintained-catalog.recorded-mover.handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value scope,
);
impl ApplicationDeclaredOperation<Catalog> for MoveName {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> ApplicationSchemaDeclarationBuilder<Catalog> {
        let op = Self::reference();
        schema
            .operation_projection_work_budget(op, 32)
            .operation_read_field(op, DirectoryIdentity::reference())
            .operation_read_entity(op, Item::reference())
            .operation_read_field(op, ItemIdentity::reference())
            .operation_read_field(op, ItemName::reference())
            .operation_write(op, ItemName::reference())
    }
}
pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Item>, Denial> {
        let result = (|| {
            let item = reader.resolve_entity(ItemIdentity::reference(), input.item.clone())?;
            if reader.field(&item, ItemName::reference())?.is_none() {
                return Ok(None);
            }
            reader.mutation_target(&item).map(Some)
        })();
        match result {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(Denial::MissingName),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Input,
        target: WorthQueryInvariantMutationTarget<Catalog, Item>,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        let result = writer
            .projected_entity(&target)
            .map_err(HandlerExecutionDenial::new)
            .and_then(|entity| {
                writer
                    .write_field(&entity, ItemName::reference(), input.replacement.clone())
                    .map_err(HandlerExecutionDenial::new)
            });
        match result {
            Ok(()) => HandlerResult::Completed(Changed),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
