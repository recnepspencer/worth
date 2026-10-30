//! Cancellation while the actual candidate currentness guard waits retains Ready custody.
use super::*;
use crate::proposal_acceptance::{
    application::candidate::{adoption::AdoptionRefusal, StageEvent},
    model_query,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
};

#[test]
fn stopped_adoption_guard_wait_preserves_the_same_candidate_for_a_fresh_request() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let original_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let original =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &original_scope)
            .unwrap();
    let mut model = Model::create(&host, &author, "model-1", 2001, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 2002, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            2003,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let loaded = storage.load_frozen(&original, &reference).unwrap();
    let source = model.branch();
    let basis = model.describe(&original).unwrap();
    progress(model.accept(&storage, &original, &loaded, 2004));
    progress(model.accept(&storage, &original, &loaded, 2004));
    progress(model.accept(&storage, &original, &loaded, 2004));
    assert!(matches!(
        model.accept(&storage, &original, &loaded, 2004).unwrap(),
        Event::Staged(StageEvent::StepCompleted)
    ));
    let Some(View::Staging(staging)) = model
        .inspect_attempt(&storage, &original, &loaded, 2004)
        .unwrap()
    else {
        panic!("the ordinary edit is genuinely staged")
    };
    let candidate = staging.branch();
    assert_ne!(candidate, source);
    assert_eq!(staging.read(&model, &original).unwrap(), 3);
    progress(model.accept(&storage, &original, &loaded, 2004));
    assert!(matches!(
        model
            .inspect_attempt(&storage, &original, &loaded, 2004)
            .unwrap(),
        Some(View::Ready)
    ));
    let pause = host
        .runtime()
        .pause_product_branch_currentness_for_test(candidate);
    let stopped = std::thread::scope(|workers| {
        let worker = workers.spawn(|| model.accept(&storage, &original, &loaded, 2004));
        let contended = pause.wait_until_reader_contended(std::time::Duration::from_secs(10));
        if contended {
            cancellation.cancel();
        }
        pause.release();
        let result = worker.join();
        assert!(
            contended,
            "the candidate's actual guarded currentness reader waits"
        );
        result.unwrap()
    });
    assert!(matches!(
        stopped,
        Err(Failure::Adoption {
            cause: AdoptionRefusal::Owner(WorthQueryExpectedProductBranchAdoptionDenial::Stopped(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            )),
            ..
        })
    ));
    let fresh_scope = authentication::request_scope();
    let fresh =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &fresh_scope)
            .unwrap();
    assert_eq!(model.branch(), source);
    assert_eq!(model.describe(&fresh).unwrap(), basis);
    assert_eq!(model.read(&fresh).unwrap().days, 7);
    assert_eq!(other.read(&fresh).unwrap().days, 5);
    assert!(matches!(
        model
            .inspect_attempt(&storage, &fresh, &loaded, 2004)
            .unwrap(),
        Some(View::Ready)
    ));
    {
        let rows = host
            .request(&fresh.principal, fresh.scope)
            .on_branch(candidate)
            .query(model_query::ReadTarget {
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
        assert_eq!(rows.rows()[0].days, 3);
    }
    assert!(matches!(
        model.accept(&storage, &fresh, &loaded, 2004).unwrap(),
        Event::Adopted
    ));
    assert_eq!(model.branch(), candidate);
    assert_eq!(model.read(&fresh).unwrap().days, 3);
    let accepted = finish(&mut model, &storage, &fresh, &loaded, 2004);
    let current = model.describe(&fresh).unwrap();
    let Event::Cached(result) = model.accept(&storage, &fresh, &loaded, 2004).unwrap() else {
        panic!("the original adoption resolves its recorded result")
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
