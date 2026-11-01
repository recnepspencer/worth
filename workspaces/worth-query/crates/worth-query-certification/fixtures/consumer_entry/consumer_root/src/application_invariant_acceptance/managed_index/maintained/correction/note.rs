//! Recorded note decisions genuinely consume a managed Parent outside their write locus.
use super::*;
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::{application_entry::*, primary_graph::*};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub directory: String,
    pub prefix: String,
    pub expect_empty: bool,
    pub replacement: String,
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {
    UnexpectedMembership,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "maintained-catalog.recorded-note.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity: "maintained-catalog.recorded-note.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "maintained-catalog.recorded-note.denial.v1" });
worth_query_operation!(pub WriteNote for Catalog, input InputBinding);
worth_query_operation_reads!(WriteNote => [Directory, DirectoryIdentity, DirectoryNote, ItemIdentity, ItemName, ItemEnabled]);
worth_query_operation_writes!(WriteNote => [DirectoryNote]);
fn scope(input: &Input) -> String {
    input.directory.clone()
}
worth_query_mutation_binding!(
    pub Binding for Input, schema Catalog, identity "maintained-catalog.recorded-note.binding.v1",
    input InputBinding, operation WriteNote, result ResultBinding,
    idempotency u64, identity "maintained-catalog.recorded-note.key.v1",
    decision WorthQueryInvariantMutationTarget<Catalog, Directory>, denial DenialBinding,
    handler identity "maintained-catalog.recorded-note.handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value scope,
);
impl ApplicationDeclaredOperation<Catalog> for WriteNote {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> ApplicationSchemaDeclarationBuilder<Catalog> {
        let op = Self::reference();
        schema
            .operation_projection_work_budget(op, 64)
            .operation_read_entity(op, Directory::reference())
            .operation_read_field(op, DirectoryIdentity::reference())
            .operation_read_field(op, DirectoryNote::reference())
            .operation_read_field(op, ItemIdentity::reference())
            .operation_read_field(op, ItemName::reference())
            .operation_read_field(op, ItemEnabled::reference())
            .operation_managed_index_read::<query::PrefixBinding, _, _>(op)
            .operation_write(op, DirectoryNote::reference())
    }
}
pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Directory>, Denial> {
        let result = (|| {
            let rows = reader.index_query::<query::PrefixBinding>(query::FindPrefix {
                directory: input.directory.clone(),
                prefix: input.prefix.clone(),
            })?;
            if rows.is_empty() != input.expect_empty {
                return Ok(None);
            }
            let directory =
                reader.resolve_entity(DirectoryIdentity::reference(), input.directory.clone())?;
            reader
                .reader()
                .require_decision_entity(&directory, Directory::reference())
                .map_err(HandlerExecutionDenial::new)?;
            if reader
                .field(&directory, DirectoryNote::reference())?
                .is_none()
            {
                return Ok(None);
            }
            reader.mutation_target(&directory).map(Some)
        })();
        match result {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(Denial::UnexpectedMembership),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Input,
        target: WorthQueryInvariantMutationTarget<Catalog, Directory>,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        let result = writer
            .projected_entity(&target)
            .map_err(HandlerExecutionDenial::new)
            .and_then(|entity| {
                writer
                    .write_field(
                        &entity,
                        DirectoryNote::reference(),
                        input.replacement.clone(),
                    )
                    .map_err(HandlerExecutionDenial::new)
            });
        match result {
            Ok(()) => HandlerResult::Completed(Changed),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
