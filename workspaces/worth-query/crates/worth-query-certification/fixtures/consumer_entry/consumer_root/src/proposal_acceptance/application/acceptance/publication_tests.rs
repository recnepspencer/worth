//! A genuine unpublished Claim keeps legal reads and resumes only its original native owner.
use super::*;
use crate::proposal_acceptance::{
    access,
    attempts::query,
    authentication::{self, Actor},
    host,
    schema::data::AuthoredIntent,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    primary_graph::WorthQueryApplicationIdempotencyResolutionDenialKind,
};
fn progress(event: Result<Event, Failure>) {
    assert!(matches!(event.unwrap(), Event::Progress));
}
#[test]
fn unpublished_claim_resumes_its_native_owner_after_fresh_admission_and_native_drift_checks() {
    let host = host::install();
    let author_scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &author_scope)
            .unwrap();
    let mut model = Model::create(&host, &author, "model-1", 1201, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 1202, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let input = |days| {
        AuthoredIntent::Retention(crate::proposal_acceptance::schema::data::RetentionEdit {
            model: model.identity.clone(),
            target: model.target.clone(),
            days,
        })
    };
    let reference = storage
        .record(&model, &author, "proposal-1", 1203, vec![input(3)])
        .unwrap();
    let changed = storage
        .record(&model, &author, "proposal-2", 1204, vec![input(4)])
        .unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let original_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let original =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &original_scope)
            .unwrap();
    let loaded = storage.load_frozen(&original, &reference).unwrap();
    let changed = storage.load_frozen(&original, &changed).unwrap();
    let source = model.branch();
    let basis = model.describe(&author).unwrap();
    progress(model.accept(&storage, &original, &loaded, 1205));
    let pause = host
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(std::num::NonZeroUsize::MIN);
    let stopped = std::thread::scope(|workers| {
        let worker = workers.spawn(|| model.accept(&storage, &original, &loaded, 1205));
        let reached = pause.wait_until_reached(std::time::Duration::from_secs(10));
        if reached {
            cancellation.cancel();
        }
        pause.release();
        let stopped = worker
            .join()
            .expect("World cutoff returns actual source custody");
        assert!(
            reached,
            "the Claim reaches comparison after its real owner effect"
        );
        stopped
    });
    assert!(matches!(
        stopped,
        Err(Failure::MetadataAdvance(
            WorthQuerySourceOnlyDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            ))
        ))
    ));
    let Some(Attempt {
        phase: Phase::Claim(operation),
        ..
    }) = &model.pending
    else {
        panic!("actual Claim operation remains in its original slot")
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    assert_eq!(model.branch(), source);
    assert_eq!(model.describe(&author).unwrap(), basis);
    assert_eq!(model.read(&author).unwrap().days, 7);
    assert_eq!(other.read(&author).unwrap().days, 5);
    let fresh_scope = authentication::request_scope();
    let fresh =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &fresh_scope)
            .unwrap();
    // Current World reads remain lawful; only exact unpublished key resolution refuses.
    storage.admit_attempt(&model, &fresh, &loaded).unwrap();
    let cause = storage
        .resolve_claim(&fresh, &loaded.input(1205), 1205)
        .unwrap_err();
    let WorthQueryApplicationRecoveryRequestDenial::Idempotency(cause) = cause else {
        panic!("expected the actual key-resolution refusal")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable
    );
    let drift = model.accept(&storage, &fresh, &changed, 1205);
    assert!(
        matches!(drift, Err(Failure::IntentDrift)),
        "actual changed-input response: {drift:?}"
    );
    let Some(Attempt {
        phase: Phase::Claim(operation),
        ..
    }) = &model.pending
    else {
        panic!("native drift cannot replace the original continuation")
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    progress(model.accept(&storage, &fresh, &loaded, 1205));
    progress(model.accept(&storage, &fresh, &loaded, 1205));
    let Some(Attempt {
        phase: Phase::Registered(registration),
        ..
    }) = &model.pending
    else {
        panic!("original recovery supplies one actual registration")
    };
    assert_eq!(registration.claim.data.input, loaded.input(1205));
    assert!(matches!(
        registration.evidence,
        WorthQuerySourceCompletion::Recovered { .. }
    ));
    let claims = host
        .request(&fresh.principal, fresh.scope)
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
    assert_eq!(claims.rows().len(), 1);
    assert_eq!(claims.rows()[0].data.input, loaded.input(1205));
    drop(claims);
    progress(model.accept(&storage, &fresh, &loaded, 1205));
    assert!(matches!(
        model.accept(&storage, &fresh, &loaded, 1205).unwrap(),
        Event::Staged(candidate::StageEvent::StepCompleted)
    ));
    progress(model.accept(&storage, &fresh, &loaded, 1205));
    assert!(matches!(
        model.accept(&storage, &fresh, &loaded, 1205).unwrap(),
        Event::Adopted
    ));
    let head = model.branch();
    assert_ne!(head, source);
    assert_eq!(model.read(&fresh).unwrap().days, 3);
    progress(model.accept(&storage, &fresh, &loaded, 1205));
    progress(model.accept(&storage, &fresh, &loaded, 1205));
    let Event::Completed(accepted) = model.accept(&storage, &fresh, &loaded, 1205).unwrap() else {
        panic!("the original attempt finishes ordinary delivery")
    };
    assert_eq!(
        accepted.result.steps,
        vec![StoredStepOutcome::Fresh { days: 3 }]
    );
    assert_eq!(other.read(&fresh).unwrap().days, 5);
    let current = model.describe(&fresh).unwrap();
    let Event::Cached(result) = model.accept(&storage, &fresh, &loaded, 1205).unwrap() else {
        panic!("same attempt resolves its actual stored result")
    };
    assert_eq!(result, accepted.result);
    assert_eq!(model.branch(), head);
    assert_eq!(model.describe(&fresh).unwrap(), current);
    drop(accepted);
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
