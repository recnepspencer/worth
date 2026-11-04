//! Sealed test doors over this module's private members.
use super::*;

impl SelectedRebase {
    #[doc(hidden)]
    pub fn prepare_full_for_test(
        facts: FactPayload,
        moved: Arc<[usize]>,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, PreparedRebaseDenial> {
        Self::prepare_full(facts, moved, control)
    }
}

impl SelectedRebase {
    #[doc(hidden)]
    pub fn rebase_for_test(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        retired: &BTreeSet<EntityId>,
        producer: bool,

        control: StorageControl<'_, '_>,
    ) -> RebasedSourceFacts {
        Self::rebase(
            self, execution, runtime, snapshot, retired, producer, control,
        )
    }
}
