//! Already-stopped original requests cannot record a proposal or open an acceptance.
use super::*;
use crate::proposal_acceptance::{
    application::storage::{AdmissionFailure, RecordFailure},
    failure::Failure as ApplicationFailure,
    storage_query,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
};

#[test]
fn cancelled_recording_and_acceptance_preserve_both_products_for_a_fresh_request() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 2101, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 2102, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let reference = storage
        .record(
            &model,
            &author,
            "proposal-1",
            2103,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let loaded = storage.load_frozen(&reviewer, &reference).unwrap();
    let record_cancellation = WorthQueryCancellationSource::new();
    let record_scope = WorthQueryRequestScope::new(None, record_cancellation.token());
    let stopped_author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &record_scope)
            .unwrap();
    let accept_cancellation = WorthQueryCancellationSource::new();
    let accept_scope = WorthQueryRequestScope::new(None, accept_cancellation.token());
    let stopped_reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &accept_scope)
            .unwrap();
    let head = model.branch();
    let before = model.describe(&reviewer).unwrap();
    let storage_before = describe(&host, &reviewer, storage.branch());
    record_cancellation.cancel();
    accept_cancellation.cancel();
    assert!(matches!(
        storage.record(
            &model,
            &stopped_author,
            "never-recorded",
            2104,
            vec![AuthoredIntent::Retention(edit(&model, 4))]
        ),
        Err(RecordFailure::Request(ApplicationFailure::Advancement(
            WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            )
        )))
    ));
    assert!(matches!(
        model.accept(&storage, &stopped_reviewer, &loaded, 2105),
        Err(Failure::Admission(AdmissionFailure::Request(
            ApplicationFailure::Query(WorthQueryApplicationRequestQueryDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            ))
        )))
    ));
    assert_eq!(model.branch(), head);
    assert_eq!(model.describe(&reviewer).unwrap(), before);
    assert_eq!(describe(&host, &reviewer, storage.branch()), storage_before);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    {
        let rows = host
            .request(&reviewer.principal, reviewer.scope)
            .on_branch(storage.branch())
            .query(storage_query::ReadProposals {
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
        assert_eq!(rows.rows()[0].key, reference.key);
        assert_eq!(rows.rows()[0].occurrence, reference.occurrence);
        assert_eq!(rows.rows()[0].data, *loaded.proposal());
    }
    assert!(model
        .inspect_attempt(&storage, &reviewer, &loaded, 2105)
        .unwrap()
        .is_none());
    let resolution = host
        .request(&reviewer.principal, reviewer.scope)
        .on_branch(storage.branch())
        .advanced()
        .mutate(loaded.input(2105))
        .without_source()
        .idempotency(&2105)
        .resolve_idempotency_in_program(&host)
        .unwrap()
        .into_resolution();
    assert!(matches!(
        resolution,
        WorthQueryApplicationIdempotencyResolution::Unseen
    ));
    assert_eq!(model.describe(&reviewer).unwrap(), before);
    assert_eq!(describe(&host, &reviewer, storage.branch()), storage_before);
    let fresh_scope = authentication::request_scope();
    let fresh =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &fresh_scope)
            .unwrap();
    reach_adoption(&mut model, &storage, &fresh, &loaded, 2105);
    assert_eq!(model.read(&fresh).unwrap().days, 3);
    assert_eq!(other.read(&fresh).unwrap().days, 5);
    drop(finish(&mut model, &storage, &fresh, &loaded, 2105));
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
