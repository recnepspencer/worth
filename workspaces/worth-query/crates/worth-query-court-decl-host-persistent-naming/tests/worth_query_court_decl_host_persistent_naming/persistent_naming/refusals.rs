//! Fail-closed boundaries of installed persistent names: a schemed kind can
//! neither be seeded nor created without a declared birth.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "persistent-naming.create-member.v1" });
worth_query_operation!(pub CreateMember for Schema, input Input; access {
    read_entities: [Model]; read_fields: [ModelKey::at("ModelKey")];
    writes: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue")]; creates: [Member];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "persistent-naming.create-member.v1", input Input, operation CreateMember, result edit::ResultBinding,
    idempotency String, identity "persistent-naming.create-member-key.v1", decision WorthQueryInvariantMutationTarget<Schema, Model>, denial edit::DenialBinding,
    handler identity "persistent-naming.create-member-handler.v1", program required, outputs NoApplicationMutationOutputs,
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
        let result = reader
            .resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())
            .and_then(|model| reader.mutation_target(&model));
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
            let context = writer.projected_entity(&model)?;
            let created = writer.create_entity_in_context(
                &context,
                Member::reference(),
                WorthQueryApplicationEntityKey::new(&input.key).unwrap(),
            )?;
            writer.initialize_field(&created, MemberKey::at("MemberKey"), input.key.clone())?;
            writer.initialize_field(&created, MemberValue::at("MemberValue"), 1)?;
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
        .declared_operation::<CreateMember>(
            CreateMember::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}

#[test]
fn a_seed_of_a_schemed_kind_without_its_name_refuses_installation() {
    let refused = match world::try_install(false) {
        Ok(_) => panic!("an unnamed seed of a schemed kind must refuse installation"),
        Err(refused) => refused,
    };
    assert!(
        refused.contains("must declare its persistent name"),
        "{refused}"
    );
}

#[test]
fn an_installed_operation_creating_a_schemed_member_without_a_birth_refuses() {
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "unnamed".into(),
        })
        .without_source()
        .idempotency(&"create-unnamed".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    let rendered = format!("{outcome:?}");
    assert!(
        !matches!(
            outcome,
            Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
        ),
        "Relational must refuse the unnamed birth: {rendered}"
    );
    assert!(
        rendered.contains("requires a persistent-name birth under installed scheme"),
        "{rendered}"
    );
    let on_world = request.on_branch(host.current_world());
    assert!(
        on_world
            .query(query::member::Read("unnamed".into()))
            .execute()
            .is_err(),
        "the refused creation leaves no member behind"
    );
}
