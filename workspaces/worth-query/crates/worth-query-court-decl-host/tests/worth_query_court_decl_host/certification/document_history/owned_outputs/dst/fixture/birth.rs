//! Bindings that create an entity: the authored creation, and the inverse
//! that reverses a retirement (`restore_retired`, a fresh occurrence). `e2`
//! refuses without a live `e1` (model `DepsSatisfied`).

/// `$module::{Intent, Binding, Operation, Born, Handler}`: creates member
/// `$key` with the intent's value, owned by model `a`. `$owner` is the
/// member that must be live first. `$idempotency` is the binding's key
/// type (`String` unless named).
macro_rules! birth_binding {
    ($module:ident, $operation:ident, $key:literal, $owner:expr, $identity:literal) => {
        birth_binding!($module, $operation, $key, $owner, $identity, String);
    };
    ($module:ident, $operation:ident, $key:literal, $owner:expr, $identity:literal, $idempotency:ident) => {
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
                read_entities: [Model, Member]; read_fields: [ModelKey::at("ModelKey"), MemberKey::at("MemberKey")]; creates: [Member]; writes: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue")]; links: [ModelMember];
            });
            fn scope(_: &Intent) -> String {
                "a".into()
            }
            worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
                identity $identity, input Input, operation $operation, result member_edit::ResultBinding,
                idempotency $idempotency, identity $identity, decision WorthQueryInvariantMutationTarget<Schema, Model>, denial RefusalBinding,
                handler identity $identity, program required, outputs Outputs,
                principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
                scope ModelKey, field ModelKey::at("ModelKey"), value scope,
            );
            pub struct Outputs;
            pub struct Born;
            impl WorthQueryApplicationOutputRole for Born {
                type Schema = Schema;
                type Contract = Outputs;
                type Entity = Member;
                type Action = WorthQueryCreateOutput;
                type Cardinality = WorthQueryExactlyOneOutput;
                const NAME: &'static str = concat!($identity, ".born");
            }
            impl ApplicationMutationOutputContract<Schema> for Outputs {
                const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
                    &[<Born as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
            }
            pub struct Handler;
            impl OperationHandler<Schema, Binding> for Handler {
                fn decide(
                    &self,
                    _: &Intent,
                    reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
                ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Model>, Refusal> {
                    let result = (|| {
                        let owner: Option<&str> = $owner;
                        if let Some(owner) = owner {
                            let found = reader
                                .resolve_optional_entity(MemberKey::at("MemberKey"), owner.to_owned())?;
                            if found.is_none() {
                                return Ok(Err(Refusal::OwnerAbsent));
                            }
                        }
                        let present = reader
                            .resolve_optional_entity(MemberKey::at("MemberKey"), String::from($key))?;
                        if present.is_some() {
                            return Ok(Err(Refusal::Present));
                        }
                        let model = reader.resolve_entity(ModelKey::at("ModelKey"), "a".into())?;
                        reader.mutation_target(&model).map(Ok)
                    })();
                    match result {
                        Ok(Ok(model)) => HandlerResult::Completed(model),
                        Ok(Err(refusal)) => HandlerResult::DomainDenied(refusal),
                        Err(cause) => HandlerResult::ExecutionDenied(cause),
                    }
                }
                fn build_candidate(
                    &self,
                    input: &Intent,
                    model: WorthQueryInvariantMutationTarget<Schema, Model>,
                    writer: &mut CandidateWriter<'_, Schema, Binding>,
                ) -> HandlerResult<member_edit::Edited, Refusal> {
                    let result = (|| {
                        let model = writer.projected_entity(&model)?;
                        let key = WorthQueryApplicationEntityKey::new(String::from($key)).unwrap();
                        let member = writer.create_entity_in_context(&model, Member::reference(), key)?;
                        writer.initialize_field(&member, MemberKey::at("MemberKey"), String::from($key))?;
                        writer.initialize_field(&member, MemberValue::at("MemberValue"), input.value)?;
                        writer.create_output::<Born>(&member)?;
                        writer.link(
                            ModelMember::reference(),
                            String::from(concat!("owns:", $key)),
                            &model,
                            &member,
                        )
                    })();
                    match result {
                        Ok(()) => HandlerResult::Completed(member_edit::Edited),
                        Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
                    }
                }
            }
        }
    };
}
