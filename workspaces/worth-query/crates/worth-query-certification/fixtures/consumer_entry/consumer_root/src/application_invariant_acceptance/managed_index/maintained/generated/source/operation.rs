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
worth_query_operation_reads!(RequestGeneratedItem => [Directory, DirectoryIdentity, DirectoryNote]);
worth_query_operation_writes!(RequestGeneratedItem => [DirectoryNote]);
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
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value scope,
);
pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &RequestItem,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Directory>, Denial> {
        HandlerResult::from_result((|| {
            let directory =
                reader.resolve_entity(DirectoryIdentity::reference(), input.directory.clone())?;
            reader.field(&directory, DirectoryNote::reference())?;
            Ok(reader.mutation_target(&directory)?)
        })())
    }
    fn build_candidate(
        &self,
        input: &RequestItem,
        target: WorthQueryInvariantMutationTarget<Catalog, Directory>,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        HandlerResult::from_result((|| {
            let directory = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&directory, DirectoryNote::reference(), input.name.clone())
                .map_err(HandlerExecutionDenial::new)?;
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
        .operation_read_field(operation, DirectoryIdentity::reference())
        .operation_read_field(operation, DirectoryNote::reference())
        .operation_write(operation, DirectoryNote::reference())
        .application_mutation_binding::<Binding>()
}
