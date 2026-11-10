use super::branch_history_publications::*;
use super::merge_proof_journey::{merge_request, published_merge_authority};
use super::merge_witness_authoring::*;
use super::merge_witness_schema::policy_builder;
use worth_relational::facade::merge::*;
use worth_testkit_relational::durable_medium::TestDurableMedium;
#[test]
fn retained_merge_strategy_witness_survives_prepared_execution_publication_and_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let builder = policy_builder(AspectMergePolicyKind::PreferRicher, false);
    let runtime = builder.clone().build_durable(medium.attachment()).unwrap();
    let shared = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    update_entity_status_on_branch(execution, &runtime, shared, "inactive", "main");
    update_entity_status_on_branch(execution, &runtime, shared, "active", "feature");

    let prepared = runtime
        .merge()
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
        )
        .expect("prepared merge");
    let live_witness = runtime
        .merge()
        .retain_merge_strategy_witness_from_prepared_execution(&prepared);
    let outcome = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    let live_authority =
        published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id);
    super::branch_history_recovery::checkpoint(execution, &runtime, &medium);
    let (_, recovered) = super::branch_history_recovery::recover(
        execution,
        &medium,
        builder.build_durable(medium.attachment()).unwrap(),
    );
    let recovered_authority =
        published_merge_authority(execution, &recovered, outcome.commit.commit.commit_id);

    assert_eq!(live_witness, outcome.execution_summary.strategy_witness);
    assert_eq!(
        live_witness,
        live_authority.execution_summary.strategy_witness
    );
    assert_eq!(
        live_witness,
        recovered_authority.execution_summary.strategy_witness
    );
    assert_eq!(
        live_witness.witness_digest(),
        outcome
            .execution_summary
            .proof_packet
            .strategy_witness_digest()
    );
    runtime
        .inspect_what_happened()
        .prepare_merge_support_inspection_witness(&outcome.execution_summary)
        .expect("consistent emitted packet");
}
