//! Explicit Native source edits and retirement; no producer is driven by these actions.
use super::split_model::*;
use super::*;
#[derive(Clone, serde::Serialize)]
pub enum Change {
    Edit { value: u64, width: u64 },
    RetirePublication,
    CorruptRealization,
}
#[derive(Clone, serde::Serialize)]
pub struct Control {
    pub key: String,
    pub change: Change,
}
worth_query_structured_value_binding!(pub Input for Control { identity: "dispatch.control.input" });
worth_query_operation!(pub ControlSource for Schema, input Input; access {
    read_entities: [Node, Realization]; read_fields: [Key::at("Key"), Value::at("Value"), Width::at("Width"), ParentName::at("ParentName"), RealizationKey::at("RealizationKey"), RealizedValue::at("RealizedValue")];
    read_relations: [GeneratedBy, Realizes]; writes: [Value::at("Value"), Width::at("Width")]; deletes: [Node, Realization]; unlinks: [GeneratedBy, Realizes]; links: [Realizes];
});
pub enum Decision {
    Edit(Target),
    Retire {
        source: Target,
        output: RealizationTarget,
        children: Vec<Target>,
    },
    Corrupt {
        output: RealizationTarget,
        source: Target,
        foreign: Target,
    },
}
pub struct Outputs;
pub struct RetiredSources;
impl WorthQueryApplicationOutputRoleFamily for RetiredSources {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Node;
    const PREFIX: &'static str = "source.";
    const POSTURES: ApplicationMutationOutputPostureSet =
        ApplicationMutationOutputPostureSet::RETIRE;
    const MINIMUM: usize = 0;
}
pub struct Retired;
impl WorthQueryApplicationOutputRoleFamily for Retired {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Realization;
    const PREFIX: &'static str = "retired.";
    const POSTURES: ApplicationMutationOutputPostureSet =
        ApplicationMutationOutputPostureSet::RETIRE;
    const MINIMUM: usize = 0;
}
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[
        <Retired as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
        <RetiredSources as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
    ];
}
pub fn scope(input: &Control) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Control, schema Schema,
    identity "dispatch.control", input Input, operation ControlSource, result super::producer::ResultBinding,
    idempotency u64, identity "dispatch.control.command", decision Decision, denial DenialBinding,
    handler identity "dispatch.control.handler", outputs Outputs,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Key, field Key::at("Key"), value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Control,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        let result = (|| {
            let source = reader.resolve_entity(Key::at("Key"), input.key.clone())?;
            match input.change {
                Change::Edit { .. } => {
                    reader.field(&source, Value::at("Value"))?;
                    reader.field(&source, Width::at("Width"))?;
                    Ok(Decision::Edit(reader.mutation_target(&source)?))
                }
                Change::CorruptRealization => {
                    let output = reader
                        .resolve_entity(RealizationKey::at("RealizationKey"), input.key.clone())?;
                    let links = reader
                        .relations_from(Realizes::reference(), &output)
                        .map_err(HandlerExecutionDenial::new)?;
                    assert_eq!(links.len(), 1);
                    assert_eq!(links[0].to().entity_id(), source.entity_id());
                    let foreign = reader.resolve_entity(Key::at("Key"), "root".to_string())?;
                    Ok(Decision::Corrupt {
                        output: reader.mutation_target(&output)?,
                        source: reader.mutation_target(&source)?,
                        foreign: reader.mutation_target(&foreign)?,
                    })
                }
                Change::RetirePublication => {
                    let realized = reader
                        .relations_to(Realizes::reference(), &source)
                        .map_err(HandlerExecutionDenial::new)?;
                    assert_eq!(realized.len(), 1);
                    let output = realized[0].from();
                    reader.entity(output, Realization::reference())?;
                    let descendants = reader
                        .relations_to(GeneratedBy::reference(), &source)
                        .map_err(HandlerExecutionDenial::new)?;
                    let mut children = Vec::new();
                    for edge in descendants {
                        let child = edge.from();
                        // The caller first retires each dependent publication.
                        assert!(reader
                            .relations_to(Realizes::reference(), child)
                            .map_err(HandlerExecutionDenial::new)?
                            .is_empty());
                        assert!(reader
                            .relations_to(GeneratedBy::reference(), child)
                            .map_err(HandlerExecutionDenial::new)?
                            .is_empty());
                        reader.entity(child, Node::reference())?;
                        children.push(reader.mutation_target(child)?);
                    }
                    Ok(Decision::Retire {
                        source: reader.mutation_target(&source)?,
                        output: reader.mutation_target(output)?,
                        children,
                    })
                }
            }
        })();
        match result {
            Ok(decision) => HandlerResult::Completed(decision),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &Control,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<(), Denial> {
        let result = (|| {
            match (decision, &input.change) {
                (Decision::Edit(target), Change::Edit { value, width }) => {
                    let source = writer.projected_entity(&target)?;
                    writer.write_field(&source, Value::at("Value"), *value)?;
                    writer.write_field(&source, Width::at("Width"), *width)?;
                }
                (
                    Decision::Corrupt {
                        output,
                        source,
                        foreign,
                    },
                    Change::CorruptRealization,
                ) => {
                    let output = writer.projected_entity(&output)?;
                    let source = writer.projected_entity(&source)?;
                    let foreign = writer.projected_entity(&foreign)?;
                    writer.unlink(Realizes::reference(), &output, &source)?;
                    writer.link(
                        Realizes::reference(),
                        "corrupt-realization",
                        &output,
                        &foreign,
                    )?;
                }
                (
                    Decision::Retire {
                        source,
                        output,
                        children,
                    },
                    Change::RetirePublication,
                ) => {
                    let source = writer.projected_entity(&source)?;
                    let output = writer.projected_entity(&output)?;
                    writer.unlink(Realizes::reference(), &output, &source)?;
                    writer.delete_entity(Realization::reference(), &output)?;
                    writer.retire_member::<Retired>("realization", &output)?;
                    for (index, child) in children.into_iter().enumerate() {
                        let child = writer.projected_entity(&child)?;
                        writer.unlink(GeneratedBy::reference(), &child, &source)?;
                        writer.delete_entity(Node::reference(), &child)?;
                        writer.retire_member::<RetiredSources>(&index.to_string(), &child)?;
                    }
                }
                _ => unreachable!("decision retains this exact control intent"),
            }
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
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
        .declared_operation::<ControlSource>(
            ControlSource::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
