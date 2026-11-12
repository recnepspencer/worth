use super::branch_history_publications::*;
use super::merge_proof_journey::{merge_request, published_merge_authority};
use worth_relational::facade::merge::*;
use worth_relational::facade::{history::BranchId, runtime::RelationalRuntime};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::durable_runtime;
#[test]
fn retained_merge_correspondence_witness_preserves_exact_admitted_truth() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = correspondence_runtime(execution, &medium);
    let prepared = runtime
        .merge()
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
        )
        .expect("prepared merge");
    let witness = runtime
        .merge()
        .retain_merge_correspondence_witness_from_prepared_execution(&prepared);

    assert_eq!(
        witness.request_digest(),
        prepared.request().request_digest()
    );
    assert_eq!(
        witness.branch_basis_digest(),
        prepared.artifact().branch_basis.basis_digest()
    );
    assert_eq!(
        witness.rows().len(),
        prepared.artifact().identity_discovery.candidates.len()
    );
    assert_eq!(witness.admitted_rows().count(), 1);
    assert!(witness.rows().iter().any(|row| row.posture()
        == RelationalMergeCorrespondenceWitnessPosture::Admitted
        && row.target_record().is_some()
        && matches!(
            row.reason(),
            worth_relational::facade::merge::IdentityResolutionReason::ExactStorageIdentity
        )));
    assert!(witness.rows().iter().any(|row| {
        row.posture() == RelationalMergeCorrespondenceWitnessPosture::UnavailableMissingTarget
            && row.target_record().is_none()
    }));
    assert!(!witness.witness_digest().is_empty());
}
#[test]
fn retained_merge_correspondence_witness_survives_publication_and_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = correspondence_runtime(execution, &medium);
    let outcome = super::merge_proof_journey::execute_merge(execution, &runtime);
    let live_witness = outcome.execution_summary.correspondence_witness.clone();
    let live_authority =
        published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id);
    let (_recovery, recovered) =
        super::merge_proof_journey::checkpoint_and_recover(execution, &runtime, &medium);
    let recovered_authority =
        published_merge_authority(execution, &recovered, outcome.commit.commit.commit_id);

    assert_eq!(
        live_witness,
        live_authority.execution_summary.correspondence_witness
    );
    assert_eq!(
        live_witness,
        recovered_authority.execution_summary.correspondence_witness
    );
    assert_eq!(
        live_witness.witness_digest(),
        outcome
            .execution_summary
            .proof_packet
            .correspondence_witness_digest()
    );
    runtime
        .inspect_what_happened()
        .prepare_merge_support_inspection_witness(&outcome.execution_summary)
        .expect("consistent emitted packet");
}
fn correspondence_runtime(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    medium: &std::sync::Arc<TestDurableMedium>,
) -> RelationalRuntime {
    let runtime = durable_runtime(medium);
    let shared = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    update_entity(execution, &runtime, shared, "shared-value");
    update_entity_on_branch(
        execution,
        &runtime,
        shared,
        "shared-value",
        BranchId("feature".to_string()),
    );
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    runtime
}
