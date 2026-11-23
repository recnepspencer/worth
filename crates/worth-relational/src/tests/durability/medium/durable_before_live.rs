//! A commit reaches the medium before it reaches live state, and a refusal
//! from the medium leaves live state exactly as it was.

use crate::durability::medium::RelationalDurablePublicationRefusal;
use crate::facade::history::BranchId;
use crate::facade::transactions::TransactionCommitError;
use crate::mvcc::RelationalPublicationDenial;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;

/// Everything a commit could move, observed from outside the medium.
#[derive(Debug, PartialEq, Eq)]
struct LiveFootprint {
    main_head: Option<crate::history::data::RelationalCommitReceipt>,
    durable_log_len: usize,
    latest_patch: Option<PatchStreamPosition>,
    live_entities: usize,
    published_snapshot_handles: usize,
    published_snapshot_capacity: usize,
}

fn live_footprint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> LiveFootprint {
    LiveFootprint {
        main_head: runtime.history().branch_head(&BranchId("main".to_owned())),
        durable_log_len: runtime.durability().durable_commit_count(),
        latest_patch: runtime
            .publication()
            .latest_patch()
            .map(|patch| patch.position),
        live_entities: runtime
            .storage_access()
            .storage_stats(execution)
            .unwrap()
            .live_entities,
        published_snapshot_handles: runtime.visibility.published_snapshot_handle_count(),
        published_snapshot_capacity: runtime
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
    }
}

fn commit_one(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> Result<crate::facade::transactions::CommitResult, TransactionCommitError> {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            batch_create(name),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction.commit(execution, runtime)
}

#[test]
fn a_refused_publication_was_offered_to_the_medium_and_moved_nothing_live() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let anchor = create_entity_outcome(execution, &runtime, "anchor");
    release_test_commit_snapshot(&runtime, &anchor);
    let before = live_footprint(execution, &runtime);

    let definition = runtime
        .index_authority()
        .register(crate::indexes::data::DerivedIndexDefinition {
            index_id: crate::indexes::data::DerivedIndexId(0),
            name: "refused-catalog".into(),
            branch_scoped: true,
            kind: crate::indexes::data::DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        })
        .unwrap();
    let built = runtime.index_authority().build_for_commit(
        execution,
        crate::indexes::data::DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![definition.index_id],
        },
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    medium.script_next_publication(ScriptedPublication::ProvenNoEffect);
    let refused = commit_one(execution, &runtime, "refused").expect_err("the medium refused");

    assert!(
        matches!(
            refused,
            TransactionCommitError::PublicationDenied {
                denial: RelationalPublicationDenial::DurablePublicationRefused(
                    RelationalDurablePublicationRefusal::ProvenNoEffect { .. }
                ),
                ..
            }
        ),
        "expected ProvenNoEffect durable refusal, got {refused:?}"
    );
    let offered = medium.offered_commits().pop().expect("the medium was asked");
    assert_eq!(offered.position(), PatchStreamPosition(2));
    assert_eq!(live_footprint(execution, &runtime), before);
    assert_eq!(medium.head(), PatchStreamPosition(1));
    assert_eq!(
        runtime
            .preparation_runtime_snapshot()
            .history
            .index_definition_frontier(),
        1
    );
    let accepted = commit_one(execution, &runtime, "catalog-after-refusal").unwrap();
    assert_eq!(
        accepted
            .envelope()
            .index_definition_delta
            .base_next_index_id,
        1
    );
    assert_eq!(
        accepted.envelope().index_definition_delta.definitions(),
        &[definition]
    );
    release_test_commit_snapshot(&runtime, &accepted);
}

#[test]
fn a_refused_position_is_not_consumed() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let anchor = create_entity_outcome(execution, &runtime, "anchor");
    release_test_commit_snapshot(&runtime, &anchor);

    medium.script_next_publication(ScriptedPublication::ProvenNoEffect);
    commit_one(execution, &runtime, "refused").expect_err("the medium refused");
    let refused = medium.offered_commits().pop().expect("the medium was asked");
    let accepted = commit_one(execution, &runtime, "accepted").expect("the medium accepts");

    assert_eq!(accepted.patch_position(), PatchStreamPosition(2));
    assert_ne!(
        accepted.commit.commit_id,
        refused.commit_id(),
        "an identity a refused candidate held is burned, never reissued"
    );
    assert_eq!(accepted.commit.parents, vec![anchor.commit.commit_id]);
    assert_eq!(medium.head(), PatchStreamPosition(2));
    release_test_commit_snapshot(&runtime, &accepted);
}
