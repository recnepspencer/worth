//! A model holds a member by persistent name in its own record: an installed
//! action carries a `NamedTarget` and a bare `WorthQueryName` as ordinary
//! input fields, and its handler writes them to ordinary record columns.
use super::*;
use worth_query_decl::facade::application_naming::{
    NamedTarget, NamedTargetDelegate, WorthQueryName, WorthQueryNameDelegate,
};

/// The reference model `a` holds from now on.
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    #[worth_value(delegate = NamedTargetDelegate<Member>)]
    pub target: NamedTarget<Member>,
    #[worth_value(delegate = OptionalDelegate<WorthQueryNameDelegate<Member>>)]
    pub anchor: Option<WorthQueryName<Member>>,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "persistent-naming.hold.v1" });
worth_query_operation!(pub HoldName for Schema, input Input; access {
    read_fields: [ModelKey::at("ModelKey"), HeldTarget::at("HeldTarget"), HeldAnchor::at("HeldAnchor")];
    writes: [HeldTarget::at("HeldTarget"), HeldAnchor::at("HeldAnchor")];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "persistent-naming.hold.v1", input Input, operation HoldName, result edit::ResultBinding,
    idempotency String, identity "persistent-naming.hold-key.v1", decision WorthQueryInvariantMutationTarget<Schema, Model>, denial edit::DenialBinding,
    handler identity "persistent-naming.hold-handler.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        _: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Model>, edit::Denial> {
        let result = (|| {
            let model = reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
            reader.field(&model, HeldTarget::at("HeldTarget"))?;
            reader.field(&model, HeldAnchor::at("HeldAnchor"))?;
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
        let result = writer.projected_entity(&model).and_then(|model| {
            writer.write_optional_field(
                &model,
                HeldTarget::at("HeldTarget"),
                Some(input.target.clone()),
            )?;
            writer.write_optional_field(&model, HeldAnchor::at("HeldAnchor"), input.anchor)
        });
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
        .declared_operation::<HoldName>(
            HoldName::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
