//! Owner-local disposal proves real retirement and never replaces a protected acceptance.
use super::super::super::{
    application::{
        acceptance::{DisposalClass, DisposalEvent, DisposalView},
        candidate::{lifecycle::CloseRefusal, StageEvent},
    },
    failure::Failure as ApplicationFailure,
};
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
};
mod preparation;

#[test]
fn partial_attempt_disposal_retries_actual_retired_cleanup_until_its_reader_releases() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 941, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 942, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            943,
            vec![
                AuthoredIntent::Retention(edit(&model, 3)),
                AuthoredIntent::Retention(edit(&model, 11)),
            ],
        )
        .unwrap();
    let loaded = storage.load_frozen(&reviewer, &reference).unwrap();
    let source = model.branch();
    let original = model.describe(&reviewer).unwrap();
    for _ in 0..3 {
        progress(model.accept(&storage, &reviewer, &loaded, 944));
    }
    assert!(matches!(
        model.accept(&storage, &reviewer, &loaded, 944).unwrap(),
        Event::Staged(StageEvent::StepCompleted)
    ));
    assert!(matches!(
        model.accept(&storage, &reviewer, &loaded, 944).unwrap(),
        Event::Staged(StageEvent::Terminal)
    ));
    let Some(View::Staging(staging)) = model
        .inspect_attempt(&storage, &reviewer, &loaded, 944)
        .unwrap()
    else {
        panic!("actual stopped candidate")
    };
    assert_eq!(staging.read(&model, &reviewer).unwrap(), 3);
    let Some(WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::Denied(cause),
    )) = staging.terminal_outcome(&model, &reviewer).unwrap()
    else {
        panic!("actual installed invariant refusal")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    );
    let candidate = staging.branch();
    let reader = host
        .request(&reviewer.principal, reviewer.scope)
        .on_branch(candidate)
        .retain_current_read()
        .unwrap();
    assert!(matches!(
        model.dispose_pending(&reviewer).unwrap(),
        DisposalEvent::Retained(DisposalClass::Candidate)
    ));
    let Some(View::Disposal(DisposalView::Candidate(CloseRefusal::Candidate(
        WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(native),
    )))) = model
        .inspect_attempt(&storage, &reviewer, &loaded, 944)
        .unwrap()
    else {
        panic!("original retired close failure remains owned")
    };
    assert!(matches!(
        native.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    assert!(matches!(
        model.accept(&storage, &reviewer, &loaded, 945),
        Err(Failure::Busy)
    ));
    assert!(matches!(
        model.dispose_pending(&reviewer).unwrap(),
        DisposalEvent::Retained(DisposalClass::RetiredCleanup)
    ));
    let Some(View::Disposal(DisposalView::Cleanup(cause))) = model
        .inspect_attempt(&storage, &reviewer, &loaded, 944)
        .unwrap()
    else {
        panic!("actual retry failure retained")
    };
    assert!(matches!(
        cause,
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    drop(reader);
    let DisposalEvent::Disposed(disposed) = model.dispose_pending(&reviewer).unwrap() else {
        panic!("release permits actual cleanup")
    };
    assert!(disposed.handoff().is_none());
    drop(disposed);
    assert_eq!(model.branch(), source);
    assert_eq!(model.describe(&reviewer).unwrap(), original);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    assert!(model
        .inspect_attempt(&storage, &reviewer, &loaded, 944)
        .unwrap()
        .is_none());
    assert!(matches!(
        model.accept(&storage, &reviewer, &loaded, 944),
        Err(Failure::ContinuationUnavailable)
    ));
    assert!(host.branches().pending_cleanup().is_empty());
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}

#[test]
fn registered_disposal_uses_owned_custody_after_storage_has_really_retired() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 961, 7).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            962,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let loaded = storage.load_frozen(&reviewer, &reference).unwrap();
    let source = model.branch();
    let before = model.describe(&reviewer).unwrap();
    progress(model.accept(&storage, &reviewer, &loaded, 963));
    progress(model.accept(&storage, &reviewer, &loaded, 963));
    let branch = storage.branch();
    let reader = host
        .request(&reviewer.principal, reviewer.scope)
        .on_branch(branch)
        .retain_current_read()
        .unwrap();
    let closed = storage.close(&author).unwrap_err();
    let ApplicationFailure::Close(
        WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(native),
    ) = closed.cause
    else {
        panic!("real storage retirement waits on reader")
    };
    assert!(matches!(
        native.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    let cleanup = native.into_cleanup();
    assert!(matches!(
        host.request(&reviewer.principal, reviewer.scope)
            .on_branch(branch)
            .retain_current_read(),
        Err(
            WorthQueryApplicationReadObservationDenial::ProductSelection(
                WorthQueryProductBranchAdmissionDenial::Refused(
                    WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch
                )
            )
        )
    ));
    let DisposalEvent::Disposed(disposed) = model.dispose_pending(&reviewer).unwrap() else {
        panic!("closed storage is not needed to release closed metadata")
    };
    assert!(disposed.handoff().is_none());
    drop(disposed);
    let blocked = cleanup
        .retry()
        .expect_err("the external storage reader still retains history");
    assert!(matches!(
        blocked.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    let cleanup = blocked.into_cleanup();
    drop(reader);
    assert_eq!(cleanup.retry().unwrap().retired_component_count(), 1);
    assert_eq!(model.branch(), source);
    assert_eq!(model.describe(&reviewer).unwrap(), before);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    let replacement = Storage::create(&model, &author).unwrap();
    assert!(matches!(
        model.accept(&replacement, &reviewer, &loaded, 963),
        Err(Failure::Admission(AdmissionFailure::Request(
            ApplicationFailure::StorageMismatch
        )))
    ));
    replacement.close(&author).unwrap();
    model.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}

#[test]
fn cancelled_disposal_preserves_an_unexecuted_owner_for_a_fresh_scope() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 981, 7).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            982,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let loaded = storage.load_frozen(&author, &reference).unwrap();
    let before = describe(&host, &author, storage.branch());
    progress(model.accept(&storage, &author, &loaded, 983));
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
            .inspect_attempt(&storage, &author, &loaded, 983)
            .unwrap(),
        Some(View::Claim(_))
    ));
    let fresh_scope = authentication::request_scope();
    let fresh =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &fresh_scope)
            .unwrap();
    let DisposalEvent::Disposed(disposed) = model.dispose_pending(&fresh).unwrap() else {
        panic!("fresh scope closes actual unexecuted owner")
    };
    drop(disposed);
    assert_eq!(describe(&host, &author, storage.branch()), before);
    assert_eq!(model.read(&author).unwrap().days, 7);
    let resolution = host
        .request(&author.principal, author.scope)
        .on_branch(storage.branch())
        .advanced()
        .mutate(loaded.input(983))
        .without_source()
        .idempotency(&983)
        .resolve_idempotency_in_program(&host)
        .unwrap()
        .into_resolution();
    assert!(matches!(
        resolution,
        WorthQueryApplicationIdempotencyResolution::Unseen
    ));
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
