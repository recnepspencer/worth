use worth_proof::TransitionOutcome;

use crate::facade::change_source::{RelationalChangeReceipt, RelationalChangeReceiptStale};
use crate::facade::history::{BranchId, CommitId};
use crate::facade::identity::PartitionId;
use crate::facade::runtime::RelationalRuntime;
use crate::tests::support::{
    create_entity_in_partition, create_entity_outcome, create_entity_outcome_on_branch,
    runtime_with_test_schema,
};

use super::observe;

fn mint_at_head(
    runtime: &RelationalRuntime,
    branch: &str,
    commit_id: CommitId,
    partition_id: Option<PartitionId>,
) -> RelationalChangeReceipt {
    let observation = observe(runtime, branch).observation();
    let selected = runtime
        .select_reachable_commit(&observation, commit_id)
        .into_outcome()
        .expect("commit is reachable from the branch head");
    match runtime.mint_change_receipt(selected, partition_id) {
        TransitionOutcome::Success(receipt) => receipt,
        other => panic!("a committed, retained change mints a receipt: {other:?}"),
    }
}

#[test]
fn receipt_carries_the_canonical_patch_and_the_checks_it_passed() {
    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(&runtime, "receipt");
    let commit = &outcome.commit;

    let receipt = mint_at_head(&runtime, "main", commit.commit_id, None);

    assert_eq!(receipt.runtime_instance_id(), runtime.runtime_instance_id());
    assert_eq!(receipt.commit_id(), commit.commit_id);
    assert_eq!(receipt.version_id(), commit.version_id);
    assert_eq!(receipt.selected_branch_id(), &BranchId("main".to_owned()));
    assert_eq!(receipt.authoring_branch_id(), &BranchId("main".to_owned()));
    assert_eq!(receipt.partition_id(), None);
    assert_eq!(receipt.records_filtered_out(), 0);
    let records = receipt.patch().authoritative_record_patches.len() as u64;
    assert!(records > 0);
    assert_eq!(receipt.records_examined(), records);
    assert_eq!(receipt.patch(), &receipt.patch().canonicalized());
    let work = receipt.consistency_work();
    assert_eq!(work.records_checked(), records);
    assert_eq!(
        work.semantic_changes_matched(),
        work.expected_changes_materialized()
    );
    assert_eq!(receipt.commit_identity().value(), &commit.commit_id.0);
}

#[test]
fn receipt_narrows_to_one_partition_and_counts_what_it_filtered() {
    let runtime = runtime_with_test_schema();
    create_entity_in_partition(&runtime, "elsewhere", PartitionId(7));
    let commit_id = runtime
        .history()
        .next_commit_id()
        .0
        .checked_sub(1)
        .map(CommitId)
        .expect("one commit");

    let everything = mint_at_head(&runtime, "main", commit_id, None);
    let inside = mint_at_head(&runtime, "main", commit_id, Some(PartitionId(7)));
    let outside = mint_at_head(&runtime, "main", commit_id, Some(PartitionId(11)));

    let total = everything.records_examined();
    assert_eq!(inside.partition_id(), Some(PartitionId(7)));
    assert_eq!(inside.records_examined(), total);
    assert_eq!(inside.records_filtered_out(), 0);
    assert_eq!(outside.records_examined(), total);
    assert_eq!(outside.records_filtered_out(), total);
    assert!(outside.patch().authoritative_record_patches.is_empty());
}

#[test]
fn a_fork_receipt_names_the_selected_and_the_authoring_branch() {
    let runtime = runtime_with_test_schema();
    let inherited = create_entity_outcome(&runtime, "inherited").commit.commit_id;
    let feature = BranchId("feature".to_owned());
    runtime
        .history_authority()
        .fork_branch_from(feature.clone(), &BranchId("main".to_owned()))
        .unwrap();
    create_entity_outcome_on_branch(&runtime, "feature-head", feature.clone());

    let receipt = mint_at_head(&runtime, "feature", inherited, None);

    assert_eq!(receipt.selected_branch_id(), &feature);
    assert_eq!(receipt.authoring_branch_id(), &BranchId("main".to_owned()));
}

#[test]
fn another_runtime_cannot_mint_a_receipt_for_a_selection() {
    let owner = runtime_with_test_schema();
    let other = runtime_with_test_schema();
    let commit_id = create_entity_outcome(&owner, "owned").commit.commit_id;
    create_entity_outcome(&other, "unrelated");
    let observation = observe(&owner, "main").observation();
    let selected = owner
        .select_exact_commit(&observation, commit_id)
        .into_outcome()
        .unwrap();

    let outcome = other.mint_change_receipt(selected, None);

    assert!(matches!(
        outcome,
        TransitionOutcome::Stale(RelationalChangeReceiptStale::RuntimeAuthority)
    ));
}
