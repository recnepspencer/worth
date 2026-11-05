//! Ordinary callers cannot destroy another producer's canonical records.
use super::{realization::*, *};
#[derive(Clone, serde::Serialize)]
pub struct ExternalWrite {
    pub key: String,
    pub action: u8,
}
worth_query_structured_value_binding!(pub Input for ExternalWrite { identity: "succession.destroy.input" });
worth_query_operation!(pub Operation for Schema, input Input; access {
    read_entities: [Node, Realization]; read_fields: [Key, RealizationKey, RealizedValue];
    read_relations: [Realizes]; writes: [RealizedValue]; deletes: [Node, Realization]; unlinks: [Realizes];
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
    scope Node, Facts, Key, String, ReadOnly, NoApplicationUnit, field Key::reference(), value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &ExternalWrite,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<(Target, RealizationTarget), Denial> {
        let result = (|| {
            let node = reader.resolve_entity(Key::reference(), input.key.clone())?;
            reader.entity(&node, Node::reference())?;
            let edges = reader
                .relations_to(Realizes::reference(), &node)
                .map_err(HandlerExecutionDenial::new)?;
            let output = edges
                .first()
                .expect("settled real producer realization")
                .from();
            reader.entity(output, Realization::reference())?;
            if input.action == 3 {
                reader.field(output, RealizedValue::reference())?;
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
                3 => writer.write_field(&output, RealizedValue::reference(), 42),
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
