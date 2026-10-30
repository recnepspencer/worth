//! Native drift precedes new-input eligibility; busy and missing delivery never authorize a fork.
use super::*;
#[test]
fn occupied_model_and_registered_attempts_cannot_be_replaced_by_new_metadata() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 921, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 922, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            923,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let loaded = storage.load_frozen(&reviewer, &reference).unwrap();
    let mut foreign = edit(&model, 3);
    foreign.target = other.target.clone();
    let changed = storage
        .record(
            &model,
            &author,
            "proposal-2",
            924,
            vec![AuthoredIntent::Retention(foreign)],
        )
        .unwrap();
    let changed = storage.load_frozen(&reviewer, &changed).unwrap();
    let before = model.describe(&reviewer).unwrap();
    let registered = storage.claim(&model, &reviewer, &loaded, 925).unwrap();
    drop(registered);
    let store = describe(&host, &reviewer, storage.branch());
    assert!(matches!(
        model.accept(&storage, &reviewer, &loaded, 925),
        Err(Failure::ContinuationUnavailable)
    ));
    assert_eq!(model.describe(&reviewer).unwrap(), before);
    assert_eq!(describe(&host, &reviewer, storage.branch()), store);
    progress(model.accept(&storage, &reviewer, &loaded, 926));
    assert!(matches!(
        model.accept(&storage, &reviewer, &changed, 926),
        Err(Failure::OriginalContinuationRetained)
    ));
    assert!(matches!(
        model.accept(&storage, &reviewer, &loaded, 927),
        Err(Failure::Busy)
    ));
    assert_eq!(describe(&host, &reviewer, storage.branch()), store);
    let resolve = |key, input| {
        host.request(&reviewer.principal, reviewer.scope)
            .on_branch(storage.branch())
            .advanced()
            .mutate(input)
            .without_source()
            .idempotency(&key)
            .resolve_idempotency_in_program(&host)
            .unwrap()
            .into_resolution()
    };
    assert!(matches!(
        resolve(927, loaded.input(927)),
        WorthQueryApplicationIdempotencyResolution::Unseen
    ));
    progress(model.accept(&storage, &reviewer, &loaded, 926));
    let after_claim = describe(&host, &reviewer, storage.branch());
    assert_ne!(after_claim, store);
    assert!(matches!(
        model.accept(&storage, &reviewer, &changed, 926),
        Err(Failure::IntentDrift)
    ));
    assert!(matches!(
        model.accept(&storage, &reviewer, &loaded, 927),
        Err(Failure::Busy)
    ));
    assert!(matches!(
        resolve(927, loaded.input(927)),
        WorthQueryApplicationIdempotencyResolution::Unseen
    ));
    assert_eq!(describe(&host, &reviewer, storage.branch()), after_claim);
    assert_eq!(model.describe(&reviewer).unwrap(), before);
    progress(model.accept(&storage, &reviewer, &loaded, 926));
    assert!(matches!(
        model.accept(&storage, &reviewer, &loaded, 926).unwrap(),
        Event::Staged(super::super::super::application::candidate::StageEvent::StepCompleted)
    ));
    progress(model.accept(&storage, &reviewer, &loaded, 926));
    assert!(matches!(
        model.accept(&storage, &reviewer, &loaded, 926).unwrap(),
        Event::Adopted
    ));
    let accepted = finish(&mut model, &storage, &reviewer, &loaded, 926);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    drop(accepted);
    storage.close(&author).unwrap();
    model.close(&reviewer).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
