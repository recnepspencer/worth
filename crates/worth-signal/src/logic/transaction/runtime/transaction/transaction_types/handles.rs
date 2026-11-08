use crate::clock::RuntimeInstant;
use crate::data::handle::NodeId;
use crate::data::output::ChangedRegion;
use crate::data::proof::DirtyBatchEntry;
use crate::data::telemetry::RuntimeTelemetry;
use crate::logic::checkpoint::CheckpointRuntime;
use crate::logic::events::EventBus;

use super::super::super::config::SignalRuntimeConfig;
use super::super::super::state::{
    BranchMutationLedger, ResourceRuntimeState, RuntimeObservationRegistry, TemporalRuntimeState,
};

use super::rollback::TransactionRollbackPacketSet;
use super::state::{TransactionExecutionState, TransactionScratch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::logic::transaction::runtime) enum TransactionCommitPosture {
    Visible,
    BranchLocal,
}

pub struct SignalTransaction<'a, D, I, E, Ctx, T = ()>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::logic::transaction::runtime) execution: worth_execution::ExecutionRequest<'a, 'a>,
    pub(in crate::logic::transaction::runtime) runtime_ctx: &'a mut Ctx,
    pub(in crate::logic::transaction::runtime) observations:
        &'a RuntimeObservationRegistry<D, I, E, Ctx, T>,
    pub(in crate::logic::transaction::runtime) config: &'a mut SignalRuntimeConfig<T>,
    pub(in crate::logic::transaction::runtime) graph: &'a mut crate::data::graph::SignalGraph,
    pub(in crate::logic::transaction::runtime) checkpoint: &'a mut CheckpointRuntime<D, I>,
    pub(in crate::logic::transaction::runtime) event_bus: &'a mut EventBus<E, D, Ctx>,
    pub(in crate::logic::transaction::runtime) resource: &'a mut ResourceRuntimeState,
    pub(in crate::logic::transaction::runtime) temporal: &'a mut TemporalRuntimeState,
    pub(in crate::logic::transaction::runtime) telemetry: Option<&'a mut RuntimeTelemetry>,
    pub(in crate::logic::transaction::runtime) branch_mutation_ledger: &'a mut BranchMutationLedger,
    pub(in crate::logic::transaction::runtime) branch_head_generation: &'a mut u64,
    pub(in crate::logic::transaction::runtime) branch_restore_snapshot_id:
        &'a mut Option<crate::state::SignalSnapshotId>,
    pub(crate) conditional_operation_scope: Option<
        crate::branch::owner_services::conditional_execution::SignalConditionalOperationScopeBinding,
    >,
    pub(in crate::logic::transaction::runtime) scratch: TransactionScratch<D, I, E>,
    pub(in crate::logic::transaction::runtime) rollback_packets: TransactionRollbackPacketSet<T>,
    pub(in crate::logic::transaction::runtime) poisoned: bool,
    pub(in crate::logic::transaction::runtime) finished: bool,
    pub(in crate::logic::transaction::runtime) execution_state: TransactionExecutionState,
    pub(in crate::logic::transaction::runtime) started_at: RuntimeInstant,
    pub(in crate::logic::transaction::runtime) commit_posture: TransactionCommitPosture,
}

impl<'a, D, I, E, Ctx, T> SignalTransaction<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::logic::transaction::runtime) fn with_telemetry(
        &mut self,
        update: impl FnOnce(&mut RuntimeTelemetry),
    ) {
        if let Some(telemetry) = self.telemetry.as_deref_mut() {
            update(telemetry);
        }
    }

    pub(in crate::logic::transaction::runtime) fn telemetry_snapshot(&self) -> RuntimeTelemetry {
        self.telemetry.as_deref().copied().unwrap_or_default()
    }

    pub(in crate::logic::transaction::runtime) fn captures_optional_telemetry(&self) -> bool {
        self.telemetry.is_some()
    }
}

pub struct BatchChangeSession<'tx, 'a, D, I, E, Ctx, T = ()>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::logic::transaction::runtime) tx: &'tx mut SignalTransaction<'a, D, I, E, Ctx, T>,
    pub(in crate::logic::transaction::runtime) entries: Vec<DirtyBatchEntry>,
    pub(in crate::logic::transaction::runtime) applied: bool,
}

impl<'tx, 'a, D, I, E, Ctx, T> BatchChangeSession<'tx, 'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::logic::transaction::runtime) fn new(
        tx: &'tx mut SignalTransaction<'a, D, I, E, Ctx, T>,
    ) -> Self {
        Self {
            tx,
            entries: Vec::new(),
            applied: false,
        }
    }

    pub fn mark(mut self, source: NodeId, changed_aspect: crate::data::aspect::Aspect) -> Self {
        self.entries
            .push(DirtyBatchEntry::without_regions(source, changed_aspect));
        self
    }

    pub fn mark_regions(
        mut self,
        source: NodeId,
        changed_aspect: crate::data::aspect::Aspect,
        changed_regions: &[ChangedRegion],
    ) -> Self {
        self.entries.push(DirtyBatchEntry::new(
            source,
            changed_aspect,
            changed_regions.to_vec(),
        ));
        self
    }
}
#[cfg(test)]
#[path = "handles_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

#[cfg(test)]
fn collect_owned_inline_type_sizes(census: &mut crate::inline_type_sizes::Census) {
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
