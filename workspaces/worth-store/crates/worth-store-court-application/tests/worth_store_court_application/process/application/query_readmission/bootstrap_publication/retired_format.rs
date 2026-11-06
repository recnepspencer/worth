//! A retired completion version cannot initialize or serve an existing Home.
use super::*;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};
use worth_query_host::facade::WorthQueryApplicationHomeOpenRefused;
use worth_relational::facade::transactions::*;

#[test]
fn retired_bootstrap_completion_format_refuses_existing_home_without_replaying_seeds() {
    let path = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    let callbacks = AtomicUsize::new(0);
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (mut query, mut home, _) = prepared(path.path(), &phase, true, &callbacks);
        query.publish_home_query_seed(&phase).unwrap();
        {
            let native = home.relational().runtime();
            let (_, basis) = native
                .observe_branch(&native.main_branch_identity())
                .unwrap();
            let view = native
                .read_truth()
                .project_observation(&basis.observation())
                .unwrap();
            let pin = native
                .read_truth()
                .pin_view_read_capability(&view, &[], &execution)
                .unwrap();
            let kind = pin
                .schema()
                .authority_snapshot()
                .entity_kinds
                .iter()
                .find(|kind| kind.kind_name == "worth-query-bootstrap-publication")
                .unwrap()
                .kind_id;
            let rows = view
                // This is the complete finite fixture, under the same request
                // as readmission. The bound counts examined slots, not matches.
                .bounded_entities_of_kind(&execution, kind, usize::MAX)
                .unwrap()
                .unwrap();
            let [row] = rows.records() else {
                panic!("one real completion");
            };
            let mut transaction = native
                .begin_branch_transaction(
                    &basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .unwrap();
            transaction
                .push_batch(
                    WorkerIntentBatch::new("retired bootstrap format").push(
                        MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                            UpdateEntityFieldsIntent {
                                entity_id: row.entity_id,
                                fields: AspectFieldPatch::from_locator(
                                    planned_single_field_locator(
                                        AspectKey::new("bootstrap-publication").unwrap(),
                                        FieldKey::new("format-version").unwrap(),
                                    ),
                                    AspectValue::UInt64(1),
                                ),
                            },
                        )),
                    ),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let committed = transaction.commit(&execution, native).unwrap();
            native
                .snapshots()
                .release_snapshot(&committed.snapshot)
                .unwrap();
        }
        let root = home.select_query_root(&execution).unwrap();
        drop(root);
        drop(query);
        assert!(matches!(
            home.close(&execution)
                .unwrap_or_else(|refused| panic!("actual World close: {:?}", refused.reason())),
            ApplicationHomeCloseOutcome::Closed { .. }
        ));
    })
    .unwrap();
    assert_eq!(callbacks.load(Ordering::SeqCst), 1);
    let reopened_callbacks = Arc::new(AtomicUsize::new(0));
    let request = fixture::request(
        path.path().to_path_buf(),
        build(),
        reopened_callbacks.clone(),
        Arc::new(AtomicUsize::new(0)),
    );
    let refused = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .err()
    .expect("retired development format must refuse");
    let WorthQueryApplicationHomeOpenRefused::ExistingQueryAttachment { query, .. } = &refused
    else {
        panic!("actual selected Query completion must refuse: {refused:?}");
    };
    assert!(
        query
            .detail()
            .contains("Home bootstrap completion format is unsupported"),
        "typed existing Query refusal: {}",
        query.detail()
    );
    assert_eq!(reopened_callbacks.load(Ordering::SeqCst), 0);
    refused
        .close(&execution)
        .unwrap_or_else(|refused| panic!("same opening custody closes: {refused:?}"));
}
