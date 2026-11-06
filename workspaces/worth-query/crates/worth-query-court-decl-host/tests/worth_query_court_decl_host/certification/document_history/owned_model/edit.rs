use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Edit {
    pub key: String,
    pub value: u64,
}
#[derive(Clone, Debug)]
pub struct Edited;
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub enum Denial {
    InvalidModelGraph,
    ModelMismatch,
    ValueOutOfRange,
}
worth_query_structured_value_binding!(pub ResultBinding for Edited { identity: "owned-model.edit-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "owned-model.denial.v1" });
worth_query_value_binding!(pub ProducerDenialBinding for Denial { identity: "owned-model.producer-denial.v1", scalar: String, canonical: ValueDecodeBounds::new(64, 32, 4, 4), });
macro_rules! edit_binding {
    ($module:ident, $operation:ident, $entity:ident, $facts:ident, $key:ident, $value:ident, $identity:literal) => {
        pub mod $module {
            use super::*;
            #[derive(Clone, Debug, WorthValue)]
            #[worth_value(crate = "worth_query_decl::facade::application_schema")]
            pub struct Intent(pub Edit);
            worth_query_structured_value_binding!(pub Input for Intent { identity: $identity });
            worth_query_operation!(pub $operation for Schema, input Input; access { read_fields: [$key::at(stringify!($key)), $value::at(stringify!($value))]; writes: [$value::at(stringify!($value))]; });
            fn scope(input: &Intent) -> String { input.0.key.clone() }
            worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
                identity $identity, input Input, operation $operation, result ResultBinding,
                idempotency String, identity $identity, decision WorthQueryInvariantMutationTarget<Schema, $entity>, denial DenialBinding,
                handler identity $identity, program required, outputs NoApplicationMutationOutputs,
                principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
                scope $key, field $key::at(stringify!($key)), value scope,
            );
            pub struct Handler;
            impl OperationHandler<Schema, Binding> for Handler {
                fn decide(&self, input: &Intent, reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, $entity>, Denial> {
                    if input.0.value>1000 { return HandlerResult::DomainDenied(Denial::ValueOutOfRange); }
                    let result = (|| { let entity = reader.resolve_entity($key::at(stringify!($key)), input.0.key.clone())?; reader.field(&entity, $value::at(stringify!($value)))?; reader.mutation_target(&entity) })();
                    match result { Ok(target) => HandlerResult::Completed(target), Err(error) => HandlerResult::ExecutionDenied(error) }
                }
                fn build_candidate(&self, input: &Intent, target: WorthQueryInvariantMutationTarget<Schema, $entity>, writer: &mut CandidateWriter<'_, Schema, Binding>) -> HandlerResult<Edited, Denial> {
                    let result = writer.projected_entity(&target).and_then(|entity| writer.write_field(&entity, $value::at(stringify!($value)), input.0.value));
                    match result { Ok(()) => HandlerResult::Completed(Edited), Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)) }
                }
            }
        }
    };
}
edit_binding!(
    model,
    PatchModel,
    Model,
    ModelFacts,
    ModelKey,
    ModelValue,
    "owned-model.edit-source.v1"
);
edit_binding!(
    member,
    PatchMember,
    Member,
    MemberFacts,
    MemberKey,
    MemberValue,
    "owned-model.edit-member.v1"
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<model::PatchModel>(
            model::PatchModel::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<model::Binding>()
        .declared_operation::<member::PatchMember>(
            member::PatchMember::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<member::Binding>()
}
