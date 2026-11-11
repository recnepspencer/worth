//! Full functional inspection truth on the original self-edge fixture.
use super::checkpoint_inspection_observation::capture_inspection_truth_bundle;
use super::*;
use worth_relational::facade::history::BranchId;
use worth_testkit_relational::aspect_storage::publications::changed_entities;
#[test]
fn inspection_truth_bundle_recovery_parity_holds_for_current_and_historical_surfaces() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let created = create_entity_outcome(execution, &runtime, "bundle");
    let entity = changed_entities(&created)[0];
    let _relation = super::checkpoint_shipping_lookup::create_relation_outcome(
        execution, &runtime, entity, entity, "self",
    );
    medium.file_checkpoint(capture(execution, &runtime));

    let expected = capture_inspection_truth_bundle(
        execution,
        &runtime,
        &BranchId("main".to_string()),
        entity,
        created.version_id,
    );
    let (recovered, _) = recover(execution, &medium);
    let actual = capture_inspection_truth_bundle(
        execution,
        &recovered,
        &BranchId("main".to_string()),
        entity,
        created.version_id,
    );

    assert_eq!(expected, actual);
}
