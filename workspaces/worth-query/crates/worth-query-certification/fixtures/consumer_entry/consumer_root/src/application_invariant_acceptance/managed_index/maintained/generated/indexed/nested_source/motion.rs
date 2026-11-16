//! Read the actual earlier Item output before moving a disjoint Source endpoint.
use crate::application_invariant_acceptance::managed_index::maintained as catalog;
use catalog::{bounds::Coordinates, geometry::schema::*, schema::*};
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*, *,
};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub directory: String,
    pub vertex: String,
    pub replacement: Coordinates,
    pub output: String,
    pub expected_name: String,
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {
    MissingOutputOrRegion,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "applied-nested-source.motion-input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity: "applied-nested-source.motion-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "applied-nested-source.motion-denial.v1" });
worth_query_operation!(pub MoveAfterOutput for Catalog, input InputBinding);
worth_query_operation_reads!(MoveAfterOutput => [Vertex, Item]);
fn scope(input: &Input) -> String {
    input.directory.clone()
}
worth_query_mutation_binding!(pub Binding for Input, schema Catalog, identity "applied-nested-source.motion-binding.v1",
    input InputBinding, operation MoveAfterOutput, result ResultBinding,
    idempotency u64, identity "applied-nested-source.motion-key.v1",
    decision WorthQueryInvariantMutationTarget<Catalog, Vertex>, denial DenialBinding,
    handler identity "applied-nested-source.motion-handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(), outputs NoApplicationMutationOutputs,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity, field DirectoryIdentity::at("DirectoryIdentity"), value scope,);

pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Vertex>, Denial> {
        let result = (|| {
            let output =
                reader.resolve_entity(ItemIdentity::at("ItemIdentity"), input.output.clone())?;
            let name = reader.field(&output, ItemName::at("ItemName"))?;
            let vertex = reader
                .resolve_entity(VertexIdentity::at("VertexIdentity"), input.vertex.clone())?;
            let region = reader.field(&vertex, VertexRegion::at("VertexRegion"))?;
            if name != Some(input.expected_name.clone().into()) || region.is_none() {
                return Ok(None);
            }
            reader.mutation_target(&vertex).map(Some)
        })();
        match result {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(Denial::MissingOutputOrRegion),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Input,
        target: WorthQueryInvariantMutationTarget<Catalog, Vertex>,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        HandlerResult::from_result((|| {
            let vertex = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&vertex, VertexRegion::at("VertexRegion"), input.replacement)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(Changed)
        })())
    }
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let op = MoveAfterOutput::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 64)
        .operation_read_field(op, DirectoryIdentity::at("DirectoryIdentity"))
        .operation_read_entity(op, Vertex::reference())
        .operation_read_field(op, VertexIdentity::at("VertexIdentity"))
        .operation_read_field(op, VertexRegion::at("VertexRegion"))
        .operation_read_entity(op, Item::reference())
        .operation_read_field(op, ItemIdentity::at("ItemIdentity"))
        .operation_read_field(op, ItemName::at("ItemName"))
        .operation_write(op, VertexRegion::at("VertexRegion"))
        .application_mutation_binding::<Binding>()
}
impl WorthQueryApplicationRequiredOutputSource<Catalog, catalog::generated::program::Connection>
    for Binding
{
    fn demand_from_source(
        input: &Input,
    ) -> Result<catalog::generated::producer::Demand, WorthQueryRequiredOutputConnectionDenial>
    {
        Ok(catalog::generated::producer::Demand {
            directory: input.directory.clone(),
        })
    }
}
pub(in crate::application_invariant_acceptance::managed_index::maintained) struct Delivery;
impl ApplicationActionDelivery<Catalog, Binding> for Delivery {
    type Mode =
        ApplicationFixedDeliveryMode<catalog::generated::program::Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "applied-nested-source.motion-delivery.v1";
}
impl ApplicationFixedActionDelivery<Catalog, Binding> for Delivery {
    type Root = catalog::generated::program::Root;
    type Projection = ApplicationNoProjection;
}
