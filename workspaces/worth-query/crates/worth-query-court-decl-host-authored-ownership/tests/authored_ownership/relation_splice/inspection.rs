//! A normal authenticated operation inspects the real Native component answer.
use super::*;
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Input {
    pub id: u64,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "ownership.relation-splice-inspection.input.v1" });
worth_query_structured_value_binding!(pub CasesBinding for Vec<String> { identity: "ownership.relation-splice-inspection.result.v1" });
worth_query_operation!(pub InspectStageLinks for S: Scenario, input InputBinding);
impl OperationReads<InspectStageLinks> for Node {}
impl OperationReads<InspectStageLinks> for Id {}
impl OperationReads<InspectStageLinks> for PreviousStage {}
worth_query_mutation_binding!(
    pub Binding for Input, schema generic S: Scenario,
    identity "ownership.relation-splice-inspection.v1", input InputBinding, operation InspectStageLinks,
    result CasesBinding, idempotency String, identity "ownership.relation-splice-inspection.key.v1",
    decision Vec<String>, denial query::UnitBinding, handler identity "ownership.relation-splice-inspection.handler.v1",
    outputs NoApplicationMutationOutputs, source NoApplicationMutationSource,
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Node, Facts, Id, u64, ReadOnly, NoApplicationUnit,
    field Id::reference(), value_field id,
);
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let op = InspectStageLinks::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_read_entity(op, Node::reference())
        .operation_read_field(op, Id::reference())
        .operation_read_relation(op, PreviousStage::reference())
        .application_mutation_binding::<Binding<S>>()
}
pub struct Handler;
impl<S: Scenario> OperationHandler<S, Binding<S>> for Handler {
    fn decide(
        &self,
        _input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, S, Binding<S>>,
    ) -> HandlerResult<Vec<String>, ()> {
        let result = (|| {
            let mut names = Vec::new();
            for id in [10, 20, 30] {
                let Some(from) = reader.resolve_optional_entity(Id::reference(), id)? else {
                    continue;
                };
                if id == 20 {
                    let record = from.entity_id();
                    names.push(format!(
                        "@20#{}:{}:{}",
                        record.partition_value(),
                        record.local_slot_value(),
                        record.generation_value()
                    ));
                }
                for edge in reader
                    .relations_from(PreviousStage::reference(), &from)
                    .map_err(HandlerExecutionDenial::new)?
                {
                    let to = reader.field(edge.to(), Id::reference())?.unwrap();
                    let record = edge.relation_id();
                    names.push(format!(
                        "{id}->{to}#{}:{}:{}",
                        record.partition_value(),
                        record.local_slot_value(),
                        record.generation_value()
                    ));
                }
            }
            Ok(names)
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Input,
        decision: Vec<String>,
        _: &mut CandidateWriter<'_, S, Binding<S>>,
    ) -> HandlerResult<Vec<String>, ()> {
        HandlerResult::Completed(decision)
    }
}
pub fn inspect<S: Scenario>(
    host: &WorthQueryProgramApplicationRuntime<S, super::super::coupled::multi::TwoProgram>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    expected: &[&str],
    key: &str,
) -> Vec<(String, worth_relational::facade::transactions::RecordRef)> {
    let outcome = request
        .advanced()
        .mutate(Input { id: 1 })
        .without_source()
        .idempotency(&key.to_owned())
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let actual = outcome.result().unwrap_or_else(|| panic!("{outcome:?}"));
    let names: Vec<_> = actual
        .iter()
        .map(|row| row.split_once('#').unwrap().0)
        .filter(|name| !name.starts_with('@'))
        .collect();
    assert_eq!(names, expected, "{outcome:?}");
    actual
        .iter()
        .map(|row| {
            let (name, coordinates) = row.split_once('#').unwrap();
            let coordinates: Vec<_> = coordinates.split(':').collect();
            let partition = worth_relational::facade::identity::PartitionId::new(
                coordinates[0].parse().unwrap(),
            );
            let slot = coordinates[1].parse().unwrap();
            let generation = coordinates[2].parse().unwrap();
            let record = if name.starts_with('@') {
                worth_relational::facade::transactions::RecordRef::Entity(
                    worth_relational::facade::identity::EntityId::new(partition, slot, generation),
                )
            } else {
                worth_relational::facade::transactions::RecordRef::Relation(
                    worth_relational::facade::identity::RelationId::new(
                        partition, slot, generation,
                    ),
                )
            };
            (name.to_owned(), record)
        })
        .collect()
}
