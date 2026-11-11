//! Exact main and sibling subjects are captured at their original heads.
use super::branch_history_publications::fork;
use super::*;
use worth_relational::facade::history::BranchId;

#[test]
fn recovered_main_and_sibling_reexport_the_original_canonical_subjects_and_envelopes() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    let seed = create_entity_outcome(execution, &source, "catalog-relink-seed");
    release_test_commit_snapshot(&source, &seed);
    let seed_recorded = current(execution, &source);
    let (_, seed_basis) = source
        .observe_branch(&source.main_branch_identity())
        .unwrap();
    let seed_lease = source.retain_component_basis(&seed_basis).unwrap();
    drop(seed_basis);
    fork(execution, &source, "catalog-relink-sibling");
    let sibling = BranchId("catalog-relink-sibling".into());
    let sibling_commit = super::checkpoint_root_publications::create_on_branch(
        execution,
        &source,
        "catalog-relink-fork",
        sibling.clone(),
    );
    release_test_commit_snapshot(&source, &sibling_commit);
    let port = source.owner_component_services().basis_port();
    let (_, sibling_basis) = port
        .observe_branch(&source.branch_identity(&sibling).unwrap())
        .unwrap();
    let sibling_recorded = port
        .export_recorded_current_component(execution, &sibling_basis)
        .unwrap();
    let main_commit = create_entity_outcome(execution, &source, "catalog-relink-main");
    release_test_commit_snapshot(&source, &main_commit);
    let main_recorded = current(execution, &source);
    medium.file_checkpoint(capture(execution, &source));
    source.release_component_basis(seed_lease).unwrap();
    let (recovered, authority) = recover(execution, &medium);
    for (commit_id, recorded) in [
        (seed.commit.commit_id, seed_recorded),
        (sibling_commit.commit.commit_id, sibling_recorded),
        (main_commit.commit.commit_id, main_recorded),
    ] {
        let original = source
            .replay()
            .canonical_commit_envelope(execution, commit_id)
            .unwrap()
            .unwrap();
        let readmitted = recovered
            .replay()
            .canonical_commit_envelope(execution, commit_id)
            .unwrap()
            .unwrap();
        assert_eq!(readmitted.commit, original.commit);
        assert_eq!(readmitted.branch_context, original.branch_context);
        assert_eq!(readmitted.semantic_result(), original.semantic_result());
        let original_frame = source
            .durability()
            .canonical_commit_artifacts(execution)
            .map(Result::unwrap)
            .find(|artifact| artifact.commit_id() == commit_id)
            .unwrap();
        let readmitted_frame = recovered
            .durability()
            .canonical_commit_artifacts(execution)
            .map(Result::unwrap)
            .find(|artifact| artifact.commit_id() == commit_id)
            .unwrap();
        assert_eq!(readmitted_frame.as_bytes(), original_frame.as_bytes());
        let basis = authority
            .admit_recorded_component(execution, &recovered, &recorded)
            .unwrap()
            .into_basis();
        assert_recorded_basis(&recovered, &basis, &recorded);
        assert_eq!(
            recovered
                .owner_component_services()
                .basis_port()
                .export_recorded_retained_component(execution, &basis)
                .unwrap(),
            recorded
        );
    }
}
