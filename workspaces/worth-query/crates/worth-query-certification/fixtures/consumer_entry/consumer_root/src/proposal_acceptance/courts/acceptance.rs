//! Public ingress owns one actual continuation through adoption and result delivery.
use super::super::{
    access::{self, status, ReadKind},
    application::{
        acceptance::{Event, Failure, View},
        model::{Model, ModelCloseRefusal},
        storage::{AdmissionFailure, LoadedProposal, Storage},
    },
    authentication::{self, Actor, Caller},
    host::{self, Host},
    schema::data::AuthoredIntent,
};
use super::retention_action::edit;
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, product::*, runtime::ExecutionAllocationPolicy,
};
mod arbitration;
mod disclosure;
mod disposal;
mod previous;
mod revisions;
mod versions;

fn progress(event: Result<Event, Failure>) {
    assert!(matches!(event.unwrap(), Event::Progress));
}
fn reach_adoption<'host>(
    model: &mut Model<'host>,
    storage: &Storage<'host>,
    caller: &Caller<'_>,
    loaded: &LoadedProposal,
    key: u64,
) {
    progress(model.accept(storage, caller, loaded, key));
    progress(model.accept(storage, caller, loaded, key));
    progress(model.accept(storage, caller, loaded, key));
    assert!(matches!(
        model.accept(storage, caller, loaded, key).unwrap(),
        Event::Staged(super::super::application::candidate::StageEvent::StepCompleted)
    ));
    progress(model.accept(storage, caller, loaded, key));
    assert!(matches!(
        model.accept(storage, caller, loaded, key).unwrap(),
        Event::Adopted
    ));
}
fn finish<'host>(
    model: &mut Model<'host>,
    storage: &Storage<'host>,
    caller: &Caller<'_>,
    loaded: &LoadedProposal,
    key: u64,
) -> super::super::application::acceptance::Accepted {
    progress(model.accept(storage, caller, loaded, key));
    progress(model.accept(storage, caller, loaded, key));
    let Event::Completed(accepted) = model.accept(storage, caller, loaded, key).unwrap() else {
        panic!("actual previous-head close completes acceptance")
    };
    accepted
}
fn describe(
    host: &Host,
    caller: &Caller<'_>,
    branch: WorthQueryProductBranch,
) -> WorthQueryExpectedProductBasisDescription {
    host.with_application_advancement(caller.scope, |phase| {
        host.on_branch(branch)
            .select(&phase)
            .unwrap()
            .describe_basis()
    })
    .unwrap()
}
#[test]
fn ingress_adopts_once_and_recovers_a_result_read_gap_with_the_same_owner() {
    let host = host::install();
    let (mut model, other, storage, reference) = {
        let scope = authentication::request_scope();
        let author =
            authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
        let model = Model::create(&host, &author, "model-1", 901, 7).unwrap();
        let other = Model::create(&host, &author, "model-2", 902, 5).unwrap();
        let storage = Storage::create(&model, &author).unwrap();
        let before = model.describe(&author).unwrap();
        let reference = storage
            .record(
                &model,
                &author,
                "proposal-1",
                903,
                vec![AuthoredIntent::Retention(edit(&model, 3))],
            )
            .unwrap();
        assert_eq!(model.describe(&author).unwrap(), before);
        assert_eq!(model.read(&author).unwrap().days, 7);
        (model, other, storage, reference)
    };
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let loaded = storage.load_frozen(&reviewer, &reference).unwrap();
    let source = model.branch();
    reach_adoption(&mut model, &storage, &reviewer, &loaded, 904);
    let head = model.branch();
    assert_ne!(head, source);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    let Some(View::Adopted {
        adopted,
        result: None,
    }) = model
        .inspect_attempt(&storage, &reviewer, &loaded, 904)
        .unwrap()
    else {
        panic!("actual adoption remains model-owned")
    };
    assert_eq!(adopted.previous_branch(), source);
    assert_eq!(adopted.metadata().attempt, 904);
    let change = |readable, key| {
        host.request(&author.principal, author.scope)
            .on_branch(storage.branch())
            .advanced()
            .mutate(status::SetStatus {
                identity: access::grant_key("model-1", 2, ReadKind::Result),
                active: true,
                readable,
            })
            .without_source()
            .idempotency(&key)
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    };
    assert!(matches!(
        change(false, 905),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let failure = model.accept(&storage, &reviewer, &loaded, 904).unwrap_err();
    let Failure::Admission(AdmissionFailure::Request(super::super::failure::Failure::Query(
        WorthQueryApplicationRequestQueryDenial::Admission(cause),
    ))) = failure
    else {
        panic!("result READ refuses before protected continuation lookup")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                WorthQueryOperationAuthorizationRefusalKind::PermissionDenied
            )
        )
    );
    assert_eq!(model.branch(), head);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    let disposal = model.dispose_pending(&reviewer).unwrap();
    assert!(matches!(
        disposal,
        super::super::application::acceptance::DisposalEvent::Retained(
            super::super::application::acceptance::DisposalClass::CommittedContinuationRetained
        )
    ));
    assert!(!format!("{disposal:?}").contains(&reference.key));
    assert_eq!(model.branch(), head);
    let close = model.close(&reviewer).unwrap_err();
    assert!(matches!(close.cause, ModelCloseRefusal::Pending));
    model = close.model;
    assert!(matches!(
        change(true, 906),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let Some(View::Adopted {
        adopted,
        result: None,
    }) = model
        .inspect_attempt(&storage, &reviewer, &loaded, 904)
        .unwrap()
    else {
        panic!("restore READ resumes identical adoption")
    };
    assert_eq!(adopted.previous_branch(), source);
    let accepted = finish(&mut model, &storage, &reviewer, &loaded, 904);
    assert_eq!(
        accepted.result.steps,
        vec![super::super::attempts::data::StoredStepOutcome::Fresh { days: 3 }]
    );
    assert!(accepted
        .performed
        .previous_close()
        .unwrap()
        .handoff
        .is_none());
    let expected = storage
        .read_result(&model, &reviewer, &loaded, &accepted.result.claim)
        .unwrap();
    assert_eq!(expected, accepted.result);
    let current = model.describe(&reviewer).unwrap();
    for _ in 0..2 {
        let Event::Cached(result) = model.accept(&storage, &reviewer, &loaded, 904).unwrap() else {
            panic!("same attempt returns actual stored result")
        };
        assert_eq!(result, expected);
        assert_eq!(model.branch(), head);
        assert_eq!(model.describe(&reviewer).unwrap(), current);
    }
    assert!(model
        .inspect_attempt(&storage, &reviewer, &loaded, 904)
        .unwrap()
        .is_none());
    drop(accepted);
    storage.close(&author).unwrap();
    model.close(&reviewer).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
