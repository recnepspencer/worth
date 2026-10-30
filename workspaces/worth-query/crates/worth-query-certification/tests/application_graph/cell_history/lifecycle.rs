//! Structural cell changes consume observed ownership and ordinary graph writers.
use super::schema::*;
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, WorthValue)]
pub struct Replacement {
    pub key: u64,
    pub value: u64,
}
#[derive(Clone, Debug, WorthValue)]
pub struct ChangeCell {
    pub feature: u64,
    pub cell: u64,
    pub replacement: Option<Replacement>,
}
#[derive(Clone, Debug)]
pub struct Changed;
#[derive(Clone, Debug)]
pub enum Denied {
    CellNotOwned,
    ReplacementExists,
}
pub struct Decision {
    feature: WorthQueryInvariantMutationTarget<Schema, Feature>,
    cell: WorthQueryInvariantMutationTarget<Schema, Cell>,
}
worth_query_structured_value_binding!(pub InputBinding for ChangeCell {identity:"cell-history.change.input.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Changed {identity:"cell-history.change.result.v1"});
worth_query_structured_value_binding!(pub DenialBinding for Denied {identity:"cell-history.change.denial.v1"});
worth_query_operation!(pub Change for Schema,input InputBinding);
worth_query_operation_reads!(Change=>[Feature,Cell,FeatureKey,OwnsCell,CellKey,CellValue]);
worth_query_operation_creates!(Change=>[Cell]);
worth_query_operation_deletes!(Change=>[Cell]);
worth_query_operation_links!(Change=>[OwnsCell]);
worth_query_operation_unlinks!(Change=>[OwnsCell]);
worth_query_operation_writes!(Change=>[CellKey,CellValue]);
fn feature(input: &ChangeCell) -> u64 {
    input.feature
}
worth_query_mutation_binding!(pub Binding for ChangeCell,schema Schema,
    identity "cell-history.change.binding.v1",input InputBinding,operation Change,
    result ResultBinding,idempotency u64,identity "cell-history.change.key.v1",
    decision Decision,denial DenialBinding,
    handler identity "cell-history.change.handler.v1",
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
    let operation = Change::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 64)
        .operation_read_entity(operation, Feature::reference())
        .operation_read_entity(operation, Cell::reference())
        .operation_read_field(operation, FeatureKey::reference())
        .operation_read_relation(operation, OwnsCell::reference())
        .operation_read_field(operation, CellKey::reference())
        .operation_read_field(operation, CellValue::reference())
        .operation_create(operation, Cell::reference())
        .operation_delete(operation, Cell::reference())
        .operation_link(operation, OwnsCell::reference())
        .operation_unlink(operation, OwnsCell::reference())
        .operation_write(operation, CellKey::reference())
        .operation_write(operation, CellValue::reference())
        .application_mutation_binding::<Binding>()
}

pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &ChangeCell,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, Denied> {
        let feature = reader.scope().clone();
        let observe = (|| -> Result<_, HandlerExecutionDenial> {
            reader.entity(&feature, Feature::reference())?;
            reader.field(&feature, FeatureKey::reference())?;
            let Some(cell) = reader.resolve_optional_entity(CellKey::reference(), input.cell)?
            else {
                return Ok(None);
            };
            let owned = reader
                .relations_to(OwnsCell::reference(), &cell)
                .map_err(HandlerExecutionDenial::new)?
                .into_iter()
                .any(|edge| edge.from().entity_id() == feature.entity_id());
            if !owned {
                return Ok(None);
            }
            reader.entity(&cell, Cell::reference())?;
            reader.field(&cell, CellValue::reference())?;
            Ok(Some(Decision {
                feature: reader.mutation_target(&feature)?,
                cell: reader.mutation_target(&cell)?,
            }))
        })();
        let decision = match observe {
            Ok(Some(decision)) => decision,
            Ok(None) => return HandlerResult::DomainDenied(Denied::CellNotOwned),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        if let Some(replacement) = &input.replacement {
            match reader.resolve_optional_entity(CellKey::reference(), replacement.key) {
                Ok(None) => {}
                Ok(Some(_)) => return HandlerResult::DomainDenied(Denied::ReplacementExists),
                Err(error) => return HandlerResult::ExecutionDenied(error),
            }
        }
        HandlerResult::Completed(decision)
    }

    fn build_candidate(
        &self,
        input: &ChangeCell,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Changed, Denied> {
        let build = |writer: &mut CandidateWriter<'_, Schema, Binding>| {
            let feature = writer.projected_entity(&decision.feature)?;
            let cell = writer.projected_entity(&decision.cell)?;
            writer.unlink(OwnsCell::reference(), &feature, &cell)?;
            writer.delete_entity(Cell::reference(), &cell)?;
            if let Some(replacement) = &input.replacement {
                let key = WorthQueryApplicationEntityKey::new(format!("cell-{}", replacement.key))
                    .unwrap();
                let cell = writer.create_entity_in_context(&feature, Cell::reference(), key)?;
                writer.initialize_field(&cell, CellKey::reference(), replacement.key)?;
                writer.initialize_field(&cell, CellValue::reference(), replacement.value)?;
                writer.link(
                    OwnsCell::reference(),
                    format!("owns-{}", replacement.key),
                    &feature,
                    &cell,
                )?;
            }
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        };
        match build(writer) {
            Ok(()) => HandlerResult::Completed(Changed),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
