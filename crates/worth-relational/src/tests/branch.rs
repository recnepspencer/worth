mod owner_seal;
mod retired_names;
mod sealed_owner_surface;

use crate::branch::RelationalForkDenial;
use crate::history::data::BranchId;
use crate::tests::support::{create_entity_outcome, runtime_with_test_schema};

#[test]
fn missing_source_denial_leaves_registry_and_catalog_unchanged() {
    let runtime = runtime_with_test_schema();
    let before_cells = runtime.history.branch_cells_snapshot();
    let before_catalog_count = runtime.history.catalog_len();
    let missing = BranchId("missing-source".to_owned());

    assert!(matches!(
        runtime.observe_fork_source(&missing),
        Err(RelationalForkDenial::SourceBranchMissing)
    ));

    assert_eq!(runtime.history.branch_cells_snapshot(), before_cells);
    assert_eq!(runtime.history.branch_count(), 1);
    assert_eq!(
        runtime.history.catalog_len(),
        before_catalog_count,
        "a missing source must not materialize or publish an artifact"
    );
}

#[test]
fn root_owned_artifact_keeps_fork_available_when_catalog_accelerator_is_missing() {
    let runtime = runtime_with_test_schema();
    let commit = create_entity_outcome(&runtime, "fork-source");
    let source = BranchId("main".to_owned());
    let target = BranchId("missing-artifact-target".to_owned());
    let (_, basis) = runtime
        .observe_fork_source(&source)
        .expect("committed source is forkable");

    assert!(runtime
        .history_authority()
        .remove_commit_envelope_for_test(commit.commit.commit_id));
    let before_source = runtime
        .history
        .branch_cell(&source)
        .expect("source remains registered")
        .checkpoint();
    let before_catalog_count = runtime.history.catalog_len();

    let forked = runtime
        .fork_branch(target.clone(), basis)
        .expect("the exact source root, not the catalog accelerator, authorizes the fork");

    assert_eq!(forked.shared_commit_id(), Some(commit.commit.commit_id));
    assert_eq!(runtime.history.branch_count(), 2);
    let after_source = runtime
        .history
        .branch_cell(&source)
        .expect("source remains registered")
        .checkpoint();
    assert_eq!(after_source.observation, before_source.observation);
    assert_eq!(after_source.truth_version, before_source.truth_version);
    assert!(runtime.history.branch_cell(&target).is_some());
    assert_eq!(runtime.history.catalog_len(), before_catalog_count);
}
