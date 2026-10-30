//! Real target retirement cannot turn an old proposal's display name into a new occurrence.
use super::*;
use crate::proposal_acceptance::{
    application::candidate::PreparationRefusal,
    schema::data::{RetentionEdit, TargetReference},
    target_replacement,
};

#[test]
fn target_replacement_requires_management_and_refuses_the_old_proposal_occurrence() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 1801, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 1802, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            1803,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let frozen = storage.load_frozen(&reviewer, &reference).unwrap();
    let input = target_replacement::ReplaceTarget {
        model: model.identity.clone(),
        target: model.target.clone(),
        key: "replacement-target".to_owned(),
        days: 7,
    };
    let source = model.branch();
    let before = model.describe(&author).unwrap();
    host.request(&reviewer.principal, reviewer.scope)
        .on_branch(source)
        .advanced()
        .mutate(edit(&model, 3))
        .without_source()
        .assess_current_capability_authorization()
        .unwrap();
    let denied = host
        .request(&reviewer.principal, reviewer.scope)
        .on_branch(source)
        .advanced()
        .mutate(input.clone())
        .without_source()
        .idempotency(&1804)
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap_err();
    let WorthQueryApplicationRequestMutationDenial::Authorization(cause) = denied else {
        panic!("replacement requires its actual management ability: {denied:?}")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            WorthQueryOperationAuthorizationRefusalKind::PermissionDenied
        )
    );
    assert_eq!(model.describe(&author).unwrap(), before);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    let mut foreign = input.clone();
    foreign.target = other.target.clone();
    let refused = host
        .request(&author.principal, author.scope)
        .on_branch(source)
        .advanced()
        .mutate(foreign)
        .without_source()
        .idempotency(&1805)
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        refused,
        WorthQueryApplicationMutationOutcome::DomainDenied(
            target_replacement::Denial::TargetMismatch
        )
    ));
    drop(refused);
    assert_eq!(model.describe(&author).unwrap(), before);
    let outcome = host
        .request(&author.principal, author.scope)
        .on_branch(source)
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(&1806)
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            &outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "actual target replacement outcome: {outcome:?}"
    );
    drop(outcome);
    let observed = model.read(&reviewer).unwrap();
    assert_eq!(observed.name, model.target.label);
    assert_ne!(observed.occurrence, model.target.occurrence);
    assert_eq!(observed.days, 7);
    assert_eq!(model.branch(), source);
    assert_ne!(model.describe(&author).unwrap(), before);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    let replacement = TargetReference {
        label: observed.name,
        occurrence: observed.occurrence,
    };
    let current = model.describe(&reviewer).unwrap();
    let storage_before = describe(&host, &reviewer, storage.branch());
    assert!(matches!(
        model.accept(&storage, &reviewer, &frozen, 1807),
        Err(Failure::Eligibility(PreparationRefusal::TargetMismatch))
    ));
    assert!(model
        .inspect_attempt(&storage, &reviewer, &frozen, 1807)
        .unwrap()
        .is_none());
    let resolution = host
        .request(&reviewer.principal, reviewer.scope)
        .on_branch(storage.branch())
        .advanced()
        .mutate(frozen.input(1807))
        .without_source()
        .idempotency(&1807)
        .resolve_idempotency_in_program(&host)
        .unwrap()
        .into_resolution();
    assert!(matches!(
        resolution,
        WorthQueryApplicationIdempotencyResolution::Unseen
    ));
    assert_eq!(model.branch(), source);
    assert_eq!(model.describe(&reviewer).unwrap(), current);
    assert_eq!(describe(&host, &reviewer, storage.branch()), storage_before);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-2",
            1808,
            vec![AuthoredIntent::Retention(RetentionEdit {
                model: model.identity.clone(),
                target: replacement,
                days: 3,
            })],
        )
        .unwrap();
    let fresh = storage.load_frozen(&reviewer, &reference).unwrap();
    reach_adoption(&mut model, &storage, &reviewer, &fresh, 1809);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    drop(finish(&mut model, &storage, &reviewer, &fresh, 1809));
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
