//! The authored edit that demands the producer's output: it writes the
//! model's value and, when `collide`, also creates and names one member under
//! the very birth the producer's primary takes, so the envelope's composed
//! plan mints one name twice.
use super::*;

#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
    pub value: u64,
    pub collide: bool,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "persistent-naming.shape.v1" });
worth_query_operation!(pub Shape for Schema, input Input; access {
    read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue")];
    writes: [ModelValue::at("ModelValue"), MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
    creates: [Member]; links: [ModelMember];
});
fn scope(input: &Intent) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "persistent-naming.shape.v1", input Input, operation Shape, result edit::ResultBinding,
    idempotency String, identity "persistent-naming.shape-key.v1", decision WorthQueryInvariantMutationTarget<Schema, Model>, denial edit::DenialBinding,
    handler identity "persistent-naming.shape-handler.v1", program required,
    coupled_outputs |visitor| { visitor.required::<super::StartPieces>(); },
    outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
impl WorthQueryApplicationRequiredOutputSource<Schema, super::StartPieces> for Binding {
    fn demand_from_source(
        input: &Intent,
    ) -> Result<super::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(super::Demand(input.key.clone()))
    }
}

pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Model>, edit::Denial> {
        let result = (|| {
            let model = reader.resolve_entity(ModelKey::at("ModelKey"), input.key.clone())?;
            reader.field(&model, ModelValue::at("ModelValue"))?;
            reader.mutation_target(&model)
        })();
        match result {
            Ok(model) => HandlerResult::Completed(model),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &Intent,
        model: WorthQueryInvariantMutationTarget<Schema, Model>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            let model = writer.projected_entity(&model)?;
            writer.write_field(&model, ModelValue::at("ModelValue"), input.value)?;
            if input.collide {
                let key = format!("shape-{}-collide", input.key);
                let member = writer.create_entity_in_context(
                    &model,
                    Member::reference(),
                    WorthQueryApplicationEntityKey::new(key.clone()).unwrap(),
                )?;
                writer.initialize_field(&member, MemberKey::at("MemberKey"), key.clone())?;
                writer.initialize_field(&member, MemberValue::at("MemberValue"), 1)?;
                writer.link(
                    ModelMember::reference(),
                    format!("owns:{key}"),
                    &model,
                    &member,
                )?;
                let occurrence = OccurrencePath::root(&super::occurrence(&input.key)).unwrap();
                let mut names = writer.naming_scope(occurrence);
                let birth = names.birth(MemberRole::Authored);
                names.name_entity(&member, birth)?;
            }
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<Shape>(
            Shape::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
