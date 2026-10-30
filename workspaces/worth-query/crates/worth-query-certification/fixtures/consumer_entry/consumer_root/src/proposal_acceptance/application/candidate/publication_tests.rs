//! An interrupted candidate edit retains genuine publication recovery before adoption.
use super::*;
use crate::proposal_acceptance::{
    application::acceptance::{Event, View},
    attempts::data::StoredStepOutcome,
    authentication::{self, Actor},
    host,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::WorthQueryManagedComputationInterruption,
};

fn progress(event: Event) {
    assert!(matches!(event, Event::Progress));
}

#[test]
fn unpublished_candidate_edit_recovers_before_adopting_the_unchanged_model() {
    let host = host::install();
    let author_scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &author_scope)
            .unwrap();
    let mut model = Model::create(&host, &author, "model-1", 1301, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 1302, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            1303,
            vec![AuthoredIntent::Retention(RetentionEdit {
                model: model.identity.clone(),
                target: model.target.clone(),
                days: 3,
            })],
        )
        .unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let original_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let original =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &original_scope)
            .unwrap();
    let loaded = storage.load_frozen(&original, &reference).unwrap();
    let source = model.branch();
    let basis = model.describe(&author).unwrap();
    progress(model.accept(&storage, &original, &loaded, 1304).unwrap());
    progress(model.accept(&storage, &original, &loaded, 1304).unwrap());
    progress(model.accept(&storage, &original, &loaded, 1304).unwrap());
    let Some(View::Staging(staging)) = model
        .inspect_attempt(&storage, &original, &loaded, 1304)
        .unwrap()
    else {
        panic!("the genuine candidate is staged")
    };
    let candidate = staging.branch();
    assert_ne!(candidate, source);
    assert_eq!(staging.read(&model, &original).unwrap(), 7);
    assert!(staging.step.is_none());
    let pause = host
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(std::num::NonZeroUsize::MIN);
    let stopped = std::thread::scope(|workers| {
        let worker = workers.spawn(|| model.accept(&storage, &original, &loaded, 1304));
        let reached = pause.wait_until_reached(std::time::Duration::from_secs(10));
        if reached {
            cancellation.cancel();
        }
        pause.release();
        let stopped = worker
            .join()
            .expect("the candidate operation returns custody");
        assert!(reached, "the real candidate edit reaches World comparison");
        stopped.unwrap()
    });
    assert!(matches!(
        stopped,
        Event::Staged(StageEvent::Refused(staging::StageRefusal::Advance(
            WorthQuerySourceOnlyDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            ))
        )))
    ));
    let fresh_scope = authentication::request_scope();
    let fresh =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &fresh_scope)
            .unwrap();
    let Some(View::Staging(staging)) = model
        .inspect_attempt(&storage, &fresh, &loaded, 1304)
        .unwrap()
    else {
        panic!("the original candidate survives interruption")
    };
    let Some(Step::Pending(operation)) = &staging.step else {
        panic!("the original ordinary operation retains unpublished custody")
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    assert_eq!(staging.branch(), candidate);
    assert!(staging.completed.is_empty());
    assert_eq!(model.branch(), source);
    assert_eq!(model.describe(&fresh).unwrap(), basis);
    assert_eq!(model.read(&fresh).unwrap().days, 7);
    assert_eq!(other.read(&fresh).unwrap().days, 5);
    assert!(matches!(
        model.accept(&storage, &fresh, &loaded, 1304).unwrap(),
        Event::Staged(StageEvent::Progress)
    ));
    let Some(View::Staging(staging)) = model
        .inspect_attempt(&storage, &fresh, &loaded, 1304)
        .unwrap()
    else {
        panic!("recovery preserves the candidate")
    };
    let Some(Step::Pending(operation)) = &staging.step else {
        panic!("native recovery remains owned before keyed completion")
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::RecoveryPerformed
    );
    assert_eq!(staging.branch(), candidate);
    assert!(matches!(
        model.accept(&storage, &fresh, &loaded, 1304).unwrap(),
        Event::Staged(StageEvent::StepCompleted)
    ));
    let Some(View::Staging(staging)) = model
        .inspect_attempt(&storage, &fresh, &loaded, 1304)
        .unwrap()
    else {
        panic!("only the recovered step has completed")
    };
    assert!(staging.step.is_none());
    assert!(matches!(
        staging.completed.as_slice(),
        [WorthQuerySourceCompletion::Recovered { .. }]
    ));
    assert_eq!(staging.branch(), candidate);
    assert_eq!(staging.read(&model, &fresh).unwrap(), 3);
    assert_eq!(model.branch(), source);
    assert_eq!(model.describe(&fresh).unwrap(), basis);
    assert_eq!(model.read(&fresh).unwrap().days, 7);
    assert_eq!(other.read(&fresh).unwrap().days, 5);
    progress(model.accept(&storage, &fresh, &loaded, 1304).unwrap());
    assert!(matches!(
        model.accept(&storage, &fresh, &loaded, 1304).unwrap(),
        Event::Adopted
    ));
    assert_eq!(model.branch(), candidate);
    assert_eq!(model.read(&fresh).unwrap().days, 3);
    progress(model.accept(&storage, &fresh, &loaded, 1304).unwrap());
    progress(model.accept(&storage, &fresh, &loaded, 1304).unwrap());
    let Event::Completed(accepted) = model.accept(&storage, &fresh, &loaded, 1304).unwrap() else {
        panic!("the original adoption finishes result delivery and previous-head close")
    };
    assert_eq!(accepted.result.steps, vec![StoredStepOutcome::Recovered]);
    let current = model.describe(&fresh).unwrap();
    let Event::Cached(result) = model.accept(&storage, &fresh, &loaded, 1304).unwrap() else {
        panic!("the completed attempt resolves its recorded result")
    };
    assert_eq!(result, accepted.result);
    assert_eq!(model.branch(), candidate);
    assert_eq!(model.describe(&fresh).unwrap(), current);
    assert_eq!(other.read(&fresh).unwrap().days, 5);
    drop(accepted);
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
