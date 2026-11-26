//! Bindings that retire an entity and the outputs derived from it, with no
//! cascade: the authored retirement, and the inverse that reverses a creation
//! (`retire_created`). `e1` refuses while `e2` is live (model
//! `DepsSatisfied`).

/// `$module::{Intent, Binding, Operation, Died, Handler}`: retires member
/// `$key`, its model edge and its `ChildOf` outputs. `$owned` is the member
/// that must not be live. `$idempotency` is the binding's key type
/// (`String` unless named).
macro_rules! death_binding {
    ($module:ident, $operation:ident, $key:literal, $owned:expr, $identity:literal) => {
        death_binding!($module, $operation, $key, $owned, $identity, String);
    };
    ($module:ident, $operation:ident, $key:literal, $owned:expr, $identity:literal, $idempotency:ident) => {
        pub mod $module {
            use super::*;
            #[derive(Clone, Debug, WorthValue)]
            #[worth_value(crate = "worth_query_decl::facade::application_schema")]
            pub struct Intent;
            worth_query_structured_value_binding!(pub Input for Intent { identity: $identity });
            // The operation's schema name is its type name: unique per binding.
            pub type Operation = $operation;
            worth_query_operation!(pub $operation for Schema, input Input; access {
                read_entities: [Model, Member, ChildOutput]; read_fields: [ModelKey::at("ModelKey"), MemberKey::at("MemberKey"), MemberValue::at("MemberValue"), ChildKey::at("ChildKey")]; read_relations: [ModelMember, ChildOf]; deletes: [Member, ChildOutput]; unlinks: [ModelMember, ChildOf];
            });
            fn scope(_: &Intent) -> String {
                "a".into()
            }
            worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
                identity $identity, input Input, operation $operation, result member_edit::ResultBinding,
                idempotency $idempotency, identity $identity, decision Targets, denial RefusalBinding,
                handler identity $identity, program required, outputs Outputs,
                principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
                scope ModelKey, field ModelKey::at("ModelKey"), value scope,
            );
            pub struct Outputs;
            pub struct Died;
            impl WorthQueryApplicationOutputRole for Died {
                type Schema = Schema;
                type Contract = Outputs;
                type Entity = Member;
                type Action = WorthQueryRetireOutput;
                type Cardinality = WorthQueryExactlyOneOutput;
                const NAME: &'static str = concat!($identity, ".died");
            }
            pub struct DiedA;
            impl WorthQueryApplicationOutputRole for DiedA {
                type Schema = Schema;
                type Contract = Outputs;
                type Entity = ChildOutput;
                type Action = WorthQueryRetireOutput;
                type Cardinality = WorthQueryAtMostOneOutput;
                const NAME: &'static str = concat!($identity, ".died-a");
            }
            pub struct DiedB;
            impl WorthQueryApplicationOutputRole for DiedB {
                type Schema = Schema;
                type Contract = Outputs;
                type Entity = ChildOutput;
                type Action = WorthQueryRetireOutput;
                type Cardinality = WorthQueryAtMostOneOutput;
                const NAME: &'static str = concat!($identity, ".died-b");
            }
            impl ApplicationMutationOutputContract<Schema> for Outputs {
                const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
                    <Died as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                    <DiedA as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                    <DiedB as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                ];
            }
            pub struct Targets {
                member: WorthQueryInvariantMutationTarget<Schema, Member>,
                owners: Vec<WorthQueryInvariantMutationTarget<Schema, Model>>,
                /// Each derived output, `true` for `dst-a`'s.
                children: Vec<(WorthQueryInvariantMutationTarget<Schema, ChildOutput>, bool)>,
            }
            pub struct Handler;
            impl OperationHandler<Schema, Binding> for Handler {
                fn decide(
                    &self,
                    _: &Intent,
                    reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
                ) -> HandlerResult<Targets, Refusal> {
                    let result = (|| {
                        let owned: Option<&str> = $owned;
                        if let Some(owned) = owned {
                            let found = reader
                                .resolve_optional_entity(MemberKey::at("MemberKey"), owned.to_owned())?;
                            if found.is_some() {
                                return Ok(Err(Refusal::OwnedLive));
                            }
                        }
                        let Some(member) = reader
                            .resolve_optional_entity(MemberKey::at("MemberKey"), String::from($key))?
                        else {
                            return Ok(Err(Refusal::Absent));
                        };
                        reader.entity(&member, Member::reference())?;
                        reader.field(&member, MemberValue::at("MemberValue"))?;
                        let mut owners = Vec::new();
                        for edge in reader
                            .relations_to(ModelMember::reference(), &member)
                            .map_err(HandlerExecutionDenial::new)?
                        {
                            reader.entity(edge.from(), Model::reference())?;
                            owners.push(reader.mutation_target(edge.from())?);
                        }
                        let mut children = Vec::new();
                        for edge in reader
                            .relations_from(ChildOf::reference(), &member)
                            .map_err(HandlerExecutionDenial::new)?
                        {
                            reader.entity(edge.to(), ChildOutput::reference())?;
                            let key: Option<String> = reader.field(edge.to(), ChildKey::at("ChildKey"))?;
                            let a = key.is_some_and(|key| key.starts_with("dst-a:"));
                            children.push((reader.mutation_target(edge.to())?, a));
                        }
                        let member = reader.mutation_target(&member)?;
                        Ok(Ok(Targets { member, owners, children }))
                    })();
                    match result {
                        Ok(Ok(targets)) => HandlerResult::Completed(targets),
                        Ok(Err(refusal)) => HandlerResult::DomainDenied(refusal),
                        Err(cause) => HandlerResult::ExecutionDenied(cause),
                    }
                }
                fn build_candidate(
                    &self,
                    _: &Intent,
                    targets: Targets,
                    writer: &mut CandidateWriter<'_, Schema, Binding>,
                ) -> HandlerResult<member_edit::Edited, Refusal> {
                    let result = (|| {
                        let member = writer.projected_entity(&targets.member)?;
                        for owner in &targets.owners {
                            let owner = writer.projected_entity(owner)?;
                            writer.unlink(ModelMember::reference(), &owner, &member)?;
                        }
                        for (child, a) in &targets.children {
                            let child = writer.projected_entity(child)?;
                            writer.unlink(ChildOf::reference(), &member, &child)?;
                            writer.delete_entity(ChildOutput::reference(), &child)?;
                            if *a {
                                writer.retire_output::<DiedA>(&child)?;
                            } else {
                                writer.retire_output::<DiedB>(&child)?;
                            }
                        }
                        writer.delete_entity(Member::reference(), &member)?;
                        writer.retire_output::<Died>(&member)
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
