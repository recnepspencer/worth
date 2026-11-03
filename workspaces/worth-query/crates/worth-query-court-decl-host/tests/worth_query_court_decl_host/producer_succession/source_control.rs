use super::*;
#[derive(Clone, serde::Serialize)]
pub struct Control {
    pub key: String,
    pub width: u64,
}
worth_query_structured_value_binding!(pub Input for Control { identity: "succession.control.input" });
worth_query_operation!(pub ControlSource for Schema, input Input; access { read_entities: [Node]; read_fields: [Key, Width]; writes: [Width]; });
pub fn scope(input: &Control) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Control, schema Schema,
 identity "succession.control", input Input, operation ControlSource, result ResultBinding,
 idempotency u64, identity "succession.control.command", decision Target, denial DenialBinding,
 handler identity "succession.control.handler", outputs Outputs,
 principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
 scope Node, Facts, Key, String, ReadOnly, NoApplicationUnit, field Key::reference(), value scope,
);
pub struct Outputs;
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[];
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Control,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Target, Denial> {
        let result = (|| {
            let node = reader.resolve_entity(Key::reference(), input.key.clone())?;
            reader.field(&node, Width::reference())?;
            reader.mutation_target(&node)
        })();
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &Control,
        target: Target,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<(), Denial> {
        let result = (|| {
            let node = writer.projected_entity(&target)?;
            writer.write_field(&node, Width::reference(), input.width)
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
    schema.correction::<correction::Resize>()
}
