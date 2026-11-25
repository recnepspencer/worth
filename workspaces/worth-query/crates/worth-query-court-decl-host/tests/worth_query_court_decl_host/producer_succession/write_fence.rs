//! Ordinary callers cannot destroy another producer's canonical records.
use super::{realization::*, *};
#[derive(Clone, serde::Serialize)]
pub struct ExternalWrite {
    pub key: String,
    pub action: u8,
}
worth_query_structured_value_binding!(pub Input for ExternalWrite { identity: "succession.destroy.input" });
worth_query_operation!(pub Operation for Schema, input Input; access {
    read_entities: [Node, Realization]; read_fields: [Key::at("Key"), RealizationKey::at("RealizationKey"), RealizedValue::at("RealizedValue")];
    read_relations: [Realizes]; writes: [RealizedValue::at("RealizedValue")]; deletes: [Node, Realization]; unlinks: [Realizes];
});
pub fn scope(input: &ExternalWrite) -> String {
    input.key.clone()
}
pub struct Outputs;
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[];
}
worth_query_mutation_binding!(pub Binding for ExternalWrite, schema Schema,
    identity "succession.destroy", input Input, operation Operation, result ResultBinding,
    idempotency u64, identity "succession.destroy.command", decision (Target, RealizationTarget), denial DenialBinding,
    handler identity "succession.destroy.handler", outputs Outputs,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Key, field Key::at("Key"), value scope,
);
use std::cell::Cell;
thread_local! {
    static LAST_REALIZES_EDGE: Cell<Option<worth_relational::facade::identity::RelationId>> =
        const { Cell::new(None) };
}
pub fn take_last_realizes_edge() -> Option<worth_relational::facade::identity::RelationId> {
    LAST_REALIZES_EDGE.with(Cell::take)
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn declared_durable_capabilities() -> Option<
        &'static [worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability],
    > {
        Some(&[worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &ExternalWrite,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<(Target, RealizationTarget), Denial> {
        let result = (|| {
            let node = reader.resolve_entity(Key::at("Key"), input.key.clone())?;
            reader.entity(&node, Node::reference())?;
            let edges = reader
                .relations_to(Realizes::reference(), &node)
                .map_err(HandlerExecutionDenial::new)?;
            let edge = edges
                .first()
                .expect("settled real producer realization");
            LAST_REALIZES_EDGE.with(|slot| slot.set(Some(edge.relation_id())));
            let output = edge.from();
            reader.entity(output, Realization::reference())?;
            if input.action == 3 {
                reader.field(output, RealizedValue::at("RealizedValue"))?;
            }
            Ok((
                reader.mutation_target(&node)?,
                reader.mutation_target(output)?,
            ))
        })();
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &ExternalWrite,
        target: (Target, RealizationTarget),
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<(), Denial> {
        let result = (|| {
            let node = writer.projected_entity(&target.0)?;
            let output = writer.projected_entity(&target.1)?;
            match input.action {
                0 => writer.unlink(Realizes::reference(), &output, &node),
                1 => writer.delete_entity(Realization::reference(), &output),
                2 => writer.delete_entity(Node::reference(), &node),
                3 => writer.write_field(&output, RealizedValue::at("RealizedValue"), 42),
                _ => unreachable!(),
            }
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation(
            Operation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
