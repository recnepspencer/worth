//! This actual source action owns the generated feature's fixed output discharge.
use super::super::super::schema::*;
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
#[derive(Clone, Debug, serde::Serialize)]
pub struct RequestEdge {
    pub directory: String,
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {}
worth_query_structured_value_binding!(pub InputBinding for RequestEdge { identity: "maintained-generated-geometry.request-input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity: "maintained-generated-geometry.request-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "maintained-generated-geometry.request-denial.v1" });
worth_query_operation!(pub RequestGeneratedEdge for Catalog, input InputBinding);
worth_query_operation_reads!(RequestGeneratedEdge => [Directory, DirectoryIdentity, DirectoryNote]);
worth_query_operation_writes!(RequestGeneratedEdge => [DirectoryNote]);
fn scope(input: &RequestEdge) -> String {
    input.directory.clone()
}
worth_query_mutation_binding!(pub Binding for RequestEdge, schema Catalog, identity "maintained-generated-geometry.request-binding.v1",
    input InputBinding, operation RequestGeneratedEdge, result ResultBinding,
    idempotency u64, identity "maintained-generated-geometry.request-key.v1",
    decision WorthQueryInvariantMutationTarget<Catalog, Directory>, denial DenialBinding,
    handler identity "maintained-generated-geometry.request-handler.v1", program required,
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
        input: &RequestEdge,
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
        _input: &RequestEdge,
        target: WorthQueryInvariantMutationTarget<Catalog, Directory>,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        HandlerResult::from_result((|| {
            let directory = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(
                    &directory,
                    DirectoryNote::reference(),
                    "generated-geometry".to_owned(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            Ok(Changed)
        })())
    }
}
pub(in super::super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let operation = RequestGeneratedEdge::reference();
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
