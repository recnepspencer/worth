//! A hidden registry slot is allocated before any branch can retire.
use super::*;

#[derive(Debug)]
pub(crate) enum WorthQueryCleanupReservationDenial {
    Capacity,
    SizeOverflow,
    BackingAllocation(std::collections::TryReserveError),
    Request(worth_execution::WorkCeilingDenial),
    Storage(worth_execution::ExecutionOrderedMapDenial),
}

impl WorthQueryProductBranchOwnerCleanupRegistry {
    pub(crate) fn reserve_funded_retirement(
        &self,
        branch: &worth_runtime_world::facade::ProductBranchIdentity,
        incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryProductBranchOwnerCleanupReservation, WorthQueryCleanupReservationDenial>
    {
        let width = branch.name().as_str().len();
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<CleanupEntry>())
            .ok()
            .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
            .and_then(|bytes| bytes.checked_add(width as u64))
            .ok_or(WorthQueryCleanupReservationDenial::SizeOverflow)?;
        request
            .run_native_with_checkpoint(|context| {
                if context.has_accounting() {
                    context.checkpoint(width as u64)
                } else {
                    context.poll()
                }
            })
            .map_err(WorthQueryCleanupReservationDenial::Request)?
            .0
            .map_err(|cause| {
                WorthQueryCleanupReservationDenial::Request(
                    worth_execution::WorkCeilingDenial::Stopped(cause),
                )
            })?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(WorthQueryCleanupReservationDenial::Request)?;
        let scope = CleanupScope::ApplicationRetirement(WorthQueryRetiredProductOccurrence::new(
            branch.clone(),
            incarnation,
        ));
        self.reserve_entry(scope, memory, Some(request))
    }
}

#[must_use = "cleanup capacity must be installed or released"]
pub(crate) struct WorthQueryProductBranchOwnerCleanupReservation {
    registry: WorthQueryProductBranchOwnerCleanupRegistry,
    identity: u64,
    entry: Option<Arc<CleanupEntry>>,
    armed: bool,
}

impl WorthQueryProductBranchOwnerCleanupReservation {
    pub(super) fn new(
        registry: WorthQueryProductBranchOwnerCleanupRegistry,
        identity: u64,
        entry: Arc<CleanupEntry>,
    ) -> Self {
        Self {
            registry,
            identity,
            entry: Some(entry),
            armed: true,
        }
    }

    pub(crate) fn install_retirement(
        self,
        report: worth_runtime_world::facade::ProductBranchRetirementReport,
    ) -> u64 {
        self.install(WorthQueryProductBranchOwnerCleanupRecord::from_retirement(
            report,
        ))
    }

    pub(crate) fn install_unpublished(
        self,
        cleanup: worth_runtime_world::facade::ProductUnpublishedCleanup,
    ) -> u64 {
        self.install(WorthQueryProductBranchOwnerCleanupRecord::from_unpublished(
            cleanup,
        ))
    }

    pub(in crate::domain_computation::execution_runtime::product_world::owner_cleanup) fn install(
        mut self,
        record: WorthQueryProductBranchOwnerCleanupRecord,
    ) -> u64 {
        let entry = self
            .entry
            .take()
            .expect("a reservation owns its hidden entry");
        *lock(&entry.record) = Some(record);
        let mut state = lock(&self.registry.state);
        debug_assert!(state
            .entries
            .get(&self.identity)
            .is_some_and(|installed| Arc::ptr_eq(installed, &entry)));
        state.reserved = state.reserved.checked_sub(1).expect("one reserved slot");
        entry.active.store(true, Ordering::Release);
        self.armed = false;
        drop(state);
        self.registry.after_install();
        self.identity
    }
}

impl Drop for WorthQueryProductBranchOwnerCleanupReservation {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut state = lock(&self.registry.state);
        let removed = state.entries.remove(&self.identity);
        assert!(removed, "the hidden unique slot remains reserved");
        state.reserved = state.reserved.checked_sub(1).expect("one reserved slot");
        self.armed = false;
    }
}
