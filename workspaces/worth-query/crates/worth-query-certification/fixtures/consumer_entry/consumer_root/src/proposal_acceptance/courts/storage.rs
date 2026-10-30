//! Only stored data survives recording; loading cannot move either model head.
use super::super::{
    application::{model::Model, storage::Storage},
    authentication::{self, Actor},
    host,
    schema::data::*,
};
use worth_query_host::facade::product::WorthQueryExpectedProductBasisDescription;

#[test]
fn ordinary_proposal_record_and_typed_reload_leave_both_model_bases_current() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let model = Model::create(&host, &author, "model-1", 101, 7).unwrap();
    let other = Model::create(&host, &reviewer, "model-2", 102, 5).unwrap();
    assert_ne!(model.identity, other.identity);
    let expected = model.describe(&author).unwrap();
    let other_expected = other.describe(&reviewer).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    assert_ne!(storage.branch(), model.branch());
    let intent = AuthoredIntent::Retention(RetentionEdit {
        model: model.identity.clone(),
        target: model.target.clone(),
        days: 3,
    });
    let reference = {
        let recording_scope = authentication::request_scope();
        let recording_author =
            authentication::authenticate(host.installed_schema(), Actor::Author, &recording_scope)
                .unwrap();
        let decoy = |days| {
            vec![AuthoredIntent::Retention(RetentionEdit {
                model: model.identity.clone(),
                target: model.target.clone(),
                days,
            })]
        };
        storage
            .record(&model, &recording_author, "proposal-0", 200, decoy(2))
            .unwrap();
        let reference = storage
            .record(
                &model,
                &recording_author,
                "proposal-1",
                201,
                vec![intent.clone()],
            )
            .unwrap();
        storage
            .record(&model, &recording_author, "proposal-2", 202, decoy(4))
            .unwrap();
        reference
    };
    // record() returned only descriptive references: its request and selection ended.
    let loaded = storage.load(&reviewer, &reference).unwrap();
    assert_eq!(loaded.format_version, 1);
    assert_eq!(loaded.revision, reference.revision);
    assert_eq!(loaded.model, model.identity);
    assert_eq!(loaded.intents, vec![intent]);
    let decoded: WorthQueryExpectedProductBasisDescription =
        serde_json::from_str(&loaded.expected_basis).unwrap();
    assert_eq!(decoded, expected);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    assert_eq!(other.read(&author).unwrap().days, 5);
    host.with_application_advancement(&scope, |phase| {
        host.branches().select_expected(&decoded, &phase).unwrap();
        host.branches()
            .select_expected(&other_expected, &phase)
            .unwrap();
    })
    .unwrap();
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&reviewer).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}

#[test]
fn new_model_with_reused_local_name_receives_a_distinct_owner_issued_incarnation() {
    use super::acceptance::{finish, reach_adoption};
    use crate::proposal_acceptance::application::{acceptance, storage::AdmissionFailure};
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let first = Model::create(&host, &author, "model-1", 301, 7).unwrap();
    let identity = first.identity.clone();
    let old_storage = Storage::create(&first, &author).unwrap();
    let old = old_storage
        .record(
            &first,
            &author,
            "proposal-1",
            303,
            vec![AuthoredIntent::Retention(RetentionEdit {
                model: first.identity.clone(),
                target: first.target.clone(),
                days: 3,
            })],
        )
        .unwrap();
    let old = old_storage.load_frozen(&reviewer, &old).unwrap();
    first.close(&author).unwrap();
    let mut replacement = Model::create(&host, &reviewer, "model-1", 302, 5).unwrap();
    assert_eq!(identity.label, replacement.identity.label);
    assert_eq!(identity.runtime, replacement.identity.runtime);
    assert_ne!(identity.occurrence, replacement.identity.occurrence);
    assert_eq!(replacement.read(&author).unwrap().days, 5);
    let head = replacement.branch();
    let before = replacement.describe(&reviewer).unwrap();
    let old_storage_basis = || {
        host.with_application_advancement(reviewer.scope, |phase| {
            host.on_branch(old_storage.branch())
                .select(&phase)
                .unwrap()
                .describe_basis()
        })
        .unwrap()
    };
    let storage_before = old_storage_basis();
    assert!(matches!(
        replacement.accept(&old_storage, &reviewer, &old, 304),
        Err(acceptance::Failure::Admission(
            AdmissionFailure::StorageModelMismatch
        ))
    ));
    assert_eq!(replacement.branch(), head);
    assert_eq!(replacement.describe(&reviewer).unwrap(), before);
    assert_eq!(old_storage_basis(), storage_before);
    assert_eq!(replacement.read(&reviewer).unwrap().days, 5);
    let storage = Storage::create(&replacement, &author).unwrap();
    let reference = storage
        .record(
            &replacement,
            &author,
            "proposal-1",
            305,
            vec![AuthoredIntent::Retention(RetentionEdit {
                model: replacement.identity.clone(),
                target: replacement.target.clone(),
                days: 3,
            })],
        )
        .unwrap();
    let loaded = storage.load_frozen(&reviewer, &reference).unwrap();
    reach_adoption(&mut replacement, &storage, &reviewer, &loaded, 306);
    assert_eq!(replacement.read(&reviewer).unwrap().days, 3);
    drop(finish(&mut replacement, &storage, &reviewer, &loaded, 306));
    old_storage.close(&author).unwrap();
    storage.close(&author).unwrap();
    replacement.close(&reviewer).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
