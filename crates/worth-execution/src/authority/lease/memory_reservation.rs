use super::{ExecutionResourceLease, LeaseDenial, ResourceReservation};

/// Owned custody of admitted payload-backing bytes in the execution ledger.
///
/// The allocation owner reserves before allocating and keeps this ticket until
/// the backing is freed. This API does not intercept allocations or charge
/// allocator overhead and the ledger's own Vec/HashMap bookkeeping. It grants
/// no workers, work allowance, or permission to execute after cancellation.
///
/// The ticket can outlive its originating lease. Moving it transfers custody;
/// sharing backing requires sharing its owner rather than duplicating tickets.
///
/// Tickets cannot be constructed without live execution admission.
///
/// ```compile_fail
/// use worth_execution::ExecutionMemoryReservation;
/// let ticket = ExecutionMemoryReservation {};
/// ```
///
/// One charge cannot be cloned into independent owners.
///
/// ```compile_fail
/// use worth_execution::ExecutionMemoryReservation;
/// fn duplicate(ticket: &ExecutionMemoryReservation) -> ExecutionMemoryReservation {
///     ticket.clone()
/// }
/// ```
pub struct ExecutionMemoryReservation {
    reservation: ResourceReservation,
}

impl ExecutionMemoryReservation {
    /// The unchanged payload-backing amount admitted to the current lineage.
    pub fn charged_bytes(&self) -> u64 {
        self.reservation.memory_bytes
    }
}

impl std::fmt::Debug for ExecutionMemoryReservation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExecutionMemoryReservation")
            .field("charged_bytes", &self.charged_bytes())
            .finish_non_exhaustive()
    }
}

impl ExecutionResourceLease<'_> {
    /// Atomically admit payload-backing bytes against this lease, every ancestor,
    /// and the process authority before the caller allocates that backing.
    ///
    /// For replacement growth, reserve the new backing while the old backing
    /// and ticket remain live. Free the old backing before dropping its ticket.
    pub fn reserve_memory(&self, bytes: u64) -> Result<ExecutionMemoryReservation, LeaseDenial> {
        self.reserve_retained_memory(bytes)
            .map(|reservation| ExecutionMemoryReservation { reservation })
    }

    /// Transfer the same live backing charge to this lease's lineage.
    ///
    /// The old charge remains intact on refusal. This does not resize storage,
    /// release the backing, or admit a temporary replacement allocation.
    pub fn transfer_memory(
        &self,
        ticket: &mut ExecutionMemoryReservation,
    ) -> Result<(), LeaseDenial> {
        let bytes = ticket.charged_bytes();
        self.rebind_retained_memory(&mut ticket.reservation, bytes)
    }
}
