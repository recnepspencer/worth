//! A durable branch remains authoritative; retirement refuses before movement.
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::{
    RuntimeWorldBranchRetirementDenial, RuntimeWorldCancellationSource, RuntimeWorldServiceDenial,
};

#[test]
fn durable_branch_retirement_refuses_before_root_movement_and_the_home_keeps_publishing() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    drop(fixture.bootstrap_root(&execution));
    let before = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert!(matches!(
        fixture
            .home
            .world()
            .branch_port()
            .retire_product_branch(&before),
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchRetirementDenial::DurableBranchPublicationRequired
        ))
    ));
    let after = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(before.branch_identity(), after.branch_identity());
    assert_eq!(before.selected_commit(), after.selected_commit());
    assert_eq!(before.reference_generation(), after.reference_generation());
    drop(before);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        after,
        crate::batches::create(0),
        &cancellation.token(),
    );
    let performed = fixture.publish(&execution, prepared, &cancellation.token());
    assert_eq!(
        performed
            .component_results()
            .relational_commit_result()
            .unwrap()
            .semantic_result()
            .created_entities()
            .len(),
        1
    );
    drop(performed.consume());
    super::world_occurrence_readmission::close(fixture.home, &execution);
}
