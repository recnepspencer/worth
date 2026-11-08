//! A producer source created by this edit is present only in the real candidate.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
    pub value: u64,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.synchronous-new-source.v1" });
worth_query_operation!(pub CreateSource for Schema, input Input; access {
    read_entities: [Model]; read_fields: [ModelKey::at("ModelKey")];
    creates: [Model]; writes: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue")];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.synchronous-new-source.v1", input Input, operation CreateSource, result ResultBinding,
    idempotency String, identity "owned-outputs.synchronous-new-source.v1", decision WorthQueryInvariantMutationTarget<Schema, Model>, denial DenialBinding,
    handler identity "owned-outputs.synchronous-new-source.v1", program required,
    coupled_outputs |visitor| { visitor.required::<program::StartFirst>(); visitor.required::<program::StartFromFirst>(); visitor.required::<program::StartSecond>(); },
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
            .and_then(|model| reader.mutation_target(&model))
        {
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
            let parent = writer.projected_entity(&target)?;
            let source = writer.create_entity_in_context(
                &parent,
                Model::reference(),
                WorthQueryApplicationEntityKey::new(input.key.clone()).unwrap(),
            )?;
            writer.initialize_field(&source, ModelKey::at("ModelKey"), input.key.clone())?;
            writer.initialize_field(&source, ModelValue::at("ModelValue"), input.value)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let schema = schema
        .declared_operation::<CreateSource>(
            CreateSource::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>();
    restricted::declare(schema)
}
macro_rules! demand {
    ($connection:ident, $family:ident) => {
        impl WorthQueryApplicationRequiredOutputSource<Schema, program::$connection> for Binding {
            fn demand_from_source(
                input: &Intent,
            ) -> Result<$family::Demand, WorthQueryRequiredOutputConnectionDenial> {
                Ok($family::Demand(input.key.clone()))
            }
        }
    };
}
demand!(StartFirst, first);
demand!(StartSecond, second);
impl WorthQueryApplicationRequiredOutputSource<Schema, program::StartFromFirst> for Binding {
    fn demand_from_source(
        input: &Intent,
    ) -> Result<from_first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(from_first::Demand(format!("first:{}", input.key)))
    }
}

fn run(worker: bool) {
    use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = if worker {
        world::install_worker_only()
    } else {
        world::install()
    };
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "fresh".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"new-source-edit".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    let head = courts::native_head(&host);
    assert_eq!(
        head.1,
        [before.0],
        "the source and all three producers are one commit"
    );
    assert_eq!(
        head.0,
        outcome.receipt().unwrap().commit_reference().commit_id.0
    );
    assert_eq!(
        request
            .query(query::model::Read("fresh".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        2
    );
    assert_eq!(
        request
            .query(query::first_output::Read("first:fresh".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        20
    );
    assert_eq!(
        request
            .query(query::second_output::Read("second:fresh".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        1002
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 3
    );
    assert_eq!(
        request
            .query(query::child_output::Read("from-first:first:fresh".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        2000,
        "the later producer consumes the earlier producer's actual D output"
    );
    from_first::settle(&host, &request, "first:fresh");
    first::settle(&host, &request, "fresh");
    second::settle(&host, &request, "fresh");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 3
    );
    assert_eq!(courts::native_head(&host), head);
    let replay = request
        .advanced()
        .mutate(Intent {
            key: "fresh".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"new-source-edit".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed) = replay else {
        panic!("the complete creating edit must replay as one unit: {replay:?}")
    };
    assert_eq!(
        replayed.commit_reference(),
        outcome.receipt().unwrap().commit_reference()
    );
    assert_eq!(courts::native_head(&host), head);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 3
    );
}
#[test]
fn new_sources_and_their_initial_producers_commit_together() {
    run(false);
}
#[test]
fn worker_only_new_sources_and_their_initial_producers_commit_together() {
    run(true);
}

fn refused_new_source(value: u64) -> u64 {
    if value == 2 {
        u64::MAX
    } else {
        second::derived(value)
    }
}
fn refusal(worker: bool) {
    use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_configured::<program::Program>(
        limits,
        1,
        None,
        world::Configuration {
            second_derivation: Some(refused_new_source),
        },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let checkpoint = super::failure::checkpoint(&host);
    let before = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "fresh".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"refused-new-source".to_owned())
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
        .expect("actual synchronous producer refusal");
    assert!(
        producer.subject().contains("owned-outputs.second.initial"),
        "{producer:?}"
    );
    assert_eq!(
        producer.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied
    );
    assert_eq!(producer.domain_reason(), Some("DerivedValueOutOfRange"));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 3
    );
    assert_eq!(courts::native_head(&host), before);
    super::failure::assert_model_unchanged(&host, &checkpoint);
}
#[test]
fn a_later_refusal_discards_the_new_source_and_its_earlier_output() {
    refusal(false);
}
#[test]
fn worker_only_a_later_refusal_discards_the_new_source_and_its_earlier_output() {
    refusal(true);
}

mod cold;

pub(crate) mod restricted;

mod identity;

mod control;
