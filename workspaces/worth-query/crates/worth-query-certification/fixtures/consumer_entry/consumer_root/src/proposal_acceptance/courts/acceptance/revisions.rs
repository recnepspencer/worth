//! Native proposal revision distinguishes registered frozen intent from an unregistered stale load.
use super::*;
use crate::proposal_acceptance::{
    attempts::{claim, data::StoredStepOutcome, query},
    failure::Failure as ApplicationFailure,
    revision,
    schema::data::ProposalReference,
    storage_query,
};

fn current_reference(host: &Host, storage: &Storage<'_>, caller: &Caller<'_>) -> ProposalReference {
    let rows = host
        .request(&caller.principal, caller.scope)
        .on_branch(storage.branch())
        .query(storage_query::ReadProposals {
            model: "model-1".to_owned(),
        })
        .execute_governed(
            access::ReadDomain::reference(),
            access::InspectDomain::reference(),
            access::ReadAccess {
                model: "model-1".to_owned(),
            },
        )
        .unwrap();
    let mut selected = rows.rows().iter().filter(|row| row.key == "proposal-1");
    let row = selected
        .next()
        .expect("the genuinely revised proposal is committed");
    assert!(selected.next().is_none());
    ProposalReference {
        key: row.key.clone(),
        occurrence: row.occurrence.clone(),
        model: row.data.model.clone(),
        revision: row.data.revision,
    }
}

#[test]
fn ordinary_revision_preserves_registered_frozen_input_and_refuses_unregistered_stale_input() {
    for registered in [false, true] {
        revise_then_accept(registered);
    }
}

fn revise_then_accept(registered: bool) {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 1701, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 1702, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let original = storage
        .record(
            &model,
            &author,
            "proposal-1",
            1703,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let frozen = storage.load_frozen(&reviewer, &original).unwrap();
    let before = model.describe(&reviewer).unwrap();
    let source = model.branch();
    if registered {
        progress(model.accept(&storage, &reviewer, &frozen, 1704));
        progress(model.accept(&storage, &reviewer, &frozen, 1704));
        assert!(matches!(
            model
                .inspect_attempt(&storage, &reviewer, &frozen, 1704)
                .unwrap(),
            Some(View::Registered)
        ));
    }
    let storage_before = describe(&host, &reviewer, storage.branch());
    let outcome = host
        .request(&author.principal, author.scope)
        .on_branch(storage.branch())
        .advanced()
        .mutate(revision::ReviseProposal {
            reference: original.clone(),
            intents: vec![AuthoredIntent::Retention(edit(&model, 4))],
        })
        .without_source()
        .idempotency(&1705)
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(&outcome, WorthQueryApplicationMutationOutcome::Committed { result, .. } if result.revision == 2)
    );
    drop(outcome);
    let revised = current_reference(&host, &storage, &reviewer);
    assert_eq!(revised.occurrence, original.occurrence);
    assert_eq!(revised.model, original.model);
    assert_eq!(revised.revision, 2);
    let latest = storage.load_frozen(&reviewer, &revised).unwrap();
    assert_eq!(latest.proposal().format_version, 1);
    assert_eq!(latest.proposal().revision, 2);
    assert_eq!(latest.proposal().model, frozen.proposal().model);
    assert_eq!(
        latest.proposal().expected_basis,
        frozen.proposal().expected_basis
    );
    let AuthoredIntent::Retention(input) = &latest.proposal().intents[0];
    assert_eq!(input.days, 4);
    let AuthoredIntent::Retention(input) = &frozen.proposal().intents[0];
    assert_eq!(input.days, 3);
    assert_eq!(frozen.proposal().revision, 1);
    assert!(matches!(
        storage.load(&reviewer, &original),
        Err(ApplicationFailure::ProposalRevisionChanged)
    ));
    assert_ne!(describe(&host, &reviewer, storage.branch()), storage_before);
    assert_eq!(model.branch(), source);
    assert_eq!(model.describe(&reviewer).unwrap(), before);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    let revised_basis = describe(&host, &reviewer, storage.branch());
    let stale_revision = host
        .request(&author.principal, author.scope)
        .on_branch(storage.branch())
        .advanced()
        .mutate(revision::ReviseProposal {
            reference: original.clone(),
            intents: vec![AuthoredIntent::Retention(edit(&model, 5))],
        })
        .without_source()
        .idempotency(&1709)
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        stale_revision,
        WorthQueryApplicationMutationOutcome::DomainDenied(revision::Denial::ProposalChanged)
    ));
    drop(stale_revision);
    assert_eq!(describe(&host, &reviewer, storage.branch()), revised_basis);
    assert_eq!(
        storage.load(&reviewer, &revised).unwrap(),
        *latest.proposal()
    );
    if registered {
        assert!(matches!(
            model.accept(&storage, &reviewer, &latest, 1704),
            Err(Failure::IntentDrift)
        ));
        progress(model.accept(&storage, &reviewer, &frozen, 1704));
        assert!(matches!(
            model.accept(&storage, &reviewer, &frozen, 1704).unwrap(),
            Event::Staged(super::super::super::application::candidate::StageEvent::StepCompleted)
        ));
        progress(model.accept(&storage, &reviewer, &frozen, 1704));
        assert!(matches!(
            model.accept(&storage, &reviewer, &frozen, 1704).unwrap(),
            Event::Adopted
        ));
        assert_eq!(model.read(&reviewer).unwrap().days, 3);
        let accepted = finish(&mut model, &storage, &reviewer, &frozen, 1704);
        assert_eq!(accepted.result.claim.data.input, frozen.input(1704));
        assert_eq!(
            accepted.result.steps,
            vec![StoredStepOutcome::Fresh { days: 3 }]
        );
        let head = model.branch();
        let current = model.describe(&reviewer).unwrap();
        let Event::Cached(result) = model.accept(&storage, &reviewer, &frozen, 1704).unwrap()
        else {
            panic!("the registered original version retains its actual cached result")
        };
        assert_eq!(result, accepted.result);
        assert_eq!(model.branch(), head);
        assert_eq!(model.describe(&reviewer).unwrap(), current);
        assert!(matches!(
            model.accept(&storage, &reviewer, &latest, 1704),
            Err(Failure::IntentDrift)
        ));
        drop(accepted);
    } else {
        progress(model.accept(&storage, &reviewer, &frozen, 1707));
        assert!(matches!(
            model.accept(&storage, &reviewer, &frozen, 1707).unwrap(),
            Event::Stopped
        ));
        let Some(View::ClaimTerminal(terminal)) = model
            .inspect_attempt(&storage, &reviewer, &frozen, 1707)
            .unwrap()
        else {
            panic!("the stale unregistered input owns its real terminal operation")
        };
        assert!(matches!(
            terminal.native_outcome(),
            Some(WorthQueryApplicationMutationOutcome::DomainDenied(
                claim::Denial::ProposalChanged
            ))
        ));
        assert_eq!(model.branch(), source);
        assert_eq!(model.describe(&reviewer).unwrap(), before);
        assert_eq!(model.read(&reviewer).unwrap().days, 7);
        assert_eq!(describe(&host, &reviewer, storage.branch()), revised_basis);
        {
            let claims = host
                .request(&reviewer.principal, reviewer.scope)
                .on_branch(storage.branch())
                .query(query::ReadClaims {
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
            assert!(claims.rows().is_empty());
        }
        let super::super::super::application::acceptance::DisposalEvent::Disposed(disposed) =
            model.dispose_pending(&reviewer).unwrap()
        else {
            panic!("close the actual stale Claim terminal before another attempt")
        };
        drop(disposed);
        reach_adoption(&mut model, &storage, &reviewer, &latest, 1708);
        assert_eq!(model.read(&reviewer).unwrap().days, 4);
        let accepted = finish(&mut model, &storage, &reviewer, &latest, 1708);
        assert_eq!(accepted.result.claim.data.input, latest.input(1708));
        assert_eq!(
            accepted.result.steps,
            vec![StoredStepOutcome::Fresh { days: 4 }]
        );
        drop(accepted);
    }
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
