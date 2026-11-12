//! A D-only membership cannot supply the dependent source's C READ grant.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
    pub value: u64,
    pub create: bool,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.synchronous-restricted-source.v1" });
worth_query_operation!(pub CreateRestrictedSource for Schema, input Input; access {
    read_entities: [Model]; read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue")]; creates: [Member];
    writes: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue"), ModelValue::at("ModelValue")]; links: [ModelMember];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.synchronous-restricted-source.v1", input Input, operation CreateRestrictedSource, result ResultBinding,
    idempotency String, identity "owned-outputs.synchronous-restricted-source.v1", decision WorthQueryInvariantMutationTarget<Schema, Model>, denial DenialBinding,
    handler identity "owned-outputs.synchronous-restricted-source.v1", program required,
    coupled_outputs |visitor| { visitor.required::<program::StartRestrictedOwned>(); },
    outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        _: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Model>, Denial> {
        match reader
            .resolve_entity(ModelKey::at("ModelKey"), "a".into())
            .and_then(|model| {
                reader.field(&model, ModelValue::at("ModelValue"))?;
                reader.mutation_target(&model)
            }) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Intent,
        target: WorthQueryInvariantMutationTarget<Schema, Model>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Edited, Denial> {
        let result = (|| {
            let model = writer.projected_entity(&target)?;
            if !input.create {
                return writer.write_field(&model, ModelValue::at("ModelValue"), 2);
            }
            let member = writer.create_entity_in_context(
                &model,
                Member::reference(),
                WorthQueryApplicationEntityKey::new(input.key.clone()).unwrap(),
            )?;
            writer.initialize_field(&member, MemberKey::at("MemberKey"), input.key.clone())?;
            writer.initialize_field(&member, MemberValue::at("MemberValue"), input.value)?;
            writer.link(
                ModelMember::reference(),
                "new-restricted-source".to_owned(),
                &model,
                &member,
            )
        })();
        match result {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<CreateRestrictedSource>(
            CreateRestrictedSource::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
impl WorthQueryApplicationRequiredOutputSource<Schema, program::StartRestrictedOwned> for Binding {
    fn demand_from_source(
        input: &Intent,
    ) -> Result<restricted_owned::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(restricted_owned::Demand(input.key.clone()))
    }
}
fn run(worker: bool) {
    use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let mut refusals = Vec::new();
    for create in [true, false] {
        let host = if worker {
            world::install_worker_only()
        } else {
            world::install()
        };
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        assert_eq!(
            request
                .query(query::member::Read("m".into()))
                .execute()
                .unwrap()
                .rows()
                .len(),
            1,
            "C grants this principal READ for the existing owned member"
        );
        let checkpoint = super::super::failure::checkpoint(&host);
        let head = courts::native_head(&host);
        let contacts = host.producer_contacts_on_this_thread_for_test();
        // At D the new ModelMember edge completes the installed policy path. At C
        // this same membership and its READ permission do not exist.
        let outcome = request
            .advanced()
            .mutate(Intent {
                key: "fresh-member".into(),
                value: 2,
                create,
            })
            .without_source()
            .idempotency(&"restricted-new-source".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = outcome
        else {
            panic!("{outcome:?}")
        };
        let producer = denial
            .coupled_producer_denial()
            .expect("the actual dependent source READ denial");
        assert_eq!(
            producer.kind(),
            WorthQueryOutputDemandDenialKind::SourceScope(
                WorthQueryEntityResolutionRefusalKind::UnknownEntity
            ),
            "{producer:?}"
        );
        assert!(
            !format!("{producer:?}").contains("LocalSlot"),
            "no privately resolved D generation"
        );
        refusals.push((producer.kind(), producer.subject().to_owned()));
        assert!(
            producer
                .subject()
                .contains("owned-outputs.restricted-owned.initial"),
            "{producer:?}"
        );
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
        assert_eq!(courts::native_head(&host), head);
        super::super::failure::assert_model_unchanged(&host, &checkpoint);
    }
    assert_eq!(
        refusals[0], refusals[1],
        "a C-denied audience cannot distinguish source presence at D"
    );
}
#[test]
fn a_new_source_cannot_borrow_its_candidates_read_grant() {
    run(false);
}
#[test]
fn worker_only_a_new_source_cannot_borrow_its_candidates_read_grant() {
    run(true);
}
