//! Loaded typed inputs stage on the exact fork and adopt the application's actual slot.
use super::{
    super::{
        application::{
            candidate::{lifecycle::CloseRefusal, StageEvent, Staging},
            model::Model,
            storage::Storage,
        },
        authentication::{self, Actor},
        host,
        schema::data::AuthoredIntent,
    },
    retention_action::edit,
};
use worth_query_host::facade::{
    application_entry::*,
    primary_graph::*,
    product::{
        WorthQueryApplicationProductBranchCleanupDenial, WorthQueryProductBranchOwnerCleanupDenial,
    },
};
mod mismatch;

#[test]
fn stored_edit_stages_under_a_fresh_caller_then_adopts_the_actual_model_head() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 701, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 702, 5).unwrap();
    let source = model.branch();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = {
        let recording_scope = authentication::request_scope();
        let recording_author =
            authentication::authenticate(host.installed_schema(), Actor::Author, &recording_scope)
                .unwrap();
        storage
            .record(
                &model,
                &recording_author,
                "proposal-1",
                703,
                vec![AuthoredIntent::Retention(edit(&model, 3))],
            )
            .unwrap()
    };
    let fresh_scope = authentication::request_scope();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &fresh_scope)
            .unwrap();
    let loaded = storage.load(&reviewer, &reference).unwrap();
    let AuthoredIntent::Retention(input) = &loaded.intents[0];
    assert_eq!(input.days, 3);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    let mut staging = Staging::prepare(&model, &storage, &reviewer, &reference, 704)
        .unwrap_or_else(|cause| {
            panic!("fresh ordinary input and exact basis must prepare: {cause:?}")
        });
    let candidate = staging.branch();
    assert_ne!(candidate, source);
    assert_eq!(staging.read(&model, &reviewer).unwrap(), 7);
    assert!(matches!(
        staging.advance(&model, &reviewer),
        StageEvent::StepCompleted
    ));
    assert_eq!(staging.read(&model, &reviewer).unwrap(), 3);
    assert_eq!(model.branch(), source);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    let ready = staging
        .finish()
        .unwrap_or_else(|_| panic!("the actual ordinary step completed"));
    let adopted = ready
        .adopt(&mut model, &reviewer)
        .unwrap_or_else(|failure| panic!("guarded actual-slot replacement: {:?}", failure.cause));
    assert_eq!(model.branch(), candidate);
    assert_eq!(adopted.previous_branch(), source);
    assert!(adopted.handoff().is_none());
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    let metadata = adopted.metadata();
    assert_eq!(metadata.attempt, 704);
    assert_eq!(metadata.proposal.occurrence, reference.occurrence);
    assert_eq!(metadata.proposal.revision, reference.revision);
    assert_eq!(adopted.completed().len(), 1);
    match &adopted.completed()[0] {
        WorthQuerySourceCompletion::Fresh { result, .. } => assert_eq!(result.days, 3),
        WorthQuerySourceCompletion::Recovered { .. } => {
            panic!("first candidate execution is fresh")
        }
    }
    let performed = adopted
        .close_previous(&reviewer)
        .unwrap_or_else(|failure| panic!("post-adoption source close: {:?}", failure.cause));
    assert!(performed.previous_close().handoff.is_none());
    assert_eq!(model.branch(), candidate);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    drop(performed);
    mismatch::refuse_copied_source(&host, &model, &storage, &reviewer, &reference);
    storage.close(&author).unwrap();
    model.close(&reviewer).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}

#[test]
fn second_intent_invariant_refusal_retains_the_first_staged_edit_and_original_model() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let model = Model::create(&host, &author, "model-1", 721, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 722, 5).unwrap();
    let source = model.branch();
    let expected = model.describe(&reviewer).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            723,
            vec![
                AuthoredIntent::Retention(edit(&model, 3)),
                AuthoredIntent::Retention(edit(&model, 11)),
            ],
        )
        .unwrap();
    let mut staging = Staging::prepare(&model, &storage, &reviewer, &reference, 724)
        .unwrap_or_else(|cause| panic!("both granted typed operations must prepare: {cause:?}"));
    assert_eq!(staging.read(&model, &reviewer).unwrap(), 7);
    assert!(matches!(
        staging.advance(&model, &reviewer),
        StageEvent::StepCompleted
    ));
    assert_eq!(staging.read(&model, &reviewer).unwrap(), 3);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    assert!(matches!(
        staging.advance(&model, &reviewer),
        StageEvent::Terminal
    ));
    let Some(WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::Denied(cause),
    )) = staging.terminal_outcome(&model, &reviewer).unwrap()
    else {
        panic!("the second ordinary commit must reach the installed retention law")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    );
    assert!(cause.custom_invariant_violation_identity().is_some());
    assert!(cause.custom_invariant_failure().is_none());
    assert_eq!(staging.read(&model, &reviewer).unwrap(), 3);
    assert_eq!(model.branch(), source);
    assert_eq!(model.describe(&reviewer).unwrap(), expected);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    let staging = staging
        .finish()
        .err()
        .expect("a refused second step cannot enter adoption");
    let reader = host
        .request(&reviewer.principal, reviewer.scope)
        .on_branch(staging.branch())
        .retain_current_read()
        .unwrap();
    let failure = staging
        .close(&reviewer)
        .err()
        .expect("a real held reader defers retired history cleanup");
    let CloseRefusal::Candidate(
        WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(native),
    ) = &failure.cause
    else {
        panic!(
            "expected actual retired owner cleanup custody: {:?}",
            failure.cause
        )
    };
    assert!(matches!(
        native.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    let (cleanup, handoff) = failure
        .into_retired_cleanup()
        .unwrap_or_else(|failure| panic!("terminal retired staging disposal: {:?}", failure.cause));
    assert!(handoff.is_none());
    assert!(!cleanup.pending_work().is_empty());
    let blocked = cleanup
        .retry()
        .expect_err("the original external reader still retains history");
    assert!(matches!(
        blocked.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    let cleanup = blocked.into_cleanup();
    drop(reader);
    let settled = cleanup.retry().unwrap();
    assert_eq!(settled.retired_component_count(), 1);
    assert_eq!(model.branch(), source);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
