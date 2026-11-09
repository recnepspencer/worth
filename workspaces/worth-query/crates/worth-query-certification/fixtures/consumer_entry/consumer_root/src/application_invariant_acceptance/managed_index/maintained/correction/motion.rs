//! Actual observed VertexRegion writes retain their genuine coordinate preimage.
use super::super::{bounds::Coordinates, geometry::schema::*};
use super::*;
use worth_query_decl::facade::{application_operation::*, *};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub directory: String,
    pub vertex: String,
    pub replacement: Coordinates,
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {
    MissingRegion,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "maintained-catalog.recorded-motion.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity: "maintained-catalog.recorded-motion.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "maintained-catalog.recorded-motion.denial.v1" });
worth_query_operation!(pub MoveVertex for Catalog, input InputBinding);
worth_query_operation_reads!(MoveVertex => [Vertex]);

fn scope(input: &Input) -> String {
    input.directory.clone()
}
worth_query_mutation_binding!(
    pub Binding for Input, schema Catalog, identity "maintained-catalog.recorded-motion.binding.v1",
    input InputBinding, operation MoveVertex, result ResultBinding,
    idempotency u64, identity "maintained-catalog.recorded-motion.key.v1",
    decision WorthQueryInvariantMutationTarget<Catalog, Vertex>, denial DenialBinding,
    handler identity "maintained-catalog.recorded-motion.handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity,
    field DirectoryIdentity::at("DirectoryIdentity"), value scope,
);
impl ApplicationDeclaredOperation<Catalog> for MoveVertex {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> ApplicationSchemaDeclarationBuilder<Catalog> {
        let op = Self::reference();
        schema
            .operation_projection_work_budget(op, 32)
            .operation_read_field(op, DirectoryIdentity::at("DirectoryIdentity"))
            .operation_read_entity(op, Vertex::reference())
            .operation_read_field(op, VertexIdentity::at("VertexIdentity"))
            .operation_read_field(op, VertexRegion::at("VertexRegion"))
            .operation_write(op, VertexRegion::at("VertexRegion"))
    }
}
pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Vertex>, Denial> {
        let result = (|| {
            let vertex = reader
                .resolve_entity(VertexIdentity::at("VertexIdentity"), input.vertex.clone())?;
            if reader
                .field(&vertex, VertexRegion::at("VertexRegion"))?
                .is_none()
            {
                return Ok(None);
            }
            reader.mutation_target(&vertex).map(Some)
        })();
        match result {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(Denial::MissingRegion),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Input,
        target: WorthQueryInvariantMutationTarget<Catalog, Vertex>,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        let result = writer
            .projected_entity(&target)
            .map_err(HandlerExecutionDenial::new)
            .and_then(|entity| {
                writer
                    .write_field(&entity, VertexRegion::at("VertexRegion"), input.replacement)
                    .map_err(HandlerExecutionDenial::new)
            });
        match result {
            Ok(()) => HandlerResult::Completed(Changed),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
