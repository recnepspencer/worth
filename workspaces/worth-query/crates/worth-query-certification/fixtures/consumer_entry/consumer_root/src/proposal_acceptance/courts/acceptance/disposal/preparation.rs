//! An ordinary stale selection can be abandoned without inventing candidate custody.
use super::*;
use crate::proposal_acceptance::{
    application::candidate::{PreparationFailure, PreparationRefusal},
    courts::retention_action::{complete, construct},
};

#[test]
fn stale_preparation_disposal_releases_the_model_for_a_fresh_recorded_basis() {
    for restored in [false, true] {
        refuse_then_dispose(restored);
    }
}

fn refuse_then_dispose(restored: bool) {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 1401, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 1402, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            1403,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let loaded = storage.load_frozen(&reviewer, &reference).unwrap();
    let source = model.branch();
    let original_basis = model.describe(&reviewer).unwrap();
    progress(model.accept(&storage, &reviewer, &loaded, 1404));
    progress(model.accept(&storage, &reviewer, &loaded, 1404));
    assert!(matches!(
        model
            .inspect_attempt(&storage, &reviewer, &loaded, 1404)
            .unwrap(),
        Some(View::Registered)
    ));
    complete(
        construct(&host, &model, &author, edit(&model, 8), 1405),
        &host
            .request(&author.principal, author.scope)
            .on_branch(source),
        8,
    );
    assert_eq!(model.read(&reviewer).unwrap().days, 8);
    if restored {
        complete(
            construct(&host, &model, &author, edit(&model, 7), 1408),
            &host
                .request(&author.principal, author.scope)
                .on_branch(source),
            7,
        );
    }
    let current_days = if restored { 7 } else { 8 };
    assert_eq!(model.read(&reviewer).unwrap().days, current_days);
    let moved = model.describe(&reviewer).unwrap();
    assert_ne!(moved, original_basis);
    assert!(matches!(
        model.accept(&storage, &reviewer, &loaded, 1404).unwrap(),
        Event::Stopped
    ));
    assert!(matches!(
        model
            .inspect_attempt(&storage, &reviewer, &loaded, 1404)
            .unwrap(),
        Some(View::Preparation(PreparationFailure::Refused {
            cause: PreparationRefusal::Selection(
                WorthQueryExpectedProductBasisSelectionDenial::Admission(
                    WorthQueryProductBranchAdmissionDenial::Refused(
                        WorthQueryProductBranchAdmissionRefusalKind::ObservationStaleSourceHead
                    )
                )
            ),
            handoff: None,
        }))
    ));
    assert_eq!(model.branch(), source);
    assert_eq!(model.describe(&reviewer).unwrap(), moved);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    let refused = match model.close(&author) {
        Err(refused) => refused,
        Ok(_) => panic!("pending preparation returns the whole model"),
    };
    assert!(matches!(refused.cause, ModelCloseRefusal::Pending));
    let mut model = refused.model;
    let cancellation = WorthQueryCancellationSource::new();
    let cancelled_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let cancelled =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &cancelled_scope)
            .unwrap();
    cancellation.cancel();
    assert!(matches!(
        model.dispose_pending(&cancelled),
        Err(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ))
    ));
    assert!(matches!(
        model
            .inspect_attempt(&storage, &reviewer, &loaded, 1404)
            .unwrap(),
        Some(View::Preparation(_))
    ));
    let fresh_scope = authentication::request_scope();
    let fresh =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &fresh_scope)
            .unwrap();
    let disposed = model.dispose_pending(&fresh).unwrap();
    let DisposalEvent::Disposed(disposed) = disposed else {
        panic!("an owner-free stale preparation can be abandoned: {disposed:?}")
    };
    assert!(disposed.handoff().is_none());
    drop(disposed);
    assert_eq!(model.branch(), source);
    assert_eq!(model.describe(&fresh).unwrap(), moved);
    assert_eq!(model.read(&fresh).unwrap().days, current_days);
    assert_eq!(other.read(&fresh).unwrap().days, 5);
    assert!(matches!(
        model.accept(&storage, &fresh, &loaded, 1404),
        Err(Failure::ContinuationUnavailable)
    ));
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-2",
            1406,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let loaded = storage.load_frozen(&fresh, &reference).unwrap();
    reach_adoption(&mut model, &storage, &fresh, &loaded, 1407);
    assert_ne!(model.branch(), source);
    assert_eq!(model.read(&fresh).unwrap().days, 3);
    assert_eq!(other.read(&fresh).unwrap().days, 5);
    drop(finish(&mut model, &storage, &fresh, &loaded, 1407));
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
