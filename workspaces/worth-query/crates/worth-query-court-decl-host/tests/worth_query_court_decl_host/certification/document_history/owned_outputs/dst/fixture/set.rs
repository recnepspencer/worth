//! The binding that sets a live entity's value; the same occurrence and
//! generation stay.

/// `$module::{Intent, Binding, Operation, Handler}`: writes member `$key`'s
/// value, refusing `Absent` when it is not live.
macro_rules! set_binding {
    ($module:ident, $operation:ident, $key:literal, $identity:literal) => {
        pub mod $module {
            use super::*;
            #[derive(Clone, Debug, WorthValue)]
            #[worth_value(crate = "worth_query_decl::facade::application_schema")]
            pub struct Intent {
                pub value: u64,
            }
            worth_query_structured_value_binding!(pub Input for Intent { identity: $identity });
            // The operation's schema name is its type name: unique per binding.
            pub type Operation = $operation;
            worth_query_operation!(pub $operation for Schema, input Input; access {
                read_entities: [Model, Member]; read_fields: [ModelKey::at("ModelKey"), MemberKey::at("MemberKey"), MemberValue::at("MemberValue")]; writes: [MemberValue::at("MemberValue")];
            });
            fn scope(_: &Intent) -> String {
                "a".into()
            }
            worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
                identity $identity, input Input, operation $operation, result member_edit::ResultBinding,
                idempotency String, identity $identity, decision WorthQueryInvariantMutationTarget<Schema, Member>, denial RefusalBinding,
                handler identity $identity, program required, outputs NoApplicationMutationOutputs,
                principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
                scope ModelKey, field ModelKey::at("ModelKey"), value scope,
            );
            pub struct Handler;
            impl OperationHandler<Schema, Binding> for Handler {
                fn decide(
                    &self,
                    _: &Intent,
                    reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
                ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Member>, Refusal> {
                    let result = (|| {
                        let Some(member) = reader
                            .resolve_optional_entity(MemberKey::at("MemberKey"), String::from($key))?
                        else {
                            return Ok(Err(Refusal::Absent));
                        };
                        reader.field(&member, MemberValue::at("MemberValue"))?;
                        reader.mutation_target(&member).map(Ok)
                    })();
                    match result {
                        Ok(Ok(member)) => HandlerResult::Completed(member),
                        Ok(Err(refusal)) => HandlerResult::DomainDenied(refusal),
                        Err(cause) => HandlerResult::ExecutionDenied(cause),
                    }
                }
                fn build_candidate(
                    &self,
                    input: &Intent,
                    member: WorthQueryInvariantMutationTarget<Schema, Member>,
                    writer: &mut CandidateWriter<'_, Schema, Binding>,
                ) -> HandlerResult<member_edit::Edited, Refusal> {
                    let result = writer.projected_entity(&member).and_then(|member| {
                        writer.write_field(&member, MemberValue::at("MemberValue"), input.value)
                    });
                    match result {
                        Ok(()) => HandlerResult::Completed(member_edit::Edited),
                        Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
                    }
                }
            }
        }
    };
}
