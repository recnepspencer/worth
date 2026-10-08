//! Retained scoped caches may lag their actual observing branch after restart.
use super::*;
use crate::facade::history::RelationalCommitReceipt;
use crate::indexes::data::DerivedIndexGenerationId;

#[test]
fn lagging_inherited_and_own_stream_scoped_generations_survive_checkpoint_reopen() {
    let runtime = persisted_runtime_with_test_schema();
    let anchor = create_entity_outcome(&runtime, "immutable-parent-name");
    let entity = changed_entities(&anchor)[0];
    let branch = create_branch_from_main(&runtime, "lagging-scoped-fork");
    let index = scoped_name_index(&runtime);
    let identity = runtime.branch_identity(&branch).unwrap();
    let (_, inherited_basis) = runtime.observe_branch(&identity).unwrap();
    let inherited = runtime.index_authority().build_for_basis(
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: branch.clone(),
            index_ids: vec![index.index_id],
        },
        &inherited_basis,
    );
    assert!(inherited.failed_indexes.is_empty());
    assert_eq!(inherited.generations.len(), 1);
    let inherited_generation = inherited.generations[0].generation_id;
    assert_ne!(anchor.commit.branch_id, branch);
    drop(inherited_basis);
    release_test_commit_snapshot(&runtime, &anchor);

    // These expectations come from actual committed native fixtures, never
    // from a manufactured generation or a query on the lagging index.
    let assert_reopened = |plan: RecoveryPlan,
                           generation: DerivedIndexGenerationId,
                           source: &RelationalCommitReceipt,
                           current: &RelationalCommitReceipt,
                           expected_name: &str| {
        let mut reopened = persisted_runtime_with_test_schema();
        reopened.durability_recovery().recover(plan).unwrap();
        let retained = reopened.indexes.generation(generation).unwrap();
        assert_eq!(retained.index_id, index.index_id);
        assert_eq!(retained.source_commit_id, source.commit_id);
        assert_eq!(retained.source_branch_id, branch);
        assert_eq!(retained.applicability.branch_id, branch);
        assert_eq!(retained.applicability.version_id, source.version_id);
        assert_ne!(source.commit_id, current.commit_id);
        let identity = reopened.branch_identity(&branch).unwrap();
        let (_, basis) = reopened.observe_branch(&identity).unwrap();
        assert_eq!(basis.observation().commit_receipt(), Some(current));
        assert!(
            reopened
                .index_access()
                .published_generation_for_observation(index.index_id, &basis.observation())
                .is_none(),
            "retained lagging cache is not current index authority"
        );
        let read = reopened
            .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
            .unwrap()
            .read_entity(entity)
            .unwrap();
        assert_eq!(
            read.base().and_then(read_entity_name),
            Some(expected_name.to_owned())
        );
    };

    let first = update_entity_on_branch(&runtime, entity, "fork-own-first", branch.clone());
    assert_eq!(first.commit.branch_id, branch);
    release_test_commit_snapshot(&runtime, &first);
    runtime.durability_authority().checkpoint().unwrap();
    assert_reopened(
        runtime
            .durability()
            .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification),
        inherited_generation,
        &anchor.commit,
        &first.commit,
        "fork-own-first",
    );

    // The next cache is authored on the fork's own older commit. Advance its
    // real branch once more without rebuilding that cache, then capture again.
    let own_generation = build_on_branch(&runtime, index.index_id, &first, branch.clone());
    let second = update_entity_on_branch(&runtime, entity, "fork-own-second", branch.clone());
    assert_eq!(second.commit.branch_id, branch);
    release_test_commit_snapshot(&runtime, &second);
    runtime.durability_authority().checkpoint().unwrap();
    assert_reopened(
        runtime
            .durability()
            .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification),
        own_generation,
        &first.commit,
        &second.commit,
        "fork-own-second",
    );
}
