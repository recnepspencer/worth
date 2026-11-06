//! A reused lookup name cannot rebind stored ownership endpoints to a new occurrence.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Input {
    pub restore: bool,
}
worth_query_structured_value_binding!(pub InputBinding for Input {identity:"owned-model.recreate-input.v1"});
worth_query_operation!(pub RecreateMember for Schema,input InputBinding;access {
    read_entities:[Model,Member];read_fields:[ModelKey::at("ModelKey"),MemberKey::at("MemberKey"),MemberValue::at("MemberValue")];
    writes:[MemberKey::at("MemberKey"),MemberValue::at("MemberValue")];creates:[Member];deletes:[Member];
});
pub struct Decision {
    model: WorthQueryInvariantMutationTarget<Schema, Model>,
    member: Option<WorthQueryInvariantMutationTarget<Schema, Member>>,
}
fn scope(_: &Input) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Input,schema Schema,
    identity "owned-model.recreate.v1",input InputBinding,operation RecreateMember,result edit::ResultBinding,
    idempotency String,identity "owned-model.recreate-key.v1",decision Decision,denial edit::DenialBinding,
    handler identity "owned-model.recreate-handler.v1",program required,outputs NoApplicationMutationOutputs,
    principal Actor,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope ModelKey,field ModelKey::at("ModelKey"),value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, edit::Denial> {
        let result = (|| {
            let model = reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
            let current =
                reader.resolve_optional_entity(MemberKey::at("MemberKey"), "direct".to_owned())?;
            if let Some(current) = &current {
                reader
                    .reader()
                    .require_decision_entity(current, Member::reference())
                    .map_err(HandlerExecutionDenial::new)?;
            }
            if input.restore == current.is_some() {
                return Ok(None);
            }
            Ok::<_, HandlerExecutionDenial>(Some(Decision {
                model: reader.mutation_target(&model)?,
                member: current
                    .map(|current| reader.mutation_target(&current))
                    .transpose()?,
            }))
        })();
        match result {
            Ok(Some(targets)) => HandlerResult::Completed(targets),
            Ok(None) => HandlerResult::DomainDenied(edit::Denial::InvalidModelGraph),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Input,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            if let Some(member) = decision.member {
                let member = writer.projected_entity(&member)?;
                writer.delete_entity(Member::reference(), &member)?;
            } else {
                let context = writer.projected_entity(&decision.model)?;
                let created = writer.create_entity_in_context(
                    &context,
                    Member::reference(),
                    WorthQueryApplicationEntityKey::new("recreated-member").unwrap(),
                )?;
                writer.initialize_field(
                    &created,
                    MemberKey::at("MemberKey"),
                    "direct".to_owned(),
                )?;
                writer.initialize_field(&created, MemberValue::at("MemberValue"), 11)?;
            }
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

#[test]
fn recreated_same_name_member_does_not_inherit_the_old_full_generation_ownership_edge() {
    let host = world::install();
    let original = world::member_identity(&host, host.current_world(), "direct");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for restore in [false, true] {
        let outcome = request
            .advanced()
            .mutate(Input { restore })
            .without_source()
            .idempotency(&format!("recreate:{restore}"))
            .execute_in_program(
                &host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{outcome:?}"
        );
    }
    let recreated = world::member_identity(&host, host.current_world(), "direct");
    assert_ne!(
        original, recreated,
        "Native must issue a different full occurrence"
    );
    let document = world::attach(&host);
    let edit = document
        .edit(edit::member::Intent(edit::Edit {
            key: "direct".into(),
            value: 88,
        }))
        .idempotency_owned(&request, "new-occurrence".into())
        .unwrap()
        .retain_in_program::<Program, gesture::MemberAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    let WorthQueryHistoryEditAdvance::Refused { edit, denial } = edit.advance(&request) else {
        panic!("a stored old endpoint cannot own the new member")
    };
    let WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
        WorthQueryDocumentHistoryDenial::UnownedOperationScope { subject, .. },
    )) = denial
    else {
        panic!("{denial:?}")
    };
    assert_eq!(
        (
            subject.partition_id.as_u32(),
            subject.local_slot_value(),
            subject.generation_value()
        ),
        recreated
    );
    let document = edit
        .discard(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert_eq!(
        world::member_value(
            &host,
            document
                .selection(&request)
                .unwrap()
                .physical_endpoint()
                .product_branch(),
            "direct"
        ),
        11
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
