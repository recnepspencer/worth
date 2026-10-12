use crate::branch::RelationalForkDenial;
use crate::facade::history::BranchId;
use crate::tests::support::{create_entity_outcome, runtime_with_test_schema};
use worth_foundational::FoundationalBranchTarget;

#[test]
fn metadata_only_generation_advance_stales_a_prior_fork_token() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "seed-main");
    let (descriptor, stale_basis) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("seeded main has a fork source");
    let truth_before = descriptor.truth_version();
    let generation_before = descriptor.observation().generation().get();
    let target_before = descriptor.observation().target().clone();

    runtime
        .history
        .branch_cell_mut(&BranchId("main".to_owned()))
        .expect("main cell remains registered")
        .advance_metadata()
        .expect("metadata movement is a production cell transition");

    let after_state = runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .expect("main cell remains observable");
    assert_eq!(after_state.truth_version(), truth_before);
    assert_eq!(after_state.observation().target(), &target_before);
    assert_eq!(
        after_state.observation().generation().get(),
        generation_before + 1
    );

    let catalog_before = runtime.history().immutable_commit_count();
    assert!(matches!(
        runtime.fork_branch(BranchId("stale-generation".to_owned()), stale_basis),
        Err(RelationalForkDenial::StaleSource)
    ));
    assert_eq!(runtime.history().immutable_commit_count(), catalog_before);
    assert!(runtime
        .branch_identity(&BranchId("stale-generation".to_owned()))
        .is_err());
    assert!(matches!(
        after_state.observation().target(),
        FoundationalBranchTarget::Basis(_)
    ));
}
