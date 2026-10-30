//! One ordinary feature-scoped edit carries the observed cell into publication.
use super::schema::*;
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, WorthValue)]
pub struct EditCell {
    pub feature: u64,
    pub cell: u64,
    pub value: u64,
}
#[derive(Clone, Debug)]
pub struct Edited;
#[derive(Clone, Debug)]
pub enum Denied {
    CellNotOwned,
}
worth_query_structured_value_binding!(pub InputBinding for EditCell {identity:"cell-history.edit.input.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Edited {identity:"cell-history.edit.result.v1"});
worth_query_structured_value_binding!(pub DenialBinding for Denied {identity:"cell-history.edit.denial.v1"});
worth_query_operation!(pub Edit for Schema,input InputBinding);
worth_query_operation_reads!(Edit=>[FeatureKey,OwnsCell,CellKey,CellValue]);
worth_query_operation_writes!(Edit=>[CellValue]);
fn feature(input: &EditCell) -> u64 {
    input.feature
}
worth_query_mutation_binding!(pub Binding for EditCell,schema Schema,
    identity "cell-history.edit.binding.v1",input InputBinding,operation Edit,
    result ResultBinding,idempotency u64,identity "cell-history.edit.key.v1",
    decision WorthQueryInvariantMutationTarget<Schema,Cell>,denial DenialBinding,
    handler identity "cell-history.edit.handler.v1",
    admission ApplicationConventionalAdmission,program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),outputs NoApplicationMutationOutputs,
    principal PrincipalBinding,mapping ExternalMapping,principal_entity Principal,
    principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Feature,FeatureFacts,FeatureKey,u64,ReadOnly,NoApplicationUnit,
    field FeatureKey::reference(),value feature,
);

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = Edit::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 32)
        .operation_read_field(operation, FeatureKey::reference())
        .operation_read_relation(operation, OwnsCell::reference())
        .operation_read_field(operation, CellKey::reference())
        .operation_read_field(operation, CellValue::reference())
        .operation_write(operation, CellValue::reference())
        .application_mutation_binding::<Binding>()
}

pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &EditCell,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Cell>, Denied> {
        let feature = reader.scope().clone();
        let target = (|| -> Result<_, HandlerExecutionDenial> {
            reader.field(&feature, FeatureKey::reference())?;
            let Some(cell) = reader.resolve_optional_entity(CellKey::reference(), input.cell)?
            else {
                return Ok(None);
            };
            for relation in reader
                .relations_to(OwnsCell::reference(), &cell)
                .map_err(HandlerExecutionDenial::new)?
            {
                if relation.from().entity_id() == feature.entity_id() {
                    reader.field(&cell, CellValue::reference())?;
                    return reader.mutation_target(&cell).map(Some);
                }
            }
            Ok(None)
        })();
        match target {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(Denied::CellNotOwned),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &EditCell,
        target: WorthQueryInvariantMutationTarget<Schema, Cell>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Edited, Denied> {
        let effect = writer
            .projected_entity(&target)
            .map_err(HandlerExecutionDenial::new)
            .and_then(|cell| {
                writer
                    .write_field(&cell, CellValue::reference(), input.value)
                    .map_err(HandlerExecutionDenial::new)
            });
        match effect {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
