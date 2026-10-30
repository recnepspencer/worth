//! A cached actual result cannot replace current independent READ or mutation authorization.
use super::super::super::{
    access::{self, status, ReadKind},
    application::storage::{AdmissionFailure, ClaimFailure, ResultFailure},
    failure::Failure,
    mutation_access,
};
use super::*;
use worth_query_host::facade::primary_graph::*;
fn read_denied(failure: AdmissionFailure) {
    let AdmissionFailure::Request(Failure::Query(
        WorthQueryApplicationRequestQueryDenial::Admission(cause),
    )) = failure
    else {
        panic!("expected result READ denial: {failure:?}")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                WorthQueryOperationAuthorizationRefusalKind::PermissionDenied
            )
        )
    );
}
#[test]
fn cached_results_require_current_result_read_and_independent_mutation_authority() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 821, 7).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            822,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let loaded = storage.load_frozen(&reviewer, &reference).unwrap();
    let registered = storage.claim(&model, &reviewer, &loaded, 823).unwrap();
    let claim = registered.reference().clone();
    drop(registered);
    let other = storage.claim(&model, &reviewer, &loaded, 824).unwrap();
    let other_claim = other.reference().clone();
    drop(other);
    let adopted = adopt(&mut model, &storage, &reviewer, &loaded, 823);
    let result = storage
        .record_result(&model, &reviewer, &loaded, &claim, &adopted)
        .unwrap();
    drop(
        adopted
            .close_previous(&reviewer)
            .unwrap_or_else(|failure| panic!("close previous model: {:?}", failure.cause)),
    );
    assert_eq!(
        storage
            .read_result(&model, &reviewer, &loaded, &claim)
            .unwrap(),
        result
    );
    let change_read = |readable, key| {
        host.request(&author.principal, author.scope)
            .on_branch(storage.branch())
            .advanced()
            .mutate(status::SetStatus {
                identity: access::grant_key("model-1", 2, ReadKind::Result),
                active: true,
                readable,
            })
            .without_source()
            .idempotency(&key)
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    };
    assert!(matches!(
        change_read(false, 825),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    storage.load(&reviewer, &reference).unwrap();
    let before = model.describe(&reviewer).unwrap();
    for reference in [&claim, &other_claim] {
        let ResultFailure::Admission(cause) = storage
            .read_result(&model, &reviewer, &loaded, reference)
            .unwrap_err()
        else {
            panic!("deny before protected result/association lookup")
        };
        read_denied(cause);
    }
    let ClaimFailure::Admission(cause) = storage
        .claim(&model, &reviewer, &loaded, 823)
        .err()
        .expect("cached registration also requires result READ")
    else {
        panic!("expected READ before ordinary retry resolution")
    };
    read_denied(cause);
    assert_eq!(model.describe(&reviewer).unwrap(), before);
    assert!(matches!(
        change_read(true, 826),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(
        storage
            .read_result(&model, &reviewer, &loaded, &claim)
            .unwrap(),
        result
    );
    let change_mutation = |active, key| {
        host.request(&author.principal, author.scope)
            .on_branch(model.branch())
            .advanced()
            .mutate(mutation_access::status::SetStatus {
                identity: mutation_access::grant_key("model-1", 2),
                active,
            })
            .without_source()
            .idempotency(&key)
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    };
    assert!(matches!(
        change_mutation(false, 827),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    storage.load(&reviewer, &reference).unwrap();
    let failure = storage
        .read_result(&model, &reviewer, &loaded, &claim)
        .unwrap_err();
    let ResultFailure::Admission(AdmissionFailure::Preparation(
        super::super::super::application::candidate::PreparationRefusal::Mutation(
            WorthQueryApplicationRequestMutationDenial::Authorization(cause),
        ),
    )) = failure
    else {
        panic!("mutation refusal must precede protected cached result")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing
        )
    );
    assert!(matches!(
        change_mutation(true, 828),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(
        storage
            .read_result(&model, &reviewer, &loaded, &claim)
            .unwrap(),
        result
    );
    let before = model.describe(&reviewer).unwrap();
    let retry = storage.claim(&model, &reviewer, &loaded, 823).unwrap();
    assert!(matches!(
        retry.evidence(),
        WorthQuerySourceCompletion::Recovered { .. }
    ));
    drop(retry);
    assert_eq!(model.describe(&reviewer).unwrap(), before);
    storage.close(&author).unwrap();
    model.close(&reviewer).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}

#[test]
fn frozen_input_keeps_its_issuing_storage_and_fresh_read_requirement() {
    use super::super::super::application::candidate::{PreparationFailure, PreparationRefusal};
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let model = Model::create(&host, &author, "model-1", 841, 7).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let other = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            842,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let loaded = storage.load_frozen(&reviewer, &reference).unwrap();
    let before = model.describe(&reviewer).unwrap();
    let change = |readable, key| {
        host.request(&author.principal, author.scope)
            .on_branch(storage.branch())
            .advanced()
            .mutate(status::SetStatus {
                identity: access::grant_key("model-1", 2, ReadKind::Proposal),
                active: true,
                readable,
            })
            .without_source()
            .idempotency(&key)
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    };
    assert!(matches!(
        change(false, 843),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    other.admit_proposal_read(&reviewer).unwrap();
    let failure = Staging::prepare_frozen(&model, &other, &reviewer, &loaded, 844)
        .err()
        .expect("another readable product cannot substitute for the issuer");
    assert!(matches!(
        failure,
        PreparationFailure::Before(PreparationRefusal::Request(Failure::StorageMismatch))
    ));
    let failure = other
        .claim(&model, &reviewer, &loaded, 844)
        .err()
        .expect("claim lookup has the same storage-affinity receiver");
    assert!(matches!(
        failure,
        ClaimFailure::Admission(AdmissionFailure::Request(Failure::StorageMismatch))
    ));
    let failure = Staging::prepare_frozen(&model, &storage, &reviewer, &loaded, 844)
        .err()
        .expect("matching origin still needs its current READ");
    let PreparationFailure::Before(PreparationRefusal::Request(cause)) = failure else {
        panic!("expected actual origin READ denial")
    };
    read_denied(AdmissionFailure::Request(cause));
    assert_eq!(model.describe(&reviewer).unwrap(), before);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    assert!(matches!(
        change(true, 845),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let staged = Staging::prepare_frozen(&model, &storage, &reviewer, &loaded, 844)
        .unwrap_or_else(|cause| panic!("restored original READ admits frozen data: {cause:?}"));
    staged
        .close(&reviewer)
        .unwrap_or_else(|failure| panic!("close unexecuted candidate: {:?}", failure.cause));
    assert_eq!(model.describe(&reviewer).unwrap(), before);
    storage.close(&author).unwrap();
    other.close(&author).unwrap();
    model.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
