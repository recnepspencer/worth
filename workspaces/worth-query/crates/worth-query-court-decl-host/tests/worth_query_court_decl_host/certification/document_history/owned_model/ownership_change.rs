//! Ordinary relation authoring must obey document exclusivity outside history too.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Change {
    pub from: String,
    pub to: String,
    pub member: String,
    pub preserve_previous: bool,
}
worth_query_structured_value_binding!(pub Input for Change { identity: "owned-model.ownership-change.v1" });
worth_query_operation!(pub ChangeOwnership for Schema, input Input; access {
    read_fields: [ModelKey::at("ModelKey"), MemberKey::at("MemberKey")]; read_relations: [ModelMember]; links: [ModelMember]; unlinks: [ModelMember];
});
pub struct Decision {
    from: WorthQueryInvariantMutationTarget<Schema, Model>,
    to: WorthQueryInvariantMutationTarget<Schema, Model>,
    member: WorthQueryInvariantMutationTarget<Schema, Member>,
}
fn scope(input: &Change) -> String {
    input.from.clone()
}
worth_query_mutation_binding!(pub Binding for Change, schema Schema,
    identity "owned-model.ownership-change.v1", input Input, operation ChangeOwnership, result edit::ResultBinding,
    idempotency String, identity "owned-model.ownership-change-key.v1", decision Decision, denial edit::DenialBinding,
    handler identity "owned-model.ownership-change-handler.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Change,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, edit::Denial> {
        let result = (|| {
            let from = reader.resolve_entity(ModelKey::at("ModelKey"), input.from.clone())?;
            let to = reader.resolve_entity(ModelKey::at("ModelKey"), input.to.clone())?;
            let member = reader.resolve_entity(MemberKey::at("MemberKey"), input.member.clone())?;
            reader
                .relations_from(ModelMember::reference(), &from)
                .map_err(HandlerExecutionDenial::new)?;
            Ok::<_, HandlerExecutionDenial>(Decision {
                from: reader.mutation_target(&from)?,
                to: reader.mutation_target(&to)?,
                member: reader.mutation_target(&member)?,
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Change,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = (|| {
            let from = writer.projected_entity(&decision.from)?;
            let to = writer.projected_entity(&decision.to)?;
            let member = writer.projected_entity(&decision.member)?;
            if !input.preserve_previous {
                writer.unlink(ModelMember::reference(), &from, &member)?;
            }
            writer.link(
                ModelMember::reference(),
                format!("move:{}:{}", input.to, input.member),
                &to,
                &member,
            )?;
            Ok::<_, WorthQueryApplicationAttemptDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

#[test]
fn ordinary_relation_addition_refuses_two_owners_before_publication() {
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(Change {
            from: "a".into(),
            to: "b".into(),
            member: "direct".into(),
            preserve_previous: true,
        })
        .without_source()
        .idempotency(&"double-owner".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        cause,
    )) = outcome
    else {
        panic!("{outcome:?}")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    );
    let conflict = cause
        .model_ownership_conflict()
        .expect("typed ownership conflict");
    assert_eq!(
        (
            conflict.subject.partition_id.as_u32(),
            conflict.subject.local_slot_value(),
            conflict.subject.generation_value()
        ),
        world::member_identity(&host, host.current_world(), "direct")
    );
    assert_eq!(
        world::member_value(&host, host.current_world(), "direct"),
        11
    );
}

#[test]
fn ownership_relation_move_records_one_step_and_changes_subsequent_membership() {
    type Action =
        ApplicationActionReference<Schema, ApplicationRootComposition, Entry, Binding, Completion>;
    let host = world::install();
    let document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = document
        .edit(Change {
            from: "a".into(),
            to: "b".into(),
            member: "direct".into(),
            preserve_previous: false,
        })
        .idempotency_owned(&request, "move".to_owned())
        .unwrap()
        .retain_in_program::<Program, Action>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish {
        document,
        outcome: _,
    } = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    type MemberAction = ApplicationActionReference<
        Schema,
        ApplicationRootComposition,
        Entry,
        edit::member::Binding,
        Completion,
    >;
    let edit = document
        .edit(edit::member::Intent(edit::Edit {
            key: "nested".into(),
            value: 999,
        }))
        .idempotency_owned(&request, "after-move".into())
        .unwrap()
        .retain_in_program::<Program, MemberAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    let WorthQueryHistoryEditAdvance::Refused { edit, denial } = edit.advance(&request) else {
        panic!("moved descendant must refuse")
    };
    assert!(
        matches!(
            denial,
            WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::UnownedOperationScope { .. }
            ))
        ),
        "{denial:?}"
    );
    let document = edit
        .discard(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
