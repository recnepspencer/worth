//! A committed adoption retains its evidence while its actual retired previous-head cleanup settles.
use super::super::super::application::{
    acceptance::{DisposalClass, DisposalEvent},
    candidate::adoption::FinishedAdoption,
};
use super::*;
#[test]
fn previous_head_cleanup_recovers_without_discarding_adoption_evidence_or_adopting_again() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 1001, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 1002, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            1003,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let loaded = storage.load_frozen(&reviewer, &reference).unwrap();
    let source = model.branch();
    reach_adoption(&mut model, &storage, &reviewer, &loaded, 1004);
    let head = model.branch();
    assert_ne!(head, source);
    let late = super::super::super::application::retention_action::construct(
        &host,
        source,
        &reviewer,
        &edit(&model, 4),
        super::super::super::retention::StepKey {
            attempt: 1005,
            ordinal: 0,
        },
    )
    .unwrap();
    let request = host
        .request(&reviewer.principal, reviewer.scope)
        .on_branch(source);
    let WorthQuerySourceOnlyAdvance::Completed(late) = late.advance(&request) else {
        panic!("real late publication on previous branch")
    };
    assert!(
        matches!(late.evidence(),WorthQuerySourceCompletion::Fresh{result,..} if result.days==4)
    );
    late.close(&request)
        .unwrap_or_else(|(_, cause)| panic!("close late publisher: {cause:?}"));
    let previous = request
        .query(super::super::super::model_query::ReadTarget {
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
    assert_eq!(previous.rows()[0].days, 4);
    drop(previous);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    // The live candidate reaches old7; this later old4 commit is outside its ancestry.
    let reader = request.retain_current_read().unwrap();
    progress(model.accept(&storage, &reviewer, &loaded, 1004));
    progress(model.accept(&storage, &reviewer, &loaded, 1004));
    let close = model.accept(&storage, &reviewer, &loaded, 1004).unwrap();
    assert!(
        matches!(close, Event::Stopped),
        "actual previous-head close: {close:?}"
    );
    assert_eq!(model.branch(), head);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    let Some(View::PreviousClose(failure)) = model
        .inspect_attempt(&storage, &reviewer, &loaded, 1004)
        .unwrap()
    else {
        panic!("actual previous-head failure is retained")
    };
    assert_eq!(failure.adopted.previous_branch(), source);
    let super::super::super::failure::Failure::Close(
        WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(native),
    ) = &failure.cause
    else {
        panic!("old head actually retired")
    };
    assert!(matches!(
        native.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    assert!(matches!(
        model.dispose_pending(&reviewer).unwrap(),
        DisposalEvent::Retained(DisposalClass::CommittedContinuationRetained)
    ));
    progress(model.accept(&storage, &reviewer, &loaded, 1004));
    assert!(matches!(
        model.accept(&storage, &reviewer, &loaded, 1004).unwrap(),
        Event::Stopped
    ));
    let Some(View::RetiredPrevious(owner)) = model
        .inspect_attempt(&storage, &reviewer, &loaded, 1004)
        .unwrap()
    else {
        panic!("original retired cleanup stays owned")
    };
    assert_eq!(owner.metadata().attempt, 1004);
    assert_eq!(owner.completed().len(), 1);
    assert!(matches!(
        owner.cleanup_denial(),
        Some(WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        ))
    ));
    assert!(owner.retirement_handoff().is_none());
    assert!(owner.cleanup_handoff().is_none());
    assert_eq!(model.branch(), head);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    drop(reader);
    let Event::Completed(accepted) = model.accept(&storage, &reviewer, &loaded, 1004).unwrap()
    else {
        panic!("actual cleanup settles while adoption evidence remains owned")
    };
    assert!(accepted.performed.previous_close().is_none());
    assert_eq!(
        accepted
            .performed
            .cleanup_receipt()
            .unwrap()
            .retired_component_count(),
        1
    );
    assert!(accepted.performed.adoption_handoff().is_none());
    let FinishedAdoption::RetiredCleanupSettled(settled) = &accepted.performed else {
        panic!("cleanup is not a manufactured branch-close receipt")
    };
    assert_eq!(settled.metadata().attempt, 1004);
    assert_eq!(settled.completed().len(), 1);
    assert!(
        matches!(&settled.completed()[0],WorthQuerySourceCompletion::Fresh{result,..} if result.days==3)
    );
    assert!(settled.retirement_handoff().is_none());
    assert!(settled.cleanup_handoff().is_none());
    assert_eq!(model.branch(), head);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    assert!(host.branches().pending_cleanup().is_empty());
    assert_eq!(
        storage
            .read_result(&model, &reviewer, &loaded, &accepted.result.claim)
            .unwrap(),
        accepted.result
    );
    let current = model.describe(&reviewer).unwrap();
    let Event::Cached(result) = model.accept(&storage, &reviewer, &loaded, 1004).unwrap() else {
        panic!("retry returns original stored result")
    };
    assert_eq!(result, accepted.result);
    assert_eq!(model.branch(), head);
    assert_eq!(model.describe(&reviewer).unwrap(), current);
    assert!(model
        .inspect_attempt(&storage, &reviewer, &loaded, 1004)
        .unwrap()
        .is_none());
    drop(accepted);
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
