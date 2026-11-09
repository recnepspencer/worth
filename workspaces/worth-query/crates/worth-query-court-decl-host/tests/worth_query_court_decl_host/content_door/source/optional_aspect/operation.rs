//! Ordinary admitted writes create, replace and remove the optional content aspect.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Edit {
    pub key: String,
    pub create: bool,
    pub samples: Option<Vec<i64>>,
}
#[derive(Clone, Debug)]
pub struct Edited;
worth_query_structured_value_binding!(pub Input for Edit {identity:"optional-content.edit.v1"});
worth_query_structured_value_binding!(pub Output for Edited {identity:"optional-content.edited.v1"});
worth_query_operation!(pub EditOptionalSamples for Schema,input Input);
worth_query_operation_creates!(EditOptionalSamples=>[Record]);
worth_query_operation_links!(EditOptionalSamples=>[ContainsOptionalSamples]);
worth_query_operation_removes_aspects!(EditOptionalSamples=>[Samples]);
fn scope(_: &Edit) -> String {
    "record".into()
}
worth_query_mutation_binding!(pub Binding for Edit,schema Schema,
 identity "optional-content.binding.v1",input Input,operation EditOptionalSamples,result Output,
 idempotency String,identity "optional-content.key.v1",decision Decision,denial super::super::super::operation::DenialBinding,
 handler identity "optional-content.handler.v1",program required,outputs NoApplicationMutationOutputs,
 principal Actor,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
 scope Key,field Key::at("Key"),value scope,);
pub struct Decision {
    root: WorthQueryInvariantMutationTarget<Schema, Record>,
    target: Option<WorthQueryInvariantMutationTarget<Schema, Record>>,
}
impl ApplicationDeclaredOperation<Schema> for EditOptionalSamples {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema
            .operation_read_field(Self::reference(), Key::at("Key"))
            .operation_read_field(Self::reference(), Values::at("Values"))
            .operation_read_field(Self::reference(), OptionalValues::at("OptionalValues"))
            .operation_read_field(Self::reference(), UnboundValues::at("UnboundValues"))
            .operation_write(Self::reference(), Key::at("Key"))
            .operation_write(Self::reference(), Values::at("Values"))
            .operation_create(Self::reference(), Record::reference())
            .operation_link(Self::reference(), ContainsOptionalSamples::reference())
            .operation_remove_aspect(Self::reference(), Samples::reference())
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<EditOptionalSamples>(
            EditOptionalSamples::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Edit,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, super::super::super::operation::Denial> {
        let result = (|| {
            let root = reader.resolve_entity(Key::at("Key"), "record".to_owned())?;
            let target = if input.create {
                None
            } else {
                let entity = reader.resolve_entity(Key::at("Key"), input.key.clone())?;
                reader
                    .content(&entity, samples())
                    .map_err(HandlerExecutionDenial::new)?;
                reader.field(&entity, OptionalValues::at("OptionalValues"))?;
                reader.field(&entity, UnboundValues::at("UnboundValues"))?;
                Some(reader.mutation_target(&entity)?)
            };
            Ok(Decision {
                root: reader.mutation_target(&root)?,
                target,
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Edit,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Edited, super::super::super::operation::Denial> {
        let result = (|| {
            let root = writer
                .projected_entity(&decision.root)
                .map_err(HandlerExecutionDenial::new)?;
            if input.create {
                let child = writer
                    .create_entity_in_context(
                        &root,
                        Record::reference(),
                        WorthQueryApplicationEntityKey::new(&input.key).unwrap(),
                    )
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .initialize_field(&child, Key::at("Key"), input.key.clone())
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .link(
                        ContainsOptionalSamples::reference(),
                        input.key.clone(),
                        &root,
                        &child,
                    )
                    .map_err(HandlerExecutionDenial::new)?;
            } else {
                let target = writer
                    .projected_entity(decision.target.as_ref().unwrap())
                    .map_err(HandlerExecutionDenial::new)?;
                if let Some(values) = &input.samples {
                    let grid = codec::Grid {
                        rows: 1,
                        columns: values.len().try_into().unwrap(),
                        samples: values.clone(),
                        corrupt_result: false,
                        swallow_sink_refusal: false,
                        cancel_during_encoding: None,
                    };
                    writer
                        .write_content(&target, samples(), &grid)
                        .map_err(HandlerExecutionDenial::new)?;
                } else {
                    writer
                        .remove_aspect(&target, Samples::reference())
                        .map_err(HandlerExecutionDenial::new)?;
                }
            }
            Ok(Edited)
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
