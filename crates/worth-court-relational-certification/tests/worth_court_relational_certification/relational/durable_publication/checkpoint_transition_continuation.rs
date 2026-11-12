//! Reopen resumes the original transition sequence after retired branch histories.
use super::branch_history_publications::{create_entity, fork};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence as Sequence, history::BranchId,
    mvcc::RelationalTransactionIntent,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium, durable_publication::durable_runtime,
};
#[test]
fn checkpoint_transition_sequence_resumes_after_archived_and_deleted_history() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let history = |extra_steps: bool| {
        let medium = TestDurableMedium::new();
        let mut runtime = durable_runtime(&medium);
        create_entity(execution, &runtime, "source");
        for index in 0..32 {
            let name = format!("branch-{index:03}");
            fork(execution, &runtime, &name);
            let identity = runtime.branch_identity(&BranchId(name)).unwrap();
            if extra_steps {
                let (_, basis) = runtime.observe_branch(&identity).unwrap();
                let transaction = runtime
                    .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                    .unwrap();
                runtime.archive_branch(execution, &identity).unwrap();
                runtime
                    .owner_component_services()
                    .lifecycle_port()
                    .delete_branch(execution, &identity)
                    .unwrap();
                drop(transaction);
                drop(basis);
            }
            runtime.delete_branch(execution, &identity).unwrap();
        }
        let count = if extra_steps { 128 } else { 64 };
        let checkpoint = runtime
            .durability_authority()
            .native_checkpoint(execution, execution.allocation_policy())
            .unwrap();
        assert_eq!(
            checkpoint.covered_branch_transitions(),
            Some(Sequence(count as u64))
        );
        medium.file_checkpoint(checkpoint);
        let recovered =
            super::branch_history_recovery::recover(execution, &medium, durable_runtime(&medium)).1;
        fork(execution, &recovered, "after-checkpoint");
        assert_eq!(
            medium.offered_transitions().last().unwrap().sequence(),
            Sequence(count as u64 + 1)
        );
    };
    history(false);
    history(true);
}
