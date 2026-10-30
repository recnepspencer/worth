//! A completed acceptance still admits each independent READ before its cached result.
use super::*;
use crate::proposal_acceptance::{
    application::{candidate::PreparationRefusal, storage::AdmissionFailure},
    attempts::result_query,
    failure::Failure as ApplicationFailure,
    mutation_access,
};

fn read_denied(failure: Failure, kind: ReadKind) {
    let cause = match (kind, failure) {
        (
            ReadKind::Model,
            Failure::Admission(AdmissionFailure::Preparation(PreparationRefusal::Request(
                ApplicationFailure::Query(WorthQueryApplicationRequestQueryDenial::Admission(
                    cause,
                )),
            ))),
        ) => cause,
        (
            ReadKind::Proposal | ReadKind::Result,
            Failure::Admission(AdmissionFailure::Request(ApplicationFailure::Query(
                WorthQueryApplicationRequestQueryDenial::Admission(cause),
            ))),
        ) => cause,
        (kind, cause) => panic!("expected the independent {kind:?} READ refusal: {cause:?}"),
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                WorthQueryOperationAuthorizationRefusalKind::PermissionDenied
            )
        )
    );
}

#[test]
fn cached_acceptance_requires_each_current_read_while_sibling_reads_remain_lawful() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 1501, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 1502, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            1503,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let loaded = storage.load_frozen(&reviewer, &reference).unwrap();
    reach_adoption(&mut model, &storage, &reviewer, &loaded, 1504);
    let accepted = finish(&mut model, &storage, &reviewer, &loaded, 1504);
    let result = accepted.result.clone();
    drop(accepted);
    let head = model.branch();
    assert!(matches!(
        model.accept(&storage, &reviewer, &loaded, 1504).unwrap(),
        Event::Cached(observed) if observed == result
    ));
    for kind in [ReadKind::Proposal, ReadKind::Model, ReadKind::Result] {
        let branch = match kind {
            ReadKind::Model => model.branch(),
            ReadKind::Proposal | ReadKind::Result => storage.branch(),
        };
        let change = |readable, key| {
            let outcome = host
                .request(&author.principal, author.scope)
                .on_branch(branch)
                .advanced()
                .mutate(status::SetStatus {
                    identity: access::grant_key("model-1", 2, kind),
                    active: true,
                    readable,
                })
                .without_source()
                .idempotency(&key)
                .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ));
        };
        change(false, 1510 + kind.code() * 2);
        let denied_scope = authentication::request_scope();
        let denied =
            authentication::authenticate(host.installed_schema(), Actor::Reviewer, &denied_scope)
                .unwrap();
        if kind != ReadKind::Model {
            assert_eq!(model.read(&denied).unwrap().days, 3);
        }
        if kind != ReadKind::Proposal {
            assert_eq!(
                storage.load(&denied, &reference).unwrap(),
                *loaded.proposal()
            );
        }
        if kind != ReadKind::Result {
            let rows = host
                .request(&denied.principal, denied.scope)
                .on_branch(storage.branch())
                .query(result_query::ReadResults {
                    model: model.identity.label.clone(),
                })
                .execute_governed(
                    access::ReadDomain::reference(),
                    access::InspectDomain::reference(),
                    access::ReadAccess {
                        model: model.identity.label.clone(),
                    },
                )
                .unwrap();
            assert_eq!(rows.rows().len(), 1);
            assert_eq!(rows.rows()[0].data, result);
        }
        let AuthoredIntent::Retention(input) = &loaded.proposal().intents[0];
        host.request(&denied.principal, denied.scope)
            .on_branch(model.branch())
            .advanced()
            .mutate(input.clone())
            .without_source()
            .assess_current_capability_authorization()
            .unwrap();
        let before = model.describe(&author).unwrap();
        let storage_before = describe(&host, &author, storage.branch());
        read_denied(
            model.accept(&storage, &denied, &loaded, 1504).unwrap_err(),
            kind,
        );
        assert_eq!(model.branch(), head);
        assert_eq!(model.describe(&author).unwrap(), before);
        assert_eq!(describe(&host, &author, storage.branch()), storage_before);
        assert_eq!(model.read(&author).unwrap().days, 3);
        assert_eq!(other.read(&author).unwrap().days, 5);
        change(true, 1511 + kind.code() * 2);
        let fresh_scope = authentication::request_scope();
        let fresh =
            authentication::authenticate(host.installed_schema(), Actor::Reviewer, &fresh_scope)
                .unwrap();
        let before = model.describe(&author).unwrap();
        let storage_before = describe(&host, &author, storage.branch());
        let Event::Cached(observed) = model.accept(&storage, &fresh, &loaded, 1504).unwrap() else {
            panic!("restoring the actual grant admits the same cached outcome")
        };
        assert_eq!(observed, result);
        assert_eq!(model.branch(), head);
        assert_eq!(model.describe(&author).unwrap(), before);
        assert_eq!(describe(&host, &author, storage.branch()), storage_before);
        assert_eq!(model.read(&fresh).unwrap().days, 3);
        assert_eq!(other.read(&fresh).unwrap().days, 5);
    }
    let mutation_branch = model.branch();
    let change_mutation = |active, key| {
        let outcome = host
            .request(&author.principal, author.scope)
            .on_branch(mutation_branch)
            .advanced()
            .mutate(mutation_access::status::SetStatus {
                identity: mutation_access::grant_key("model-1", 2),
                active,
            })
            .without_source()
            .idempotency(&key)
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        drop(outcome);
    };
    change_mutation(false, 1520);
    let denied_scope = authentication::request_scope();
    let denied =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &denied_scope)
            .unwrap();
    assert_eq!(model.read(&denied).unwrap().days, 3);
    assert_eq!(
        storage.load(&denied, &reference).unwrap(),
        *loaded.proposal()
    );
    {
        let rows = host
            .request(&denied.principal, denied.scope)
            .on_branch(storage.branch())
            .query(result_query::ReadResults {
                model: model.identity.label.clone(),
            })
            .execute_governed(
                access::ReadDomain::reference(),
                access::InspectDomain::reference(),
                access::ReadAccess {
                    model: model.identity.label.clone(),
                },
            )
            .unwrap();
        assert_eq!(rows.rows().len(), 1);
        assert_eq!(rows.rows()[0].data, result);
    }
    let before = model.describe(&author).unwrap();
    let storage_before = describe(&host, &author, storage.branch());
    let failure = model.accept(&storage, &denied, &loaded, 1504).unwrap_err();
    let Failure::Admission(AdmissionFailure::Preparation(PreparationRefusal::Mutation(
        WorthQueryApplicationRequestMutationDenial::Authorization(cause),
    ))) = failure
    else {
        panic!("cached ingress requires current independent mutation authority: {failure:?}")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing
        )
    );
    assert_eq!(model.branch(), head);
    assert_eq!(model.describe(&author).unwrap(), before);
    assert_eq!(describe(&host, &author, storage.branch()), storage_before);
    assert_eq!(model.read(&denied).unwrap().days, 3);
    change_mutation(true, 1521);
    let fresh_scope = authentication::request_scope();
    let fresh =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &fresh_scope)
            .unwrap();
    let before = model.describe(&author).unwrap();
    let storage_before = describe(&host, &author, storage.branch());
    let Event::Cached(observed) = model.accept(&storage, &fresh, &loaded, 1504).unwrap() else {
        panic!("restored mutation authority admits the same cached outcome")
    };
    assert_eq!(observed, result);
    assert_eq!(model.branch(), head);
    assert_eq!(model.describe(&author).unwrap(), before);
    assert_eq!(describe(&host, &author, storage.branch()), storage_before);
    assert_eq!(other.read(&fresh).unwrap().days, 5);
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
