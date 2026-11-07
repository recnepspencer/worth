//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalTransaction<'static, (), (), (), (), ()>>("crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_types/handles.rs", "SignalTransaction", 128 * 1024);
        owner.field::<worth_execution::ExecutionRequest<'static, 'static>>("execution");
        owner.field::<&'static mut ()>("runtime_ctx");
        owner.field::<&'static RuntimeObservationRegistry<(), (), (), (), ()>>("observations");
        owner.field::<&'static mut SignalRuntimeConfig<()>>("config");
        owner.field::<&'static mut crate::data::graph::SignalGraph>("graph");
        owner.field::<&'static mut CheckpointRuntime<(), ()>>("checkpoint");
        owner.field::<&'static mut EventBus<(), (), ()>>("event_bus");
        owner.field::<&'static mut ResourceRuntimeState>("resource");
        owner.field::<&'static mut TemporalRuntimeState>("temporal");
        owner.field::<Option<&'static mut RuntimeTelemetry>>("telemetry");
        owner.field::<&'static mut BranchMutationLedger>("branch_mutation_ledger");
        owner.field::<&'static mut u64>("branch_head_generation");
        owner.field::<&'static mut Option<crate::state::SignalSnapshotId>>(
            "branch_restore_snapshot_id",
        );
        owner.field::<Option<
        crate::branch::owner_services::conditional_execution::SignalConditionalOperationScopeBinding,
    >>("conditional_operation_scope");
        owner.field::<TransactionScratch<(), (), ()>>("scratch");
        owner.field::<TransactionRollbackPacketSet<()>>("rollback_packets");
        owner.field::<bool>("poisoned");
        owner.field::<bool>("finished");
        owner.field::<TransactionExecutionState>("execution_state");
        owner.field::<RuntimeInstant>("started_at");
        owner.field::<TransactionCommitPosture>("commit_posture");
    }
}
