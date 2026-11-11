use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::tests::support::*;

#[test]
fn recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = runtime_with_test_schema();
    let seed = create_entity_outcome(execution, &source, "catalog-relink-seed");
    release_test_commit_snapshot(&source, &seed);
    let sibling = create_branch_from_main(&source, "catalog-relink-sibling");
    let sibling_commit =
        create_entity_outcome_on_branch(execution, &source, "catalog-relink-fork", sibling.clone());
    release_test_commit_snapshot(&source, &sibling_commit);
    let main_commit = create_entity_outcome(execution, &source, "catalog-relink-main");
    release_test_commit_snapshot(&source, &main_commit);
    source.durability_authority().checkpoint(execution).unwrap();

    let plan = source
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    for commit_id in [
        seed.commit.commit_id,
        sibling_commit.commit.commit_id,
        main_commit.commit.commit_id,
    ] {
        let original = source.history.commit_artifact(commit_id).unwrap();
        let readmitted = recovered.history.commit_artifact(commit_id).unwrap();
        assert_eq!(readmitted.identity(), original.identity());
        assert_eq!(readmitted.parentage(), original.parentage());
        assert_eq!(readmitted.roots(), original.roots());
        assert_eq!(
            readmitted
                .canonical_payload_digest()
                .expect("resident fixture has a canonical digest"),
            original
                .canonical_payload_digest()
                .expect("resident fixture has a canonical digest")
        );
    }
}
