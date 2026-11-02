use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Input(pub GraphRow);
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "source-subgraph.generate-input.v1" });
// Existing generated entities cannot be acquired: only Source is readable.
// Generated links can be authored only between this candidate's created handles.
worth_query_operation!(pub Generate for Schema, input InputBinding; access {
    read_fields: [SourceKey, SourceValue]; read_relations: [SourceMember];
    writes: [GeneratedKey, GeneratedValue]; creates: [Generated]; links: [GeneratedMember];
});
pub struct Outputs;
macro_rules! role {
    ($role:ident, $name:literal) => {
        pub struct $role;
        impl WorthQueryApplicationOutputRole for $role {
            type Schema = Schema;
            type Contract = Outputs;
            type Entity = Generated;
            type Action = WorthQueryCreateOutput;
            type Cardinality = WorthQueryExactlyOneOutput;
            const NAME: &'static str = $name;
        }
    };
}
role!(Root, "root");
role!(Left, "left");
role!(Right, "right");
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[Root::DESCRIPTOR, Left::DESCRIPTOR, Right::DESCRIPTOR];
}
type Scope = ApplicationMutationFieldScope<
    Schema,
    Source,
    SourceFacts,
    SourceKey,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
pub struct Binding;
impl ApplicationMutationBinding<Schema> for Binding {
    type Input = Input;
    type InputBinding = InputBinding;
    type Result = edit::Edited;
    type ResultBinding = edit::ResultBinding;
    type IdempotencyKey = u64;
    type Operation = Generate;
    type Decision = Decision;
    type Denial = edit::Denial;
    type DenialBinding = edit::DenialBinding;
    type Output = Outputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<graph_read::source::Query>;
    const IDENTITY: &'static str = "source-subgraph.generate.v1";
    const HANDLER_IDENTITY: &'static str = "source-subgraph.generate-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "source-subgraph.generate-command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Source,
        SourceFacts,
        SourceKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        SourceKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Actor,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        Actor::reference()
    }
}
impl ApplicationMutationIntent<Schema> for Input {
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(SourceKey::reference(), self.0.key.clone())
    }
}
pub struct Decision {
    source: WorthQueryInvariantMutationTarget<Schema, Source>,
    values: [u64; 3],
}
pub struct Handler {
    pub omit_right_role: bool,
}
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, edit::Denial> {
        let result = (|| {
            let source = reader.resolve_entity(SourceKey::reference(), input.0.key.clone())?;
            let Some(value) = reader.field(&source, SourceValue::reference())? else {
                return Ok(Err(edit::Denial::InvalidSourceGraph));
            };
            let edges = reader
                .relations_from(SourceMember::reference(), &source)
                .map_err(HandlerExecutionDenial::new)?;
            if edges.len() != 2 {
                return Ok(Err(edit::Denial::InvalidSourceGraph));
            }
            let mut members = Vec::with_capacity(2);
            for edge in edges {
                let child = edge.to();
                let Some(key) = reader.field(child, SourceKey::reference())? else {
                    return Ok(Err(edit::Denial::InvalidSourceGraph));
                };
                let Some(value) = reader.field(child, SourceValue::reference())? else {
                    return Ok(Err(edit::Denial::InvalidSourceGraph));
                };
                // This small fixture is a star, not an arbitrary reachability walk.
                if !reader
                    .relations_from(SourceMember::reference(), child)
                    .map_err(HandlerExecutionDenial::new)?
                    .is_empty()
                {
                    return Ok(Err(edit::Denial::InvalidSourceGraph));
                }
                members.push(MemberRow { key, value });
            }
            members.sort_by(|a, b| a.key.cmp(&b.key));
            let actual = GraphRow {
                key: input.0.key.clone(),
                value,
                members,
            };
            if actual.key != "source"
                || actual.members.len() != 2
                || actual.members[0].key != "left"
                || actual.members[1].key != "right"
            {
                return Ok(Err(edit::Denial::InvalidSourceGraph));
            }
            if actual != input.0 {
                return Ok(Err(edit::Denial::SourceMismatch));
            }
            Ok(Ok(Decision {
                source: reader.mutation_target(&source)?,
                values: [value, actual.members[0].value, actual.members[1].value],
            }))
        })();
        match result {
            Ok(Ok(value)) => HandlerResult::Completed(value),
            Ok(Err(denial)) => HandlerResult::DomainDenied(denial),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        _: &Input,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            let source = writer.projected_entity(&decision.source)?;
            let mut nodes = Vec::new();
            for (key, value) in ["generated", "left", "right"]
                .into_iter()
                .zip(decision.values)
            {
                let entity = writer.create_entity_in_context(
                    &source,
                    Generated::reference(),
                    WorthQueryApplicationEntityKey::new(format!("generated:{key}")).unwrap(),
                )?;
                writer.initialize_field(&entity, GeneratedKey::reference(), key.to_owned())?;
                writer.initialize_field(&entity, GeneratedValue::reference(), value)?;
                nodes.push(entity);
            }
            writer.link(
                GeneratedMember::reference(),
                "generated:left",
                &nodes[0],
                &nodes[1],
            )?;
            writer.link(
                GeneratedMember::reference(),
                "generated:right",
                &nodes[0],
                &nodes[2],
            )?;
            writer.create_output::<Root>(&nodes[0])?;
            writer.create_output::<Left>(&nodes[1])?;
            // The hostile twin stages Right AND its link, but omits its certificate.
            if !self.omit_right_role {
                writer.create_output::<Right>(&nodes[2])?;
            }
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<Generate>(
            Generate::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
