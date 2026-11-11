use super::merge_proof_journey::published_merge_authority;
use super::merge_witness_authoring::*;
use super::merge_witness_schema::policy_builder;
use worth_relational::facade::inspection::*;
use worth_relational::facade::merge::*;
use worth_testkit_relational::durable_medium::TestDurableMedium;
#[test]
fn merge_support_inspection_witness_parity_survives_live_replay_recovery_and_compatibility_lane() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let builder = policy_builder(AspectMergePolicyKind::PreferRicher, true);
    let runtime = builder.clone().build_durable(medium.attachment()).unwrap();
    let prepared = prepared_collaboration(execution, &runtime, false);
    let outcome = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    let live_support = runtime
        .inspect_what_happened()
        .prepare_merge_support_inspection_witness(&outcome.execution_summary)
        .expect("live support");
    let published_authority =
        published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id);
    let published_support = runtime
        .inspect_what_happened()
        .prepare_published_merge_support_inspection_witness(&published_authority)
        .expect("published support");
    super::branch_history_recovery::checkpoint(execution, &runtime, &medium);
    let (_, recovered) = super::branch_history_recovery::recover(
        execution,
        &medium,
        builder.build_durable(medium.attachment()).unwrap(),
    );
    let recovered_authority =
        published_merge_authority(execution, &recovered, outcome.commit.commit.commit_id);
    let recovered_support = recovered
        .inspect_what_happened()
        .prepare_published_merge_support_inspection_witness(&recovered_authority)
        .expect("recovered support");

    assert_eq!(live_support, published_support);
    assert_eq!(published_support, recovered_support);
    assert_eq!(published_support.rows(), recovered_support.rows());
    assert_eq!(published_support.profile(), recovered_support.profile());
    assert_eq!(
        published_support.witness_digest(),
        recovered_support.witness_digest()
    );
    assert!(published_support.rows().iter().any(|row| matches!(
        row,
        RelationalMergeSupportInspectionRow::Compatibility {
            posture: RelationalMergeSupportInspectionCompatibilityPosture::UnavailablePhaseDependency,
            absence: Some(
                RelationalMergeSupportInspectionAbsenceKind::MissingCompatibilityWitnessPhaseDependency
            ),
            ..
        }
    )));
    assert_ne!(live_support.witness_digest(), "");
}
